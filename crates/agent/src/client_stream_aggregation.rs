fn aggregate_openai(events: Vec<(Option<String>, Value)>) -> AgentResult<Value> {
    let mut text = String::new();
    let mut id = None;
    let mut model = None;
    let mut finish_reason = None;
    let mut tool_calls: BTreeMap<usize, (String, String, String)> = BTreeMap::new();
    let mut final_payload = None;
    let mut usage = None;
    let mut service_tier = None;
    for (_, event) in events {
        if let Some(tier) = event.get("service_tier") {
            service_tier = Some(tier.clone());
        }
        if event.get("usage").is_some_and(Value::is_object) {
            usage = event.get("usage").cloned();
        }
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
                if let Some(content) = delta.and_then(|value| value.get("content")) {
                    let mut blocks = Vec::new();
                    crate::conversation::text_blocks(content, &mut blocks);
                    text.push_str(&blocks.concat());
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
    let mut payload = json!({
        "id": id,
        "model": model,
        "choices": [{"index": 0, "message": message, "finish_reason": finish_reason.unwrap_or_else(|| "stop".to_string())}]
    });
    if let Some(usage) = usage {
        payload["usage"] = usage;
    }
    if let Some(tier) = service_tier {
        payload["service_tier"] = tier;
    }
    Ok(payload)
}

fn aggregate_responses(events: Vec<(Option<String>, Value)>) -> AgentResult<Value> {
    let mut text = String::new();
    let mut terminal_text = None;
    let mut output_items = Vec::new();
    let mut final_payload = None;
    for (event_name, event) in events {
        if (event_name.as_deref() == Some("response.output_text.delta")
            || event.get("type").and_then(Value::as_str) == Some("response.output_text.delta"))
            && let Some(delta) = event.get("delta").and_then(Value::as_str)
        {
            text.push_str(delta);
        }
        let kind = event_name
            .as_deref()
            .or_else(|| event.get("type").and_then(Value::as_str));
        if kind == Some("response.output_text.done") {
            terminal_text = event.get("text").and_then(Value::as_str).map(str::to_owned);
        }
        if kind == Some("response.output_item.done")
            && let Some(item) = event.get("item")
        {
            output_items.push(item.clone());
        }
        if matches!(
            kind,
            Some("response.completed" | "response.incomplete" | "response.failed")
        ) {
            final_payload = event.get("response").cloned();
        } else if event.get("output_text").is_some() || event.get("status").is_some() {
            final_payload = Some(event);
        }
    }
    if let Some(terminal_text) = terminal_text {
        text = terminal_text;
    }
    if let Some(mut payload) = final_payload {
        if !output_items.is_empty() {
            let output = payload
                .get_mut("output")
                .and_then(Value::as_array_mut)
                .map(|output| {
                    for item in &output_items {
                        if !output.iter().any(|existing| existing == item) {
                            output.push(item.clone());
                        }
                    }
                    output.clone()
                })
                .unwrap_or_else(|| output_items.clone());
            payload["output"] = Value::Array(output);
        }
        if payload.get("output_text").is_none() && !text.is_empty() {
            payload["output_text"] = Value::String(text);
        }
        return Ok(payload);
    }
    let mut payload = json!({"output_text": text});
    if !output_items.is_empty() {
        payload["output"] = Value::Array(output_items);
    }
    Ok(payload)
}

fn aggregate_anthropic(events: Vec<(Option<String>, Value)>) -> AgentResult<Value> {
    let mut text = String::new();
    let mut id = None;
    let mut model = None;
    let mut stop_reason = None;
    let mut tools: BTreeMap<usize, (String, String, String)> = BTreeMap::new();
    let mut usage = serde_json::Map::new();
    for (event_name, event) in events {
        if event.get("type").and_then(Value::as_str) == Some("message")
            && event.get("content").is_some()
        {
            return Ok(event);
        }
        if let Some(snapshot) = event
            .pointer("/message/usage")
            .or_else(|| event.get("usage"))
            .and_then(Value::as_object)
        {
            usage.extend(snapshot.clone());
        }
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
    let mut payload = json!({
        "id": id,
        "model": model,
        "role": "assistant",
        "content": content,
        "stop_reason": stop_reason.unwrap_or_else(|| "end_turn".to_string())
    });
    if !usage.is_empty() {
        payload["usage"] = Value::Object(usage);
    }
    Ok(payload)
}

fn aggregate_google(events: Vec<(Option<String>, Value)>) -> AgentResult<Value> {
    let mut text = String::new();
    let mut thoughts = String::new();
    let mut function_calls = Vec::new();
    let mut usage = None;
    for (_, event) in events {
        if event.get("usageMetadata").is_some() {
            usage = event.get("usageMetadata").cloned();
        }
        if let Some(parts) = event
            .get("candidates")
            .and_then(|value| value.get(0))
            .and_then(|value| value.get("content"))
            .and_then(|value| value.get("parts"))
            .and_then(Value::as_array)
        {
            for part in parts {
                if let Some(value) = part.get("text").and_then(Value::as_str) {
                    if part.get("thought").and_then(Value::as_bool) == Some(true) {
                        thoughts.push_str(value);
                    } else {
                        text.push_str(value);
                    }
                }
                if let Some(call) = part.get("functionCall") {
                    function_calls.push(call.clone());
                }
            }
        }
    }
    let mut parts = Vec::new();
    if !thoughts.is_empty() {
        parts.push(json!({"thought": true, "text": thoughts}));
    }
    if !text.is_empty() || function_calls.is_empty() {
        parts.push(json!({"text": text}));
    }
    parts.extend(
        function_calls
            .into_iter()
            .map(|call| json!({"functionCall": call})),
    );
    let mut payload = json!({"candidates": [{"content": {"role": "model", "parts": parts}}]});
    if let Some(usage) = usage {
        payload["usageMetadata"] = usage;
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

