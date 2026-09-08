use super::*;
use dowe_agent::{NativeRequestEvent, native_harness::RedactedTextStream};

impl TerminalHost<'_> {
    pub(super) async fn send_provider(
        &mut self,
        request: &AgentRequest,
    ) -> AgentResult<AgentServerResponse> {
        self.request_events.clear();
        let provider = request
            .provider
            .as_deref()
            .ok_or_else(|| AgentError::new("provider missing"))?;
        let key = if provider == self.key_provider {
            self.api_key.as_deref()
        } else {
            None
        };
        let activity_pause = self.activity.suspend()?;
        let auth =
            super::super::chat::ensure_provider_auth(&self.auth, provider, key, !self.json_output)
                .await
                .map_err(|error| AgentError::new(error.to_string()))?
                .ok_or_else(|| AgentError::new("provider authentication canceled"))?;
        drop(activity_pause);
        let mut redactor = Redactor::for_project(&self.root);
        for secret in self.secrets() {
            redactor.add(&secret);
        }
        if let Some(secret) = &auth.secret {
            redactor.add(secret);
        }
        let mut preview = RedactedTextStream::new(redactor, 32768);
        let mut native = request.clone();
        native.stream = true;
        let role = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("harness_role"))
            .map(String::as_str)
            .unwrap_or("execute");
        let response = dowe_agent::send_native_agent_request_observed(&native, &auth, &mut |event| {
            match event {
                NativeRequestEvent::TextDelta { text } => if let Some(text) = preview.push(text) {
                    if self.json_output {
                        println!("{}", json!({"event":"response_delta","role":role,"provider":provider,"model":request.model,"text":text}));
                    } else if self.activity.enabled() {
                        self.activity.stream("preview", &text);
                    }
                },
                NativeRequestEvent::RequestStarted { attempt } | NativeRequestEvent::RequestAttempt { attempt, .. } => {
                    let mut value = serde_json::to_value(event)?;
                    value["event"] = json!("request_attempt");
                    if matches!(event, NativeRequestEvent::RequestStarted { .. }) {
                        value["interrupted"] = json!(true);
                        value["usage"] = Value::Null;
                    }
                    self.request_events.retain(|previous| previous["attempt"] != *attempt);
                    value["role"] = json!(role);
                    value["provider"] = json!(provider);
                    value["model"] = json!(request.model);
                    self.request_events.push(value);
                }
            }
            Ok(())
        }).await;
        for event in &mut self.request_events {
            event["accounted"] = json!(true);
            {
                self.usage.record_usage(
                    provider,
                    &request.model,
                    serde_json::from_value(event["usage"].clone()).ok(),
                );
            }
        }
        response
    }
}
