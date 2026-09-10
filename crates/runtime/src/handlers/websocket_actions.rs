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
    project: Arc<CompiledProject>,
    handlers: WebSocketHandlers,
    params: HashMap<String, String>,
    request_context: HashMap<String, Value>,
    headers: HeaderMap,
    cache_mode: CacheRuntimeMode,
) {
    crate::background_jobs::launch_task_statements(&project.root, &handlers.open, cache_mode);
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
                crate::background_jobs::launch_task_statements(
                    &project.root,
                    &handlers.close,
                    cache_mode,
                );
                execute_server_action(&handlers.close);
                closed = true;
                break;
            }
            Ok(Message::Text(text)) => {
                if execute_websocket_action(
                    &mut socket,
                    &project,
                    &handlers.message,
                    text.as_str(),
                    &params,
                    &request_context,
                    &headers,
                    cache_mode,
                )
                .await
                .is_err()
                {
                    break;
                }
            }
            Ok(Message::Binary(payload)) => {
                let text = String::from_utf8_lossy(&payload);
                if execute_websocket_action(
                    &mut socket,
                    &project,
                    &handlers.message,
                    &text,
                    &params,
                    &request_context,
                    &headers,
                    cache_mode,
                )
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
        crate::background_jobs::launch_task_statements(&project.root, &handlers.close, cache_mode);
        execute_server_action(&handlers.close);
    }
}

pub(crate) fn websocket_response(
    upgrade: WebSocketUpgrade,
    project: Arc<CompiledProject>,
    handlers: WebSocketHandlers,
    params: HashMap<String, String>,
    request_context: HashMap<String, Value>,
    headers: HeaderMap,
    cache_mode: CacheRuntimeMode,
) -> Response {
    upgrade
        .on_upgrade(move |socket| {
            handle_websocket(
                socket,
                project,
                handlers,
                params,
                request_context,
                headers,
                cache_mode,
            )
        })
        .into_response()
}

async fn execute_websocket_action(
    socket: &mut WebSocket,
    project: &CompiledProject,
    action: &dowe_compiler::ServerAction,
    text: &str,
    params: &HashMap<String, String>,
    request_context: &HashMap<String, Value>,
    headers: &HeaderMap,
    cache_mode: CacheRuntimeMode,
) -> Result<(), ()> {
    let body = Bytes::from(text.to_string());
    let mut context = StoreActionContext {
        project,
        root: &project.root,
        params: &params,
        body: &body,
        raw_query: None,
        headers: Some(headers),
        request_context: Some(request_context),
        request_body: None,
        bindings: HashMap::new(),
        http_results: HashMap::new(),
        bytes_results: HashMap::new(),
        handles: HashMap::new(),
        kv_handles: HashMap::new(),
        vector_handles: HashMap::new(),
        queue_handles: HashMap::new(),
        handle_databases: HashMap::new(),
        cache_mode,
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
                    let request_id = resolved_text(&context, "event.requestId", "unknown");
                    let request_type = resolved_text(&context, "event.requestType", "");
                    let model = resolved_text(&context, "event.model", "");
                    send_ws_error(
                        socket,
                        Some(&request_id),
                        Some(&request_type),
                        Some(&model),
                        error.code,
                        error.message,
                    )
                    .await?;
                    return Err(());
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

