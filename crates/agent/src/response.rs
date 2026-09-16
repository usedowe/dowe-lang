use crate::{AgentError, AgentResult};
use serde_json::Value;

pub(crate) fn response_failed(event: Option<&str>, payload: &Value) -> bool {
    [event, payload.get("type").and_then(Value::as_str)]
        .into_iter()
        .any(|kind| {
            matches!(
                kind,
                Some("error" | "response.failed" | "response.incomplete" | "response.cancelled")
            )
        })
        || payload.get("error").is_some_and(|error| !error.is_null())
        || matches!(
            payload.get("status").and_then(Value::as_str),
            Some("failed" | "cancelled" | "incomplete")
        )
        || matches!(
            payload.get("stopReason").and_then(Value::as_str),
            Some("error" | "aborted")
        )
        || payload
            .get("response")
            .is_some_and(|value| response_failed(None, value))
        || payload
            .get("message")
            .is_some_and(|value| response_failed(None, value))
}

pub fn agent_response_text(payload: &Value) -> AgentResult<String> {
    if response_failed(None, payload) {
        return Err(AgentError::new(
            "the provider returned an unsuccessful response",
        ));
    }
    if let Some(message) = payload.get("message") {
        return agent_response_text(message);
    }
    if let Some(text) = payload
        .get("output_text")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
    {
        return Ok(text.to_string());
    }
    let mut blocks = Vec::new();
    if let Some(content) = payload.pointer("/choices/0/message/content") {
        text_blocks(content, &mut blocks);
    } else if let Some(output) = payload
        .get("output")
        .or_else(|| payload.get("outputs"))
        .and_then(Value::as_array)
    {
        for message in output {
            if matches!(
                message.get("type").and_then(Value::as_str),
                Some("message" | "message.output")
            ) && message
                .get("role")
                .and_then(Value::as_str)
                .is_none_or(|role| role == "assistant")
                && let Some(content) = message.get("content")
            {
                text_blocks(content, &mut blocks);
            }
        }
    } else if let Some(content) = payload.pointer("/output/message/content") {
        text_blocks(content, &mut blocks);
    } else if let Some(content) = payload.get("content") {
        text_blocks(content, &mut blocks);
    } else if let Some(parts) = payload.pointer("/candidates/0/content/parts") {
        text_blocks(parts, &mut blocks);
    }
    let text = blocks.join("\n\n");
    if text.trim().is_empty() {
        return Err(AgentError::new(
            "the provider returned no visible assistant text; no local tools were executed",
        ));
    }
    Ok(text)
}

pub(crate) fn text_blocks(value: &Value, blocks: &mut Vec<String>) {
    if let Some(text) = value.as_str() {
        if !text.is_empty() {
            blocks.push(text.to_string());
        }
    } else if let Some(parts) = value.as_array() {
        for part in parts {
            if part.get("thought").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            if part
                .get("type")
                .and_then(Value::as_str)
                .is_none_or(|kind| matches!(kind, "text" | "output_text"))
                && let Some(text) = part
                    .get("text")
                    .and_then(Value::as_str)
                    .filter(|text| !text.is_empty())
            {
                blocks.push(text.to_string());
            }
        }
    }
}
