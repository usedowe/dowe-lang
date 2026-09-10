mod tests {
    use super::*;
    use crate::model::{AgentMessage, AgentMessageContent, AgentRequestType};
    use crate::provider::AgentAuthKind;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    include!("client_inference_tests.rs");
    include!("client_codex_tests.rs");
    include!("client_conversation_tests.rs");
    include!("client_provider_tests.rs");
    include!("client_provider_http_tests.rs");

    fn request(provider: &str, model: &str, stream: bool) -> AgentRequest {
        AgentRequest {
            request_id: "request-1".to_string(),
            provider: Some(provider.to_string()),
            request_type: AgentRequestType::Implementation,
            model: model.to_string(),
            messages: vec![AgentMessage {
                role: "user".to_string(),
                content: AgentMessageContent::Text("hello".to_string()),
            }],
            stream,
            tools: Vec::new(),
            metadata: None,
            response_format: None,
            extra: BTreeMap::new(),
        }
    }

    fn auth() -> ResolvedProviderAuth {
        ResolvedProviderAuth {
            kind: AgentAuthKind::ApiKey,
            secret: Some("test-secret".to_string()),
            env: BTreeMap::new(),
            source: "test".to_string(),
        }
    }

    #[test]
    fn openai_image_fixture_is_bounded_and_url_only_output_is_rejected() {
        let png = base64::engine::general_purpose::STANDARD.encode(b"\x89PNG\r\n\x1a\nfixture");
        let image = parse_openai_image_response(
            &json!({"data":[{"b64_json":png}]}),
            "gpt-image-1",
            "a blue circle",
        )
        .unwrap();
        assert_eq!(image.mime_type, "image/png");
        assert_eq!(image.provider, "openai");
        assert!(
            parse_openai_image_response(
                &json!({"data":[{"url":"https://example.test/image.png"}]}),
                "gpt-image-1",
                "prompt"
            )
            .is_err()
        );
        assert!(parse_openai_image_response(&json!({"data":[]}), "gpt-image-1", "prompt").is_err());
        let request = build_openai_image_request("gpt-image-1", "prompt").unwrap();
        assert_eq!(request["n"], 1);
        assert_eq!(request["response_format"], "b64_json");
    }

    #[test]
    fn builds_openai_requests_with_bearer_auth_and_no_dowe_envelope_fields() {
        let definition = provider_definition("openai").expect("provider");
        let request = request("openai", "gpt-5.5", false);
        let (url, headers, body) = build_provider_request(
            &definition,
            AgentProviderProtocol::OpenAiResponses,
            "https://example.test/v1",
            "gpt-5.5",
            &request,
            &auth(),
        )
        .expect("request");

        assert_eq!(url, "https://example.test/v1/responses");
        assert_eq!(headers[AUTHORIZATION], "Bearer test-secret");
        assert_eq!(body["model"], "gpt-5.5");
        assert!(body.get("requestType").is_none());
    }

    #[test]
    fn builds_pi_compatible_openai_codex_headers() {
        let definition = provider_definition("openai-codex").expect("provider");
        let payload = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&json!({
                "https://api.openai.com/auth": {"chatgpt_account_id": "account-1"}
            }))
            .expect("payload"),
        );
        let token = format!("header.{payload}.signature");
        let auth = ResolvedProviderAuth {
            kind: AgentAuthKind::OAuth,
            secret: Some(token),
            env: BTreeMap::new(),
            source: "stored credential".to_string(),
        };
        let mut request = request("openai-codex", "gpt-5.5", true);
        request
            .extra
            .insert("session_id".to_string(), json!("session-1"));
        let (url, headers, body) = build_provider_request(
            &definition,
            AgentProviderProtocol::OpenAiResponses,
            "https://chatgpt.com/backend-api",
            "gpt-5.5",
            &request,
            &auth,
        )
        .expect("request");

        assert_eq!(url, "https://chatgpt.com/backend-api/codex/responses");
        assert_eq!(headers["chatgpt-account-id"], "account-1");
        assert_eq!(headers["originator"], "pi");
        assert_eq!(headers["openai-beta"], "responses=experimental");
        assert_eq!(headers[ACCEPT], "text/event-stream");
        assert_eq!(headers["session-id"], "session-1");
        assert_eq!(body["store"], false);
    }

    #[test]
    fn maps_openrouter_anthropic_models_to_the_anthropic_endpoint() {
        let definition = provider_definition("openrouter").expect("provider");
        let request = request("openrouter", "anthropic/claude-sonnet-4", true);
        let (url, _, _) = build_provider_request(
            &definition,
            AgentProviderProtocol::AnthropicMessages,
            "https://openrouter.ai/api/v1",
            "anthropic/claude-sonnet-4",
            &request,
            &auth(),
        )
        .expect("request");
        assert_eq!(url, "https://openrouter.ai/api/v1/messages");
    }

    #[test]
    fn aggregates_openai_and_anthropic_sse_text() {
        let openai = parse_stream_payload(
            AgentProviderProtocol::OpenAiCompletions,
            "data: {\"id\":\"1\",\"choices\":[{\"delta\":{\"content\":\"hel\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"lo\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n",
        )
        .expect("openai");
        assert_eq!(openai["choices"][0]["message"]["content"], "hello");

        let anthropic = parse_stream_payload(
            AgentProviderProtocol::AnthropicMessages,
            "event: content_block_delta\ndata: {\"delta\":{\"type\":\"text_delta\",\"text\":\"hello\"}}\n\nevent: message_delta\ndata: {\"delta\":{\"stop_reason\":\"end_turn\"}}\n\n",
        )
        .expect("anthropic");
        assert_eq!(anthropic["content"][0]["text"], "hello");
    }
}
