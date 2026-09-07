#[test]
fn retired_codex_model_is_rejected_before_authentication_or_http() {
    let root = tempfile::tempdir().unwrap();
    for model in ["gpt-5.3-codex", "openai-codex/gpt-5.3-codex"] {
        let options = crate::AgentPrepareOptions {
            provider: Some("openai-codex".into()),
            model: Some(model.into()),
            ..Default::default()
        };
        let error = crate::prepare_agent_request(root.path(), "hello", options).unwrap_err();
        assert!(error.to_string().contains("gpt-5.5"));
        let request = request("openai-codex", model, true);
        let definition = provider_definition("openai-codex").unwrap();
        let error = build_provider_request(
            &definition,
            definition.protocol,
            "http://localhost:1",
            model,
            &request,
            &auth(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("no longer supported"));
    }
    let prepared = crate::prepare_agent_request(
        root.path(),
        "hello",
        crate::AgentPrepareOptions {
            provider: Some("openai-codex".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(prepared.request.model, "gpt-5.5");
    assert!(crate::agent_model_details("openai-codex", "gpt-5.3-codex").is_none());
}

#[test]
fn thinking_is_lowered_into_native_requests_and_rejects_unknown_models() {
    for (provider, model, level, expected) in [
        ("openai", "gpt-5.5", "high", json!({"effort":"high"})),
        ("openai", "gpt-5.5", "off", json!({"effort":"none"})),
        (
            "anthropic",
            "claude-sonnet-4-6",
            "high",
            json!({"type":"adaptive"}),
        ),
        (
            "anthropic",
            "claude-sonnet-4-6",
            "off",
            json!({"type":"disabled"}),
        ),
    ] {
        let mut request = request(provider, model, false);
        request.extra.insert("thinkingLevel".into(), json!(level));
        request.extra.insert("temperature".into(), json!(0.3));
        let definition = provider_definition(provider).unwrap();
        let (_, _, body) = build_provider_request(
            &definition,
            protocol_for_model(&definition, model),
            "http://localhost",
            model,
            &request,
            &auth(),
        )
        .unwrap();
        assert_eq!(
            body[if provider == "anthropic" {
                "thinking"
            } else {
                "reasoning"
            }],
            expected
        );
        assert!(body.get("temperature").is_none());
        assert!(body.get("thinkingLevel").is_none());
        if provider == "anthropic" && level == "high" {
            assert_eq!(body["output_config"], json!({"effort":"high"}));
        }
        assert!(
            build_provider_request(
                &definition,
                definition.protocol,
                "http://localhost",
                "unknown",
                &request,
                &auth()
            )
            .is_err()
        );
    }
    let root = tempfile::tempdir().unwrap();
    let prepared = crate::prepare_agent_request(
        root.path(),
        "hello",
        crate::AgentPrepareOptions {
            provider: Some("openai-codex".into()),
            model: Some("gpt-6-astra".into()),
            thinking_level: Some(crate::ThinkingLevel::Max),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(prepared.request.extra["thinkingLevel"], "max");
    let mut body = json!({"temperature":0.3});
    crate::inference::apply_thinking(
        "openai-codex",
        "gpt-6-astra",
        crate::ThinkingLevel::Max,
        &mut body,
    )
    .unwrap();
    assert_eq!(body, json!({"reasoning":{"effort":"max"}}));
}

#[tokio::test]
async fn thinking_and_usage_round_trip_through_http() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0; 4096];
        let body = loop {
            let count = socket.read(&mut buffer).unwrap();
            assert!(count > 0);
            bytes.extend_from_slice(&buffer[..count]);
            if let Some(end) = bytes.windows(4).position(|v| v == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..end]);
                let length: usize = headers
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse().unwrap())
                    })
                    .unwrap();
                if bytes.len() >= end + 4 + length {
                    break serde_json::from_slice::<Value>(&bytes[end + 4..end + 4 + length])
                        .unwrap();
                }
            }
        };
        assert_eq!(body["reasoning"], json!({"effort":"high"}));
        assert!(body.get("thinkingLevel").is_none());
        let sse = "event: response.completed\ndata: {\"type\":\"response.completed\",\"response\":{\"status\":\"completed\",\"output_text\":\"hello\",\"usage\":{\"input_tokens\":1000,\"output_tokens\":100}}}\n\n";
        write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", sse.len(), sse).unwrap();
    });
    let mut request = request("openai", "gpt-5.5", true);
    request.extra.insert("thinkingLevel".into(), json!("high"));
    let definition = provider_definition("openai").unwrap();
    let (url, headers, body) = build_provider_request(
        &definition,
        definition.protocol,
        &format!("http://{address}"),
        "gpt-5.5",
        &request,
        &auth(),
    )
    .unwrap();
    let response = streaming::send_observed(
        &url,
        &headers,
        body,
        &auth(),
        &request,
        definition.protocol,
        &mut |_| Ok(()),
    )
    .await
    .unwrap();
    let payload = parse_stream_payload(definition.protocol, &response).unwrap();
    assert_eq!(payload["output_text"], "hello");
    let usage = crate::agent_response_usage("openai", "gpt-5.5", &payload).unwrap();
    assert_eq!(usage.context_tokens(), 1100);
    assert!((usage.cost_usd.unwrap() - 0.008).abs() < 1e-10);
    server.join().unwrap();
}

#[test]
fn terminal_usage_survives_stream_aggregation_without_chunk_summing() {
    let plain = aggregate_openai(vec![(
        None,
        json!({"choices":[{"delta":{"content":"hello"}}]}),
    )])
    .unwrap();
    assert_eq!(
        plain,
        json!({"id":null,"model":null,"choices":[{"index":0,"message":{"role":"assistant","content":"hello"},"finish_reason":"stop"}]})
    );
    let events = vec![
        (
            None,
            json!({"choices":[{"delta":{"content":"hello"}}],"usage":{"prompt_tokens":10,"completion_tokens":1}}),
        ),
        (
            None,
            json!({"choices":[],"usage":{"prompt_tokens":10,"completion_tokens":2},"service_tier":"priority"}),
        ),
    ];
    let payload = aggregate_openai(events).unwrap();
    assert_eq!(payload["choices"][0]["message"]["content"], "hello");
    assert_eq!(
        payload["usage"],
        json!({"prompt_tokens":10,"completion_tokens":2})
    );
    assert_eq!(payload["service_tier"], "priority");
    let payload = aggregate_responses(vec![(None, json!({"type":"response.output_text.delta","delta":"hello"})), (None, json!({"type":"response.completed","response":{"id":"r1","status":"completed","usage":{"input_tokens":10,"output_tokens":2}}}))]).unwrap();
    assert_eq!(payload["output_text"], "hello");
    assert_eq!(
        payload["usage"],
        json!({"input_tokens":10,"output_tokens":2})
    );
    let payload = aggregate_anthropic(vec![(None, json!({"type":"message_start","message":{"usage":{"input_tokens":10,"output_tokens":0,"cache_read_input_tokens":5}}})), (None, json!({"type":"message_delta","usage":{"output_tokens":2}}))]).unwrap();
    assert_eq!(
        payload["usage"],
        json!({"input_tokens":10,"output_tokens":2,"cache_read_input_tokens":5})
    );
    let payload = aggregate_google(vec![
        (
            None,
            json!({"usageMetadata":{"promptTokenCount":10,"candidatesTokenCount":1}}),
        ),
        (
            None,
            json!({"usageMetadata":{"promptTokenCount":10,"candidatesTokenCount":2}}),
        ),
    ])
    .unwrap();
    assert_eq!(
        payload["usageMetadata"],
        json!({"promptTokenCount":10,"candidatesTokenCount":2})
    );
}
