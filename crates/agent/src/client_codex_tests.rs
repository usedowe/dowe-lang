fn codex_test_auth() -> ResolvedProviderAuth {
    let payload = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&json!({
            "https://api.openai.com/auth": {"chatgpt_account_id":"codex-test-account"}
        }))
        .unwrap(),
    );
    ResolvedProviderAuth {
        kind: AgentAuthKind::OAuth,
        secret: Some(format!("header.{payload}.signature")),
        env: BTreeMap::new(),
        source: "stored credential".into(),
    }
}

fn prepared_codex_request(thinking_level: Option<crate::ThinkingLevel>) -> AgentRequest {
    let root = tempfile::tempdir().unwrap();
    crate::prepare_agent_request(
        root.path(),
        "hola",
        crate::AgentPrepareOptions {
            provider: Some("openai-codex".into()),
            model: Some("gpt-5.6-luna".into()),
            thinking_level,
            ..Default::default()
        },
    )
    .unwrap()
    .request
}

#[test]
fn codex_payload_does_not_forward_generic_generation_parameters() {
    let definition = provider_definition("openai-codex").unwrap();
    for level in [None, Some(crate::ThinkingLevel::Medium)] {
        let mut request = prepared_codex_request(level);
        assert_eq!(request.request_type, crate::AgentRequestType::Clarify);
        assert!(request.extra["max_completion_tokens"].as_u64().unwrap() > 0);
        assert!(request.extra.contains_key("temperature"));
        request.extra.insert("max_output_tokens".into(), json!(123));
        request.extra.insert("max_tokens".into(), json!(456));
        request
            .extra
            .insert("arbitrary_parameter".into(), json!(true));
        if level.is_some() {
            request.tools = crate::agent_tool_definitions(crate::AgentRequestType::Implementation);
            assert!(!request.tools.is_empty());
        }
        let (_, headers, body) = build_provider_request(
            &definition,
            definition.protocol,
            "http://localhost",
            "gpt-5.6-luna",
            &request,
            &codex_test_auth(),
        )
        .unwrap();
        assert_eq!(headers[ACCEPT], "text/event-stream");
        assert!(body.get("max_output_tokens").is_none(), "{body}");
        let mut keys = vec![
            "model",
            "instructions",
            "input",
            "stream",
            "store",
            "text",
            "include",
            "tool_choice",
            "parallel_tool_calls",
            "prompt_cache_key",
        ];
        if !request.tools.is_empty() {
            keys.push("tools");
            let tools: Vec<Value> = request.tools.iter().map(|tool| json!({"type":"function","name":tool.function.name,"description":tool.function.description,"parameters":tool.function.parameters})).collect();
            assert_eq!(body["tools"], json!(tools));
        }
        if level.is_some() {
            keys.push("reasoning");
            assert_eq!(body["reasoning"], json!({"effort":"medium"}));
        }
        keys.sort_unstable();
        assert_eq!(
            body.as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            keys
        );
        assert_eq!(body["model"], "gpt-5.6-luna");
        assert_eq!(body["stream"], true);
        assert_eq!(body["store"], false);
        assert_eq!(body["prompt_cache_key"], request.extra["session_id"]);
        assert_eq!(body["text"], json!({"verbosity":"low"}));
        assert_eq!(body["include"], json!(["reasoning.encrypted_content"]));
        assert_eq!(body["tool_choice"], "auto");
        assert_eq!(body["parallel_tool_calls"], true);
        assert!(!body["instructions"].as_str().unwrap().is_empty());
        assert_eq!(body["input"][0]["role"], "user");
        let content: Value =
            serde_json::from_str(body["input"][0]["content"].as_str().unwrap()).unwrap();
        assert_eq!(content["userPrompt"], "hola");
    }
}

#[test]
fn public_openai_responses_keeps_generation_limits_and_temperature() {
    let mut request = prepared_codex_request(None);
    request.provider = Some("openai".into());
    request.model = "gpt-5.5".into();
    let definition = provider_definition("openai").unwrap();
    let (_, _, body) = build_provider_request(
        &definition,
        definition.protocol,
        "http://localhost",
        "gpt-5.5",
        &request,
        &auth(),
    )
    .unwrap();
    assert_eq!(
        body["max_output_tokens"],
        request.extra["max_completion_tokens"]
    );
    assert_eq!(body["temperature"], request.extra["temperature"]);
    assert_eq!(body["stream"], false);
    assert!(body.get("max_completion_tokens").is_none());
}

#[tokio::test]
async fn prepared_codex_request_passes_a_strict_http_endpoint() {
    use std::io::{Read, Write};
    for thinking in [None, Some(crate::ThinkingLevel::Medium)] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut data = Vec::new();
            let body = loop {
                let mut buffer = [0; 4096];
                let count = socket.read(&mut buffer).unwrap();
                assert!(count > 0);
                data.extend_from_slice(&buffer[..count]);
                if let Some(end) = data.windows(4).position(|value| value == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&data[..end]);
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse().unwrap())
                        })
                        .unwrap();
                    if data.len() >= end + 4 + length {
                        assert!(headers.starts_with("POST /codex/responses "));
                        break serde_json::from_slice::<Value>(&data[end + 4..end + 4 + length])
                            .unwrap();
                    }
                }
            };
            let invalid = [
                "max_output_tokens",
                "max_completion_tokens",
                "max_tokens",
                "temperature",
                "thinkingLevel",
                "session_id",
            ]
            .iter()
            .any(|key| body.get(*key).is_some());
            let (status, response) = if invalid {
                (
                    "400 Bad Request",
                    r#"{"detail":"Unsupported parameter: max_output_tokens"}"#,
                )
            } else {
                (
                    "200 OK",
                    "event: response.completed\ndata: {\"type\":\"response.completed\",\"response\":{\"status\":\"completed\",\"output_text\":\"Hola\",\"usage\":{\"input_tokens\":1000,\"output_tokens\":10}}}\n\n",
                )
            };
            write!(
                socket,
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                response.len()
            )
            .unwrap();
            assert_eq!(body["stream"], true);
            if thinking.is_some() {
                assert_eq!(body["reasoning"]["effort"], "medium");
            }
        });
        let request = prepared_codex_request(thinking);
        let definition = provider_definition("openai-codex").unwrap();
        let auth = codex_test_auth();
        let (url, headers, body) = build_provider_request(
            &definition,
            definition.protocol,
            &format!("http://{address}"),
            "gpt-5.6-luna",
            &request,
            &auth,
        )
        .unwrap();
        let response = streaming::send_observed(
            &url,
            &headers,
            body,
            &auth,
            &request,
            definition.protocol,
            &mut |_| Ok(()),
        )
        .await;
        server.join().unwrap();
        let payload = parse_stream_payload(definition.protocol, &response.unwrap()).unwrap();
        assert_eq!(payload["output_text"], "Hola");
        assert_eq!(
            crate::agent_response_usage("openai-codex", "gpt-5.6-luna", &payload)
                .unwrap()
                .context_tokens(),
            1010
        );
    }
}
