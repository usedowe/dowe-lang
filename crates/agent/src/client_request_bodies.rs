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
    let provider = request.provider.as_deref().unwrap_or_default();
    let limit = if matches!(
        provider,
        "deepseek"
            | "nvidia"
            | "zai"
            | "zai-coding-cn"
            | "opencode"
            | "opencode-go"
            | "cloudflare-ai-gateway"
    ) {
        "max_tokens"
    } else {
        "max_completion_tokens"
    };
    insert_common_request_fields(object, request, limit);
    if provider != "openai" {
        object.remove("prompt_cache_key");
    }
    if request.stream {
        object.insert("stream_options".into(), json!({"include_usage":true}));
    }
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
        "stream": codex || request.stream,
        "store": false,
    });
    let object = body
        .as_object_mut()
        .ok_or_else(|| AgentError::new("could not build OpenAI Responses request"))?;
    if !codex {
        insert_common_request_fields(object, request, "max_output_tokens");
    }
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
        if matches!(
            name.as_str(),
            "max_completion_tokens" | "thinkingLevel" | "dowe_harness_turns" | "task_packet"
        ) {
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

pub(crate) fn responses_message(message: &AgentMessage) -> Value {
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

pub(crate) fn anthropic_message(message: &AgentMessage) -> AgentResult<Value> {
    Ok(json!({
        "role": if message.role == "assistant" { "assistant" } else { "user" },
        "content": anthropic_content(&message.content)?,
    }))
}

