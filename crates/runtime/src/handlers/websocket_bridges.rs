async fn bridge_websocket_response(
    socket: &mut WebSocket,
    response: reqwest::Response,
    request_id: &str,
    request_type: &str,
    model: &str,
) -> Result<(), ()> {
    let status = status_from_reqwest(response.status());
    let content_type = response_content_type(&response);
    if !status.is_success() {
        let payload = match response.bytes().await {
            Ok(body) => json_from_bytes(&body),
            Err(_) => Value::String("Outbound HTTP response failed".to_string()),
        };
        let message = payload.to_string();
        return send_ws_error(
            socket,
            Some(request_id),
            Some(request_type),
            Some(model),
            "openrouter_error",
            &message,
        )
        .await;
    }
    if content_type.as_deref().is_some_and(is_sse_content_type) {
        return bridge_websocket_event_stream(socket, response, request_id, request_type, model)
            .await;
    }
    match response.bytes().await {
        Ok(body) => {
            send_ws_event(
                socket,
                "message",
                request_id,
                request_type,
                model,
                json_from_bytes(&body),
                None,
            )
            .await?;
            send_ws_done(socket, request_id, request_type, model).await
        }
        Err(_) => {
            send_ws_error(
                socket,
                Some(request_id),
                Some(request_type),
                Some(model),
                "http_error",
                "Outbound HTTP response failed",
            )
            .await
        }
    }
}

async fn bridge_websocket_event_stream(
    socket: &mut WebSocket,
    response: reqwest::Response,
    request_id: &str,
    request_type: &str,
    model: &str,
) -> Result<(), ()> {
    let mut buffer = String::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(_) => {
                return send_ws_error(
                    socket,
                    Some(request_id),
                    Some(request_type),
                    Some(model),
                    "http_error",
                    "Outbound HTTP response failed",
                )
                .await;
            }
        };
        let text = String::from_utf8_lossy(&chunk);
        for data in extract_sse_data_events(&mut buffer, &text) {
            if data == "[DONE]" {
                return send_ws_done(socket, request_id, request_type, model).await;
            }
            let payload = serde_json::from_str::<Value>(&data).unwrap_or(Value::String(data));
            let content = delta_content(&payload);
            send_ws_event(
                socket,
                "delta",
                request_id,
                request_type,
                model,
                payload,
                content,
            )
            .await?;
        }
    }
    send_ws_done(socket, request_id, request_type, model).await
}

fn extract_sse_data_events(buffer: &mut String, chunk: &str) -> Vec<String> {
    buffer.push_str(chunk);
    let mut events = Vec::new();
    while let Some(index) = buffer.find('\n') {
        let mut line = buffer[..index].to_string();
        if line.ends_with('\r') {
            line.pop();
        }
        buffer.replace_range(..=index, "");
        let trimmed = line.trim();
        if let Some(data) = trimmed.strip_prefix("data:") {
            events.push(data.trim().to_string());
        }
    }
    events
}

fn delta_content(payload: &Value) -> Option<String> {
    payload
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("delta"))
        .and_then(|delta| delta.get("content"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn resolved_text(context: &StoreActionContext<'_>, reference: &str, default: &str) -> String {
    match context.resolve_reference(reference).into_json() {
        Some(Value::String(value)) => value,
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        Some(value) => value.to_string(),
        None => default.to_string(),
    }
}

async fn send_ws_done(
    socket: &mut WebSocket,
    request_id: &str,
    request_type: &str,
    model: &str,
) -> Result<(), ()> {
    send_ws_event(
        socket,
        "done",
        request_id,
        request_type,
        model,
        done_payload(),
        None,
    )
    .await
}

async fn send_ws_error(
    socket: &mut WebSocket,
    request_id: Option<&str>,
    request_type: Option<&str>,
    model: Option<&str>,
    code: &str,
    message: &str,
) -> Result<(), ()> {
    let mut error = Map::new();
    error.insert("code".to_string(), Value::String(code.to_string()));
    error.insert("message".to_string(), Value::String(message.to_string()));
    let mut payload = Map::new();
    payload.insert("error".to_string(), Value::Object(error));
    payload.insert("metadata".to_string(), Value::Null);
    send_ws_event(
        socket,
        "error",
        request_id.unwrap_or("unknown"),
        request_type.unwrap_or(""),
        model.unwrap_or(""),
        Value::Object(payload),
        None,
    )
    .await
}

