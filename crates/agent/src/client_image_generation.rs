pub const MAX_GENERATED_IMAGE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedImage {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub provider: String,
    pub model: String,
    pub prompt: String,
}

pub fn build_openai_image_request(model: &str, prompt: &str) -> AgentResult<Value> {
    if model != "gpt-image-1" || prompt.trim().is_empty() || prompt.len() > 4096 {
        return Err(AgentError::new(
            "OpenAI image generation requires gpt-image-1 and a bounded prompt",
        ));
    }
    Ok(json!({"model": model, "prompt": prompt, "n": 1, "response_format": "b64_json"}))
}

pub fn parse_openai_image_response(
    payload: &Value,
    model: &str,
    prompt: &str,
) -> AgentResult<GeneratedImage> {
    let data = payload
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| AgentError::new("OpenAI Images response has no data array"))?;
    if data.len() != 1 {
        return Err(AgentError::new(
            "OpenAI Images response must contain exactly one image",
        ));
    }
    let item = data[0]
        .as_object()
        .ok_or_else(|| AgentError::new("OpenAI Images output is malformed"))?;
    let encoded = item.get("b64_json").and_then(Value::as_str).ok_or_else(|| AgentError::new("OpenAI Images response must contain bounded base64 image bytes; URLs are unsupported"))?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| AgentError::new("OpenAI Images output is not valid base64"))?;
    if bytes.is_empty() || bytes.len() > MAX_GENERATED_IMAGE_BYTES {
        return Err(AgentError::new("generated image is empty or exceeds 8 MiB"));
    }
    let mime_type = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        "image/jpeg"
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        return Err(AgentError::new(
            "generated image must be PNG, JPEG, or WebP",
        ));
    };
    Ok(GeneratedImage {
        bytes,
        mime_type: mime_type.into(),
        provider: "openai".into(),
        model: model.into(),
        prompt: prompt.into(),
    })
}

pub async fn send_openai_image_generation(
    auth: &ResolvedProviderAuth,
    model: &str,
    prompt: &str,
) -> AgentResult<GeneratedImage> {
    let definition = provider_definition("openai")
        .ok_or_else(|| AgentError::new("OpenAI provider is unavailable"))?;
    let base = provider_base_url(&definition, auth)?;
    if auth.kind != AgentAuthKind::ApiKey {
        return Err(AgentError::new(
            "OpenAI Images requires an API key credential",
        ));
    }
    let secret = auth
        .secret
        .as_deref()
        .ok_or_else(|| AgentError::new("OpenAI requires an API key"))?;
    let response = reqwest::Client::new()
        .post(format!("{}/images/generations", base.trim_end_matches('/')))
        .bearer_auth(secret)
        .json(&build_openai_image_request(model, prompt)?)
        .send()
        .await
        .map_err(|error| AgentError::new(error.to_string()))?;
    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| AgentError::new(error.to_string()))?;
    if bytes.len() > 12 * 1024 * 1024 {
        return Err(AgentError::new(
            "OpenAI Images response exceeds bounded response size",
        ));
    }
    let payload: Value = serde_json::from_slice(&bytes)
        .map_err(|_| AgentError::new("OpenAI Images response is invalid JSON"))?;
    if !status.is_success() {
        return Err(AgentError::new(format!(
            "OpenAI Images request failed with {}",
            status
        )));
    }
    parse_openai_image_response(&payload, model, prompt)
}

pub async fn send_native_agent_request_with_registry(
    request: &AgentRequest,
    auth: &ResolvedProviderAuth,
    registry: &ProviderRegistry,
) -> AgentResult<AgentServerResponse> {
    registry.validate()?;
    let provider_id = request.provider.as_deref().ok_or_else(|| AgentError::new("native agent requests require a provider"))?;
    let definition = registry.definition(provider_id).ok_or_else(|| AgentError::new(format!("unknown dynamic agent provider `{provider_id}`")))?;
    if !registry.contains_model(provider_id, &request.model) { return Err(AgentError::new("model is not declared by the dynamic provider")); }
    let protocol = definition.protocol;
    let base = definition.base_url.trim_end_matches('/');
    let url = match protocol {
        AgentProviderProtocol::OpenAiCompletions | AgentProviderProtocol::MistralConversations => format!("{base}/chat/completions"),
        AgentProviderProtocol::OpenAiResponses => format!("{base}/responses"),
        AgentProviderProtocol::AnthropicMessages => format!("{}/v1/messages", base.strip_suffix("/v1").unwrap_or(base)),
        AgentProviderProtocol::GoogleGenerativeAi => format!("{base}/models/{}:generateContent", percent_encode_query(&request.model)),
        AgentProviderProtocol::PiMessages => format!("{base}/messages"),
        AgentProviderProtocol::GoogleVertex | AgentProviderProtocol::BedrockConverse => return Err(AgentError::new("dynamic provider protocol is not available for this route")),
    };
    let secret = auth.secret.as_deref().ok_or_else(|| AgentError::new("dynamic provider requires an explicit request credential or declared environment credential"))?;
    let mut headers = HeaderMap::new(); headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json")); headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    match protocol { AgentProviderProtocol::AnthropicMessages => { insert_header(&mut headers, "x-api-key", secret.into())?; insert_header(&mut headers, "anthropic-version", "2023-06-01".into())?; }, AgentProviderProtocol::GoogleGenerativeAi => insert_header(&mut headers, "x-goog-api-key", secret.into())?, _ => insert_header(&mut headers, AUTHORIZATION.as_str(), format!("Bearer {secret}"))? }
    let body = match protocol { AgentProviderProtocol::OpenAiCompletions | AgentProviderProtocol::MistralConversations => openai_completions_body(&request.model, request, false)?, AgentProviderProtocol::OpenAiResponses => openai_responses_body(&request.model, request, false)?, AgentProviderProtocol::AnthropicMessages => anthropic_body(&request.model, request)?, AgentProviderProtocol::GoogleGenerativeAi => google_body(request)?, AgentProviderProtocol::PiMessages => pi_messages_body(&request.model, request)?, _ => unreachable!() };
    let response_body = streaming::send_observed(&url, &headers, body, auth, request, protocol, &mut |_| Ok(())).await?;
    let payload = if request.stream && !response_body.trim_start().starts_with('{') { parse_stream_payload(protocol, &response_body)? } else { serde_json::from_str(&response_body).map_err(|_| AgentError::new("dynamic provider returned invalid JSON"))? };
    Ok(AgentServerResponse { request_id: request.request_id.clone(), request_type: request.request_type, model: request.model.clone(), payload })
}

pub async fn send_native_agent_request(
    request: &AgentRequest,
    auth: &ResolvedProviderAuth,
) -> AgentResult<AgentServerResponse> {
    send_native_agent_request_observed(request, auth, &mut |_| Ok(())).await
}

pub async fn send_native_agent_request_observed(
    request: &AgentRequest,
    auth: &ResolvedProviderAuth,
    progress: &mut impl FnMut(&NativeRequestEvent) -> AgentResult<()>,
) -> AgentResult<AgentServerResponse> {
    let provider_id = request
        .provider
        .as_deref()
        .ok_or_else(|| AgentError::new("native agent requests require a provider"))?;
    let definition = provider_definition(provider_id)
        .ok_or_else(|| AgentError::new(format!("unknown agent provider `{provider_id}`")))?;
    let protocol = protocol_for_model(&definition, &request.model);
    let base_url = provider_base_url(&definition, auth)?;
    let model = normalize_model_id(provider_id, &request.model).to_string();
    let mut native_request = request.clone();
    native_request.stream = request.stream || provider_id == "openai-codex";
    let (url, headers, body) = build_provider_request(
        &definition,
        protocol,
        &base_url,
        &model,
        &native_request,
        auth,
    )?;
    let response_body = streaming::send_observed(
        &url,
        &headers,
        body,
        auth,
        &native_request,
        protocol,
        progress,
    )
    .await?;
    let payload = if native_request.stream
        && protocol != AgentProviderProtocol::BedrockConverse
        && !response_body.trim_start().starts_with('{')
    {
        parse_stream_payload(protocol, &response_body)?
    } else {
        serde_json::from_str(&response_body).map_err(|_error| {
            AgentError::new(format!(
                "provider `{provider_id}` returned invalid JSON: {}",
                redact_secret(&response_body, auth.secret.as_deref())
            ))
        })?
    };

    Ok(AgentServerResponse {
        request_id: request.request_id.clone(),
        request_type: request.request_type,
        model: request.model.clone(),
        payload,
    })
}

