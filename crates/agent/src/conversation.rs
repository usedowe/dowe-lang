use crate::{
    AgentError, AgentMessage, AgentMessageContent, AgentPrepareOptions, AgentPreparedRequest,
    AgentRequest, AgentRequestType, AgentResult, AgentServerResponse, prepare_agent_request,
};
use rand_core::{OsRng, RngCore};
use serde_json::Value;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct AgentConversation {
    history: Vec<AgentMessage>,
    session_id: String,
    next_request: AtomicU64,
}

impl Default for AgentConversation {
    fn default() -> Self {
        let mut bytes = [0; 16];
        OsRng.fill_bytes(&mut bytes);
        Self {
            history: Vec::new(),
            session_id: format!(
                "dowe-agent-session-{}",
                bytes
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            ),
            next_request: AtomicU64::new(0),
        }
    }
}

impl AgentConversation {
    pub fn prepare(
        &self,
        root: impl AsRef<Path>,
        prompt: &str,
        mut options: AgentPrepareOptions,
    ) -> AgentResult<AgentPreparedRequest> {
        if options
            .request_type
            .is_some_and(|kind| kind != AgentRequestType::Conversation)
        {
            return Err(AgentError::new(
                "conversation history requires the conversation request type",
            ));
        }
        options.request_type = Some(AgentRequestType::Conversation);
        let mut prepared = prepare_agent_request(root, prompt, options)?;
        prepared
            .request
            .messages
            .splice(1..1, self.history.iter().cloned());
        prepared.request.request_id = format!(
            "{}-{}",
            self.session_id,
            self.next_request.fetch_add(1, Ordering::Relaxed)
        );
        prepared
            .request
            .extra
            .insert("session_id".into(), Value::String(self.session_id.clone()));
        Ok(prepared)
    }

    pub fn record_response(
        &mut self,
        request: &AgentRequest,
        response: &AgentServerResponse,
    ) -> AgentResult<()> {
        if request.request_type != AgentRequestType::Conversation
            || response.request_id != request.request_id
            || response.request_type != request.request_type
            || response.model != request.model
            || request.extra.get("session_id").and_then(Value::as_str)
                != Some(self.session_id.as_str())
            || request.messages.len() != self.history.len() + 2
            || request.messages[0].role != "system"
            || &request.messages[1..request.messages.len() - 1] != self.history.as_slice()
        {
            return Err(AgentError::new(
                "response does not match the current conversation turn",
            ));
        }
        let user = request
            .messages
            .last()
            .expect("validated conversation messages");
        if user.role != "user" {
            return Err(AgentError::new(
                "conversation turn must end with a user message",
            ));
        }
        let text = agent_response_text(&response.payload)?;
        self.history.push(user.clone());
        self.history.push(AgentMessage {
            role: "assistant".into(),
            content: AgentMessageContent::Text(text),
        });
        Ok(())
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn response(request: &AgentRequest, payload: Value) -> AgentServerResponse {
        AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload,
        }
    }

    #[test]
    fn successful_turns_are_replayed_once_and_reset_is_explicit() {
        let root = tempfile::tempdir().unwrap();
        let mut conversation = AgentConversation::default();
        let first = conversation
            .prepare(root.path(), "Me llamo Ana", AgentPrepareOptions::default())
            .unwrap()
            .request;
        let result = response(&first, json!({"output_text":"Hola, **Ana**."}));
        conversation.record_response(&first, &result).unwrap();
        assert!(conversation.record_response(&first, &result).is_err());
        let options = AgentPrepareOptions {
            provider: Some("openai-codex".into()),
            model: Some("gpt-5.6-luna".into()),
            thinking_level: Some(crate::ThinkingLevel::Medium),
            ..Default::default()
        };
        let second = conversation
            .prepare(root.path(), "¿Cómo me llamo?", options)
            .unwrap()
            .request;
        assert_eq!(second.messages.len(), 4);
        assert_eq!(
            second.messages[1].content,
            AgentMessageContent::Text("Me llamo Ana".into())
        );
        assert_eq!(
            second.messages[2],
            AgentMessage {
                role: "assistant".into(),
                content: AgentMessageContent::Text("Hola, **Ana**.".into())
            }
        );
        assert_eq!(
            second.messages[3].content,
            AgentMessageContent::Text("¿Cómo me llamo?".into())
        );
        assert_eq!(second.extra["session_id"], first.extra["session_id"]);
        assert_ne!(second.request_id, first.request_id);
        assert!(second.tools.is_empty() && second.response_format.is_none());
        conversation.reset();
        assert!(
            conversation
                .record_response(&second, &response(&second, json!({"output_text":"Ana"})))
                .is_err()
        );
        let fresh = conversation
            .prepare(root.path(), "hola", AgentPrepareOptions::default())
            .unwrap()
            .request;
        assert_eq!(fresh.messages.len(), 2);
        assert_ne!(fresh.extra["session_id"], first.extra["session_id"]);
    }

    #[test]
    fn failed_or_foreign_responses_do_not_add_history() {
        let root = tempfile::tempdir().unwrap();
        let mut conversation = AgentConversation::default();
        let request = conversation
            .prepare(root.path(), "hola", AgentPrepareOptions::default())
            .unwrap()
            .request;
        for payload in [
            json!({"error":{"message":"failure"}}),
            json!({"output":[]}),
            json!({"type":"error","message":{"content":[{"type":"text","text":"bad"}]}}),
            json!({"choices":[{"message":{"tool_calls":[{"id":"1"}]}}]}),
        ] {
            assert!(
                conversation
                    .record_response(&request, &response(&request, payload))
                    .is_err()
            );
        }
        let mut foreign = response(&request, json!({"output_text":"hello"}));
        foreign.request_id = "other".into();
        assert!(conversation.record_response(&request, &foreign).is_err());
        assert_eq!(
            conversation
                .prepare(root.path(), "retry", AgentPrepareOptions::default())
                .unwrap()
                .request
                .messages
                .len(),
            2
        );
    }

    #[test]
    fn visible_text_is_extracted_from_all_native_response_shapes() {
        for payload in [
            json!({"choices":[{"message":{"content":"A\n\nB"}}]}),
            json!({"output_text":"A\n\nB"}),
            json!({"output":[{"type":"reasoning","summary":[{"text":"hidden"}]},{"type":"message","role":"assistant","content":[{"type":"output_text","text":"A"},{"type":"output_text","text":"B"}]}]}),
            json!({"content":[{"type":"thinking","thinking":"hidden"},{"type":"text","text":"A"},{"type":"tool_use","input":{"text":"hidden"}},{"type":"text","text":"B"}]}),
            json!({"candidates":[{"content":{"parts":[{"thought":true,"text":"hidden"},{"text":"A"},{"text":"B"}]}}]}),
            json!({"output":{"message":{"role":"assistant","content":[{"text":"A"},{"text":"B"}]}}}),
            json!({"outputs":[{"type":"message.output","role":"assistant","content":"A\n\nB"}]}),
            json!({"type":"done","message":{"role":"assistant","content":[{"type":"text","text":"A"},{"type":"text","text":"B"}]}}),
        ] {
            assert_eq!(
                agent_response_text(&payload).unwrap(),
                "A\n\nB",
                "{payload}"
            );
        }
    }
}
