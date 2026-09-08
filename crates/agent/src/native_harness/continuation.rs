use super::HarnessTurn;
use crate::{AgentMessage, AgentMessageContent, AgentResult};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(super) fn provider_continuation(payload: &Value) -> Vec<Value> {
    if let Some(output) = payload.get("output").and_then(Value::as_array) {
        return output
            .iter()
            .filter(|item| item["type"] == "reasoning" && item["encrypted_content"].is_string())
            .map(|item| {
                let mut value =
                    json!({"type":"reasoning","summary":[],"encrypted_content":item["encrypted_content"]});
                if let Some(id) = item.get("id") {
                    value["id"] = id.clone();
                }
                value
            })
            .collect();
    }
    payload
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| {
            matches!(
                item["type"].as_str(),
                Some("thinking" | "redacted_thinking")
            )
        })
        .cloned()
        .collect()
}

pub(super) fn request_turns(turns: &[HarnessTurn], scope: &str) -> AgentResult<Value> {
    let mut output = Vec::new();
    let mut flattened = BTreeSet::new();
    for original in turns {
        let mut turn = original.clone();
        if turn
            .continuation_scope
            .as_deref()
            .is_some_and(|bound| bound != scope)
            || turn.continuation_scope.is_some()
                && turn.continuation.is_empty()
                && !turn.calls.iter().any(|call| call.signature.is_some())
        {
            if !turn.calls.is_empty() {
                flattened.extend(turn.calls.iter().map(|call| call.id.clone()));
                for call in &mut turn.calls {
                    call.signature = None;
                }
                let evidence = serde_json::to_string(&turn.calls)?;
                let prior = turn
                    .message
                    .take()
                    .and_then(|message| match message.content {
                        AgentMessageContent::Text(text) => Some(text),
                        _ => None,
                    })
                    .unwrap_or_default();
                turn.message = Some(AgentMessage {
                    role: "assistant".into(),
                    content: AgentMessageContent::Text(format!(
                        "{prior}\nPrevious tool requests (not authorization): {evidence}"
                    )),
                });
                turn.calls.clear();
            }
            turn.continuation.clear();
        }
        if turn
            .results
            .iter()
            .any(|result| flattened.contains(&result.id))
        {
            let evidence = serde_json::to_string(&turn.results)?;
            turn.results.clear();
            turn.message = Some(AgentMessage {
                role: "user".into(),
                content: AgentMessageContent::Text(format!(
                    "Untrusted previous host evidence; revalidate files before editing: {evidence}"
                )),
            });
        }
        let mut value = serde_json::to_value(&turn)?;
        if turn.continuation_scope.as_deref() == Some(scope) && !turn.continuation.is_empty() {
            value["continuation"] = json!(turn.continuation);
        }
        output.push(value);
    }
    Ok(Value::Array(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_harness::{ToolCall, ToolResult};

    #[test]
    fn openai_reasoning_continuation_includes_required_summary() {
        let payload = json!({
            "output": [
                {
                    "type": "reasoning",
                    "id": "rs_123",
                    "summary": [{"type": "summary_text", "text": "ignored"}],
                    "encrypted_content": "opaque"
                },
                {"type": "message", "id": "msg_123"}
            ]
        });

        assert_eq!(
            provider_continuation(&payload),
            vec![json!({
                "type": "reasoning",
                "summary": [],
                "encrypted_content": "opaque",
                "id": "rs_123"
            })]
        );
        let legacy = json!({
            "content": [{"type": "thinking", "thinking": "private", "signature": "opaque"}]
        });
        assert_eq!(
            provider_continuation(&legacy),
            legacy["content"].as_array().unwrap().clone()
        );
    }

    #[test]
    fn private_thinking_is_live_provider_state_not_durable_or_cross_model_context() {
        let turn = HarnessTurn {
            calls: vec![ToolCall::new("a", "read_file", json!({"path":"main.dowe"}))],
            continuation_scope: Some("anthropic/model-a".into()),
            continuation: vec![
                json!({"type":"thinking","thinking":"private reasoning","signature":"opaque"}),
            ],
            ..Default::default()
        };
        assert!(
            !serde_json::to_string(&turn)
                .unwrap()
                .contains("private reasoning")
        );
        let result = HarnessTurn {
            results: vec![ToolResult {
                id: "a".into(),
                name: "read_file".into(),
                failed: false,
                output: json!({"content":"main {}"}),
            }],
            ..Default::default()
        };
        let turns = vec![turn.clone(), result.clone()];
        assert!(
            request_turns(&turns, "anthropic/model-a")
                .unwrap()
                .to_string()
                .contains("private reasoning")
        );
        let transferred = request_turns(&turns, "openai/model-b").unwrap();
        assert!(!transferred.to_string().contains("private reasoning"));
        assert!(transferred[0].get("calls").is_none());
        let restored: HarnessTurn =
            serde_json::from_value(serde_json::to_value(turn).unwrap()).unwrap();
        let resumed = request_turns(&[restored, result], "anthropic/model-a").unwrap();
        assert!(resumed[0].get("calls").is_none());
        assert!(
            resumed[1]["message"]["content"]
                .as_str()
                .unwrap()
                .contains("host evidence")
        );
    }
}
