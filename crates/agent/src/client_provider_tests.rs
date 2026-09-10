#[test]
fn endpoint_configuration_is_resolved_without_requiring_vertex_project_fields() {
    if std::env::var_os("DOWE_TEST_PROVIDER_CONFIG").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "client::tests::endpoint_configuration_is_resolved_without_requiring_vertex_project_fields"])
            .env_clear().env("DOWE_TEST_PROVIDER_CONFIG", "1").env("AWS_REGION", "eu-west-1").env("RUST_MIN_STACK", "8388608")
            .output().unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let home = tempfile::tempdir().unwrap();
    let store = crate::AgentAuthStore::new(home.path().join("auth.json"));
    for (provider, base) in [
        (
            "amazon-bedrock",
            "https://bedrock-runtime.eu-west-1.amazonaws.com",
        ),
        ("google-vertex", "https://aiplatform.googleapis.com"),
    ] {
        let definition = provider_definition(provider).unwrap();
        let credential =
            crate::resolve_provider_auth(&definition, &store, Some("synthetic-key"), None)
                .unwrap()
                .unwrap();
        assert_eq!(provider_base_url(&definition, &credential).unwrap(), base);
    }
}

#[test]
fn copilot_headers_describe_images_and_request_initiator() {
    let definition = provider_definition("github-copilot").unwrap();
    let mut request = request("github-copilot", "gpt-5.5", true);
    request.messages[0].content =
        AgentMessageContent::Parts(vec![crate::AgentMessagePart::ImageUrl {
            image_url: crate::ImageUrl {
                url: "data:image/png;base64,aGVsbG8=".into(),
            },
        }]);
    let (_, headers, _) = build_provider_request(
        &definition,
        AgentProviderProtocol::OpenAiResponses,
        definition.base_url.unwrap(),
        &request.model,
        &request,
        &auth(),
    )
    .unwrap();
    assert_eq!(headers["authorization"], "Bearer test-secret");
    assert_eq!(headers["copilot-integration-id"], "vscode-chat");
    assert_eq!(headers["editor-version"], "vscode/1.107.0");
    assert_eq!(headers["x-initiator"], "user");
    assert_eq!(headers["copilot-vision-request"], "true");
}

#[test]
fn completion_profiles_use_supported_limits_cache_and_usage_fields() {
    for provider in [
        "deepseek",
        "nvidia",
        "zai",
        "zai-coding-cn",
        "opencode",
        "opencode-go",
        "cloudflare-ai-gateway",
        "groq",
        "openrouter",
    ] {
        let mut request = request(provider, "test-model", true);
        request
            .extra
            .insert("max_completion_tokens".into(), json!(4096));
        request
            .extra
            .insert("session_id".into(), json!("conversation-1"));
        let body =
            openai_completions_body("test-model", &request, provider == "openrouter").unwrap();
        let modern_limit = provider == "groq" || provider == "openrouter";
        assert_eq!(
            body[if modern_limit {
                "max_completion_tokens"
            } else {
                "max_tokens"
            }],
            4096,
            "{provider}: {body}"
        );
        assert!(
            body.get(if modern_limit {
                "max_tokens"
            } else {
                "max_completion_tokens"
            })
            .is_none(),
            "{provider}"
        );
        assert!(body.get("prompt_cache_key").is_none(), "{provider}");
        assert_eq!(body["stream_options"]["include_usage"], true, "{provider}");
    }
}

#[test]
fn harness_requests_disable_openrouter_fallbacks_even_for_auxiliary_stages() {
    let definition = provider_definition("openrouter").unwrap();
    let mut request = request("openrouter", "deepseek/deepseek-v4-flash", false);
    request
        .extra
        .insert("dowe_harness_turns".into(), json!([]));
    let (_, _, body) = build_provider_request(
        &definition,
        AgentProviderProtocol::OpenAiCompletions,
        definition.base_url.unwrap(),
        &request.model,
        &request,
        &auth(),
    )
    .unwrap();
    assert_eq!(body["provider"]["allow_fallbacks"], false);
}

#[test]
fn native_routes_match_pi_protocols_instead_of_broad_name_guesses() {
    use AgentProviderProtocol::*;
    for (provider, model, protocol, url) in [
        (
            "cloudflare-ai-gateway",
            "gpt-4o",
            OpenAiResponses,
            "https://gateway.ai.cloudflare.com/v1/account/gateway/openai/responses",
        ),
        (
            "cloudflare-ai-gateway",
            "workers-ai/@cf/meta/llama",
            OpenAiCompletions,
            "https://gateway.ai.cloudflare.com/v1/account/gateway/compat/chat/completions",
        ),
        (
            "cloudflare-ai-gateway",
            "claude-sonnet-4.6",
            AnthropicMessages,
            "https://gateway.ai.cloudflare.com/v1/account/gateway/anthropic/v1/messages",
        ),
        (
            "fireworks",
            "accounts/fireworks/models/glm-5p3",
            OpenAiCompletions,
            "https://api.fireworks.ai/inference/v1/chat/completions",
        ),
        (
            "fireworks",
            "accounts/fireworks/models/kimi-k3",
            OpenAiCompletions,
            "https://api.fireworks.ai/inference/v1/chat/completions",
        ),
        (
            "fireworks",
            "accounts/fireworks/models/minimax-m3",
            AnthropicMessages,
            "https://api.fireworks.ai/inference/v1/messages",
        ),
        (
            "opencode",
            "minimax-m3",
            OpenAiCompletions,
            "https://opencode.ai/zen/v1/chat/completions",
        ),
        (
            "opencode",
            "gpt-5.5",
            OpenAiResponses,
            "https://opencode.ai/zen/v1/responses",
        ),
        (
            "opencode",
            "opencode/gpt-5.5",
            OpenAiResponses,
            "https://opencode.ai/zen/v1/responses",
        ),
        (
            "cloudflare-ai-gateway",
            "cloudflare-ai-gateway/gpt-4o",
            OpenAiResponses,
            "https://gateway.ai.cloudflare.com/v1/account/gateway/openai/responses",
        ),
        (
            "github-copilot",
            "github-copilot/gpt-5.5",
            OpenAiResponses,
            "https://api.individual.githubcopilot.com/responses",
        ),
        (
            "opencode",
            "gemini-3.5-flash",
            GoogleGenerativeAi,
            "https://opencode.ai/zen/v1/models/gemini-3.5-flash:streamGenerateContent?alt=sse",
        ),
        (
            "opencode-go",
            "qwen3.7-plus",
            OpenAiCompletions,
            "https://opencode.ai/zen/go/v1/chat/completions",
        ),
        (
            "opencode-go",
            "qwen3.8-flash",
            AnthropicMessages,
            "https://opencode.ai/zen/go/v1/messages",
        ),
        (
            "opencode-go",
            "gpt-5.6-luna",
            OpenAiResponses,
            "https://opencode.ai/zen/go/v1/responses",
        ),
        (
            "github-copilot",
            "gpt-5.5",
            OpenAiResponses,
            "https://api.individual.githubcopilot.com/responses",
        ),
        (
            "github-copilot",
            "gemini-3.5-flash",
            OpenAiCompletions,
            "https://api.individual.githubcopilot.com/chat/completions",
        ),
        (
            "vercel-ai-gateway",
            "openai/gpt-5.5",
            AnthropicMessages,
            "https://ai-gateway.vercel.sh/v1/messages",
        ),
        (
            "openrouter",
            "anthropic/claude-sonnet-4.6",
            AnthropicMessages,
            "https://openrouter.ai/api/v1/messages",
        ),
        (
            "mistral",
            "mistral-large-latest",
            MistralConversations,
            "https://api.mistral.ai/v1/chat/completions",
        ),
    ] {
        let definition = provider_definition(provider).unwrap();
        assert_eq!(
            protocol_for_model(&definition, model),
            protocol,
            "{provider}/{model}"
        );
        let mut credential = auth();
        credential
            .env
            .insert("CLOUDFLARE_ACCOUNT_ID".into(), "account".into());
        credential
            .env
            .insert("CLOUDFLARE_GATEWAY_ID".into(), "gateway".into());
        let base = provider_base_url(&definition, &credential).unwrap();
        let request = request(provider, model, true);
        let (actual, headers, _) = build_provider_request(
            &definition,
            protocol,
            &base,
            normalize_model_id(provider, model),
            &request,
            &credential,
        )
        .unwrap();
        assert_eq!(actual, url, "{provider}/{model}");
        if provider == "cloudflare-ai-gateway" {
            assert_eq!(headers["cf-aig-authorization"], "Bearer test-secret");
        }
        assert!(!actual.contains("test-secret"));
    }
}

#[test]
fn google_keys_are_headers_and_vertex_api_keys_use_express() {
    for provider in ["google", "google-vertex"] {
        let definition = provider_definition(provider).unwrap();
        let credential = auth();
        let base = provider_base_url(&definition, &credential).unwrap();
        for stream in [false, true] {
            let request = request(provider, "gemini-2.5-flash", stream);
            let (url, headers, _) = build_provider_request(
                &definition,
                definition.protocol,
                &base,
                &request.model,
                &request,
                &credential,
            )
            .unwrap();
            assert!(!url.contains("test-secret"));
            assert_eq!(headers["x-goog-api-key"], "test-secret");
            assert!(!headers.contains_key("authorization"));
            assert_eq!(url.ends_with("?alt=sse"), stream);
            if provider == "google-vertex" {
                assert!(
                    url.starts_with(
                        "https://aiplatform.googleapis.com/v1/publishers/google/models/"
                    ),
                    "{url}"
                );
            }
        }
    }
}

#[test]
fn azure_host_and_global_vertex_routes_are_normalized() {
    let definition = provider_definition("azure-openai-responses").unwrap();
    let mut credential = auth();
    credential.env.insert(
        "AZURE_OPENAI_BASE_URL".into(),
        "https://resource.openai.azure.com/".into(),
    );
    assert_eq!(
        provider_base_url(&definition, &credential).unwrap(),
        "https://resource.openai.azure.com/openai/v1"
    );
    let vertex = provider_definition("google-vertex").unwrap();
    credential.kind = AgentAuthKind::OAuth;
    credential
        .env
        .insert("GOOGLE_CLOUD_LOCATION".into(), "global".into());
    credential
        .env
        .insert("GOOGLE_CLOUD_PROJECT".into(), "project-id".into());
    let base = provider_base_url(&vertex, &credential).unwrap();
    let url = provider_url(
        &vertex,
        vertex.protocol,
        &base,
        "gemini-2.5-flash",
        true,
        &credential,
    )
    .unwrap();
    assert_eq!(
        url,
        "https://aiplatform.googleapis.com/v1/projects/project-id/locations/global/publishers/google/models/gemini-2.5-flash:streamGenerateContent?alt=sse"
    );
    let request = request("google-vertex", "gemini-2.5-flash", true);
    let (_, headers, _) = build_provider_request(
        &vertex,
        vertex.protocol,
        &base,
        &request.model,
        &request,
        &credential,
    )
    .unwrap();
    assert_eq!(headers["authorization"], "Bearer test-secret");
    assert!(!headers.contains_key("x-goog-api-key"));
    credential
        .env
        .insert("GOOGLE_CLOUD_LOCATION".into(), "".into());
    assert!(
        provider_url(
            &vertex,
            vertex.protocol,
            &base,
            &request.model,
            true,
            &credential
        )
        .is_err()
    );
}

#[test]
fn bedrock_images_use_converse_json_bytes_and_encoded_model_ids() {
    let mut request = request(
        "amazon-bedrock",
        "arn:aws:bedrock:region:account:inference-profile/model",
        true,
    );
    request.messages[0].content =
        AgentMessageContent::Parts(vec![crate::AgentMessagePart::ImageUrl {
            image_url: crate::ImageUrl {
                url: "data:image/png;base64,aGVsbG8=".into(),
            },
        }]);
    let body = bedrock_body(&request).unwrap();
    assert_eq!(
        body["messages"][0]["content"][0],
        json!({"image":{"format":"png","source":{"bytes":"aGVsbG8="}}})
    );
    let definition = provider_definition("amazon-bedrock").unwrap();
    let url = provider_url(
        &definition,
        definition.protocol,
        "https://example.test",
        &request.model,
        true,
        &auth(),
    )
    .unwrap();
    assert_eq!(
        url,
        "https://example.test/model/arn%3Aaws%3Abedrock%3Aregion%3Aaccount%3Ainference-profile%2Fmodel/converse"
    );
}

#[test]
fn every_catalog_provider_builds_a_text_request_without_network_or_real_auth() {
    assert_eq!(crate::builtin_provider_ids().len(), 28);
    for provider in crate::builtin_provider_ids() {
        let definition = provider_definition(provider).unwrap();
        let mut credential = if *provider == "openai-codex" {
            codex_test_auth()
        } else {
            auth()
        };
        for (key, value) in [
            (
                "AZURE_OPENAI_BASE_URL",
                "https://resource.openai.azure.com/openai/v1",
            ),
            ("CLOUDFLARE_ACCOUNT_ID", "account"),
            ("CLOUDFLARE_GATEWAY_ID", "gateway"),
            ("GOOGLE_CLOUD_PROJECT", "project"),
            ("GOOGLE_CLOUD_LOCATION", "us-central1"),
        ] {
            credential.env.insert(key.into(), value.into());
        }
        let model = if definition.default_model.is_empty() {
            "explicit-test-model"
        } else {
            definition.default_model
        };
        let request = request(provider, model, true);
        let base = provider_base_url(&definition, &credential).unwrap();
        let protocol = protocol_for_model(&definition, model);
        let (url, headers, body) =
            build_provider_request(&definition, protocol, &base, model, &request, &credential)
                .unwrap();
        assert!(!url.contains("/v1/v1/"), "{provider}: {url}");
        assert!(!url.contains('{'), "{provider}: {url}");
        assert!(
            !url.contains(credential.secret.as_deref().unwrap()),
            "{provider}: credential in URL"
        );
        assert!(
            !body
                .to_string()
                .contains(credential.secret.as_deref().unwrap()),
            "{provider}: credential in body"
        );
        assert!(
            headers.contains_key("authorization")
                || headers.contains_key("x-api-key")
                || headers.contains_key("x-goog-api-key")
                || headers.contains_key("api-key")
                || headers.contains_key("cf-aig-authorization"),
            "{provider}"
        );
    }
}
