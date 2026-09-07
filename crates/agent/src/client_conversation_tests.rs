#[test]
fn failed_streams_cannot_be_committed_as_partial_successes() {
    for protocol in [
        AgentProviderProtocol::OpenAiResponses,
        AgentProviderProtocol::OpenAiCompletions,
        AgentProviderProtocol::AnthropicMessages,
        AgentProviderProtocol::GoogleGenerativeAi,
        AgentProviderProtocol::GoogleVertex,
        AgentProviderProtocol::MistralConversations,
        AgentProviderProtocol::PiMessages,
    ] {
        for failure in [
            "event: error\ndata: {\"error\":{\"message\":\"secret-test-value\"}}\n\n",
            "event: response.failed\ndata: {\"response\":{\"status\":\"failed\"}}\n\n",
        ] {
            let stream = format!(
                "data: {{\"choices\":[{{\"delta\":{{\"content\":\"partial\"}}}}]}}\n\n{failure}"
            );
            let error = parse_stream_payload(protocol, &stream).unwrap_err();
            assert!(error.to_string().contains("unsuccessful"));
            assert!(!error.to_string().contains("secret-test-value"));
        }
    }
}

#[test]
fn google_stream_keeps_thoughts_out_of_visible_conversation_text() {
    let payload = aggregate_google(vec![
        (None, json!({"candidates":[{"content":{"parts":[{"thought":true,"text":"hidden"}]}}]})),
        (None, json!({"candidates":[{"content":{"parts":[{"text":"visible"},{"functionCall":{"name":"not_executed","args":{}}}]}}],"usageMetadata":{"promptTokenCount":10,"candidatesTokenCount":3}})),
    ]).unwrap();
    assert_eq!(crate::agent_response_text(&payload).unwrap(), "visible");
    assert_eq!(payload["usageMetadata"]["promptTokenCount"], 10);
    assert_eq!(
        payload["candidates"][0]["content"]["parts"][0],
        json!({"thought":true,"text":"hidden"})
    );
}

#[test]
fn conversation_history_lowers_to_every_native_protocol() {
    let root = tempfile::tempdir().unwrap();
    let mut conversation = crate::AgentConversation::default();
    let first = conversation
        .prepare(
            root.path(),
            "Me llamo Ana",
            crate::AgentPrepareOptions::default(),
        )
        .unwrap()
        .request;
    conversation
        .record_response(
            &first,
            &crate::AgentServerResponse {
                request_id: first.request_id.clone(),
                request_type: first.request_type,
                model: first.model.clone(),
                payload: json!({"output_text":"Hola Ana"}),
            },
        )
        .unwrap();
    let request = conversation
        .prepare(
            root.path(),
            "¿Mi nombre?",
            crate::AgentPrepareOptions::default(),
        )
        .unwrap()
        .request;
    let with_system = serde_json::to_value(&request.messages).unwrap();
    let turns = json!([
        {"role":"user","content":"Me llamo Ana"},
        {"role":"assistant","content":"Hola Ana"},
        {"role":"user","content":"¿Mi nombre?"}
    ]);
    for body in [
        openai_completions_body("model", &request, false).unwrap(),
        openai_completions_body("model", &request, true).unwrap(),
        mistral_body("model", &request).unwrap(),
    ] {
        assert_eq!(body["messages"], with_system);
        assert!(body.get("response_format").is_none());
        assert!(body.get("tools").is_none());
    }
    for codex in [false, true] {
        let body = openai_responses_body("model", &request, codex).unwrap();
        assert_eq!(body["input"], turns);
        assert!(
            body["instructions"]
                .as_str()
                .unwrap()
                .contains("user's language")
        );
        assert!(body.pointer("/text/format").is_none());
        assert!(body.get("tools").is_none());
        assert_eq!(body["prompt_cache_key"], request.extra["session_id"]);
    }
    let anthropic = anthropic_body("model", &request).unwrap();
    assert_eq!(anthropic["messages"], turns);
    let google = google_body(&request).unwrap();
    assert_eq!(
        google["contents"],
        json!([
            {"role":"user","parts":[{"text":"Me llamo Ana"}]},
            {"role":"model","parts":[{"text":"Hola Ana"}]},
            {"role":"user","parts":[{"text":"¿Mi nombre?"}]}
        ])
    );
    let bedrock = bedrock_body(&request).unwrap();
    assert_eq!(
        bedrock["messages"],
        json!([
            {"role":"user","content":[{"text":"Me llamo Ana"}]},
            {"role":"assistant","content":[{"text":"Hola Ana"}]},
            {"role":"user","content":[{"text":"¿Mi nombre?"}]}
        ])
    );
    let pi = pi_messages_body("model", &request).unwrap();
    assert_eq!(pi["context"]["messages"], with_system);
    assert_eq!(pi["context"]["tools"], json!([]));
}
