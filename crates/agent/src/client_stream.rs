use super::*;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum NativeRequestEvent {
    RequestStarted {
        attempt: u8,
    },
    TextDelta {
        text: String,
    },
    RequestAttempt {
        attempt: u8,
        status: Option<u16>,
        succeeded: bool,
        usage: Option<crate::AgentUsage>,
    },
}

struct Frames {
    pending: Vec<u8>,
}
impl Frames {
    fn feed(
        &mut self,
        bytes: &[u8],
        protocol: AgentProviderProtocol,
        progress: &mut impl FnMut(&NativeRequestEvent) -> AgentResult<()>,
    ) -> AgentResult<()> {
        self.pending.extend_from_slice(bytes);
        if self.pending.len() > 2 * 1024 * 1024 {
            return Err(AgentError::new("provider SSE frame exceeds limit"));
        }
        loop {
            let boundary = self
                .pending
                .windows(2)
                .position(|s| s == b"\n\n")
                .map(|end| end + 2)
                .into_iter()
                .chain(
                    self.pending
                        .windows(4)
                        .position(|s| s == b"\r\n\r\n")
                        .map(|end| end + 4),
                )
                .min();
            let Some(end) = boundary else {
                break;
            };
            let frame = String::from_utf8(self.pending.drain(..end).collect())
                .map_err(|_| AgentError::new("provider SSE contains invalid UTF-8"))?;
            if !frame
                .lines()
                .filter_map(|line| line.strip_prefix("data:"))
                .any(|data| !data.trim().is_empty() && data.trim() != "[DONE]")
            {
                continue;
            }
            for (name, value) in parse_sse_events(&frame)? {
                if let Some(text) = visible_delta(protocol, name.as_deref(), &value) {
                    progress(&NativeRequestEvent::TextDelta { text })?;
                }
            }
        }
        Ok(())
    }
}

fn visible_delta(
    protocol: AgentProviderProtocol,
    name: Option<&str>,
    value: &Value,
) -> Option<String> {
    use AgentProviderProtocol::*;
    match protocol {
        OpenAiResponses
            if name.or_else(|| value["type"].as_str()) == Some("response.output_text.delta") =>
        {
            value["delta"].as_str().map(str::to_owned)
        }
        OpenAiCompletions | MistralConversations => value["choices"][0]["delta"]["content"]
            .as_str()
            .map(str::to_owned),
        AnthropicMessages if value["delta"]["type"] == "text_delta" => {
            value["delta"]["text"].as_str().map(str::to_owned)
        }
        GoogleGenerativeAi | GoogleVertex => {
            let parts = value["candidates"][0]["content"]["parts"].as_array()?;
            let text: String = parts
                .iter()
                .filter(|part| part["thought"] != true)
                .filter_map(|part| part["text"].as_str())
                .collect();
            (!text.is_empty()).then_some(text)
        }
        _ => None,
    }
}

pub(super) async fn send_observed(
    url: &str,
    headers: &HeaderMap,
    body: Value,
    auth: &ResolvedProviderAuth,
    request: &AgentRequest,
    protocol: AgentProviderProtocol,
    progress: &mut impl FnMut(&NativeRequestEvent) -> AgentResult<()>,
) -> AgentResult<String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| AgentError::new(e.to_string()))?;
    for attempt in 0..=2 {
        progress(&NativeRequestEvent::RequestStarted { attempt })?;
        let mut response = match client
            .post(url)
            .headers(headers.clone())
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                progress(&NativeRequestEvent::RequestAttempt {
                    attempt,
                    status: None,
                    succeeded: false,
                    usage: None,
                })?;
                return Err(AgentError::new(redact_secret(
                    &error.to_string(),
                    auth.secret.as_deref(),
                )));
            }
        };
        let status = response.status();
        let delay = response
            .headers()
            .get("retry-after")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(1 << attempt)
            .min(30);
        let sse = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("text/event-stream"));
        let mut bytes = Vec::new();
        let mut frames = Frames {
            pending: Vec::new(),
        };
        let result: AgentResult<()> = async {
            while let Some(chunk) = response.chunk().await.map_err(|error| {
                AgentError::new(redact_secret(&error.to_string(), auth.secret.as_deref()))
            })? {
                if bytes.len() + chunk.len() > 16 * 1024 * 1024 {
                    return Err(AgentError::new("provider response exceeds limit"));
                }
                bytes.extend_from_slice(&chunk);
                if status.is_success() && sse {
                    frames.feed(&chunk, protocol, progress)?;
                }
            }
            Ok(())
        }
        .await;
        let text = String::from_utf8_lossy(&bytes).to_string();
        let payload = if sse {
            parse_stream_payload(protocol, &text).ok()
        } else {
            serde_json::from_str(&text).ok()
        };
        let usage = payload.as_ref().and_then(|payload| {
            crate::agent_response_usage(
                request.provider.as_deref().unwrap_or_default(),
                &request.model,
                payload,
            )
        });
        progress(&NativeRequestEvent::RequestAttempt {
            attempt,
            status: Some(status.as_u16()),
            succeeded: status.is_success() && result.is_ok(),
            usage,
        })?;
        result?;
        if status.is_success() {
            return Ok(text);
        }
        if attempt < 2 && is_retryable(status) {
            tokio::time::sleep(Duration::from_secs(delay)).await;
            continue;
        }
        return Err(AgentError::new(format!(
            "provider request returned {}: {}",
            status_text(status),
            redact_secret(&text, auth.secret.as_deref())
        )));
    }
    Err(AgentError::new("provider retry loop exhausted"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fragmented_sse_exposes_only_visible_text() {
        let mut frames = Frames { pending: vec![] };
        let mut output = Vec::new();
        let source = "event: content_block_delta\r\ndata: {\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"private\"}}\r\n\r\nevent: content_block_delta\r\ndata: {\"delta\":{\"type\":\"text_delta\",\"text\":\"héllo\"}}\r\n\r\n: heartbeat\r\n\r\ndata: [DONE]\r\n\r\n";
        for byte in source.as_bytes() {
            frames
                .feed(
                    &[*byte],
                    AgentProviderProtocol::AnthropicMessages,
                    &mut |event| {
                        output.push(serde_json::to_value(event)?);
                        Ok(())
                    },
                )
                .unwrap();
        }
        assert_eq!(output, vec![json!({"event":"text_delta","text":"héllo"})]);
        assert!(frames.pending.is_empty());
    }
    #[test]
    fn thinking_tools_and_signatures_are_not_visible_deltas() {
        assert!(
            visible_delta(
                AgentProviderProtocol::OpenAiResponses,
                Some("response.reasoning_text.delta"),
                &json!({"delta":"private"})
            )
            .is_none()
        );
        assert!(visible_delta(AgentProviderProtocol::OpenAiCompletions, None, &json!({"choices":[{"delta":{"tool_calls":[{"function":{"arguments":"private"}}]}}]})).is_none());
        assert_eq!(
            visible_delta(
                AgentProviderProtocol::GoogleGenerativeAi,
                None,
                &json!({"candidates":[{"content":{"parts":[{"thought":true,"text":"private"},{"text":"visible"}]}}]})
            ),
            Some("visible".into())
        );
    }
}
