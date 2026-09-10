fn build_provider_request(
    definition: &AgentProviderDefinition,
    protocol: AgentProviderProtocol,
    base_url: &str,
    model: &str,
    request: &AgentRequest,
    auth: &ResolvedProviderAuth,
) -> AgentResult<(String, HeaderMap, Value)> {
    crate::validate_agent_model(definition.id, model)?;
    let url = provider_url(definition, protocol, base_url, model, request.stream, auth)?;
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "accept",
        HeaderValue::from_static(
            if request.stream && protocol != AgentProviderProtocol::BedrockConverse {
                "text/event-stream"
            } else {
                "application/json"
            },
        ),
    );
    headers.insert("user-agent", HeaderValue::from_static("dowe-agent/1.0"));
    apply_auth_headers(definition, protocol, &mut headers, auth)?;
    if definition.id == "github-copilot" {
        for (name, value) in [
            ("user-agent", "GitHubCopilotChat/0.35.0"),
            ("editor-version", "vscode/1.107.0"),
            ("editor-plugin-version", "copilot-chat/0.35.0"),
            ("copilot-integration-id", "vscode-chat"),
            ("openai-intent", "conversation-edits"),
        ] {
            insert_header(&mut headers, name, value.into())?;
        }
        insert_header(
            &mut headers,
            "x-initiator",
            if request
                .extra
                .get("dowe_harness_turns")
                .and_then(Value::as_array)
                .and_then(|turns| turns.last())
                .map_or_else(
                    || {
                        request
                            .messages
                            .last()
                            .is_some_and(|message| message.role != "user")
                    },
                    |turn| turn.pointer("/message/role").and_then(Value::as_str) != Some("user"),
                )
            {
                "agent"
            } else {
                "user"
            }
            .into(),
        )?;
        if request.messages.iter().any(|message| matches!(&message.content, AgentMessageContent::Parts(parts) if parts.iter().any(|part| matches!(part, AgentMessagePart::ImageUrl { .. }))))
            || request.extra.get("dowe_harness_turns").and_then(Value::as_array).is_some_and(|turns| turns.iter().any(|turn| turn.pointer("/message/content").and_then(Value::as_array).is_some_and(|parts| parts.iter().any(|part| part["type"] == "image_url"))))
        {
            insert_header(&mut headers, "copilot-vision-request", "true".into())?;
        }
    }
    if definition.id == "openai-codex" {
        headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
        if let Some(session_id) = request.extra.get("session_id").and_then(Value::as_str) {
            insert_header(&mut headers, "session-id", session_id.to_string())?;
            insert_header(&mut headers, "x-client-request-id", session_id.to_string())?;
        }
    }
    let mut body = match protocol {
        AgentProviderProtocol::OpenAiCompletions => {
            openai_completions_body(model, request, definition.id == "openrouter")?
        }
        AgentProviderProtocol::OpenAiResponses => {
            openai_responses_body(model, request, definition.id == "openai-codex")?
        }
        AgentProviderProtocol::AnthropicMessages => anthropic_body(model, request)?,
        AgentProviderProtocol::GoogleGenerativeAi | AgentProviderProtocol::GoogleVertex => {
            google_body(request)?
        }
        AgentProviderProtocol::MistralConversations => mistral_body(model, request)?,
        AgentProviderProtocol::BedrockConverse => bedrock_body(request)?,
        AgentProviderProtocol::PiMessages => pi_messages_body(model, request)?,
    };
    crate::native_harness::apply_harness_turns(protocol, request, &mut body)?;
    if let Some(level) = request.extra.get("thinkingLevel") {
        let level = serde_json::from_value(level.clone())
            .map_err(|_| AgentError::new("invalid thinking level"))?;
        crate::inference::apply_thinking(definition.id, model, level, &mut body)?;
    }
    Ok((url, headers, body))
}

fn provider_url(
    definition: &AgentProviderDefinition,
    protocol: AgentProviderProtocol,
    base_url: &str,
    model: &str,
    stream: bool,
    auth: &ResolvedProviderAuth,
) -> AgentResult<String> {
    let base = base_url.trim_end_matches('/');
    let url = match protocol {
        AgentProviderProtocol::OpenAiCompletions | AgentProviderProtocol::MistralConversations => {
            if definition.id == "cloudflare-ai-gateway" {
                format!("{base}/compat/chat/completions")
            } else if matches!(
                definition.id,
                "fireworks" | "opencode" | "opencode-go" | "vercel-ai-gateway"
            ) {
                format!(
                    "{}/v1/chat/completions",
                    base.strip_suffix("/v1").unwrap_or(base)
                )
            } else {
                format!("{base}/chat/completions")
            }
        }
        AgentProviderProtocol::OpenAiResponses => {
            if definition.id == "openai-codex" {
                format!("{base}/codex/responses")
            } else if definition.id == "cloudflare-ai-gateway" {
                format!("{base}/openai/responses")
            } else if matches!(definition.id, "opencode" | "opencode-go") {
                format!("{}/v1/responses", base.strip_suffix("/v1").unwrap_or(base))
            } else {
                format!("{base}/responses")
            }
        }
        AgentProviderProtocol::AnthropicMessages => {
            if definition.id == "cloudflare-ai-gateway" {
                format!("{base}/anthropic/v1/messages")
            } else {
                format!("{}/v1/messages", base.strip_suffix("/v1").unwrap_or(base))
            }
        }
        AgentProviderProtocol::GoogleGenerativeAi => {
            let base = if definition.id == "opencode" {
                format!("{}/v1", base.strip_suffix("/v1").unwrap_or(base))
            } else {
                base.to_string()
            };
            let action = if stream {
                "streamGenerateContent?alt=sse"
            } else {
                "generateContent"
            };
            format!("{base}/models/{}:{action}", percent_encode_query(model))
        }
        AgentProviderProtocol::GoogleVertex => {
            let action = if stream {
                "streamGenerateContent?alt=sse"
            } else {
                "generateContent"
            };
            if auth.kind == AgentAuthKind::ApiKey && auth.secret.is_some() {
                return Ok(format!(
                    "{base}/v1/publishers/google/models/{}:{action}",
                    percent_encode_query(model)
                ));
            }
            let location = auth
                .env
                .get("GOOGLE_CLOUD_LOCATION")
                .cloned()
                .or_else(|| std::env::var("GOOGLE_CLOUD_LOCATION").ok())
                .ok_or_else(|| {
                    AgentError::new("Google Vertex AI requires GOOGLE_CLOUD_LOCATION")
                })?;
            let project = auth
                .env
                .get("GOOGLE_CLOUD_PROJECT")
                .cloned()
                .or_else(|| std::env::var("GOOGLE_CLOUD_PROJECT").ok())
                .ok_or_else(|| AgentError::new("Google Vertex AI requires GOOGLE_CLOUD_PROJECT"))?;
            if location.is_empty()
                || !location
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
            {
                return Err(AgentError::new("invalid Google Vertex location"));
            }
            let host = if location == "global" {
                "aiplatform.googleapis.com".to_string()
            } else {
                format!("{location}-aiplatform.googleapis.com")
            };
            format!(
                "https://{host}/v1/projects/{}/locations/{location}/publishers/google/models/{}:{action}",
                percent_encode_query(&project),
                percent_encode_query(model)
            )
        }
        AgentProviderProtocol::BedrockConverse => {
            let encoded_model = percent_encode_query(model);
            format!("{base}/model/{encoded_model}/converse")
        }
        AgentProviderProtocol::PiMessages => format!("{base}/messages"),
    };
    Ok(url)
}

fn apply_auth_headers(
    definition: &AgentProviderDefinition,
    protocol: AgentProviderProtocol,
    headers: &mut HeaderMap,
    auth: &ResolvedProviderAuth,
) -> AgentResult<()> {
    let secret = auth.secret.as_deref();
    if definition.id == "openai-codex" {
        let token = secret
            .ok_or_else(|| AgentError::new("OpenAI Codex requires a ChatGPT account credential"))?;
        let account_id = openai_codex_account_id(token)?;
        insert_header(headers, AUTHORIZATION.as_str(), format!("Bearer {token}"))?;
        insert_header(headers, "chatgpt-account-id", account_id)?;
        insert_header(headers, "originator", "pi".to_string())?;
        insert_header(headers, "OpenAI-Beta", "responses=experimental".to_string())?;
        return Ok(());
    }
    if secret.is_none() && auth.kind == AgentAuthKind::Ambient {
        if definition.id == "amazon-bedrock" {
            return Err(AgentError::new(
                "Amazon Bedrock ambient AWS credentials require the AWS SDK credential chain; configure AWS_BEARER_TOKEN_BEDROCK or use an API key",
            ));
        }
        if definition.id == "google-vertex" {
            return Err(AgentError::new(
                "Google Vertex ambient credentials require an access-token exchange; set GOOGLE_CLOUD_API_KEY for direct Dowe Agent requests",
            ));
        }
    }
    let secret = secret.ok_or_else(|| {
        AgentError::new(format!(
            "provider `{}` has no usable request credential",
            definition.id
        ))
    })?;
    if definition.id == "cloudflare-ai-gateway" {
        insert_header(headers, "cf-aig-authorization", format!("Bearer {secret}"))?;
    }
    match protocol {
        AgentProviderProtocol::AnthropicMessages => {
            if definition.id == "github-copilot"
                || definition.id == "kimi-coding"
                || auth.kind == AgentAuthKind::OAuth
                || (definition.id == "anthropic" && auth.source == "ANTHROPIC_AUTH_TOKEN")
            {
                insert_header(headers, AUTHORIZATION.as_str(), format!("Bearer {secret}"))?;
            } else if definition.id != "cloudflare-ai-gateway" {
                insert_header(headers, "x-api-key", secret.to_string())?;
            }
            insert_header(headers, "anthropic-version", "2023-06-01".to_string())?;
        }
        AgentProviderProtocol::GoogleGenerativeAi => {
            insert_header(headers, "x-goog-api-key", secret.to_string())?;
        }
        AgentProviderProtocol::GoogleVertex => {
            if auth.kind == AgentAuthKind::ApiKey {
                insert_header(headers, "x-goog-api-key", secret.to_string())?;
            } else {
                insert_header(headers, AUTHORIZATION.as_str(), format!("Bearer {secret}"))?;
            }
        }
        AgentProviderProtocol::BedrockConverse | AgentProviderProtocol::PiMessages => {
            insert_header(headers, AUTHORIZATION.as_str(), format!("Bearer {secret}"))?;
        }
        AgentProviderProtocol::OpenAiCompletions | AgentProviderProtocol::OpenAiResponses => {
            if definition.id == "azure-openai-responses" {
                insert_header(headers, "api-key", secret.to_string())?;
            } else if definition.id == "cloudflare-ai-gateway" {
                insert_header(headers, "cf-aig-authorization", format!("Bearer {secret}"))?;
            } else {
                insert_header(headers, AUTHORIZATION.as_str(), format!("Bearer {secret}"))?;
            }
        }
        AgentProviderProtocol::MistralConversations => {
            insert_header(headers, AUTHORIZATION.as_str(), format!("Bearer {secret}"))?;
        }
    }
    for (name, value) in &auth.env {
        if name.starts_with("DOWE_AGENT_HEADER_") {
            let header_name = name.trim_start_matches("DOWE_AGENT_HEADER_");
            insert_header(headers, header_name, value.clone())?;
        }
    }
    Ok(())
}

