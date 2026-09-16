use crate::model::{AgentRequestType, AgentToolDefinition, AgentToolFunction};
use serde_json::{Value, json};

pub fn agent_tool_definitions(request_type: AgentRequestType) -> Vec<AgentToolDefinition> {
    match request_type {
        AgentRequestType::Conversation
        | AgentRequestType::Clarify
        | AgentRequestType::SpecPlan
        | AgentRequestType::VisionUi => Vec::new(),
        AgentRequestType::Implementation => vec![
            function_tool(
                "convert_svg",
                "Convert a project-local SVG through the shared parse.svg implementation into Dowe Svg/Path source or normalized runtime vector data. The request is read-only and requires no approval.",
                json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "colors": { "type": "string", "enum": ["original", "tokens"], "default": "original" },
                        "format": { "type": "string", "enum": ["source", "data"], "default": "source" }
                    },
                    "required": ["path"]
                }),
            ),
            function_tool(
                "read_file",
                "Request a specific file path for local inspection by the Dowe binary.",
                json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "reason": { "type": "string" }
                    },
                    "required": ["path", "reason"]
                }),
            ),
            function_tool(
                "write_file",
                "Request a local file write. The Dowe binary must validate and approve before executing.",
                json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" },
                        "skill": { "type": "string" },
                        "reason": { "type": "string" }
                    },
                    "required": ["path", "content", "skill", "reason"]
                }),
            ),
            function_tool(
                "run_validation",
                "Request a declared validation command such as cargo test, harness check, or codegraph check.",
                json!({
                    "type": "object",
                    "properties": {
                        "command": { "type": "string" },
                        "reason": { "type": "string" }
                    },
                    "required": ["command", "reason"]
                }),
            ),
            function_tool(
                "explain_codegraph_node",
                "Request a focused CodeGraph explanation for a path or node id.",
                json!({
                    "type": "object",
                    "properties": {
                        "selector": { "type": "string" },
                        "reason": { "type": "string" }
                    },
                    "required": ["selector", "reason"]
                }),
            ),
        ],
    }
}

pub fn read_only_agent_tool_definitions() -> Vec<AgentToolDefinition> {
    agent_tool_definitions(AgentRequestType::Implementation)
        .into_iter()
        .filter(|tool| {
            matches!(
                tool.function.name.as_str(),
                "convert_svg" | "read_file" | "explain_codegraph_node"
            )
        })
        .collect()
}

/// Shape-only compatibility declarations for older conversation clients.
/// The clean read runner rejects these calls; they never grant execution.
pub fn conversation_compatibility_tool_definitions() -> Vec<AgentToolDefinition> {
    vec![
        function_tool(
            "shell",
            "Compatibility declaration only; ASK cannot execute shell commands.",
            json!({"type":"object","properties":{"command":{"type":"string"},"cwd":{"type":"string"},"reason":{"type":"string"}},"required":["command","cwd","reason"]}),
        ),
        function_tool(
            "propose_instruction_update",
            "Compatibility declaration only; ASK cannot mutate project instructions.",
            json!({"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"},"reason":{"type":"string"}},"required":["path","content","reason"]}),
        ),
    ]
}

fn function_tool(name: &str, description: &str, parameters: Value) -> AgentToolDefinition {
    AgentToolDefinition {
        tool_type: "function".to_string(),
        function: AgentToolFunction {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        },
    }
}
