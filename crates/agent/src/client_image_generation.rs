pub const MAX_GENERATED_IMAGE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_IMAGE_PROMPT_BYTES: usize = 32_000;

const OPENAI_IMAGE_MODELS: &[&str] = &[
    "gpt-image-1",
    "gpt-image-1-mini",
    "gpt-image-1.5",
    "gpt-image-2",
    "gpt-image-2-2026-04-21",
    "gpt-image-2.5-sunburst",
    "gpt-image-2.5-sunburst-2026-09-08",
    "gpt-image-2.5-flare",
    "gpt-image-2.5-flare-2026-09-08",
    "chatgpt-image-latest",
];

const OPENAI_IMAGE_MODEL_CATALOG: &[(&str, &str)] = &[
    ("gpt-image-1", "GPT Image 1"),
    ("gpt-image-1-mini", "GPT Image 1 mini"),
    ("gpt-image-1.5", "GPT Image 1.5"),
    ("gpt-image-2", "GPT Image 2"),
    ("gpt-image-2-2026-04-21", "GPT Image 2 (2026-04-21)"),
    ("gpt-image-2.5-sunburst", "GPT Image 2.5 Sunburst"),
    (
        "gpt-image-2.5-sunburst-2026-09-08",
        "GPT Image 2.5 Sunburst (2026-09-08)",
    ),
    ("gpt-image-2.5-flare", "GPT Image 2.5 Flare"),
    (
        "gpt-image-2.5-flare-2026-09-08",
        "GPT Image 2.5 Flare (2026-09-08)",
    ),
    ("chatgpt-image-latest", "ChatGPT Image latest"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedImage {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub provider: String,
    pub model: String,
    pub prompt: String,
}

pub fn is_openai_image_model(model: &str) -> bool {
    OPENAI_IMAGE_MODELS.contains(&normalize_model_id("openai", model))
}

pub fn openai_image_models() -> &'static [(&'static str, &'static str)] {
    OPENAI_IMAGE_MODEL_CATALOG
}

pub fn build_openai_image_request(model: &str, prompt: &str) -> AgentResult<Value> {
    let model = normalize_model_id("openai", model);
    if !is_openai_image_model(model)
        || prompt.trim().is_empty()
        || prompt.len() > MAX_IMAGE_PROMPT_BYTES
    {
        return Err(AgentError::new(
            "OpenAI image generation requires a supported GPT Image model and a bounded prompt",
        ));
    }
    Ok(json!({
        "model": model,
        "prompt": prompt,
        "n": 1,
        "background": "auto",
        "quality": "auto",
        "size": "auto"
    }))
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
    let mime_type = image_mime_type(&bytes)
        .ok_or_else(|| AgentError::new("generated image must be PNG, JPEG, or WebP"))?;
    let model = normalize_model_id("openai", model);
    Ok(GeneratedImage {
        bytes,
        mime_type: mime_type.into(),
        provider: "openai".into(),
        model: model.into(),
        prompt: prompt.into(),
    })
}

pub fn image_mime_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        Some("image/jpeg")
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

pub async fn send_openai_image_generation(
    auth: &ResolvedProviderAuth,
    model: &str,
    prompt: &str,
) -> AgentResult<GeneratedImage> {
    send_openai_image_request(auth, model, prompt, &[]).await
}

pub async fn send_openai_image_edit(
    auth: &ResolvedProviderAuth,
    model: &str,
    prompt: &str,
    references: &[(std::path::PathBuf, Vec<u8>)],
) -> AgentResult<GeneratedImage> {
    if references.is_empty() {
        return Err(AgentError::new(
            "OpenAI image editing requires at least one reference image",
        ));
    }
    send_openai_image_request(auth, model, prompt, references).await
}

async fn send_openai_image_request(
    auth: &ResolvedProviderAuth,
    model: &str,
    prompt: &str,
    references: &[(std::path::PathBuf, Vec<u8>)],
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
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|error| {
            AgentError::new(format!("could not prepare OpenAI Images client: {error}"))
        })?;
    let request = if references.is_empty() {
        client
            .post(format!("{}/images/generations", base.trim_end_matches('/')))
            .bearer_auth(secret)
            .json(&build_openai_image_request(model, prompt)?)
    } else {
        let normalized_model = normalize_model_id("openai", model);
        if !is_openai_image_model(normalized_model)
            || prompt.trim().is_empty()
            || prompt.len() > MAX_IMAGE_PROMPT_BYTES
        {
            return Err(AgentError::new(
                "OpenAI image editing requires a supported GPT Image model and a bounded prompt",
            ));
        }
        let mut form = reqwest::multipart::Form::new()
            .text("model", normalized_model.to_string())
            .text("prompt", prompt.to_string())
            .text("n", "1")
            .text("background", "auto")
            .text("quality", "auto")
            .text("size", "auto");
        let image_field = if references.len() == 1 {
            "image"
        } else {
            "image[]"
        };
        for (path, bytes) in references {
            let mime = image_mime_type(bytes).ok_or_else(|| {
                AgentError::at_path(path, "reference image must be PNG, JPEG, or WebP")
            })?;
            if bytes.len() > MAX_GENERATED_IMAGE_BYTES {
                return Err(AgentError::at_path(
                    path,
                    "reference image exceeds the 8 MiB safety limit",
                ));
            }
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("reference-image.png");
            let part = reqwest::multipart::Part::bytes(bytes.clone())
                .file_name(file_name.to_string())
                .mime_str(mime)
                .map_err(|error| {
                    AgentError::new(format!("invalid reference image MIME type: {error}"))
                })?;
            form = form.part(image_field, part);
        }
        client
            .post(format!("{}/images/edits", base.trim_end_matches('/')))
            .bearer_auth(secret)
            .multipart(form)
    };
    let response = request.send().await.map_err(|error| {
        AgentError::new(format!("OpenAI Images network request failed: {error}"))
    })?;
    let status = response.status();
    let request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let bytes = response
        .bytes()
        .await
        .map_err(|error| AgentError::new(error.to_string()))?;
    if bytes.len() > 12 * 1024 * 1024 {
        return Err(AgentError::new(
            "OpenAI Images response exceeds bounded response size",
        ));
    }
    let payload: Value = match serde_json::from_slice(&bytes) {
        Ok(payload) => payload,
        Err(_) if !status.is_success() => {
            return Err(AgentError::new(format!(
                "OpenAI Images request failed with {status}: provider returned invalid JSON{}",
                request_id
                    .as_deref()
                    .map(|id| format!(" (request id {id})"))
                    .unwrap_or_default()
            )));
        }
        Err(_) => {
            return Err(AgentError::new(format!(
                "OpenAI Images response is invalid JSON{}",
                request_id
                    .as_deref()
                    .map(|id| format!(" (request id {id})"))
                    .unwrap_or_default()
            )));
        }
    };
    if !status.is_success() {
        return Err(openai_image_error(status, request_id.as_deref(), &payload));
    }
    parse_openai_image_response(&payload, model, prompt)
}

fn openai_image_error(status: StatusCode, request_id: Option<&str>, payload: &Value) -> AgentError {
    let error = payload.get("error").and_then(Value::as_object);
    let message = error
        .and_then(|value| value.get("message"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("provider did not return an error message");
    let code = error
        .and_then(|value| value.get("code"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(|value| format!(", code {value}"))
        .unwrap_or_default();
    let request_id = request_id
        .filter(|value| !value.is_empty())
        .map(|value| format!(", request id {value}"))
        .unwrap_or_default();
    AgentError::new(format!(
        "OpenAI Images request failed with {status}{code}{request_id}: {message}"
    ))
}

pub async fn send_native_agent_request_with_registry(
    request: &AgentRequest,
    auth: &ResolvedProviderAuth,
    registry: &ProviderRegistry,
) -> AgentResult<AgentServerResponse> {
    registry.validate()?;
    let provider_id = request
        .provider
        .as_deref()
        .ok_or_else(|| AgentError::new("native agent requests require a provider"))?;
    let definition = registry.definition(provider_id).ok_or_else(|| {
        AgentError::new(format!("unknown dynamic agent provider `{provider_id}`"))
    })?;
    if !registry.contains_model(provider_id, &request.model) {
        return Err(AgentError::new(
            "model is not declared by the dynamic provider",
        ));
    }
    let protocol = definition.protocol;
    let base = definition.base_url.trim_end_matches('/');
    let url = match protocol {
        AgentProviderProtocol::OpenAiCompletions | AgentProviderProtocol::MistralConversations => {
            format!("{base}/chat/completions")
        }
        AgentProviderProtocol::OpenAiResponses => format!("{base}/responses"),
        AgentProviderProtocol::AnthropicMessages => {
            format!("{}/v1/messages", base.strip_suffix("/v1").unwrap_or(base))
        }
        AgentProviderProtocol::GoogleGenerativeAi => format!(
            "{base}/models/{}:generateContent",
            percent_encode_query(&request.model)
        ),
        AgentProviderProtocol::PiMessages => format!("{base}/messages"),
        AgentProviderProtocol::GoogleVertex | AgentProviderProtocol::BedrockConverse => {
            return Err(AgentError::new(
                "dynamic provider protocol is not available for this route",
            ));
        }
    };
    let secret = auth.secret.as_deref().ok_or_else(|| AgentError::new("dynamic provider requires an explicit request credential or declared environment credential"))?;
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    match protocol {
        AgentProviderProtocol::AnthropicMessages => {
            insert_header(&mut headers, "x-api-key", secret.into())?;
            insert_header(&mut headers, "anthropic-version", "2023-06-01".into())?;
        }
        AgentProviderProtocol::GoogleGenerativeAi => {
            insert_header(&mut headers, "x-goog-api-key", secret.into())?
        }
        _ => insert_header(
            &mut headers,
            AUTHORIZATION.as_str(),
            format!("Bearer {secret}"),
        )?,
    }
    let body = match protocol {
        AgentProviderProtocol::OpenAiCompletions | AgentProviderProtocol::MistralConversations => {
            openai_completions_body(&request.model, request, false)?
        }
        AgentProviderProtocol::OpenAiResponses => {
            openai_responses_body(&request.model, request, false)?
        }
        AgentProviderProtocol::AnthropicMessages => anthropic_body(&request.model, request)?,
        AgentProviderProtocol::GoogleGenerativeAi => google_body(request)?,
        AgentProviderProtocol::PiMessages => pi_messages_body(&request.model, request)?,
        _ => unreachable!(),
    };
    let response_body =
        streaming::send_observed(&url, &headers, body, auth, request, protocol, &mut |_| {
            Ok(())
        })
        .await?;
    let payload = if request.stream && !response_body.trim_start().starts_with('{') {
        parse_stream_payload(protocol, &response_body)?
    } else {
        serde_json::from_str(&response_body)
            .map_err(|_| AgentError::new("dynamic provider returned invalid JSON"))?
    };
    Ok(AgentServerResponse {
        request_id: request.request_id.clone(),
        request_type: request.request_type,
        model: request.model.clone(),
        payload,
    })
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
