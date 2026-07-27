use crate::authoring::{get_public_skill, public_skills};
use crate::error::AgentResult;
use crate::examples::search_public_examples;
use crate::project::project_context;
use serde_json::{Value, json};
use std::path::Path;

const PROTOCOL_VERSION: &str = "2025-11-25";

pub fn handle_mcp_message(root: impl AsRef<Path>, line: &str) -> AgentResult<Option<String>> {
    let request = match serde_json::from_str::<Value>(line) {
        Ok(request) => request,
        Err(error) => return response(error_response(Value::Null, -32700, error.to_string())),
    };
    let id = request.get("id").cloned();
    let method = request.get("method").and_then(Value::as_str);
    if id.is_none() {
        return Ok(None);
    }
    let id = id.unwrap_or(Value::Null);
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
    let result = match method {
        Some("initialize") => initialize(&params),
        Some("ping") => Ok(json!({})),
        Some("tools/list") => Ok(json!({ "tools": tool_definitions() })),
        Some("tools/call") => call_tool(root.as_ref(), &params),
        Some("resources/list") => Ok(json!({ "resources": resources() })),
        Some("resources/read") => read_resource(root.as_ref(), &params),
        Some(_) => Err((-32601, "method not found".to_string())),
        None => Err((-32600, "request method must be a string".to_string())),
    };
    match result {
        Ok(result) => response(json!({ "jsonrpc": "2.0", "id": id, "result": result })),
        Err((code, message)) => response(error_response(id, code, message)),
    }
}

fn initialize(params: &Value) -> Result<Value, (i64, String)> {
    let protocol_version = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(PROTOCOL_VERSION);
    Ok(json!({
        "protocolVersion": protocol_version,
        "capabilities": {
            "tools": { "listChanged": false },
            "resources": { "subscribe": false, "listChanged": false }
        },
        "serverInfo": {
            "name": "dowe-agent",
            "version": env!("CARGO_PKG_VERSION")
        },
        "instructions": "Use Dowe authoring skills and curated examples for user projects. Private /agents/skills are only for developing Dowe itself and are never exposed here."
    }))
}

fn call_tool(root: &Path, params: &Value) -> Result<Value, (i64, String)> {
    let name = required_string(params, "name")?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let result = match name {
        "dowe_skills_list" => Ok(json!({ "skills": public_skills() })),
        "dowe_skills_get" => {
            let id = required_string(&arguments, "id")?;
            let full = optional_bool(&arguments, "full")?.unwrap_or(false);
            get_public_skill(id, full)
                .map(|skill| json!(skill))
                .map_err(|error| error.to_string())
        }
        "dowe_examples_search" => {
            let query = required_string(&arguments, "query")?;
            let limit = optional_usize(&arguments, "limit")?.unwrap_or(5);
            search_public_examples(query, limit)
                .map(|search| json!(search))
                .map_err(|error| error.to_string())
        }
        "dowe_project_context" => project_context(root)
            .map(|context| json!(context))
            .map_err(|error| error.to_string()),
        _ => return Err((-32602, format!("unknown Dowe tool `{name}`"))),
    };
    Ok(match result {
        Ok(structured_content) => tool_result(structured_content, false),
        Err(message) => tool_result(json!({ "error": message }), true),
    })
}

fn read_resource(root: &Path, params: &Value) -> Result<Value, (i64, String)> {
    let uri = required_string(params, "uri")?;
    let (mime_type, text) = if uri == "dowe://context/project" {
        let context = project_context(root).map_err(|error| (-32603, error.to_string()))?;
        let text =
            serde_json::to_string_pretty(&context).map_err(|error| (-32603, error.to_string()))?;
        ("application/json", text)
    } else if let Some(id) = uri
        .strip_prefix("dowe://skills/")
        .and_then(|path| path.strip_suffix("/full"))
    {
        let skill = get_public_skill(id, true).map_err(|error| (-32602, error.to_string()))?;
        ("text/markdown", skill.content)
    } else if let Some(id) = uri.strip_prefix("dowe://skills/") {
        let skill = get_public_skill(id, false).map_err(|error| (-32602, error.to_string()))?;
        ("text/markdown", skill.content)
    } else {
        return Err((-32602, format!("unknown Dowe resource `{uri}`")));
    };
    Ok(json!({
        "contents": [{ "uri": uri, "mimeType": mime_type, "text": text }]
    }))
}

fn tool_result(structured_content: Value, is_error: bool) -> Value {
    let text = serde_json::to_string_pretty(&structured_content)
        .unwrap_or_else(|_| "unable to serialize tool result".to_string());
    json!({
        "content": [{ "type": "text", "text": text }],
        "structuredContent": structured_content,
        "isError": is_error
    })
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, (i64, String)> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| (-32602, format!("`{key}` must be a non-empty string")))
}

fn optional_bool(value: &Value, key: &str) -> Result<Option<bool>, (i64, String)> {
    match value.get(key) {
        Some(value) => value
            .as_bool()
            .map(Some)
            .ok_or_else(|| (-32602, format!("`{key}` must be a boolean"))),
        None => Ok(None),
    }
}

fn optional_usize(value: &Value, key: &str) -> Result<Option<usize>, (i64, String)> {
    match value.get(key) {
        Some(value) => value
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .map(Some)
            .ok_or_else(|| (-32602, format!("`{key}` must be a positive integer"))),
        None => Ok(None),
    }
}

fn resources() -> Vec<Value> {
    let mut resources = Vec::new();
    for skill in public_skills() {
        resources.push(json!({
            "uri": format!("dowe://skills/{}", skill.id),
            "name": format!("{} compact", skill.name),
            "description": skill.description,
            "mimeType": "text/markdown"
        }));
        resources.push(json!({
            "uri": format!("dowe://skills/{}/full", skill.id),
            "name": format!("{} full", skill.name),
            "description": format!("{} Includes declared references and examples.", skill.description),
            "mimeType": "text/markdown"
        }));
    }
    resources.push(json!({
        "uri": "dowe://context/project",
        "name": "Dowe project context",
        "description": "Compact project, harness, CodeGraph, source and skill discovery.",
        "mimeType": "application/json"
    }));
    resources
}

fn tool_definitions() -> Vec<Value> {
    vec![
        tool_definition(
            "dowe_skills_list",
            "List public Dowe source-authoring skills.",
            json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        ),
        tool_definition(
            "dowe_skills_get",
            "Get a compact or full public Dowe source-authoring skill.",
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "full": { "type": "boolean", "default": false }
                },
                "required": ["id"],
                "additionalProperties": false
            }),
        ),
        tool_definition(
            "dowe_examples_search",
            "Search up to five curated Dowe source examples.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "maxLength": 512 },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 5, "default": 5 }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
        ),
        tool_definition(
            "dowe_project_context",
            "Read compact Dowe project, harness and CodeGraph context.",
            json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        ),
    ]
}

fn tool_definition(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
        "annotations": {
            "readOnlyHint": true,
            "destructiveHint": false,
            "idempotentHint": true,
            "openWorldHint": false
        }
    })
}

fn error_response(id: Value, code: i64, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

fn response(value: Value) -> AgentResult<Option<String>> {
    Ok(Some(value.to_string()))
}
