#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadArgs {
    path: String,
    #[serde(default = "first_line")]
    offset: usize,
    #[serde(default = "page_size")]
    limit: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListArgs {
    path: String,
    #[serde(default)]
    offset: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchArgs {
    path: String,
    query: String,
    #[serde(default = "first_line")]
    offset: usize,
}
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum SkillSource {
    #[default]
    Embedded,
    Project,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillArgs {
    id: String,
    #[serde(default)]
    resource: Option<String>,
    #[serde(default = "first_line")]
    offset: usize,
    #[serde(default)]
    source: SkillSource,
    #[serde(default)]
    hash: Option<String>,
}
fn first_line() -> usize {
    1
}
fn page_size() -> usize {
    200
}

fn normalize_embedded_skill_request(
    id: &str,
    resource: Option<&str>,
) -> AgentResult<(String, Option<String>)> {
    let id = id.trim();
    if id.starts_with("references/") {
        let matches = crate::public_skills()
            .into_iter()
            .filter(|skill| skill.resources.iter().any(|path| path == id))
            .collect::<Vec<_>>();
        return match (matches.as_slice(), resource) {
            ([skill], None) => Ok((skill.id.clone(), Some(id.to_string()))),
            ([skill], Some(bundle)) => {
                let normalized_bundle =
                    bundle.trim().strip_prefix("dowe-").unwrap_or(bundle.trim());
                if normalized_bundle != skill.id {
                    return Err(AgentError::new(format!(
                        "embedded skill resource `{id}` does not match bundle `{bundle}`; expected `{}`",
                        skill.id
                    )));
                }
                Ok((skill.id.clone(), Some(id.to_string())))
            }
            ([], _) => Err(AgentError::new(format!(
                "unknown embedded skill resource `{id}`"
            ))),
            _ => Err(AgentError::new(format!(
                "embedded skill resource `{id}` is ambiguous; specify the skill id"
            ))),
        };
    }
    let normalized_id = id.strip_prefix("dowe-").unwrap_or(id).to_string();
    let Some(resource) = resource else {
        return Ok((normalized_id, None));
    };
    let resource = resource.trim();
    if resource.starts_with("references/")
        && resource
            .strip_prefix("references/")
            .and_then(|name| name.strip_suffix(".md"))
            .is_some()
    {
        let matches = crate::public_skills()
            .into_iter()
            .filter(|skill| skill.resources.iter().any(|path| path == resource))
            .collect::<Vec<_>>();
        let skill = match matches.as_slice() {
            [skill] => skill,
            [] => {
                return Ok((normalized_id, Some(resource.to_string())));
            }
            _ => {
                return Err(AgentError::new(format!(
                    "embedded skill resource `{resource}` is ambiguous; specify the skill id"
                )));
            }
        };
        let basename = resource
            .strip_prefix("references/")
            .and_then(|name| name.strip_suffix(".md"))
            .expect("declared reference resources have a Markdown basename");
        let normalized_basename = basename.replace('_', "-");
        let basename_is_unambiguous = crate::public_skills()
            .into_iter()
            .flat_map(|skill| skill.resources)
            .filter_map(|path| {
                path.strip_prefix("references/")
                    .and_then(|name| name.strip_suffix(".md"))
                    .map(|name| name.replace('_', "-"))
            })
            .filter(|name| name == &normalized_basename)
            .count()
            == 1;
        if normalized_id != skill.id
            && normalized_id != basename
            && (!basename_is_unambiguous || normalized_id.replace('_', "-") != normalized_basename)
        {
            return Err(AgentError::new(format!(
                "embedded skill resource `{resource}` does not match skill id `{id}`; expected `{}` or `{basename}`",
                skill.id
            )));
        }
        return Ok((skill.id.clone(), Some(resource.to_string())));
    }
    if let Some(alias_id) = resource.strip_prefix("bundles/") {
        let is_exact_alias =
            alias_id == normalized_id || alias_id.replace('/', "-") == normalized_id;
        let is_parent_alias = normalized_id
            .strip_prefix(alias_id)
            .is_some_and(|suffix| suffix.starts_with('/'));
        if is_exact_alias || is_parent_alias {
            return Ok((normalized_id, None));
        }
        return Err(AgentError::new(format!(
            "legacy skill bundle alias `{resource}` does not match logical skill id `{normalized_id}`"
        )));
    }
    if resource.contains('/') || resource != normalized_id {
        return Ok((normalized_id, Some(resource.to_string())));
    }
    let shorthand = match normalized_id.as_str() {
        "core" => "references/main.md".to_string(),
        bundle => {
            let skill = crate::public_skills()
                .into_iter()
                .find(|skill| skill.id == bundle)
                .ok_or_else(|| AgentError::new(format!("unknown public Dowe skill `{id}`")))?;
            let matches = skill
                .resources
                .into_iter()
                .filter(|path| {
                    path.rsplit('/')
                        .next()
                        .is_some_and(|name| name.strip_suffix(".md") == Some(bundle))
                })
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(AgentError::new(format!(
                    "resource shorthand `{resource}` is not unambiguous for `{id}`"
                )));
            }
            matches.into_iter().next().unwrap()
        }
    };
    Ok((normalized_id, Some(shorthand)))
}

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
                for line in &lines {
                    let size = serde_json::to_string(line)?.len() + 2;
                    if encoded_size + size > self.config.max_output_bytes {
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

#[cfg(test)]
mod image_generation_contract_tests {
    use super::*;

    #[test]
    fn schema_does_not_advertise_overwrite_and_reference_rejection_is_explicit() {
        let definition =
            HarnessTools::definitions_for_capabilities(HarnessRole::Execute, false, true)
                .into_iter()
                .find(|tool| tool.function.name == "generate_image")
                .expect("generate_image definition");
        let properties = &definition.function.parameters["properties"];
        assert!(properties.get("overwrite").is_none());
        assert!(properties.get("reference_image_path").is_some());
        assert!(
            definition
                .function
                .description
                .contains("reference_image_path is unsupported in v1")
        );

        let root = tempfile::tempdir().expect("temporary project root");
        let mut tools = HarnessTools::new(root.path(), "image-contract", HarnessConfig::default())
            .expect("harness tools");
        let error = tools
            .prepare(
                &ToolCall::new(
                    "reference",
                    "generate_image",
                    json!({
                        "prompt": "a textured background",
                        "destination": "assets/background.png",
                        "reason": "test unsupported reference",
                        "reference_image_path": "assets/reference.png"
                    }),
                ),
                HarnessRole::Execute,
            )
            .expect_err("reference images must remain rejected");
        assert!(
            error
                .to_string()
                .contains("reference_image_path is not supported")
        );
    }
}

#[cfg(test)]
mod skill_tool_definition_contract_tests {
    use super::*;

    #[test]
    fn mutating_tools_require_skill_id_not_hash() {
        let definitions =
            HarnessTools::definitions_for_capabilities(HarnessRole::Execute, false, false);
        let get_skill = definitions
            .iter()
            .find(|tool| tool.function.name == "get_skill")
            .expect("get_skill definition");
        assert!(get_skill.function.description.contains("logical skill id"));
        assert!(
            get_skill
                .function
                .description
                .contains("integrity metadata only")
        );

        for name in ["write_file", "write_asset", "edit_file"] {
            let definition = definitions
                .iter()
                .find(|tool| tool.function.name == name)
                .expect("mutating tool definition");
            let skill = &definition.function.parameters["properties"]["skill"];
            assert_eq!(skill["type"], "string");
            assert!(
                skill["description"]
                    .as_str()
                    .expect("skill description")
                    .contains("never use the hash")
            );
        }
    }
}

#[cfg(test)]
mod codegraph_contract_tests {
    use super::*;

    #[test]
    fn codegraph_advertises_only_bounded_read_tools() {
        let names = HarnessTools::definitions_for_capabilities(HarnessRole::Codegraph, true, true)
            .into_iter()
            .map(|tool| tool.function.name)
            .collect::<Vec<_>>();
        assert!(names.iter().all(|name| matches!(
            name.as_str(),
            "ask_user" | "get_skill" | "read_file" | "list_files" | "search"
        )));
    }

    #[test]
    fn codegraph_rejects_mutating_and_execution_calls() {
        let root = tempfile::tempdir().unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "codegraph", HarnessConfig::default()).unwrap();
        for name in [
            "write_file",
            "edit_file",
            "write_asset",
            "propose_instruction_update",
            "shell",
            "generate_image",
            "capture_web_screenshot",
        ] {
            let call = ToolCall::new("codegraph", name, json!({}));
            assert!(
                tools.prepare(&call, HarnessRole::Codegraph).is_err(),
                "accepted {name}"
            );
        }
    }
}

#[cfg(test)]
mod list_files_tests {
    use super::*;

    #[test]
    fn missing_directory_returns_empty_successful_result() {
        let root = tempfile::tempdir().unwrap();
        let tools = HarnessTools::new(root.path(), "list-files", HarnessConfig::default()).unwrap();
        let result = tools
            .execute_read(&ToolCall::new(
                "list",
                "list_files",
                json!({"path": "assets", "offset": 7}),
            ))
            .unwrap();

        assert_eq!(
            result,
            json!({
                "exists": false,
                "paths": [],
                "next_offset": 0,
                "truncated": false,
            })
        );
    }

    #[test]
    fn existing_non_directory_returns_an_error() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("assets"), "not a directory").unwrap();
        let tools = HarnessTools::new(root.path(), "list-files", HarnessConfig::default()).unwrap();

        assert!(
            tools
                .execute_read(&ToolCall::new(
                    "list",
                    "list_files",
                    json!({"path": "assets"}),
                ))
                .is_err()
        );
    }
}

fn definition(
    name: &str,
    description: &str,
    properties: Value,
    required: &[&str],
) -> AgentToolDefinition {
    AgentToolDefinition {
        tool_type: "function".into(),
        function: AgentToolFunction {
            name: name.into(),
            description: description.into(),
            parameters: json!({"type":"object","properties":properties,"required":required,"additionalProperties":false}),
        },
    }
}

#[cfg(test)]
mod skill_normalization_tests {
    use super::*;

    #[test]
    fn normalizes_legacy_embedded_skill_forms() {
        assert_eq!(
            normalize_embedded_skill_request("dowe-views", Some("views")).unwrap(),
            ("views".to_string(), Some("references/views.md".to_string()))
        );
        assert_eq!(
            normalize_embedded_skill_request("dowe-core", Some("core")).unwrap(),
            ("core".to_string(), Some("references/main.md".to_string()))
        );
        assert_eq!(
            normalize_embedded_skill_request("references/composition.md", None).unwrap(),
            (
                "views".to_string(),
                Some("references/composition.md".to_string())
            )
        );
    }

    #[test]
    fn normalizes_swapped_resource_and_bundle_forms() {
        for resource in ["composition.md", "components.md"] {
            let id = format!("references/{resource}");
            assert_eq!(
                normalize_embedded_skill_request(&id, Some("views")).unwrap(),
                ("views".to_string(), Some(id))
            );
        }
    }

    #[test]
    fn infers_views_owner_from_declared_reference_ids() {
        for (id, resource) in [
            ("composition", "references/composition.md"),
            ("components", "references/components.md"),
            ("reference-ui", "references/reference-ui.md"),
        ] {
            assert_eq!(
                normalize_embedded_skill_request(id, Some(resource)).unwrap(),
                ("views".to_string(), Some(resource.to_string()))
            );
        }
    }

    #[test]
    fn rejects_declared_reference_id_mismatch() {
        let error =
            normalize_embedded_skill_request("composition", Some("references/components.md"))
                .unwrap_err()
                .to_string();
        assert!(error.contains("does not match skill id `composition`"));
    }

    #[test]
    fn rejects_swapped_resource_bundle_mismatch() {
        let error = normalize_embedded_skill_request("references/composition.md", Some("server"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("does not match bundle `server`"));
    }

    #[test]
    fn normalizes_matching_legacy_bundle_alias_to_compact_unit() {
        assert_eq!(
            normalize_embedded_skill_request("dowe-core", Some("bundles/core")).unwrap(),
            ("core".to_string(), None)
        );
    }

    #[test]
    fn normalizes_nested_legacy_bundle_alias_to_compact_unit() {
        assert_eq!(
            normalize_embedded_skill_request("dowe-views-pages", Some("bundles/views/pages"))
                .unwrap(),
            ("views-pages".to_string(), None)
        );
    }

    #[test]
    fn normalizes_hierarchical_legacy_bundle_aliases() {
        for (id, resource) in [
            ("core/validation", "bundles/core/validation"),
            ("core/validation", "bundles/core"),
            ("views/pages", "bundles/views"),
        ] {
            assert_eq!(
                normalize_embedded_skill_request(id, Some(resource)).unwrap(),
                (id.to_string(), None)
            );
        }
    }

    #[test]
    fn rejects_mismatched_legacy_bundle_alias() {
        let error = normalize_embedded_skill_request("core", Some("bundles/views"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("legacy skill bundle alias `bundles/views`"));
        assert!(error.contains("logical skill id `core`"));

        let error = normalize_embedded_skill_request(
            "core/validation",
            Some("bundles/core-validation-extra"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("legacy skill bundle alias"));

        let error = normalize_embedded_skill_request("core/validation", Some("bundles/corex"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("legacy skill bundle alias"));
    }

    #[test]
    fn rejects_ambiguous_or_unknown_resource_shorthand() {
        let error = normalize_embedded_skill_request("references/workflow.md", None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("ambiguous"));
        let error =
            normalize_embedded_skill_request("dowe-domain-modeling", Some("domain-modeling"))
                .unwrap_err()
                .to_string();
        assert!(error.contains("not unambiguous"));
    }
}
