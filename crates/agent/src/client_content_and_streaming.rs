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

pub(crate) fn google_parts(message: &AgentMessage) -> AgentResult<Vec<Value>> {
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

pub(crate) fn bedrock_parts(message: &AgentMessage) -> AgentResult<Vec<Value>> {
    match &message.content {
        AgentMessageContent::Text(text) => Ok(vec![json!({"text": text})]),
        AgentMessageContent::Parts(parts) => parts
            .iter()
            .map(|part| match part {
                AgentMessagePart::Text { text } => Ok(json!({"text": text})),
                AgentMessagePart::ImageUrl { image_url } => {
                    let (media_type, data) = parse_data_url(&image_url.url)?;
                    let format = media_type
                        .strip_prefix("image/")
                        .filter(|format| matches!(*format, "png" | "jpeg" | "gif" | "webp"))
                        .ok_or_else(|| AgentError::new("unsupported Bedrock image format"))?;
                    Ok(json!({"image":{"format":format,"source":{"bytes":data}}}))
                }
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

fn parse_stream_payload(protocol: AgentProviderProtocol, body: &str) -> AgentResult<Value> {
    let events = parse_sse_events(body)?;
    if events
        .iter()
        .any(|(name, payload)| crate::conversation::response_failed(name.as_deref(), payload))
    {
        return Err(AgentError::new(
            "the provider returned an unsuccessful stream",
        ));
    }
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

