pub async fn dev_websocket_handler(
    State(state): State<DevRuntimeState>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let mut receiver = state.events.subscribe();

    upgrade
        .on_upgrade(move |mut socket| async move {
            while let Ok(event) = receiver.recv().await {
                let Ok(message) = serde_json::to_string(&event) else {
                    continue;
                };
                if socket.send(Message::Text(message.into())).await.is_err() {
                    break;
                }
            }
        })
        .into_response()
}

async fn handle_websocket(
    mut socket: WebSocket,
    project: CompiledProject,
    handlers: WebSocketHandlers,
) {
    execute_server_action(&handlers.open);
    let mut closed = false;

    while let Some(message) = socket.next().await {
        match message {
            Ok(Message::Ping(payload)) => {
                if socket.send(Message::Pong(payload)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                execute_server_action(&handlers.close);
                closed = true;
                break;
            }
            Ok(Message::Text(text)) => {
                if execute_websocket_action(&mut socket, &project, &handlers.message, text.as_str())
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Ok(Message::Binary(payload)) => {
                let text = String::from_utf8_lossy(&payload);
                if execute_websocket_action(&mut socket, &project, &handlers.message, &text)
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }

    if !closed {
        execute_server_action(&handlers.close);
    }
}

pub(crate) fn websocket_response(
    upgrade: WebSocketUpgrade,
    project: CompiledProject,
    handlers: WebSocketHandlers,
) -> Response {
    upgrade
        .on_upgrade(move |socket| handle_websocket(socket, project, handlers))
        .into_response()
}

async fn execute_websocket_action(
    socket: &mut WebSocket,
    project: &CompiledProject,
    action: &dowe_compiler::ServerAction,
    text: &str,
) -> Result<(), ()> {
    let body = Bytes::from(text.to_string());
    let params = HashMap::new();
    let mut context = StoreActionContext {
        project,
        root: &project.root,
        params: &params,
        body: &body,
        request_body: None,
        bindings: HashMap::new(),
        http_results: HashMap::new(),
        handles: HashMap::new(),
        kv_handles: HashMap::new(),
        handle_databases: HashMap::new(),
    };
    for statement in &action.statements {
        match statement {
            ServerStatement::WebSocketSendJson(statement) => {
                send_websocket_json_statement(socket, &context, statement).await?;
            }
            ServerStatement::WebSocketSseBridge(statement) => {
                bridge_websocket_sse(socket, &mut context, statement).await?;
            }
            _ => {
                if let Err(error) = context.execute_statement(statement).await {
                    send_ws_error(socket, None, None, None, error.code, error.message).await?;
                }
            }
        }
    }
    Ok(())
}

async fn send_websocket_json_statement(
    socket: &mut WebSocket,
    context: &StoreActionContext<'_>,
    statement: &WebSocketSendJsonStatement,
) -> Result<(), ()> {
    match context.evaluate(&statement.value) {
        Ok(ResolvedValue::Json(value)) => send_ws_value(socket, value).await,
        Ok(ResolvedValue::Missing) => {
            send_ws_error(
                socket,
                None,
                None,
                None,
                "invalid_response",
                "WebSocket response value is missing",
            )
            .await
        }
        Err(error) => send_ws_error(socket, None, None, None, error.code, error.message).await,
    }
}

async fn bridge_websocket_sse(
    socket: &mut WebSocket,
    context: &mut StoreActionContext<'_>,
    statement: &WebSocketSseBridgeStatement,
) -> Result<(), ()> {
    let request_id = resolved_text(context, &statement.request_id, "unknown");
    let request_type = resolved_text(context, &statement.request_type, "clarify");
    let model = resolved_text(context, &statement.model, "");
    let Some(result) = context.http_results.remove(&statement.upstream) else {
        return send_ws_error(
            socket,
            Some(&request_id),
            Some(&request_type),
            Some(&model),
            "invalid_response",
            "HTTP response binding is missing",
        )
        .await;
    };
    match result {
        HttpActionResult::Buffered { status, body, .. } if status.is_success() => {
            send_ws_event(
                socket,
                "message",
                &request_id,
                &request_type,
                &model,
                body,
                None,
            )
            .await?;
            send_ws_done(socket, &request_id, &request_type, &model).await
        }
        HttpActionResult::Buffered { body, .. } => {
            let message = body.to_string();
            send_ws_error(
                socket,
                Some(&request_id),
                Some(&request_type),
                Some(&model),
                "openrouter_error",
                &message,
            )
            .await
        }
        HttpActionResult::Proxy(response) => {
            bridge_websocket_response(socket, response, &request_id, &request_type, &model).await
        }
    }
}

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
        request_type.unwrap_or("clarify"),
        model.unwrap_or(""),
        Value::Object(payload),
        None,
    )
    .await
}

async fn send_ws_event(
    socket: &mut WebSocket,
    event: &str,
    request_id: &str,
    request_type: &str,
    model: &str,
    payload: Value,
    content: Option<String>,
) -> Result<(), ()> {
    let mut output = Map::new();
    output.insert("event".to_string(), Value::String(event.to_string()));
    output.insert(
        "requestId".to_string(),
        Value::String(request_id.to_string()),
    );
    output.insert(
        "requestType".to_string(),
        Value::String(request_type.to_string()),
    );
    output.insert("model".to_string(), Value::String(model.to_string()));
    output.insert("payload".to_string(), payload);
    if let Some(content) = content {
        output.insert("content".to_string(), Value::String(content));
    }
    send_ws_value(socket, Value::Object(output)).await
}

async fn send_ws_value(socket: &mut WebSocket, value: Value) -> Result<(), ()> {
    socket
        .send(Message::Text(value.to_string().into()))
        .await
        .map_err(|_| ())
}

fn done_payload() -> Value {
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(true));
    Value::Object(output)
}
