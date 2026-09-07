fn provider_http_fixture(
    response: String,
    path: String,
    expected_headers: HeaderMap,
    expected_body: Value,
    redirect: bool,
) -> (String, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let location = format!("{base}/must-not-follow");
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0; 4096];
        loop {
            let count = socket.read(&mut buffer).unwrap();
            assert!(count > 0);
            bytes.extend_from_slice(&buffer[..count]);
            assert!(bytes.len() < 1_000_000);
            if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                let header_text = String::from_utf8_lossy(&bytes[..end]);
                let headers: BTreeMap<String, String> = header_text
                    .lines()
                    .skip(1)
                    .filter_map(|line| line.split_once(':'))
                    .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_string()))
                    .collect();
                let length: usize = headers["content-length"].parse().unwrap();
                if bytes.len() < end + 4 + length {
                    continue;
                }
                assert_eq!(
                    header_text.lines().next().unwrap(),
                    format!("POST {path} HTTP/1.1")
                );
                for (name, value) in &expected_headers {
                    assert_eq!(headers[name.as_str()], value.to_str().unwrap(), "{name}");
                }
                let body: Value =
                    serde_json::from_slice(&bytes[end + 4..end + 4 + length]).unwrap();
                assert_eq!(body, expected_body);
                break;
            }
        }
        let status = if redirect { "302 Found" } else { "200 OK" };
        let extra = if redirect {
            format!("Location: {location}\r\n")
        } else {
            String::new()
        };
        write!(socket, "HTTP/1.1 {status}\r\nContent-Type: text/event-stream\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{response}", response.len()).unwrap();
    });
    (base, server)
}

#[tokio::test]
async fn native_protocols_round_trip_through_local_http_with_images_tools_and_usage() {
    let completion = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":2}}\n\ndata: [DONE]\n\n";
    let anthropic = "event: message_start\ndata: {\"message\":{\"usage\":{\"input_tokens\":10}}}\n\nevent: content_block_delta\ndata: {\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\nevent: message_delta\ndata: {\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":2}}\n\n";
    let google = "data: {\"candidates\":[{\"content\":{\"parts\":[{\"thought\":true,\"text\":\"hidden\"},{\"text\":\"Hello\"}]}}],\"usageMetadata\":{\"promptTokenCount\":10,\"candidatesTokenCount\":2}}\n\n";
    let responses = "event: response.completed\ndata: {\"response\":{\"status\":\"completed\",\"output_text\":\"Hello\",\"usage\":{\"input_tokens\":10,\"output_tokens\":2}}}\n\n";
    let bedrock = "{\"output\":{\"message\":{\"content\":[{\"text\":\"Hello\"}]}},\"usage\":{\"inputTokens\":10,\"outputTokens\":2}}";
    for (provider, wire) in [
        ("deepseek", completion),
        ("minimax", anthropic),
        ("google", google),
        ("google-vertex", google),
        ("mistral", completion),
        ("openai", responses),
        ("amazon-bedrock", bedrock),
    ] {
        let definition = provider_definition(provider).unwrap();
        let credential = auth();
        let mut request = request(provider, definition.default_model, true);
        request.messages[0].content = AgentMessageContent::Parts(vec![
            crate::AgentMessagePart::Text {
                text: "Hello".into(),
            },
            crate::AgentMessagePart::ImageUrl {
                image_url: crate::ImageUrl {
                    url: "data:image/png;base64,aGVsbG8=".into(),
                },
            },
        ]);
        request.tools = crate::agent_tool_definitions(AgentRequestType::Implementation);
        request
            .extra
            .insert("max_completion_tokens".into(), json!(4096));
        let protocol = protocol_for_model(&definition, &request.model);
        let base = provider_base_url(&definition, &credential).unwrap();
        let (url, headers, body) = build_provider_request(
            &definition,
            protocol,
            &base,
            &request.model,
            &request,
            &credential,
        )
        .unwrap();
        match protocol {
            AgentProviderProtocol::OpenAiCompletions
            | AgentProviderProtocol::MistralConversations => {
                assert_eq!(
                    body["messages"][0]["content"][1]["image_url"]["url"],
                    "data:image/png;base64,aGVsbG8="
                );
                assert_eq!(body["tools"][0]["type"], "function");
            }
            AgentProviderProtocol::AnthropicMessages => {
                assert_eq!(
                    body["messages"][0]["content"][1]["source"],
                    json!({"type":"base64","media_type":"image/png","data":"aGVsbG8="})
                );
                assert!(body["tools"][0]["input_schema"].is_object());
            }
            AgentProviderProtocol::GoogleGenerativeAi | AgentProviderProtocol::GoogleVertex => {
                assert_eq!(
                    body["contents"][0]["parts"][1]["inlineData"],
                    json!({"mimeType":"image/png","data":"aGVsbG8="})
                );
                assert!(body["tools"][0]["functionDeclarations"].is_array());
            }
            AgentProviderProtocol::OpenAiResponses => {
                assert_eq!(
                    body["input"][0]["content"][1],
                    json!({"type":"input_image","image_url":"data:image/png;base64,aGVsbG8="})
                );
                assert_eq!(body["tools"][0]["type"], "function");
            }
            AgentProviderProtocol::BedrockConverse => {
                assert_eq!(
                    body["messages"][0]["content"][1]["image"],
                    json!({"format":"png","source":{"bytes":"aGVsbG8="}})
                );
                assert!(body["toolConfig"]["tools"][0]["toolSpec"].is_object());
            }
            AgentProviderProtocol::PiMessages => unreachable!(),
        }
        let parsed_url = reqwest::Url::parse(&url).unwrap();
        let path = format!(
            "{}{}",
            parsed_url.path(),
            parsed_url
                .query()
                .map(|query| format!("?{query}"))
                .unwrap_or_default()
        );
        let (local, server) = provider_http_fixture(
            wire.into(),
            path.clone(),
            headers.clone(),
            body.clone(),
            false,
        );
        let text = streaming::send_observed(
            &format!("{local}{path}"),
            &headers,
            body,
            &credential,
            &request,
            protocol,
            &mut |_| Ok(()),
        )
        .await
        .unwrap();
        server.join().unwrap();
        let payload = if protocol == AgentProviderProtocol::BedrockConverse {
            serde_json::from_str(&text).unwrap()
        } else {
            parse_stream_payload(protocol, &text).unwrap()
        };
        assert_eq!(
            crate::agent_response_text(&payload).unwrap(),
            "Hello",
            "{provider}"
        );
        let usage = crate::agent_response_usage(provider, &request.model, &payload).unwrap();
        assert_eq!(usage.input, 10, "{provider}");
        assert_eq!(usage.output, 2, "{provider}");
    }
}

#[tokio::test]
async fn provider_redirects_are_not_followed_and_errors_redact_credentials() {
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_static("test-secret"));
    let body = json!({"prompt":"hello"});
    let (url, server) = provider_http_fixture(
        "test-secret".into(),
        "/".into(),
        headers.clone(),
        body.clone(),
        true,
    );
    let mut credential = auth();
    credential.secret = Some("test-secret".into());
    let error = streaming::send_observed(
        &url,
        &headers,
        body,
        &credential,
        &request("openai", "gpt-5.5", false),
        AgentProviderProtocol::OpenAiResponses,
        &mut |_| Ok(()),
    )
    .await
    .unwrap_err();
    server.join().unwrap();
    assert!(error.to_string().contains("302"));
    assert!(!error.to_string().contains("test-secret"));
}
