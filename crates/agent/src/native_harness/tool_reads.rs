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
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillArgs {
    id: String,
    #[serde(default)]
    resource: Option<String>,
    #[serde(default = "first_line")]
    offset: usize,
}
fn first_line() -> usize {
    1
}
fn page_size() -> usize {
    200
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
                self.loaded
                    .lock()
                    .expect("loaded skills lock poisoned")
                    .insert(format!("{hash}:{offset}"));
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
                let mut names = Vec::new();
                for entry in fs::read_dir(self.path(&args.path)?)?.take(10001) {
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
                let (content, hash, dependencies) = if let Some(resource) = &args.resource {
                    let document = crate::get_public_skill_resource(&args.id, resource)?;
                    let hash = digest(document.content.as_bytes());
                    (document.content, hash, vec!["core".to_string()])
                } else {
                    let unit = skill_unit(&args.id)?;
                    (unit.content, unit.hash, unit.dependencies)
                };
                let key = format!("{hash}:{}", args.offset);
                if self
                    .loaded
                    .lock()
                    .map_err(|_| AgentError::new("loaded skills lock poisoned"))?
                    .contains(&key)
                {
                    return Ok(json!({"id":args.id,"hash":hash,"status":"already_loaded"}));
                }
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
                    json!({"id":args.id,"hash":hash,"offset":args.offset,"content":page,"dependencies":dependencies,"next_offset":args.offset + count,"truncated":count < lines.len()}),
                )
            }
            _ => Err(AgentError::new("not a skill tool")),
        }
    }

    pub fn definitions(role: HarnessRole, images: bool) -> Vec<AgentToolDefinition> {
        Self::definitions_for_capabilities(role, images, false)
    }

    pub fn definitions_for_capabilities(role: HarnessRole, images: bool, image_generation: bool) -> Vec<AgentToolDefinition> {
        if role == HarnessRole::Compact { return vec![]; }
        let mut tools = vec![
            definition("ask_user", "Ask one bounded contextual question. Host interaction only; this never executes a tool or grants permission.", json!({"id":{"type":"string","maxLength":64},"text":{"type":"string","maxLength":1024},"options":{"type":"array","maxItems":8,"items":{"type":"string","maxLength":256}}}), &["id", "text"]),
            definition("get_skill", "Load one fixed Dowe skill unit or declared bundle resource before authoring. Follow dependencies. Page large resources by offset.", json!({"id":{"type":"string"},"resource":{"type":"string"},"offset":{"type":"integer","minimum":1}}), &["id"]),
            definition("read_file", "Read a bounded application text file page. Environment values are hidden.", json!({"path":{"type":"string"},"offset":{"type":"integer","minimum":1},"limit":{"type":"integer","minimum":1,"maximum":1000}}), &["path"]),
            definition("list_files", "List a single application directory, excluding private/generated paths. Use offset to page.", json!({"path":{"type":"string"},"offset":{"type":"integer","minimum":0}}), &["path"]),
            definition("search", "Search literal text within one bounded file page.", json!({"path":{"type":"string"},"query":{"type":"string"},"offset":{"type":"integer","minimum":1}}), &["path", "query"]),
        ];
        if role == HarnessRole::Execute {
            let common = json!({"path":{"type":"string"},"skill":{"type":"string"},"reason":{"type":"string"},"content":{"type":"string"}});
            tools.push(definition("write_file", "Propose a Dowe application file write.", common, &["path", "skill", "reason", "content"]));
                tools.push(definition("propose_instruction_update", "Propose a complete root AGENTS.md or .agents/AGENTS.md update after the user clearly requests an instruction change. Summarize the instruction first; host approval remains required and this tool never grants it.", json!({"path":{"type":"string","enum":["AGENTS.md",".agents/AGENTS.md"]},"content":{"type":"string","maxLength":65536},"reason":{"type":"string","maxLength":1024}}), &["path", "content", "reason"]));
            tools.push(definition("write_asset", "Propose an approved bounded binary asset write using explicit base64.", json!({"path":{"type":"string"},"skill":{"type":"string"},"reason":{"type":"string"},"content_base64":{"type":"string"}}), &["path", "skill", "reason", "content_base64"]));
            tools.push(definition("edit_file", "Propose one exact unique replacement in a Dowe application file.", json!({"path":{"type":"string"},"skill":{"type":"string"},"reason":{"type":"string"},"old_text":{"type":"string"},"new_text":{"type":"string"}}), &["path", "skill", "reason", "old_text", "new_text"]));
            tools.push(definition("shell", "Request a general shell command; every call requires approval.", json!({"command":{"type":"string"},"cwd":{"type":"string"},"reason":{"type":"string"},"pty":{"type":"boolean"},"resource":{"type":"string"}}), &["command", "cwd", "reason"]));
            if images { tools.push(definition("capture_web_screenshot", "Capture a bounded PNG from an already-running loopback web URL.", json!({"url":{"type":"string"},"reason":{"type":"string"}}), &["url", "reason"])); }
            if image_generation { tools.push(definition("generate_image", "Generate independent illustrations, photos, textures, or device artwork only; use Dowe components, Icon, Svg, or Brand for logos, controls, and UI-shaped regions. Requires exact approval before writing under public/assets. reference_image_path is unsupported in v1.", json!({"prompt":{"type":"string","minLength":1,"maxLength":4096},"destination":{"type":"string"},"reason":{"type":"string","minLength":1,"maxLength":1024},"reference_image_path":{"type":"string"}}), &["prompt", "destination", "reason"])); }
        }
        tools
    }

}

    #[cfg(test)]
mod image_generation_contract_tests {
    use super::*;

    #[test]
    fn schema_does_not_advertise_overwrite_and_reference_rejection_is_explicit() {
        let definition = HarnessTools::definitions_for_capabilities(
            HarnessRole::Execute,
            false,
            true,
        )
        .into_iter()
        .find(|tool| tool.function.name == "generate_image")
        .expect("generate_image definition");
        let properties = &definition.function.parameters["properties"];
        assert!(properties.get("overwrite").is_none());
        assert!(properties.get("reference_image_path").is_some());
        assert!(definition
            .function
            .description
            .contains("reference_image_path is unsupported in v1"));

        let root = tempfile::tempdir().expect("temporary project root");
        let mut tools = HarnessTools::new(
            root.path(),
            "image-contract",
            HarnessConfig::default(),
        )
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
        assert!(error
            .to_string()
            .contains("reference_image_path is not supported"));
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
        assert!(names.iter().all(|name| matches!(name.as_str(), "ask_user" | "get_skill" | "read_file" | "list_files" | "search")));
    }

    #[test]
    fn codegraph_rejects_mutating_and_execution_calls() {
        let root = tempfile::tempdir().unwrap();
        let mut tools = HarnessTools::new(root.path(), "codegraph", HarnessConfig::default()).unwrap();
        for name in ["write_file", "edit_file", "write_asset", "propose_instruction_update", "shell", "generate_image", "capture_web_screenshot"] {
            let call = ToolCall::new("codegraph", name, json!({}));
            assert!(tools.prepare(&call, HarnessRole::Codegraph).is_err(), "accepted {name}");
        }
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
