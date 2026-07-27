use crate::model::{AgentRequestType, AgentToolDefinition, AgentToolFunction};
use serde_json::{Value, json};

pub fn agent_tool_definitions(request_type: AgentRequestType) -> Vec<AgentToolDefinition> {
    match request_type {
        AgentRequestType::Clarify | AgentRequestType::SpecPlan | AgentRequestType::VisionUi => {
            Vec::new()
        }
        AgentRequestType::Implementation => vec![
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
                        "reason": { "type": "string" }
                    },
                    "required": ["path", "content", "reason"]
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
