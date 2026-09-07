use crate::{AgentError, AgentMessage, AgentResult, agent_response_text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl ToolCall {
    pub fn new(id: &str, name: &str, arguments: Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments,
            signature: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: String,
    pub name: String,
    pub failed: bool,
    pub output: Value,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HarnessTurn {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<AgentMessage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<ToolCall>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<ToolResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation_scope: Option<String>,
    #[serde(default, skip_serializing)]
    pub continuation: Vec<Value>,
}

pub fn response_turn(payload: &Value) -> AgentResult<HarnessTurn> {
    if crate::conversation::response_failed(None, payload) {
        return Err(AgentError::new(
            "provider returned an unsuccessful response",
        ));
    }
    let mut turn = HarnessTurn {
        continuation: super::continuation::provider_continuation(payload),
        ..Default::default()
    };
    if let Ok(text) = agent_response_text(payload) {
        turn.message = Some(AgentMessage {
            role: "assistant".into(),
            content: crate::AgentMessageContent::Text(text),
        });
    }
    if let Some(calls) = payload
        .pointer("/choices/0/message/tool_calls")
        .and_then(Value::as_array)
    {
        for call in calls {
            turn.calls
                .push(parse_call(call, "id", "function", "arguments")?);
        }
    }
    if let Some(output) = payload.get("output").and_then(Value::as_array) {
        for call in output.iter().filter(|item| item["type"] == "function_call") {
            turn.calls
                .push(parse_call(call, "call_id", "", "arguments")?);
        }
    }
    if let Some(content) = payload.get("content").and_then(Value::as_array) {
        for call in content.iter().filter(|item| item["type"] == "tool_use") {
            turn.calls.push(parse_call(call, "id", "", "input")?);
        }
    }
    if let Some(parts) = payload
        .pointer("/candidates/0/content/parts")
        .and_then(Value::as_array)
    {
        for (index, part) in parts.iter().enumerate() {
            if let Some(call) = part.get("functionCall") {
                let mut call = call.clone();
                if call.get("id").is_none() {
                    call["id"] = json!(format!("google-{index}-{}", super::identifier()));
                }
                let mut call = parse_call(&call, "id", "", "args")?;
                call.signature = part
                    .get("thoughtSignature")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                turn.calls.push(call);
            }
        }
    }
    if let Some(parts) = payload
        .pointer("/output/message/content")
        .and_then(Value::as_array)
    {
        for part in parts {
            if let Some(call) = part.get("toolUse") {
                turn.calls.push(parse_call(call, "toolUseId", "", "input")?);
            }
        }
    }
    let mut ids = BTreeSet::new();
    if turn.calls.len() > 32 || turn.calls.iter().any(|call| !ids.insert(&call.id)) {
        return Err(AgentError::new("duplicate or excessive tool calls"));
    }
    if turn.message.is_none() && turn.calls.is_empty() {
        return Err(AgentError::new(
            "provider returned neither visible text nor tool calls",
        ));
    }
    Ok(turn)
}

fn parse_call(value: &Value, id_key: &str, inner: &str, arguments: &str) -> AgentResult<ToolCall> {
    let id = value
        .get(id_key)
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty() && id.len() <= 256)
        .ok_or_else(|| AgentError::new("tool call requires a bounded id"))?;
    let function = if inner.is_empty() {
        value
    } else {
        &value[inner]
    };
    let name = function
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty() && name.len() <= 64)
        .ok_or_else(|| AgentError::new("tool call requires a bounded name"))?;
    let arguments = match function.get(arguments) {
        Some(Value::String(text)) if text.len() <= 1048576 => serde_json::from_str(text)?,
        Some(value) if value.is_object() && value.to_string().len() <= 1048576 => value.clone(),
        _ => {
            return Err(AgentError::new(
                "tool arguments must be a bounded JSON object",
            ));
        }
    };
    if !arguments.is_object() {
        return Err(AgentError::new("tool arguments must be an object"));
    }
    Ok(ToolCall::new(id, name, arguments))
}
