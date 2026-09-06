use crate::error::{AgentError, AgentResult};
use crate::model::{
    AgentMessage, AgentMessageContent, AgentMessagePart, AgentRequest, AgentServerResponse,
    AgentToolDefinition,
};
use crate::oauth::openai_codex_account_id;
use crate::provider::{
    AgentAuthKind, AgentProviderDefinition, AgentProviderProtocol, ResolvedProviderAuth,
    normalize_model_id, protocol_for_model, provider_base_url, provider_definition,
};
use reqwest::StatusCode;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::Duration;

pub async fn send_agent_request(
    server_url: &str,
    request: &AgentRequest,
) -> AgentResult<AgentServerResponse> {
    let url = format!("{}/api/v1/agent", server_url.trim_end_matches('/'));
    let response = reqwest::Client::new()
        .post(url)
        .json(request)
        .send()
        .await
        .map_err(|error| AgentError::new(error.to_string()))?;
    let status = response.status();
    let payload = response
        .json::<Value>()
        .await
        .unwrap_or_else(|error| Value::String(error.to_string()));

    if !status.is_success() {
        return Err(AgentError::new(format!(
            "llm server returned {}: {}",
            status_text(status),
            payload
        )));
    }

    serde_json::from_value::<AgentServerResponse>(payload)
        .map_err(|error| AgentError::new(error.to_string()))
}

pub async fn send_native_agent_request(
    request: &AgentRequest,
    auth: &ResolvedProviderAuth,
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
    let (url, mut headers, body) = build_provider_request(
        &definition,
        protocol,
        &base_url,
        &model,
        &native_request,
        auth,
    )?;
    let response_body = send_with_retry(&url, &mut headers, body, auth.secret.as_deref()).await?;
    let payload = if native_request.stream && protocol != AgentProviderProtocol::BedrockConverse {
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

fn build_provider_request(
    definition: &AgentProviderDefinition,
    protocol: AgentProviderProtocol,
    base_url: &str,
    model: &str,
    request: &AgentRequest,
    auth: &ResolvedProviderAuth,
) -> AgentResult<(String, HeaderMap, Value)> {
    let url = provider_url(definition, protocol, base_url, model, request.stream, auth)?;
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("accept", HeaderValue::from_static("application/json"));
    headers.insert("user-agent", HeaderValue::from_static("dowe-agent/1.0"));
    apply_auth_headers(definition, protocol, &mut headers, auth)?;
    if definition.id == "openai-codex" && request.stream {
        headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
        if let Some(session_id) = request.extra.get("session_id").and_then(Value::as_str) {
            insert_header(&mut headers, "session-id", session_id.to_string())?;
            insert_header(&mut headers, "x-client-request-id", session_id.to_string())?;
        }
    }
    let body = match protocol {
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
                format!("{base}/openai/v1/chat/completions")
            } else if matches!(definition.id, "vercel-ai-gateway" | "mistral") {
                format!("{base}/v1/chat/completions")
            } else {
                format!("{base}/chat/completions")
            }
        }
        AgentProviderProtocol::OpenAiResponses => {
            if definition.id == "openai-codex" {
                format!("{base}/codex/responses")
            } else {
                format!("{base}/responses")
            }
        }
        AgentProviderProtocol::AnthropicMessages => {
            if definition.id == "cloudflare-ai-gateway" {
                format!("{base}/anthropic/v1/messages")
            } else if definition.id == "openrouter" && base.ends_with("/v1") {
                format!("{}/v1/messages", base.trim_end_matches("/v1"))
            } else {
                format!("{base}/v1/messages")
            }
        }
        AgentProviderProtocol::GoogleGenerativeAi => {
            let mut value = format!("{base}/models/{model}:streamGenerateContent");
            if !stream {
                value = format!("{base}/models/{model}:generateContent");
            }
            if let Some(key) = auth.secret.as_deref() {
                value.push_str("?key=");
                value.push_str(&percent_encode_query(key));
            }
            value
        }
        AgentProviderProtocol::GoogleVertex => {
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
            let action = if stream {
                "streamGenerateContent"
            } else {
                "generateContent"
            };
            format!(
                "https://{location}-aiplatform.googleapis.com/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:{action}"
            )
        }
        AgentProviderProtocol::BedrockConverse => {
            let encoded_model = model.replace('/', "%2F");
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
    match protocol {
        AgentProviderProtocol::AnthropicMessages => {
            if definition.id == "github-copilot"
                || definition.id == "kimi-coding"
                || auth.kind == AgentAuthKind::OAuth
                || (definition.id == "anthropic" && auth.source == "ANTHROPIC_AUTH_TOKEN")
            {
                insert_header(headers, AUTHORIZATION.as_str(), format!("Bearer {secret}"))?;
            } else {
                insert_header(headers, "x-api-key", secret.to_string())?;
            }
            insert_header(headers, "anthropic-version", "2023-06-01".to_string())?;
        }
        AgentProviderProtocol::GoogleGenerativeAi => {}
        AgentProviderProtocol::GoogleVertex => {
            insert_header(headers, AUTHORIZATION.as_str(), format!("Bearer {secret}"))?;
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

fn openai_completions_body(
    model: &str,
    request: &AgentRequest,
    openrouter: bool,
) -> AgentResult<Value> {
    let mut body = json!({
        "model": model,
        "messages": request.messages,
        "stream": request.stream,
    });
    let object = body
        .as_object_mut()
        .ok_or_else(|| AgentError::new("could not build OpenAI request"))?;
    insert_common_request_fields(object, request, "max_completion_tokens");
    if !request.tools.is_empty() {
        object.insert("tools".to_string(), serde_json::to_value(&request.tools)?);
    }
    if let Some(response_format) = &request.response_format {
        object.insert("response_format".to_string(), response_format.clone());
    }
    if openrouter {
        object.insert("provider".to_string(), json!({"allow_fallbacks": true}));
    }
    Ok(body)
}

fn openai_responses_body(model: &str, request: &AgentRequest, codex: bool) -> AgentResult<Value> {
    let mut instructions = Vec::new();
    let mut input = Vec::new();
    for message in &request.messages {
        if message.role == "system" {
            instructions.push(message_text(message));
            continue;
        }
        input.push(responses_message(message));
    }
    let mut body = json!({
        "model": model,
        "instructions": instructions.join("\n\n"),
        "input": input,
        "stream": request.stream,
        "store": false,
    });
    let object = body
        .as_object_mut()
        .ok_or_else(|| AgentError::new("could not build OpenAI Responses request"))?;
    insert_common_request_fields(object, request, "max_output_tokens");
    if !request.tools.is_empty() {
        object.insert("tools".to_string(), responses_tools(&request.tools)?);
    }
    if codex {
        object.insert("text".to_string(), json!({"verbosity": "low"}));
        object.insert(
            "include".to_string(),
            json!(["reasoning.encrypted_content"]),
        );
        object.insert("tool_choice".to_string(), json!("auto"));
        object.insert("parallel_tool_calls".to_string(), json!(true));
        if let Some(session_id) = request.extra.get("session_id") {
            object.insert("prompt_cache_key".to_string(), session_id.clone());
        }
    }
    Ok(body)
}

fn anthropic_body(model: &str, request: &AgentRequest) -> AgentResult<Value> {
    let mut messages = Vec::new();
    let mut system = Vec::new();
    for message in &request.messages {
        if message.role == "system" {
            system.push(message_text(message));
        } else {
            messages.push(anthropic_message(message)?);
        }
    }
    let max_tokens = request
        .extra
        .get("max_completion_tokens")
        .cloned()
        .unwrap_or_else(|| json!(4096));
    let mut body = json!({
        "model": model,
        "max_tokens": max_tokens,
        "messages": messages,
        "stream": request.stream,
    });
    let object = body
        .as_object_mut()
        .ok_or_else(|| AgentError::new("could not build Anthropic request"))?;
    if !system.is_empty() {
        object.insert("system".to_string(), json!(system.join("\n\n")));
    }
    if !request.tools.is_empty() {
        object.insert("tools".to_string(), anthropic_tools(&request.tools)?);
    }
    if let Some(temperature) = request.extra.get("temperature") {
        object.insert("temperature".to_string(), temperature.clone());
    }
    Ok(body)
}

fn google_body(request: &AgentRequest) -> AgentResult<Value> {
    let mut contents = Vec::new();
    let mut system = Vec::new();
    for message in &request.messages {
        if message.role == "system" {
            system.push(message_text(message));
            continue;
        }
        let role = if message.role == "assistant" {
            "model"
        } else {
            "user"
        };
        contents.push(json!({"role": role, "parts": google_parts(message)?}));
    }
    let mut config = serde_json::Map::new();
    if let Some(value) = request.extra.get("temperature") {
        config.insert("temperature".to_string(), value.clone());
    }
    if let Some(value) = request.extra.get("max_completion_tokens") {
        config.insert("maxOutputTokens".to_string(), value.clone());
    }
    let mut body = json!({"contents": contents, "generationConfig": config});
    let object = body
        .as_object_mut()
        .ok_or_else(|| AgentError::new("could not build Google request"))?;
    if !system.is_empty() {
        object.insert(
            "systemInstruction".to_string(),
            json!({"parts": [{"text": system.join("\n\n") }]}),
        );
    }
    if !request.tools.is_empty() {
        object.insert(
            "tools".to_string(),
            json!([{"functionDeclarations": google_tools(&request.tools)?}]),
        );
    }
    Ok(body)
}

fn mistral_body(model: &str, request: &AgentRequest) -> AgentResult<Value> {
    let mut body = openai_completions_body(model, request, false)?;
    if let Some(object) = body.as_object_mut()
        && let Some(value) = object.remove("max_completion_tokens")
    {
        object.insert("max_tokens".to_string(), value);
    }
    Ok(body)
}

fn bedrock_body(request: &AgentRequest) -> AgentResult<Value> {
    let mut messages = Vec::new();
    let mut system = Vec::new();
    for message in &request.messages {
        if message.role == "system" {
            system.push(json!({"text": message_text(message)}));
            continue;
        }
        let role = if message.role == "assistant" {
            "assistant"
        } else {
            "user"
        };
        messages.push(json!({"role": role, "content": bedrock_parts(message)?}));
    }
    let mut body = json!({
        "messages": messages,
        "inferenceConfig": {},
    });
    let object = body
        .as_object_mut()
        .ok_or_else(|| AgentError::new("could not build Bedrock request"))?;
    if !system.is_empty() {
        object.insert("system".to_string(), Value::Array(system));
    }
    if let Some(value) = request.extra.get("max_completion_tokens") {
        object
            .get_mut("inferenceConfig")
            .and_then(Value::as_object_mut)
            .map(|config| config.insert("maxTokens".to_string(), value.clone()));
    }
    if !request.tools.is_empty() {
        object.insert(
            "toolConfig".to_string(),
            json!({"tools": bedrock_tools(&request.tools)?}),
        );
    }
    Ok(body)
}

fn pi_messages_body(model: &str, request: &AgentRequest) -> AgentResult<Value> {
    Ok(json!({
        "model": model,
        "context": {
            "messages": request.messages,
            "tools": request.tools,
        },
        "options": {
            "stream": request.stream,
            "temperature": request.extra.get("temperature"),
            "maxTokens": request.extra.get("max_completion_tokens"),
            "sessionId": request.extra.get("session_id"),
        }
    }))
}

fn insert_common_request_fields(
    object: &mut serde_json::Map<String, Value>,
    request: &AgentRequest,
    max_tokens_name: &str,
) {
    for (name, value) in &request.extra {
        if name == "max_completion_tokens" {
            continue;
        }
        if name == "session_id" {
            object.insert("prompt_cache_key".to_string(), value.clone());
        } else {
            object.insert(name.clone(), value.clone());
        }
    }
    if let Some(value) = request.extra.get("max_completion_tokens") {
        object.insert(max_tokens_name.to_string(), value.clone());
    }
}

fn responses_message(message: &AgentMessage) -> Value {
    let content = match &message.content {
        AgentMessageContent::Text(text) => Value::String(text.clone()),
        AgentMessageContent::Parts(parts) => Value::Array(
            parts
                .iter()
                .map(|part| match part {
                    AgentMessagePart::Text { text } => json!({"type": "input_text", "text": text}),
                    AgentMessagePart::ImageUrl { image_url } => {
                        json!({"type": "input_image", "image_url": image_url.url})
                    }
                })
                .collect(),
        ),
    };
    json!({"role": message.role, "content": content})
}

fn anthropic_message(message: &AgentMessage) -> AgentResult<Value> {
    Ok(json!({
        "role": if message.role == "assistant" { "assistant" } else { "user" },
        "content": anthropic_content(&message.content)?,
    }))
}

fn anthropic_content(content: &AgentMessageContent) -> AgentResult<Value> {
    match content {
        AgentMessageContent::Text(text) => Ok(Value::String(text.clone())),
        AgentMessageContent::Parts(parts) => Ok(Value::Array(
            parts
                .iter()
                .map(anthropic_part)
                .collect::<AgentResult<Vec<_>>>()?,
        )),
    }
}

fn anthropic_part(part: &AgentMessagePart) -> AgentResult<Value> {
    match part {
        AgentMessagePart::Text { text } => Ok(json!({"type": "text", "text": text})),
        AgentMessagePart::ImageUrl { image_url } => {
            let (media_type, data) = parse_data_url(&image_url.url)?;
            Ok(json!({
                "type": "image",
                "source": {"type": "base64", "media_type": media_type, "data": data}
            }))
        }
    }
}

fn google_parts(message: &AgentMessage) -> AgentResult<Vec<Value>> {
    match &message.content {
        AgentMessageContent::Text(text) => Ok(vec![json!({"text": text})]),
        AgentMessageContent::Parts(parts) => parts
            .iter()
            .map(|part| match part {
                AgentMessagePart::Text { text } => Ok(json!({"text": text})),
                AgentMessagePart::ImageUrl { image_url } => {
                    let (mime_type, data) = parse_data_url(&image_url.url)?;
                    Ok(json!({"inlineData": {"mimeType": mime_type, "data": data}}))
                }
            })
            .collect(),
    }
}

fn bedrock_parts(message: &AgentMessage) -> AgentResult<Vec<Value>> {
    match &message.content {
        AgentMessageContent::Text(text) => Ok(vec![json!({"text": text})]),
        AgentMessageContent::Parts(parts) => parts
            .iter()
            .map(|part| match part {
                AgentMessagePart::Text { text } => Ok(json!({"text": text})),
                AgentMessagePart::ImageUrl { .. } => Err(AgentError::new(
                    "Bedrock native requests currently support text input only",
                )),
            })
            .collect(),
    }
}

fn responses_tools(tools: &[AgentToolDefinition]) -> AgentResult<Value> {
    Ok(Value::Array(
        tools
            .iter()
            .map(|tool| {
                json!({
                    "type": "function",
                    "name": tool.function.name,
                    "description": tool.function.description,
                    "parameters": tool.function.parameters,
                })
            })
            .collect(),
    ))
}

fn anthropic_tools(tools: &[AgentToolDefinition]) -> AgentResult<Value> {
    Ok(Value::Array(
        tools
            .iter()
            .map(|tool| {
                json!({
                    "name": tool.function.name,
                    "description": tool.function.description,
                    "input_schema": tool.function.parameters,
                })
            })
            .collect(),
    ))
}

fn google_tools(tools: &[AgentToolDefinition]) -> AgentResult<Vec<Value>> {
    Ok(tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.function.name,
                "description": tool.function.description,
                "parameters": tool.function.parameters,
            })
        })
        .collect())
}

fn bedrock_tools(tools: &[AgentToolDefinition]) -> AgentResult<Vec<Value>> {
    Ok(tools
        .iter()
        .map(|tool| {
            json!({
                "toolSpec": {
                    "name": tool.function.name,
                    "description": tool.function.description,
                    "inputSchema": {"json": tool.function.parameters},
                }
            })
        })
        .collect())
}

fn message_text(message: &AgentMessage) -> String {
    match &message.content {
        AgentMessageContent::Text(text) => text.clone(),
        AgentMessageContent::Parts(parts) => parts
            .iter()
            .filter_map(|part| match part {
                AgentMessagePart::Text { text } => Some(text.as_str()),
                AgentMessagePart::ImageUrl { .. } => None,
            })
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn parse_data_url(value: &str) -> AgentResult<(String, String)> {
    let (header, data) = value
        .strip_prefix("data:")
        .and_then(|value| value.split_once(","))
        .ok_or_else(|| AgentError::new("image input must be a data URL"))?;
    let mime_type = header
        .split(';')
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AgentError::new("image data URL has no MIME type"))?;
    Ok((mime_type.to_string(), data.to_string()))
}

async fn send_with_retry(
    url: &str,
    headers: &mut HeaderMap,
    body: Value,
    secret: Option<&str>,
) -> AgentResult<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|error| AgentError::new(error.to_string()))?;
    for attempt in 0..=2 {
        let response = client
            .post(url)
            .headers(headers.clone())
            .json(&body)
            .send()
            .await
            .map_err(|error| AgentError::new(redact_secret(&error.to_string(), secret)))?;
        let status = response.status();
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .map(|seconds| seconds.min(30));
        let text = response
            .text()
            .await
            .map_err(|error| AgentError::new(redact_secret(&error.to_string(), secret)))?;
        if status.is_success() {
            return Ok(text);
        }
        if attempt < 2 && is_retryable(status) {
            let delay = retry_after.unwrap_or(1_u64 << attempt).min(30);
            tokio::time::sleep(Duration::from_secs(delay)).await;
            continue;
        }
        return Err(AgentError::new(format!(
            "provider request returned {}: {}",
            status_text(status),
            redact_secret(&text, secret)
        )));
    }
    Err(AgentError::new(
        "provider request retry loop ended unexpectedly",
    ))
}

fn parse_stream_payload(protocol: AgentProviderProtocol, body: &str) -> AgentResult<Value> {
    let events = parse_sse_events(body)?;
    match protocol {
        AgentProviderProtocol::AnthropicMessages => aggregate_anthropic(events),
        AgentProviderProtocol::GoogleGenerativeAi | AgentProviderProtocol::GoogleVertex => {
            aggregate_google(events)
        }
        AgentProviderProtocol::PiMessages => aggregate_pi_messages(events),
        AgentProviderProtocol::OpenAiResponses => aggregate_responses(events),
        _ => aggregate_openai(events),
    }
}

fn parse_sse_events(body: &str) -> AgentResult<Vec<(Option<String>, Value)>> {
    let mut events = Vec::new();
    let mut event_name = None;
    let mut data = Vec::new();
    for line in body.lines().chain(std::iter::once("")) {
        if line.is_empty() {
            if !data.is_empty() {
                let value = data.join("\n");
                if value != "[DONE]" {
                    let parsed = serde_json::from_str(&value).map_err(|error| {
                        AgentError::new(format!("provider returned invalid SSE JSON: {error}"))
                    })?;
                    events.push((event_name.take(), parsed));
                } else {
                    event_name = None;
                }
                data.clear();
            }
            continue;
        }
        if let Some(value) = line.strip_prefix("event:") {
            event_name = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            data.push(value.trim_start().to_string());
        }
    }
    if events.is_empty() {
        let value: Value = serde_json::from_str(body)
            .map_err(|error| AgentError::new(format!("provider returned invalid JSON: {error}")))?;
        return Ok(vec![(None, value)]);
    }
    Ok(events)
}

fn aggregate_openai(events: Vec<(Option<String>, Value)>) -> AgentResult<Value> {
    let mut text = String::new();
    let mut id = None;
    let mut model = None;
    let mut finish_reason = None;
    let mut tool_calls: BTreeMap<usize, (String, String, String)> = BTreeMap::new();
    let mut final_payload = None;
    for (_, event) in events {
        id = id.or_else(|| event.get("id").and_then(Value::as_str).map(str::to_string));
        model = model.or_else(|| {
            event
                .get("model")
                .and_then(Value::as_str)
                .map(str::to_string)
        });
        if event.get("choices").is_some() {
            if let Some(choice) = event.get("choices").and_then(|value| value.get(0)) {
                finish_reason = choice
                    .get("finish_reason")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(finish_reason);
                let delta = choice.get("delta").or_else(|| choice.get("message"));
                if let Some(content) = delta
                    .and_then(|value| value.get("content"))
                    .and_then(Value::as_str)
                {
                    text.push_str(content);
                }
                if let Some(calls) = delta
                    .and_then(|value| value.get("tool_calls"))
                    .and_then(Value::as_array)
                {
                    for call in calls {
                        let index = call.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                        let entry = tool_calls
                            .entry(index)
                            .or_insert_with(|| (String::new(), String::new(), String::new()));
                        if let Some(value) = call.get("id").and_then(Value::as_str) {
                            entry.0 = value.to_string();
                        }
                        if let Some(function) = call.get("function") {
                            if let Some(value) = function.get("name").and_then(Value::as_str) {
                                entry.1.push_str(value);
                            }
                            if let Some(value) = function.get("arguments").and_then(Value::as_str) {
                                entry.2.push_str(value);
                            }
                        }
                    }
                }
            }
            final_payload = Some(event);
        } else if event.get("message").is_some() {
            final_payload = Some(event);
        }
    }
    let mut message = json!({"role": "assistant", "content": text});
    if !tool_calls.is_empty() {
        message["tool_calls"] = Value::Array(
            tool_calls
                .into_iter()
                .map(|(index, (id, name, arguments))| {
                    json!({"index": index, "id": id, "type": "function", "function": {"name": name, "arguments": arguments}})
                })
                .collect(),
        );
    }
    if let Some(payload) = final_payload
        && payload.get("choices").is_none()
        && payload.get("output_text").is_some()
    {
        return Ok(payload);
    }
    Ok(json!({
        "id": id,
        "model": model,
        "choices": [{"index": 0, "message": message, "finish_reason": finish_reason.unwrap_or_else(|| "stop".to_string())}]
    }))
}

fn aggregate_responses(events: Vec<(Option<String>, Value)>) -> AgentResult<Value> {
    let mut text = String::new();
    let mut final_payload = None;
    for (event_name, event) in events {
        if (event_name.as_deref() == Some("response.output_text.delta")
            || event.get("type").and_then(Value::as_str) == Some("response.output_text.delta"))
            && let Some(delta) = event.get("delta").and_then(Value::as_str)
        {
            text.push_str(delta);
        }
        if event.get("output_text").is_some() || event.get("status").is_some() {
            final_payload = Some(event);
        }
    }
    if let Some(mut payload) = final_payload {
        if payload.get("output_text").is_none() && !text.is_empty() {
            payload["output_text"] = Value::String(text);
        }
        return Ok(payload);
    }
    Ok(json!({"output_text": text}))
}

fn aggregate_anthropic(events: Vec<(Option<String>, Value)>) -> AgentResult<Value> {
    let mut text = String::new();
    let mut id = None;
    let mut model = None;
    let mut stop_reason = None;
    let mut tools: BTreeMap<usize, (String, String, String)> = BTreeMap::new();
    for (event_name, event) in events {
        let kind = event_name
            .as_deref()
            .or_else(|| event.get("type").and_then(Value::as_str));
        if kind == Some("message_start") {
            let message = event.get("message").unwrap_or(&event);
            id = message
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string);
            model = message
                .get("model")
                .and_then(Value::as_str)
                .map(str::to_string);
        } else if kind == Some("content_block_start") {
            let index = event.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
            if let Some(tool) = event
                .get("content_block")
                .and_then(|value| value.get("type"))
                .and_then(Value::as_str)
                .filter(|value| *value == "tool_use")
            {
                let block = event.get("content_block").unwrap_or(&Value::Null);
                tools.insert(
                    index,
                    (
                        block
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        block
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        String::new(),
                    ),
                );
                let _ = tool;
            }
        } else if kind == Some("content_block_delta") {
            let delta = event.get("delta").unwrap_or(&Value::Null);
            match delta.get("type").and_then(Value::as_str) {
                Some("text_delta") => text.push_str(
                    delta
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or_default(),
                ),
                Some("input_json_delta") => {
                    let index = event.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                    tools
                        .entry(index)
                        .or_insert_with(|| (String::new(), String::new(), String::new()))
                        .2
                        .push_str(
                            delta
                                .get("partial_json")
                                .and_then(Value::as_str)
                                .unwrap_or_default(),
                        );
                }
                _ => {}
            }
        } else if kind == Some("message_delta") {
            stop_reason = event
                .get("delta")
                .and_then(|value| value.get("stop_reason"))
                .and_then(Value::as_str)
                .map(str::to_string);
        }
    }
    let mut content = vec![json!({"type": "text", "text": text})];
    for (_, (id, name, input)) in tools {
        let arguments = serde_json::from_str::<Value>(&input).unwrap_or_else(|_| json!({}));
        content.push(json!({"type": "tool_use", "id": id, "name": name, "input": arguments}));
    }
    Ok(json!({
        "id": id,
        "model": model,
        "role": "assistant",
        "content": content,
        "stop_reason": stop_reason.unwrap_or_else(|| "end_turn".to_string())
    }))
}

fn aggregate_google(events: Vec<(Option<String>, Value)>) -> AgentResult<Value> {
    let mut text = String::new();
    let mut function_calls = Vec::new();
    for (_, event) in events {
        if let Some(parts) = event
            .get("candidates")
            .and_then(|value| value.get(0))
            .and_then(|value| value.get("content"))
            .and_then(|value| value.get("parts"))
            .and_then(Value::as_array)
        {
            for part in parts {
                if let Some(value) = part.get("text").and_then(Value::as_str) {
                    text.push_str(value);
                }
                if let Some(call) = part.get("functionCall") {
                    function_calls.push(call.clone());
                }
            }
        }
    }
    let mut payload =
        json!({"candidates": [{"content": {"role": "model", "parts": [{"text": text}]}}]});
    if !function_calls.is_empty() {
        payload["candidates"][0]["content"]["parts"] = Value::Array(
            function_calls
                .into_iter()
                .map(|call| json!({"functionCall": call}))
                .collect(),
        );
    }
    Ok(payload)
}

fn aggregate_pi_messages(events: Vec<(Option<String>, Value)>) -> AgentResult<Value> {
    let mut text = String::new();
    let mut final_event = None;
    for (_, event) in events {
        match event.get("type").and_then(Value::as_str) {
            Some("text_delta") => text.push_str(
                event
                    .get("delta")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            ),
            Some("done") | Some("error") => final_event = Some(event),
            _ => {}
        }
    }
    if let Some(event) = final_event {
        return Ok(event);
    }
    Ok(
        json!({"choices": [{"message": {"role": "assistant", "content": text}, "finish_reason": "stop"}]}),
    )
}

fn insert_header(headers: &mut HeaderMap, name: &str, value: String) -> AgentResult<()> {
    let name = HeaderName::try_from(name).map_err(|error| AgentError::new(error.to_string()))?;
    let value =
        HeaderValue::from_str(&value).map_err(|error| AgentError::new(error.to_string()))?;
    headers.insert(name, value);
    Ok(())
}

fn percent_encode_query(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn is_retryable(status: StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 425 | 429 | 500 | 502 | 503 | 504)
}

fn redact_secret(value: &str, secret: Option<&str>) -> String {
    match secret.filter(|secret| !secret.is_empty()) {
        Some(secret) => value.replace(secret, "[REDACTED]"),
        None => value.to_string(),
    }
}

fn status_text(status: StatusCode) -> String {
    status
        .canonical_reason()
        .map(|reason| format!("{} {reason}", status.as_u16()))
        .unwrap_or_else(|| status.as_u16().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AgentMessage, AgentMessageContent, AgentRequestType};
    use crate::provider::AgentAuthKind;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

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
