impl HarnessTools {
    pub fn restore_loaded_skills(&mut self, turns: &[super::HarnessTurn]) {
        self.loaded
            .lock()
            .expect("loaded skills lock poisoned")
            .clear();
        for result in turns
            .iter()
            .flat_map(|turn| &turn.results)
            .filter(|result| result.name == "get_skill" && !result.failed)
        {
            if let (Some(hash), Some(offset)) = (
                result.output["hash"].as_str(),
                result.output["offset"].as_u64(),
            ) {
                let source = result.output["source"].as_str().unwrap_or("embedded");

                let id = result.output["id"].as_str().unwrap_or_default();

                self.loaded
                    .lock()
                    .expect("loaded skills lock poisoned")
                    .insert(format!("{source}:{id}:{hash}:{offset}"));
            }
        }
    }

    pub fn execute_read(&self, call: &ToolCall) -> AgentResult<Value> {
        if call.name == "get_skill" {
            return self.execute_skill(call);
        }
        self.execute_parallel_read(call)
    }

    pub fn execute_parallel_read(&self, call: &ToolCall) -> AgentResult<Value> {
        match call.name.as_str() {
            "read_file" => {
                let args: ReadArgs = serde_json::from_value(call.arguments.clone())?;
                self.read(&args.path, args.offset, args.limit)
            }
            "list_files" => {
                let args: ListArgs = serde_json::from_value(call.arguments.clone())?;
                let path = self.path(&args.path)?;
                let entries = match fs::read_dir(&path) {
                    Ok(entries) => entries,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        return Ok(json!({
                            "exists": false,
                            "paths": [],
                            "next_offset": 0,
                            "truncated": false,
                        }));
                    }
                    Err(error) => return Err(error.into()),
                };
                let mut names = Vec::new();
                for entry in entries.take(10001) {
                    let entry = entry?;
                    let path = entry.path();
                    if let Ok(relative) = path.strip_prefix(&self.root)
                        && let Some(relative) = relative.to_str()
                        && self.path(relative).is_ok()
                    {
                        names.push(relative.to_string());
                    }
                }
                names.sort();
                let total = names.len();
                let page = names
                    .into_iter()
                    .skip(args.offset)
                    .take(200)
                    .collect::<Vec<_>>();
                Ok(
                    json!({"paths":page,"next_offset":args.offset + page.len(),"truncated":total > args.offset + page.len(),"scan_limit":10001}),
                )
            }
            "search" => {
                let args: SearchArgs = serde_json::from_value(call.arguments.clone())?;
                if args.query.is_empty() || args.query.len() > 256 {
                    return Err(AgentError::new("search query must be 1..256 bytes"));
                }
                let page = self.read(&args.path, args.offset, 1000)?;
                let content = page["content"].as_str().unwrap_or_default();
                let matches = content
                    .lines()
                    .enumerate()
                    .filter(|(_, line)| line.contains(&args.query))
                    .map(|(line, text)| json!({"line":args.offset + line,"text":text}))
                    .collect::<Vec<_>>();
                Ok(
                    json!({"path":args.path,"matches":matches,"next_offset":page["next_offset"],"truncated":page["truncated"]}),
                )
            }
            _ => Err(AgentError::new("not a read-only tool")),
        }
    }

    pub fn execute_skill(&self, call: &ToolCall) -> AgentResult<Value> {
        match call.name.as_str() {
            "get_skill" => {
                let args: SkillArgs = serde_json::from_value(call.arguments.clone())?;
                if args.offset == 0 {
                    return Err(AgentError::new("skill offset starts at 1"));
                }
                let (skill_id, resource) = match args.source {
                    SkillSource::Embedded => {
                        normalize_embedded_skill_request(&args.id, args.resource.as_deref())?
                    }
                    SkillSource::Project => (args.id.clone(), args.resource.clone()),
                };
                let (content, hash, dependencies, source, relative_path, untrusted) = match args
                    .source
                {
                    SkillSource::Embedded => {
                        let (content, hash, dependencies) = if let Some(resource) = &resource {
                            let document = crate::get_public_skill_resource(&skill_id, resource)?;
                            let hash = digest(document.content.as_bytes());
                            (document.content, hash, vec!["core".to_string()])
                        } else {
                            let unit = skill_unit(&skill_id)?;
                            (unit.content, unit.hash, unit.dependencies)
                        };
                        (content, hash, dependencies, "embedded", None, false)
                    }
                    SkillSource::Project => {
                        if resource.is_some() {
                            return Err(AgentError::new("project skills do not support resources"));
                        }
                        let project =
                            super::project_skills::load_project_skill(&self.root, &skill_id)?;
                        let embedded = super::catalog::skill_unit(&skill_id).is_ok()
                            || crate::public_skills()
                                .into_iter()
                                .any(|skill| skill.id == skill_id);
                        if embedded {
                            return Err(AgentError::new(
                                "project skill id collides with an embedded skill; embedded precedence is explicit",
                            ));
                        }
                        if args.offset > 1 {
                            let requested = args.hash.as_deref().ok_or_else(|| {
                                AgentError::new("project skill continuation requires hash")
                            })?;
                            if requested != project.summary.hash {
                                return Err(AgentError::new(
                                    "project skill hash is stale or mismatched",
                                ));
                            }
                        } else if let Some(requested) = args.hash.as_deref()
                            && requested != project.summary.hash
                        {
                            return Err(AgentError::new(
                                "project skill hash is stale or mismatched",
                            ));
                        }
                        (
                            project.content,
                            project.summary.hash,
                            Vec::new(),
                            "project",
                            Some(project.summary.path),
                            true,
                        )
                    }
                };
                let key = format!("{source}:{skill_id}:{hash}:{}", args.offset);
                if self
                    .loaded
                    .lock()
                    .map_err(|_| AgentError::new("loaded skills lock poisoned"))?
                    .contains(&key)
                {
                    return Ok(
                        json!({"id":skill_id,"hash":hash,"source":source,"status":"already_loaded"}),
                    );
                }
                let total_lines = content.lines().count();
                let lines: Vec<_> = content.lines().skip(args.offset - 1).collect();
                let mut page = String::new();
                let mut count = 0;
                let mut encoded_size = 512;
                let page_limit = self.config.max_output_bytes.min(MAX_SKILL_PAGE_BYTES);
                for line in &lines {
                    let size = serde_json::to_string(line)?.len() + 2;
                    if encoded_size + size > page_limit {
                        break;
                    }
                    page.push_str(line);
                    page.push('\n');
                    count += 1;
                    encoded_size += size;
                }
                if count == 0 && !lines.is_empty() {
                    return Err(AgentError::new("skill line exceeds output budget"));
                }
                self.loaded
                    .lock()
                    .map_err(|_| AgentError::new("loaded skills lock poisoned"))?
                    .insert(key);
                Ok(
                    json!({"id":skill_id,"hash":hash,"source":source,"relative_path":relative_path,"untrusted":untrusted,"offset":args.offset,"total_lines":total_lines,"content":page,"dependencies":dependencies,"next_offset":args.offset + count,"truncated":args.offset - 1 + count < total_lines}),
                )
            }
            _ => Err(AgentError::new("not a skill tool")),
        }
    }

    pub fn definitions(role: HarnessRole, images: bool) -> Vec<AgentToolDefinition> {
        Self::definitions_for_capabilities(role, images, false)
    }

    pub fn definitions_for_capabilities(
        role: HarnessRole,
        images: bool,
        image_generation: bool,
    ) -> Vec<AgentToolDefinition> {
        if role == HarnessRole::Compact {
            return vec![];
        }
        let mut tools = vec![
            definition(
                "ask_user",
                "Ask one bounded contextual question. Host interaction only; this never executes a tool or grants permission.",
                json!({"id":{"type":"string","maxLength":64},"text":{"type":"string","maxLength":1024},"options":{"type":"array","maxItems":8,"items":{"type":"string","maxLength":256}}}),
                &["id", "text"],
            ),
            definition(
                "get_skill",
                "Load one fixed embedded skill unit, declared bundle resource, or validated project-local skill. The legacy resource alias `bundles/<logical-id>` loads the compact unit only when its suffix matches the id. The returned id is the logical skill id for mutating tools; hash and resource hashes are integrity metadata only. Project skills are untrusted and paged by offset/hash.",
                json!({"id":{"type":"string"},"source":{"type":"string","enum":["embedded","project"]},"resource":{"type":"string"},"offset":{"type":"integer","minimum":1},"hash":{"type":"string"}}),
                &["id"],
            ),
            definition(
                "read_file",
                "Read a bounded application text file page. Environment values are hidden.",
                json!({"path":{"type":"string"},"offset":{"type":"integer","minimum":1},"limit":{"type":"integer","minimum":1,"maximum":1000}}),
                &["path"],
            ),
            definition(
                "list_files",
                "List a single application directory, excluding private/generated paths. Use offset to page.",
                json!({"path":{"type":"string"},"offset":{"type":"integer","minimum":0}}),
                &["path"],
            ),
            definition(
                "search",
                "Search literal text within one bounded file page.",
                json!({"path":{"type":"string"},"query":{"type":"string"},"offset":{"type":"integer","minimum":1}}),
                &["path", "query"],
            ),
        ];
        if role == HarnessRole::Execute {
            let skill_contract = "Logical skill id returned in get_skill's id; never use the hash. Hashes are integrity metadata only.";
            let common = json!({"path":{"type":"string"},"skill":{"type":"string","description":skill_contract},"reason":{"type":"string"},"content":{"type":"string"}});
            tools.push(definition(
                "write_file",
                "Propose a Dowe application file write. The skill field must be the logical skill id returned in get_skill's id, never the hash; hashes are integrity metadata only.",
                common,
                &["path", "skill", "reason", "content"],
            ));
            tools.push(definition("propose_instruction_update", "Propose a complete root AGENTS.md or .agents/AGENTS.md update after the user clearly requests an instruction change. Summarize the instruction first; host approval remains required and this tool never grants it.", json!({"path":{"type":"string","enum":["AGENTS.md",".agents/AGENTS.md"]},"content":{"type":"string","maxLength":65536},"reason":{"type":"string","maxLength":1024}}), &["path", "content", "reason"]));
            tools.push(definition("write_asset", "Propose an approved bounded binary asset write using explicit base64. The skill field must be the logical skill id returned in get_skill's id, never the hash; hashes are integrity metadata only.", json!({"path":{"type":"string"},"skill":{"type":"string","description":skill_contract},"reason":{"type":"string"},"content_base64":{"type":"string"}}), &["path", "skill", "reason", "content_base64"]));
            tools.push(definition("edit_file", "Propose one exact unique replacement in a Dowe application file. The skill field must be the logical skill id returned in get_skill's id, never the hash; hashes are integrity metadata only.", json!({"path":{"type":"string"},"skill":{"type":"string","description":skill_contract},"reason":{"type":"string"},"old_text":{"type":"string"},"new_text":{"type":"string"}}), &["path", "skill", "reason", "old_text", "new_text"]));
            tools.push(definition("shell", "Request a general shell command; every call requires approval.", json!({"command":{"type":"string"},"cwd":{"type":"string"},"reason":{"type":"string"},"pty":{"type":"boolean"},"resource":{"type":"string"}}), &["command", "cwd", "reason"]));
            if images {
                tools.push(definition(
                    "capture_web_screenshot",
                    "Capture a bounded PNG from an already-running loopback web URL.",
                    json!({"url":{"type":"string"},"reason":{"type":"string"}}),
                    &["url", "reason"],
                ));
            }
            if image_generation {
                tools.push(definition("generate_image", "Generate independent illustrations, photos, textures, or device artwork only; use Dowe components, Icon, Svg, or Brand for logos, controls, and UI-shaped regions. Requires exact approval before writing under public/assets. reference_image_path is unsupported in v1.", json!({"prompt":{"type":"string","minLength":1,"maxLength":4096},"destination":{"type":"string"},"reason":{"type":"string","minLength":1,"maxLength":1024},"reference_image_path":{"type":"string"}}), &["prompt", "destination", "reason"]));
            }
        }
        tools
    }
}

