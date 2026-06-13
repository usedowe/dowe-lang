use crate::server::DevRuntimeState;
use crate::server_actions::{
    execute_resolved_log, execute_server_action, execute_server_action_with_resolver,
};
use axum::body::{Body, Bytes};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::header::{
    ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, CACHE_CONTROL, CONTENT_TYPE,
    ORIGIN, VARY,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use dowe_compiler::{
    AgentResponseEndpoint, CompiledProject, CorsConfig, DoweType, EndpointBehavior,
    HttpActionJsonEndpoint, HttpConnectionValue, HttpMethod, HttpProxyEndpoint, HttpResponseMode,
    KvActionJsonEndpoint, KvConnectionValue, KvCredential, KvRemoteConnection, OutboundHttpRequest,
    ServerConfig, ServerKvStatement, ServerMiddleware, ServerMiddlewareResponseBody,
    ServerMiddlewareStatement, ServerSecret, ServerStatement, ServerStoreStatement,
    StoreActionJsonEndpoint, StoreConnection, StoreConnectionValue, StoreCredential, StoreFilter,
    StoreLiteral, StoreRemoteConnection, StoreTransactionEndpoint, StoreTransactionOperation,
    ViewPage, WebOutput, WebSocketHandlers, WebSocketSendJsonStatement,
    WebSocketSseBridgeStatement, normalize_cors_method, normalize_http_header_name,
};
use dowe_crypto::{
    JwtValidationOptions, decrypt_jwe_dir_a256gcm, encrypt_jwe_dir_a256gcm, sign_jws_hs256,
    verify_jws_hs256,
};
use dowe_kv::{KvDatabase, RemoteKvClient, RemoteKvConfig, open_database as open_kv_database};
use dowe_store::{
    Database, RemoteStoreClient, RemoteStoreConfig, StoreRecord, StoreValue, init_database,
    open_database,
};
use futures_util::StreamExt;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn backend_handler(
    State(state): State<DevRuntimeState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let project = state.project.read().await;
    server_response(
        &project,
        &project.backend,
        &state.dev_origins,
        method,
        uri.path(),
        headers,
        body,
    )
    .await
}

pub async fn backend_declared_websocket_handler(
    state: DevRuntimeState,
    upgrade: WebSocketUpgrade,
    path: String,
) -> Response {
    let project = state.project.read().await;
    let Some(route) = project.backend.find_websocket(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    websocket_response(upgrade, project.clone(), route.handlers)
}

pub async fn desktop_handler(
    State(state): State<DevRuntimeState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let project = state.project.read().await;
    if let Some(server) = &project.desktop_server
        && (server.has_endpoint_path(uri.path())
            || method == Method::OPTIONS && is_preflight(&headers))
    {
        return server_response(
            &project,
            server,
            &state.dev_origins,
            method,
            uri.path(),
            headers,
            body,
        )
        .await;
    }

    if method == Method::GET {
        desktop_static_response(&project, uri.path())
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn desktop_declared_websocket_handler(
    state: DevRuntimeState,
    upgrade: WebSocketUpgrade,
    path: String,
) -> Response {
    let project = state.project.read().await;
    let Some(server) = &project.desktop_server else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(route) = server.find_websocket(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    websocket_response(upgrade, project.clone(), route.handlers)
}

pub(crate) async fn server_response(
    project: &CompiledProject,
    server: &ServerConfig,
    dev_origins: &[String],
    method: Method,
    path: &str,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if method == Method::OPTIONS && is_preflight(&headers) && server.cors.enabled {
        return cors_preflight_response(server, dev_origins, path, &headers);
    }

    let response = match HttpMethod::from_str(method.as_str()) {
        Ok(method) => match server.find_endpoint(&method, path) {
            Some(matched) => {
                let middleware_context =
                    match execute_middlewares(project, &matched.endpoint.middlewares, &headers) {
                        MiddlewareFlow::Continue(context) => context,
                        MiddlewareFlow::Respond(response) => {
                            return cors_actual_response(
                                &server.cors,
                                dev_origins,
                                &headers,
                                response,
                            );
                        }
                    };
                if !matches!(
                    &matched.endpoint.behavior,
                    EndpointBehavior::HttpProxy(_)
                        | EndpointBehavior::HttpActionJson(_)
                        | EndpointBehavior::AgentResponse(_)
                        | EndpointBehavior::StoreActionJson(_)
                        | EndpointBehavior::KvActionJson(_)
                ) {
                    execute_server_action_with_resolver(&matched.endpoint.action, |reference| {
                        resolve_request_reference(reference, &matched.params, &middleware_context)
                            .map(log_json_text)
                    });
                }

                match matched.endpoint.behavior {
                    EndpointBehavior::StaticText(text) => text_response(StatusCode::OK, text),
                    EndpointBehavior::TextTemplate(text) => text_response(
                        StatusCode::OK,
                        render_text_template(&text, &matched.params, &middleware_context),
                    ),
                    EndpointBehavior::UserGreeting => {
                        let id = matched.params.get("id").cloned().unwrap_or_default();
                        text_response(StatusCode::OK, format!("Hello User {id}!"))
                    }
                    EndpointBehavior::CreatePostJson => created_json_response(&body),
                    EndpointBehavior::HttpProxy(response) => {
                        execute_http_proxy(
                            project,
                            &project.root,
                            &matched.endpoint.action,
                            &response,
                            &matched.params,
                            &body,
                        )
                        .await
                    }
                    EndpointBehavior::HttpActionJson(response) => {
                        execute_http_action_json(
                            project,
                            &project.root,
                            &matched.endpoint.action,
                            &response,
                            &matched.params,
                            &body,
                        )
                        .await
                    }
                    EndpointBehavior::AgentResponse(response) => {
                        execute_agent_response(
                            project,
                            &project.root,
                            &matched.endpoint.action,
                            &response,
                            &matched.params,
                            &body,
                        )
                        .await
                    }
                    EndpointBehavior::StoreInsertJson(insert) => {
                        match execute_store_insert(
                            project,
                            &insert.connection,
                            &insert.table,
                            &insert.value,
                        )
                        .await
                        {
                            Ok(value) => json_response(StatusCode::OK, value),
                            Err(error) => store_error_response(error),
                        }
                    }
                    EndpointBehavior::StoreQueryJson(query) => {
                        match execute_store_query(project, &query.connection, &query.sql).await {
                            Ok(value) => json_response(StatusCode::OK, value),
                            Err(error) => store_error_response(error),
                        }
                    }
                    EndpointBehavior::StoreTransactionJson(transaction) => {
                        match execute_store_transaction(&project.root, &transaction) {
                            Ok(value) => json_response(StatusCode::OK, value),
                            Err(error) => store_error_response(error),
                        }
                    }
                    EndpointBehavior::StoreActionJson(response) => {
                        execute_store_action_json(
                            project,
                            &project.root,
                            &matched.endpoint.action,
                            &response,
                            &matched.params,
                            &body,
                        )
                        .await
                    }
                    EndpointBehavior::KvActionJson(response) => {
                        execute_kv_action_json(
                            project,
                            &project.root,
                            &matched.endpoint.action,
                            &response,
                            &matched.params,
                            &body,
                        )
                        .await
                    }
                }
            }
            None => {
                if server.has_endpoint_path(path) {
                    StatusCode::METHOD_NOT_ALLOWED.into_response()
                } else {
                    StatusCode::NOT_FOUND.into_response()
                }
            }
        },
        Err(_) => StatusCode::METHOD_NOT_ALLOWED.into_response(),
    };

    cors_actual_response(&server.cors, dev_origins, &headers, response)
}

pub async fn views_handler(State(state): State<DevRuntimeState>, uri: Uri) -> Response {
    if uri.path() == "/_dowe/dev/ws" {
        return StatusCode::BAD_REQUEST.into_response();
    }

    if uri.path() == "/_dowe/dev/client.js" {
        return dev_client_response();
    }

    let project = state.project.read().await;

    if uri.path() == "/design.css" {
        return design_css_response(&project, "web/design.css");
    }

    if uri.path() == "/router.js" {
        return javascript_response(project.web.router_js.clone());
    }

    if uri.path() == "/env.json" {
        return json_response_text(project.environment_config.client_json());
    }

    if let Some(response) = font_response(&project, uri.path()) {
        return response;
    }

    if let Some(response) = chunk_response(&project.web, uri.path()) {
        return response;
    }

    if let Some(page) = project
        .web
        .pages
        .iter()
        .find(|page| page.route_path == uri.path())
    {
        return render_page(page);
    }

    StatusCode::NOT_FOUND.into_response()
}

fn desktop_static_response(project: &CompiledProject, path: &str) -> Response {
    if path == "/_dowe/dev/client.js" {
        return dev_client_response();
    }
    if path == "/design.css" {
        return design_css_response(project, "apps/desktop/web/design.css");
    }
    if path == "/router.js" {
        return javascript_response(project.desktop_web.router_js.clone());
    }
    if path == "/env.json" {
        return json_response_text(project.environment_config.client_json());
    }
    if let Some(response) = font_response(project, path) {
        return response;
    }
    if let Some(response) = chunk_response(&project.desktop_web, path) {
        return response;
    }
    let Some(page) = project
        .desktop_web
        .pages
        .iter()
        .find(|page| page.route_path == path)
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let html_path = if page.route_path == "/" {
        project.root.join(".dowe/apps/desktop/web/index.html")
    } else {
        let file_name = page.route_path.trim_matches('/').replace('/', "-");
        project
            .root
            .join(".dowe/apps/desktop/web/pages")
            .join(format!("{file_name}.html"))
    };
    match fs::read_to_string(html_path) {
        Ok(html) => Html(inject_dev_client(&html)).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

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

enum MiddlewareFlow {
    Continue(HashMap<String, Value>),
    Respond(Response),
}

enum MiddlewareStep {
    Continue,
    Return(MiddlewareFlow),
}

fn execute_middlewares(
    project: &CompiledProject,
    middlewares: &[ServerMiddleware],
    headers: &HeaderMap,
) -> MiddlewareFlow {
    let mut request_context = HashMap::new();
    for middleware in middlewares {
        let mut execution = MiddlewareExecution {
            project,
            headers,
            request_context: &mut request_context,
            bindings: HashMap::new(),
        };
        match execution.execute(&middleware.action.statements) {
            Ok(MiddlewareFlow::Continue(context)) => request_context = context,
            Ok(MiddlewareFlow::Respond(response)) => return MiddlewareFlow::Respond(response),
            Err(error) => return MiddlewareFlow::Respond(error),
        }
    }
    MiddlewareFlow::Continue(request_context)
}

struct MiddlewareExecution<'a> {
    project: &'a CompiledProject,
    headers: &'a HeaderMap,
    request_context: &'a mut HashMap<String, Value>,
    bindings: HashMap<String, Value>,
}

impl<'a> MiddlewareExecution<'a> {
    fn execute(
        &mut self,
        statements: &[ServerMiddlewareStatement],
    ) -> Result<MiddlewareFlow, Response> {
        for statement in statements {
            match self.execute_statement(statement)? {
                MiddlewareStep::Continue => {}
                MiddlewareStep::Return(flow) => return Ok(flow),
            }
        }
        Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "middleware_error",
            "Middleware did not return a result",
        ))
    }

    fn execute_statement(
        &mut self,
        statement: &ServerMiddlewareStatement,
    ) -> Result<MiddlewareStep, Response> {
        match statement {
            ServerMiddlewareStatement::Log(log) => {
                execute_resolved_log(log, |reference| {
                    self.resolve_reference(reference).map(log_json_text)
                });
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::Header { binding, name } => {
                let value = self
                    .headers
                    .get(name.as_str())
                    .and_then(|value| value.to_str().ok())
                    .map(|value| Value::String(value.to_string()))
                    .unwrap_or(Value::Null);
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::Bearer { binding, source } => {
                let value = self
                    .resolve_reference(source)
                    .and_then(|value| value.as_str().map(parse_bearer_token))
                    .flatten()
                    .map(Value::String)
                    .unwrap_or(Value::Null);
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::JwtVerify {
                binding,
                token,
                secret,
                algorithm,
            } => {
                let token = self.resolve_string(token);
                let secret = self.secret_value(secret)?;
                let value = if algorithm == "HS256" {
                    match token.as_deref().and_then(|token| {
                        verify_jws_hs256(token, &secret, &JwtValidationOptions::default()).ok()
                    }) {
                        Some(claims) => jwt_result(true, Some(claims)),
                        None => jwt_result(false, None),
                    }
                } else {
                    jwt_result(false, None)
                };
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::JwtDecrypt {
                binding,
                token,
                key,
                algorithm,
                encryption,
            } => {
                let token = self.resolve_string(token);
                let key = self.secret_value(key)?;
                let value = if algorithm == "dir" && encryption == "A256GCM" {
                    match token.as_deref().and_then(|token| {
                        decrypt_jwe_dir_a256gcm(token, &key, &JwtValidationOptions::default()).ok()
                    }) {
                        Some(claims) => jwt_result(true, Some(claims)),
                        None => jwt_result(false, None),
                    }
                } else {
                    jwt_result(false, None)
                };
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::JwtSign {
                binding,
                claims,
                secret,
                algorithm,
            } => {
                let secret = self.secret_value(secret)?;
                let claims = self.evaluate(claims);
                let value = if algorithm == "HS256" {
                    sign_jws_hs256(&claims, &secret)
                        .map(Value::String)
                        .unwrap_or(Value::Null)
                } else {
                    Value::Null
                };
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::JwtEncrypt {
                binding,
                claims,
                key,
                algorithm,
                encryption,
            } => {
                let key = self.secret_value(key)?;
                let claims = self.evaluate(claims);
                let value = if algorithm == "dir" && encryption == "A256GCM" {
                    encrypt_jwe_dir_a256gcm(&claims, &key)
                        .map(Value::String)
                        .unwrap_or(Value::Null)
                } else {
                    Value::Null
                };
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::IfValid {
                binding,
                statements,
            } => {
                let valid = self
                    .bindings
                    .get(binding)
                    .and_then(|value| value.get("valid"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if valid {
                    match self.execute(statements)? {
                        MiddlewareFlow::Continue(context) => {
                            return Ok(MiddlewareStep::Return(MiddlewareFlow::Continue(context)));
                        }
                        MiddlewareFlow::Respond(response) => {
                            return Ok(MiddlewareStep::Return(MiddlewareFlow::Respond(response)));
                        }
                    }
                }
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::Continue { context } => {
                if let Some(context) = context {
                    if let Value::Object(values) = self.evaluate(context) {
                        for (key, value) in values {
                            self.request_context.insert(key, value);
                        }
                    }
                }
                Ok(MiddlewareStep::Return(MiddlewareFlow::Continue(
                    self.request_context.clone(),
                )))
            }
            ServerMiddlewareStatement::Response { status, body } => {
                let response = match body {
                    ServerMiddlewareResponseBody::Text(value) => {
                        text_response(status_from_u16(*status), value.clone())
                    }
                    ServerMiddlewareResponseBody::Json(value) => {
                        json_response(status_from_u16(*status), self.evaluate(value))
                    }
                };
                Ok(MiddlewareStep::Return(MiddlewareFlow::Respond(response)))
            }
        }
    }

    fn resolve_string(&self, reference: &str) -> Option<String> {
        self.resolve_reference(reference)
            .and_then(|value| value.as_str().map(str::to_string))
    }

    fn resolve_reference(&self, reference: &str) -> Option<Value> {
        if let Some(value) = self.bindings.get(reference) {
            return Some(value.clone());
        }
        if let Some(path) = reference.strip_prefix("req.context.") {
            return read_context_path(self.request_context, path).cloned();
        }
        if let Some((root, path)) = reference.split_once('.')
            && let Some(value) = self.bindings.get(root)
        {
            return read_json_path(value, path).cloned();
        }
        if let Some(env_name) = reference.strip_prefix("env.") {
            return self.env_value(env_name).map(Value::String);
        }
        None
    }

    fn evaluate(&self, value: &StoreLiteral) -> Value {
        match value {
            StoreLiteral::Null => Value::Null,
            StoreLiteral::Bool(value) => Value::Bool(*value),
            StoreLiteral::Number(value) => number_json(value),
            StoreLiteral::String(value) => Value::String(value.clone()),
            StoreLiteral::Reference(reference) => self
                .resolve_reference(reference)
                .unwrap_or_else(|| Value::String(reference.clone())),
            StoreLiteral::Array(values) => {
                Value::Array(values.iter().map(|value| self.evaluate(value)).collect())
            }
            StoreLiteral::Object(entries) => Value::Object(
                entries
                    .iter()
                    .map(|(key, value)| (key.clone(), self.evaluate(value)))
                    .collect(),
            ),
        }
    }

    fn secret_value(&self, secret: &ServerSecret) -> Result<String, Response> {
        match secret {
            ServerSecret::Environment(name) => self.env_value(name).ok_or_else(|| {
                json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "jwt_secret_missing",
                    "JWT secret is not configured",
                )
            }),
        }
    }

    fn env_value(&self, name: &str) -> Option<String> {
        self.project
            .environment_config
            .variable(name)
            .and_then(|variable| variable.resolved_value.clone())
    }
}

fn parse_bearer_token(value: &str) -> Option<String> {
    let mut parts = value.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;
    if parts.next().is_some() || token.is_empty() || !scheme.eq_ignore_ascii_case("Bearer") {
        return None;
    }
    Some(token.to_string())
}

fn jwt_result(valid: bool, claims: Option<Value>) -> Value {
    let mut output = Map::new();
    output.insert("valid".to_string(), Value::Bool(valid));
    if let Some(claims) = claims {
        output.insert("claims".to_string(), claims);
    }
    Value::Object(output)
}

fn read_context_path<'a>(context: &'a HashMap<String, Value>, path: &str) -> Option<&'a Value> {
    let (root, rest) = path.split_once('.').unwrap_or((path, ""));
    let value = context.get(root)?;
    if rest.is_empty() {
        Some(value)
    } else {
        read_json_path(value, rest)
    }
}

fn resolve_request_reference(
    reference: &str,
    params: &HashMap<String, String>,
    context: &HashMap<String, Value>,
) -> Option<Value> {
    if let Some(name) = reference.strip_prefix("req.params.") {
        return params.get(name).map(|value| Value::String(value.clone()));
    }
    if let Some(path) = reference.strip_prefix("req.context.") {
        return read_context_path(context, path).cloned();
    }
    None
}

fn render_text_template(
    template: &str,
    params: &HashMap<String, String>,
    context: &HashMap<String, Value>,
) -> String {
    let mut output = template.to_string();
    for (key, value) in params {
        output = output.replace(&format!("{{req.params.{key}}}"), value);
    }
    replace_context_tokens(output, context)
}

fn replace_context_tokens(mut output: String, context: &HashMap<String, Value>) -> String {
    while let Some(start) = output.find("{req.context.") {
        let Some(relative_end) = output[start..].find('}') else {
            break;
        };
        let end = start + relative_end;
        let token = &output[start + "{req.context.".len()..end];
        let replacement = read_context_path(context, token)
            .map(|value| match value {
                Value::String(value) => value.clone(),
                value => value.to_string(),
            })
            .unwrap_or_default();
        output.replace_range(start..=end, &replacement);
    }
    output
}

pub(crate) fn is_preflight(headers: &HeaderMap) -> bool {
    headers.contains_key(ORIGIN) && headers.contains_key(ACCESS_CONTROL_REQUEST_METHOD)
}

fn cors_preflight_response(
    server: &ServerConfig,
    dev_origins: &[String],
    path: &str,
    headers: &HeaderMap,
) -> Response {
    let cors = &server.cors;
    let Some(origin) = origin_header(headers) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(allowed_origin) = cors.allowed_origin(origin, dev_origins) else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let Some(requested_method) = request_method_header(headers) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(requested_method) = normalize_cors_method(requested_method) else {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    };
    let route_methods = server.methods_for_path(path);
    if route_methods.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }
    if !route_methods
        .iter()
        .any(|method| method.as_str() == requested_method)
        || !cors.allows_method(requested_method)
    {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let Some(requested_headers) = requested_header_names(headers) else {
        return StatusCode::FORBIDDEN.into_response();
    };
    if !cors.allows_headers(&requested_headers) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let mut response = StatusCode::NO_CONTENT.into_response();
    apply_cors_base_headers(cors, &allowed_origin, &mut response);
    insert_header(
        &mut response,
        "access-control-allow-methods",
        &allow_methods_value(cors, &route_methods),
    );
    insert_header(
        &mut response,
        "access-control-allow-headers",
        &allow_headers_value(cors, &requested_headers),
    );
    if let Some(max_age) = cors.max_age {
        insert_header(
            &mut response,
            "access-control-max-age",
            &max_age.to_string(),
        );
    }
    response
}

fn cors_actual_response(
    cors: &CorsConfig,
    dev_origins: &[String],
    headers: &HeaderMap,
    mut response: Response,
) -> Response {
    let Some(origin) = origin_header(headers) else {
        return response;
    };
    let Some(allowed_origin) = cors.allowed_origin(origin, dev_origins) else {
        return response;
    };
    apply_cors_base_headers(cors, &allowed_origin, &mut response);
    if !cors.expose_headers.is_empty() {
        insert_header(
            &mut response,
            "access-control-expose-headers",
            &cors.expose_headers.join(", "),
        );
    }
    response
}

fn apply_cors_base_headers(cors: &CorsConfig, allowed_origin: &str, response: &mut Response) {
    insert_header(response, "access-control-allow-origin", allowed_origin);
    if cors.credentials {
        insert_header(response, "access-control-allow-credentials", "true");
    }
    if allowed_origin != "*" {
        append_vary_origin(response);
    }
}

fn origin_header(headers: &HeaderMap) -> Option<&str> {
    headers.get(ORIGIN).and_then(|value| value.to_str().ok())
}

fn request_method_header(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(ACCESS_CONTROL_REQUEST_METHOD)
        .and_then(|value| value.to_str().ok())
}

fn requested_header_names(headers: &HeaderMap) -> Option<Vec<String>> {
    let Some(value) = headers.get(ACCESS_CONTROL_REQUEST_HEADERS) else {
        return Some(Vec::new());
    };
    let value = value.to_str().ok()?;
    let mut output = Vec::new();
    for header in value.split(',') {
        let header = header.trim();
        if header.is_empty() {
            continue;
        }
        output.push(normalize_http_header_name(header)?);
    }
    Some(output)
}

fn allow_methods_value(cors: &CorsConfig, route_methods: &[HttpMethod]) -> String {
    route_methods
        .iter()
        .filter_map(|method| {
            let method = method.as_str();
            if cors.allows_method(method) {
                Some(method)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn allow_headers_value(cors: &CorsConfig, requested_headers: &[String]) -> String {
    if requested_headers.is_empty() {
        cors.headers.join(", ")
    } else {
        requested_headers.join(", ")
    }
}

fn insert_header(response: &mut Response, name: &'static str, value: &str) {
    if let Ok(value) = HeaderValue::from_str(value) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(name), value);
    }
}

fn append_vary_origin(response: &mut Response) {
    let next = match response
        .headers()
        .get(VARY)
        .and_then(|value| value.to_str().ok())
    {
        Some(current)
            if current
                .split(',')
                .any(|value| value.trim().eq_ignore_ascii_case("origin")) =>
        {
            return;
        }
        Some(current) if !current.is_empty() => format!("{current}, Origin"),
        _ => "Origin".to_string(),
    };
    insert_header(response, "vary", &next);
}

fn text_response(status: StatusCode, body: String) -> Response {
    (status, [(CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}

fn created_json_response(body: &Bytes) -> Response {
    let Ok(value) = serde_json::from_slice::<Value>(body) else {
        return text_response(StatusCode::BAD_REQUEST, "Invalid JSON".to_string());
    };

    let Some(input) = value.as_object() else {
        return text_response(StatusCode::BAD_REQUEST, "Expected JSON object".to_string());
    };

    let mut output = Map::new();
    output.insert("created".to_string(), Value::Bool(true));

    for (key, value) in input {
        output.insert(key.clone(), value.clone());
    }

    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json; charset=utf-8")],
        Value::Object(output).to_string(),
    )
        .into_response()
}

async fn execute_store_insert(
    project: &CompiledProject,
    connection: &StoreConnection,
    table: &str,
    value: &StoreLiteral,
) -> dowe_store::StoreResult<Value> {
    let record = literal_record(value);
    if let Some(client) = remote_client_for_connection(project, connection)? {
        return client.insert(table, record_json(&record)).await;
    }
    init_database(&project.root, &connection.database)?;
    let database = open_database(&project.root, &connection.database)?;
    Ok(record_json(&database.insert(table, record)?))
}

async fn execute_store_query(
    project: &CompiledProject,
    connection: &StoreConnection,
    sql: &str,
) -> dowe_store::StoreResult<Value> {
    if let Some(client) = remote_client_for_connection(project, connection)? {
        return client.query(sql).await;
    }
    init_database(&project.root, &connection.database)?;
    let database = open_database(&project.root, &connection.database)?;
    database.query_json(sql)
}

fn remote_client_for_connection(
    project: &CompiledProject,
    connection: &StoreConnection,
) -> dowe_store::StoreResult<Option<RemoteStoreClient>> {
    let Some(remote) = &connection.remote else {
        return Ok(None);
    };
    let credential = match &remote.credential {
        StoreCredential::Token(value) | StoreCredential::Password(value) => {
            connection_value(project, value)?
        }
    };
    Ok(Some(RemoteStoreClient::new(RemoteStoreConfig {
        host: connection_value(project, &remote.host)?,
        database: connection.database.clone(),
        user: connection_value(project, &remote.user)?,
        credential,
    })?))
}

fn connection_value(
    project: &CompiledProject,
    value: &StoreConnectionValue,
) -> dowe_store::StoreResult<String> {
    match value {
        StoreConnectionValue::Static(value) => Ok(value.clone()),
        StoreConnectionValue::Environment(name) => project
            .environment_config
            .variable(name)
            .and_then(|variable| variable.resolved_value.clone())
            .ok_or_else(|| {
                dowe_store::StoreError::Remote(format!(
                    "Store environment variable `{name}` is not configured"
                ))
            }),
    }
}

async fn execute_store_action_json(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &StoreActionJsonEndpoint,
    params: &HashMap<String, String>,
    body: &Bytes,
) -> Response {
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        request_body: None,
        bindings: HashMap::new(),
        http_results: HashMap::new(),
        handles: HashMap::new(),
        kv_handles: HashMap::new(),
        handle_databases: HashMap::new(),
    };
    match context
        .execute(action)
        .await
        .and_then(|_| context.evaluate(&response.value))
    {
        Ok(ResolvedValue::Json(value)) => json_response(status_from_u16(response.status), value),
        Ok(ResolvedValue::Missing) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "invalid_response",
            "Response value is missing",
        ),
        Err(error) => json_error(error.status, error.code, error.message),
    }
}

async fn execute_kv_action_json(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &KvActionJsonEndpoint,
    params: &HashMap<String, String>,
    body: &Bytes,
) -> Response {
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        request_body: None,
        bindings: HashMap::new(),
        http_results: HashMap::new(),
        handles: HashMap::new(),
        kv_handles: HashMap::new(),
        handle_databases: HashMap::new(),
    };
    match context
        .execute(action)
        .await
        .and_then(|_| context.evaluate(&response.value))
    {
        Ok(ResolvedValue::Json(value)) => json_response(status_from_u16(response.status), value),
        Ok(ResolvedValue::Missing) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "invalid_response",
            "Response value is missing",
        ),
        Err(error) => json_error(error.status, error.code, error.message),
    }
}

async fn execute_http_action_json(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &HttpActionJsonEndpoint,
    params: &HashMap<String, String>,
    body: &Bytes,
) -> Response {
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        request_body: None,
        bindings: HashMap::new(),
        http_results: HashMap::new(),
        handles: HashMap::new(),
        kv_handles: HashMap::new(),
        handle_databases: HashMap::new(),
    };
    match context
        .execute(action)
        .await
        .and_then(|_| context.evaluate(&response.value))
    {
        Ok(ResolvedValue::Json(value)) => json_response(status_from_u16(response.status), value),
        Ok(ResolvedValue::Missing) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "invalid_response",
            "Response value is missing",
        ),
        Err(error) => json_error(error.status, error.code, error.message),
    }
}

async fn execute_http_proxy(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &HttpProxyEndpoint,
    params: &HashMap<String, String>,
    body: &Bytes,
) -> Response {
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        request_body: None,
        bindings: HashMap::new(),
        http_results: HashMap::new(),
        handles: HashMap::new(),
        kv_handles: HashMap::new(),
        handle_databases: HashMap::new(),
    };
    match context.execute(action).await {
        Ok(()) => match context.http_results.remove(&response.binding) {
            Some(result) => http_result_response(result).await,
            None => json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid_response",
                "HTTP response binding is missing",
            ),
        },
        Err(error) => json_error(error.status, error.code, error.message),
    }
}

async fn execute_agent_response(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &AgentResponseEndpoint,
    params: &HashMap<String, String>,
    body: &Bytes,
) -> Response {
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        request_body: None,
        bindings: HashMap::new(),
        http_results: HashMap::new(),
        handles: HashMap::new(),
        kv_handles: HashMap::new(),
        handle_databases: HashMap::new(),
    };
    for statement in &action.statements {
        if matches!(statement, ServerStatement::Http(_))
            && request_stream_enabled(context.resolve_reference(&response.request).into_json())
        {
            return json_error(
                StatusCode::BAD_REQUEST,
                "agent_http_stream_unsupported",
                "Use /api/v1/agent/ws for Dowe Agent streaming requests.",
            );
        }
        if let Err(error) = context.execute_statement(statement).await {
            return json_error(error.status, error.code, error.message);
        }
    }
    let request = context
        .resolve_reference(&response.request)
        .into_json()
        .unwrap_or(Value::Null);
    match context.http_results.remove(&response.upstream) {
        Some(HttpActionResult::Buffered { status, body, .. }) if status.is_success() => {
            json_response(StatusCode::OK, agent_http_success(request, body))
        }
        Some(HttpActionResult::Buffered { status, body, .. }) => {
            json_response(status, openrouter_error(body))
        }
        Some(HttpActionResult::Proxy(upstream)) => agent_proxy_response(request, upstream).await,
        None => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "invalid_response",
            "HTTP response binding is missing",
        ),
    }
}

async fn http_result_response(result: HttpActionResult) -> Response {
    match result {
        HttpActionResult::Buffered {
            status,
            content_type,
            raw,
            ..
        } => body_response(status, content_type, raw),
        HttpActionResult::Proxy(response) => {
            let status = status_from_reqwest(response.status());
            let content_type = response_content_type(&response);
            if content_type.as_deref().is_some_and(is_sse_content_type) {
                return streaming_body_response(status, content_type, response.bytes_stream());
            }
            match response.bytes().await {
                Ok(body) => body_response(status, content_type, body),
                Err(_) => json_error(
                    StatusCode::BAD_GATEWAY,
                    "http_error",
                    "Outbound HTTP response failed",
                ),
            }
        }
    }
}

async fn agent_proxy_response(request: Value, response: reqwest::Response) -> Response {
    let status = status_from_reqwest(response.status());
    match response.bytes().await {
        Ok(body) if status.is_success() => json_response(
            StatusCode::OK,
            agent_http_success(request, json_from_bytes(&body)),
        ),
        Ok(body) => json_response(status, openrouter_error(json_from_bytes(&body))),
        Err(_) => json_error(
            StatusCode::BAD_GATEWAY,
            "http_error",
            "Outbound HTTP response failed",
        ),
    }
}

fn body_response(status: StatusCode, content_type: Option<String>, body: Bytes) -> Response {
    let mut response = (status, body).into_response();
    if let Some(content_type) = content_type {
        insert_header(&mut response, "content-type", &content_type);
    }
    response
}

fn streaming_body_response(
    status: StatusCode,
    content_type: Option<String>,
    stream: impl futures_util::Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> Response {
    let mut response = Response::new(Body::from_stream(stream));
    *response.status_mut() = status;
    if let Some(content_type) = content_type {
        insert_header(&mut response, "content-type", &content_type);
    }
    response
}

fn status_from_reqwest(status: reqwest::StatusCode) -> StatusCode {
    StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::OK)
}

fn response_content_type(response: &reqwest::Response) -> Option<String> {
    response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

fn is_sse_content_type(value: &str) -> bool {
    value
        .split(';')
        .next()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("text/event-stream"))
}

fn json_from_bytes(body: &Bytes) -> Value {
    serde_json::from_slice::<Value>(body)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(body).to_string()))
}

fn http_binding_json(
    status: StatusCode,
    content_type: Option<String>,
    body: Option<Value>,
) -> Value {
    let mut output = Map::new();
    output.insert(
        "status".to_string(),
        Value::Number(u64::from(status.as_u16()).into()),
    );
    if let Some(content_type) = content_type {
        output.insert("contentType".to_string(), Value::String(content_type));
    }
    if let Some(body) = body {
        output.insert("json".to_string(), body);
    }
    Value::Object(output)
}

fn agent_chat_body(source: Value) -> Value {
    let mut object = source.as_object().cloned().unwrap_or_default();
    object.remove("requestId");
    object.remove("request_id");
    let request_type = object
        .remove("requestType")
        .or_else(|| object.remove("request_type"));
    if let Some(request_type) = request_type {
        let mut metadata = object
            .remove("metadata")
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default();
        metadata.insert("dowe_request_type".to_string(), request_type);
        object.insert("metadata".to_string(), Value::Object(metadata));
    }
    Value::Object(object)
}

fn request_stream_enabled(request: Option<Value>) -> bool {
    request
        .as_ref()
        .and_then(|value| value.get("stream"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn agent_http_success(request: Value, payload: Value) -> Value {
    let mut output = Map::new();
    output.insert(
        "requestId".to_string(),
        request_field(&request, "requestId", "request_id"),
    );
    output.insert(
        "requestType".to_string(),
        request_field(&request, "requestType", "request_type"),
    );
    output.insert(
        "model".to_string(),
        request.get("model").cloned().unwrap_or(Value::Null),
    );
    output.insert("payload".to_string(), payload);
    Value::Object(output)
}

fn request_field(request: &Value, camel: &str, snake: &str) -> Value {
    request
        .get(camel)
        .or_else(|| request.get(snake))
        .cloned()
        .unwrap_or(Value::Null)
}

fn openrouter_error(payload: Value) -> Value {
    let mut error = Map::new();
    error.insert(
        "code".to_string(),
        Value::String("openrouter_error".to_string()),
    );
    error.insert(
        "message".to_string(),
        Value::String("OpenRouter returned an error.".to_string()),
    );
    error.insert("upstream".to_string(), payload);
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(false));
    output.insert("error".to_string(), Value::Object(error));
    Value::Object(output)
}

enum StoreHandle {
    Local(Database),
    Remote(RemoteStoreClient),
}

enum KvHandle {
    Local(KvDatabase),
    Remote(RemoteKvClient),
}

struct StoreActionContext<'a> {
    project: &'a CompiledProject,
    root: &'a Path,
    params: &'a HashMap<String, String>,
    body: &'a Bytes,
    request_body: Option<Value>,
    bindings: HashMap<String, Value>,
    http_results: HashMap<String, HttpActionResult>,
    handles: HashMap<String, StoreHandle>,
    kv_handles: HashMap<String, KvHandle>,
    handle_databases: HashMap<String, String>,
}

enum HttpActionResult {
    Buffered {
        status: StatusCode,
        content_type: Option<String>,
        body: Value,
        raw: Bytes,
    },
    Proxy(reqwest::Response),
}

enum ResolvedValue {
    Json(Value),
    Missing,
}

struct StoreActionError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
}

impl StoreActionError {
    fn invalid_body(message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_body",
            message,
        }
    }

    fn not_found(message: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message,
        }
    }

    fn store() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "store_error",
            message: "Store operation failed",
        }
    }

    fn kv() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "kv_error",
            message: "KV operation failed",
        }
    }

    fn http() -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code: "http_error",
            message: "Outbound HTTP request failed",
        }
    }

    fn missing_http() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "invalid_response",
            message: "HTTP response binding is missing",
        }
    }

    fn from_store(error: dowe_store::StoreError) -> Self {
        match error {
            dowe_store::StoreError::Authentication(_) => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "store_authentication",
                message: "Store authentication failed",
            },
            dowe_store::StoreError::Authorization(_) => Self {
                status: StatusCode::FORBIDDEN,
                code: "store_authorization",
                message: "Store authorization failed",
            },
            dowe_store::StoreError::NotFound(_) => Self::not_found("Record not found"),
            dowe_store::StoreError::AlreadyExists(_)
            | dowe_store::StoreError::TransactionConflict(_) => Self {
                status: StatusCode::CONFLICT,
                code: "store_conflict",
                message: "Store operation conflicted",
            },
            dowe_store::StoreError::InvalidName(_) | dowe_store::StoreError::InvalidQuery(_) => {
                Self {
                    status: StatusCode::BAD_REQUEST,
                    code: "store_invalid_request",
                    message: "Store request is invalid",
                }
            }
            _ => Self::store(),
        }
    }

    fn from_kv(error: dowe_kv::KvError) -> Self {
        match error {
            dowe_kv::KvError::Authentication(_) => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "kv_authentication",
                message: "KV authentication failed",
            },
            dowe_kv::KvError::Authorization(_) => Self {
                status: StatusCode::FORBIDDEN,
                code: "kv_authorization",
                message: "KV authorization failed",
            },
            dowe_kv::KvError::NotFound(_) => Self::not_found("KV key not found"),
            dowe_kv::KvError::InvalidName(_) | dowe_kv::KvError::InvalidRequest(_) => Self {
                status: StatusCode::BAD_REQUEST,
                code: "kv_invalid_request",
                message: "KV request is invalid",
            },
            _ => Self::kv(),
        }
    }
}

impl<'a> StoreActionContext<'a> {
    async fn execute(
        &mut self,
        action: &dowe_compiler::ServerAction,
    ) -> Result<(), StoreActionError> {
        for statement in &action.statements {
            self.execute_statement(statement).await?;
        }
        Ok(())
    }

    async fn execute_statement(
        &mut self,
        statement: &ServerStatement,
    ) -> Result<(), StoreActionError> {
        match statement {
            ServerStatement::Log(log) => execute_resolved_log(log, |reference| {
                self.resolve_reference(reference)
                    .into_json()
                    .map(log_json_text)
            }),
            ServerStatement::RequestJson { binding, schema } => {
                let value =
                    serde_json::from_slice::<Value>(self.body).map_err(|_| StoreActionError {
                        status: StatusCode::BAD_REQUEST,
                        code: "invalid_json",
                        message: "Invalid JSON body",
                    })?;
                let value = if let Some(schema) = schema {
                    typed_json_value(&value, schema)?
                } else if value.is_object() {
                    value
                } else {
                    return Err(StoreActionError::invalid_body("Expected JSON object"));
                };
                self.request_body = Some(value.clone());
                self.bindings.insert(binding.clone(), value);
            }
            ServerStatement::Http(statement) => self.execute_http(statement).await?,
            ServerStatement::AgentChat(statement) => {
                let source = self
                    .resolve_reference(&statement.source)
                    .into_json()
                    .ok_or_else(StoreActionError::missing_http)?;
                self.bindings
                    .insert(statement.binding.clone(), agent_chat_body(source));
            }
            ServerStatement::WebSocketJson(statement) => {
                let value =
                    serde_json::from_slice::<Value>(self.body).map_err(|_| StoreActionError {
                        status: StatusCode::BAD_REQUEST,
                        code: "invalid_json",
                        message: "Invalid JSON body",
                    })?;
                self.request_body = Some(value.clone());
                self.bindings.insert(statement.binding.clone(), value);
            }
            ServerStatement::WebSocketSendJson(_) | ServerStatement::WebSocketSseBridge(_) => {}
            ServerStatement::Store(statement) => self.execute_store(statement).await?,
            ServerStatement::Kv(statement) => self.execute_kv(statement).await?,
        }
        Ok(())
    }

    async fn execute_http(
        &mut self,
        statement: &OutboundHttpRequest,
    ) -> Result<(), StoreActionError> {
        let url = format!(
            "{}{}",
            self.http_base(&statement.base)?.trim_end_matches('/'),
            statement.path
        );
        let client = reqwest::Client::new();
        let mut request = match statement.method {
            HttpMethod::Get => client.get(url),
            HttpMethod::Post => client.post(url),
            HttpMethod::Put => client.put(url),
            HttpMethod::Patch => client.patch(url),
            HttpMethod::Delete => client.delete(url),
        };
        if let Some(secret) = &statement.bearer {
            request = request.bearer_auth(self.secret_value(secret)?);
        }
        if let Some(json) = &statement.json {
            let value = self.evaluate(json)?.into_json().unwrap_or(Value::Null);
            request = request.json(&value);
        }
        let response = request.send().await.map_err(|_| StoreActionError::http())?;
        match statement.mode {
            HttpResponseMode::Proxy => {
                let status = status_from_reqwest(response.status());
                let content_type = response_content_type(&response);
                self.bindings.insert(
                    statement.binding.clone(),
                    http_binding_json(status, content_type, None),
                );
                self.http_results
                    .insert(statement.binding.clone(), HttpActionResult::Proxy(response));
            }
            HttpResponseMode::Json => {
                let status = status_from_reqwest(response.status());
                let content_type = response_content_type(&response);
                let raw = response
                    .bytes()
                    .await
                    .map_err(|_| StoreActionError::http())?;
                let body = serde_json::from_slice::<Value>(&raw)
                    .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&raw).to_string()));
                self.bindings.insert(
                    statement.binding.clone(),
                    http_binding_json(status, content_type.clone(), Some(body.clone())),
                );
                self.http_results.insert(
                    statement.binding.clone(),
                    HttpActionResult::Buffered {
                        status,
                        content_type,
                        body,
                        raw,
                    },
                );
            }
        }
        Ok(())
    }

    async fn execute_store(
        &mut self,
        statement: &ServerStoreStatement,
    ) -> Result<(), StoreActionError> {
        match statement {
            ServerStoreStatement::Handle {
                binding,
                database,
                remote,
            } => {
                let database_name = database.clone();
                let handle = if let Some(remote) = remote {
                    StoreHandle::Remote(self.remote_client(database, remote)?)
                } else {
                    init_database(self.root, database).map_err(StoreActionError::from_store)?;
                    let database =
                        open_database(self.root, database).map_err(StoreActionError::from_store)?;
                    StoreHandle::Local(database)
                };
                self.handles.insert(binding.clone(), handle);
                self.handle_databases.insert(binding.clone(), database_name);
            }
            ServerStoreStatement::List {
                binding,
                handle,
                table,
            } => {
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let records = database
                            .records(table)
                            .map_err(StoreActionError::from_store)?;
                        Value::Array(records.iter().map(record_json).collect())
                    }
                    StoreHandle::Remote(client) => client
                        .list(table)
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Read {
                binding,
                handle,
                table,
                filter,
                required,
            } => {
                let expected = self.filter_value(filter)?;
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let record = database
                            .records(table)
                            .map_err(StoreActionError::from_store)?
                            .into_iter()
                            .find(|record| record_matches(record, &filter.field, &expected));
                        if record.is_none() && *required {
                            return Err(StoreActionError::not_found("Record not found"));
                        }
                        record.as_ref().map(record_json).unwrap_or(Value::Null)
                    }
                    StoreHandle::Remote(client) => client
                        .read(table, &filter.field, expected.to_json(), *required)
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Insert {
                binding,
                handle,
                table,
                value,
                required,
            } => {
                let record = self.literal_record(value)?;
                validate_required_fields(&record, required)?;
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let inserted = database
                            .insert(table, record)
                            .map_err(StoreActionError::from_store)?;
                        record_json(&inserted)
                    }
                    StoreHandle::Remote(client) => client
                        .insert(table, record_json(&record))
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Update {
                binding,
                handle,
                table,
                filter,
                value,
                required,
                matches,
            } => {
                self.validate_matches(matches)?;
                let expected = self.filter_value(filter)?;
                let patch = self.literal_record(value)?;
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let changed = database
                            .update(table, &filter.field, &expected, patch)
                            .map_err(StoreActionError::from_store)?;
                        if changed == 0 && *required {
                            return Err(StoreActionError::not_found("Record not found"));
                        }
                        changed_json(changed)
                    }
                    StoreHandle::Remote(client) => client
                        .update(
                            table,
                            &filter.field,
                            expected.to_json(),
                            record_json(&patch),
                            *required,
                        )
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Delete {
                binding,
                handle,
                table,
                filter,
                required,
            } => {
                let expected = self.filter_value(filter)?;
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let changed = database
                            .delete(table, &filter.field, &expected)
                            .map_err(StoreActionError::from_store)?;
                        if changed == 0 && *required {
                            return Err(StoreActionError::not_found("Record not found"));
                        }
                        changed_json(changed)
                    }
                    StoreHandle::Remote(client) => client
                        .delete(table, &filter.field, expected.to_json(), *required)
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Query {
                binding,
                handle,
                sql,
            } => {
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => database
                        .query_json(sql)
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::Remote(client) => client
                        .query(sql)
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Transaction {
                binding,
                handle,
                operations,
                return_binding,
            } => {
                if matches!(self.handle(handle)?, StoreHandle::Remote(_)) {
                    return Err(StoreActionError::store());
                }
                let database_name = self
                    .handle_databases
                    .get(handle)
                    .cloned()
                    .ok_or_else(StoreActionError::store)?;
                let transaction = StoreTransactionEndpoint {
                    database: database_name,
                    operations: operations.clone(),
                    return_binding: return_binding.clone(),
                };
                let value = execute_store_transaction(self.root, &transaction)
                    .map_err(|_| StoreActionError::store())?;
                self.bindings.insert(binding.clone(), value);
            }
        }
        Ok(())
    }

    async fn execute_kv(&mut self, statement: &ServerKvStatement) -> Result<(), StoreActionError> {
        match statement {
            ServerKvStatement::Handle {
                binding,
                database,
                persist,
                remote,
            } => {
                let handle = if let Some(remote) = remote {
                    KvHandle::Remote(self.kv_remote_client(database, *persist, remote)?)
                } else {
                    KvHandle::Local(
                        open_kv_database(self.root, database, *persist)
                            .map_err(StoreActionError::from_kv)?,
                    )
                };
                self.kv_handles.insert(binding.clone(), handle);
            }
            ServerKvStatement::Get {
                binding,
                handle,
                key,
                required,
            } => {
                let value = match self.kv_handle(handle)? {
                    KvHandle::Local(database) => {
                        let value = database.get(key).map_err(StoreActionError::from_kv)?;
                        if value.is_none() && *required {
                            return Err(StoreActionError::not_found("KV key not found"));
                        }
                        value.unwrap_or(Value::Null)
                    }
                    KvHandle::Remote(client) => client
                        .get(key, *required)
                        .await
                        .map_err(StoreActionError::from_kv)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerKvStatement::Set {
                binding,
                handle,
                key,
                value,
            } => {
                let value = self.evaluate(value)?.into_json().unwrap_or(Value::Null);
                let output = match self.kv_handle(handle)? {
                    KvHandle::Local(database) => {
                        database
                            .set(key, value)
                            .map_err(StoreActionError::from_kv)?;
                        kv_set_json(key)
                    }
                    KvHandle::Remote(client) => client
                        .set(key, value)
                        .await
                        .map_err(StoreActionError::from_kv)?,
                };
                self.bindings.insert(binding.clone(), output);
            }
            ServerKvStatement::Delete {
                binding,
                handle,
                key,
            } => {
                let output = match self.kv_handle(handle)? {
                    KvHandle::Local(database) => {
                        kv_delete_json(database.delete(key).map_err(StoreActionError::from_kv)?)
                    }
                    KvHandle::Remote(client) => client
                        .delete(key)
                        .await
                        .map_err(StoreActionError::from_kv)?,
                };
                self.bindings.insert(binding.clone(), output);
            }
            ServerKvStatement::Keys {
                binding,
                handle,
                prefix,
            } => {
                let output = match self.kv_handle(handle)? {
                    KvHandle::Local(database) => Value::Array(
                        database
                            .keys(prefix.as_deref())
                            .map_err(StoreActionError::from_kv)?
                            .into_iter()
                            .map(Value::String)
                            .collect(),
                    ),
                    KvHandle::Remote(client) => client
                        .keys(prefix.as_deref())
                        .await
                        .map_err(StoreActionError::from_kv)?,
                };
                self.bindings.insert(binding.clone(), output);
            }
            ServerKvStatement::Clear { binding, handle } => {
                let output = match self.kv_handle(handle)? {
                    KvHandle::Local(database) => {
                        kv_clear_json(database.clear().map_err(StoreActionError::from_kv)?)
                    }
                    KvHandle::Remote(client) => {
                        client.clear().await.map_err(StoreActionError::from_kv)?
                    }
                };
                self.bindings.insert(binding.clone(), output);
            }
        }
        Ok(())
    }

    fn handle(&self, handle: &str) -> Result<&StoreHandle, StoreActionError> {
        self.handles.get(handle).ok_or_else(StoreActionError::store)
    }

    fn kv_handle(&self, handle: &str) -> Result<&KvHandle, StoreActionError> {
        self.kv_handles.get(handle).ok_or_else(StoreActionError::kv)
    }

    fn remote_client(
        &self,
        database: &str,
        remote: &StoreRemoteConnection,
    ) -> Result<RemoteStoreClient, StoreActionError> {
        let credential = match &remote.credential {
            StoreCredential::Token(value) | StoreCredential::Password(value) => {
                self.connection_value(value)?
            }
        };
        RemoteStoreClient::new(RemoteStoreConfig {
            host: self.connection_value(&remote.host)?,
            database: database.to_string(),
            user: self.connection_value(&remote.user)?,
            credential,
        })
        .map_err(StoreActionError::from_store)
    }

    fn connection_value(&self, value: &StoreConnectionValue) -> Result<String, StoreActionError> {
        match value {
            StoreConnectionValue::Static(value) => Ok(value.clone()),
            StoreConnectionValue::Environment(name) => self
                .project
                .environment_config
                .variable(name)
                .and_then(|variable| variable.resolved_value.clone())
                .ok_or_else(StoreActionError::store),
        }
    }

    fn http_base(&self, value: &HttpConnectionValue) -> Result<String, StoreActionError> {
        match value {
            HttpConnectionValue::Static(value) => Ok(value.clone()),
            HttpConnectionValue::Environment(name) => {
                self.env_value(name).ok_or_else(|| StoreActionError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "http_env_missing",
                    message: "HTTP environment variable is not configured",
                })
            }
        }
    }

    fn secret_value(&self, secret: &ServerSecret) -> Result<String, StoreActionError> {
        match secret {
            ServerSecret::Environment(name) => {
                self.env_value(name).ok_or_else(|| StoreActionError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "http_secret_missing",
                    message: "HTTP secret is not configured",
                })
            }
        }
    }

    fn env_value(&self, name: &str) -> Option<String> {
        self.project
            .environment_config
            .variable(name)
            .and_then(|variable| variable.resolved_value.clone())
    }

    fn kv_remote_client(
        &self,
        database: &str,
        persist: bool,
        remote: &KvRemoteConnection,
    ) -> Result<RemoteKvClient, StoreActionError> {
        let credential = match &remote.credential {
            KvCredential::Token(value) | KvCredential::Password(value) => {
                self.kv_connection_value(value)?
            }
        };
        RemoteKvClient::new(RemoteKvConfig {
            host: self.kv_connection_value(&remote.host)?,
            database: database.to_string(),
            user: self.kv_connection_value(&remote.user)?,
            credential,
            persist,
        })
        .map_err(StoreActionError::from_kv)
    }

    fn kv_connection_value(&self, value: &KvConnectionValue) -> Result<String, StoreActionError> {
        match value {
            KvConnectionValue::Static(value) => Ok(value.clone()),
            KvConnectionValue::Environment(name) => self
                .project
                .environment_config
                .variable(name)
                .and_then(|variable| variable.resolved_value.clone())
                .ok_or_else(StoreActionError::kv),
        }
    }

    fn filter_value(&self, filter: &StoreFilter) -> Result<StoreValue, StoreActionError> {
        Ok(StoreValue::from_json(
            self.evaluate(&filter.value)?
                .into_json()
                .unwrap_or(Value::Null),
        ))
    }

    fn literal_record(&self, value: &StoreLiteral) -> Result<StoreRecord, StoreActionError> {
        let StoreLiteral::Object(entries) = value else {
            return Ok(StoreRecord::new());
        };
        let mut record = StoreRecord::new();
        for (key, value) in entries {
            match self.evaluate(value)? {
                ResolvedValue::Json(value) => {
                    record.insert(key.clone(), StoreValue::from_json(value));
                }
                ResolvedValue::Missing => {}
            }
        }
        Ok(record)
    }

    fn validate_matches(
        &self,
        matches: &[dowe_compiler::StoreMatchField],
    ) -> Result<(), StoreActionError> {
        let Some(Value::Object(body)) = &self.request_body else {
            return Ok(());
        };
        for expected in matches {
            let Some(body_value) = body.get(&expected.field) else {
                continue;
            };
            let expected_value = self
                .evaluate(&expected.value)?
                .into_json()
                .unwrap_or(Value::Null);
            if body_value != &expected_value {
                return Err(StoreActionError::invalid_body(
                    "Request body does not match route authority",
                ));
            }
        }
        Ok(())
    }

    fn evaluate(&self, value: &StoreLiteral) -> Result<ResolvedValue, StoreActionError> {
        Ok(match value {
            StoreLiteral::Null => ResolvedValue::Json(Value::Null),
            StoreLiteral::Bool(value) => ResolvedValue::Json(Value::Bool(*value)),
            StoreLiteral::Number(value) => ResolvedValue::Json(number_json(value)),
            StoreLiteral::String(value) => ResolvedValue::Json(Value::String(value.clone())),
            StoreLiteral::Reference(value) => self.resolve_reference(value),
            StoreLiteral::Array(values) => {
                let mut output = Vec::new();
                for value in values {
                    if let ResolvedValue::Json(value) = self.evaluate(value)? {
                        output.push(value);
                    }
                }
                ResolvedValue::Json(Value::Array(output))
            }
            StoreLiteral::Object(entries) => {
                let mut output = Map::new();
                for (key, value) in entries {
                    if let ResolvedValue::Json(value) = self.evaluate(value)? {
                        output.insert(key.clone(), value);
                    }
                }
                ResolvedValue::Json(Value::Object(output))
            }
        })
    }

    fn resolve_reference(&self, reference: &str) -> ResolvedValue {
        if reference == "now" {
            return ResolvedValue::Json(Value::String(timestamp()));
        }
        if reference == "req.params.id" {
            return self
                .params
                .get("id")
                .map(|value| ResolvedValue::Json(Value::String(value.clone())))
                .unwrap_or(ResolvedValue::Missing);
        }
        if let Some(name) = reference.strip_prefix("req.params.") {
            return self
                .params
                .get(name)
                .map(|value| ResolvedValue::Json(Value::String(value.clone())))
                .unwrap_or(ResolvedValue::Missing);
        }
        if let Some(value) = self.bindings.get(reference) {
            return ResolvedValue::Json(value.clone());
        }
        if let Some((binding, path)) = reference.split_once('.')
            && let Some(value) = self.bindings.get(binding)
        {
            return read_json_path(value, path)
                .map(|value| ResolvedValue::Json(value.clone()))
                .unwrap_or(ResolvedValue::Missing);
        }
        ResolvedValue::Json(Value::String(reference.to_string()))
    }
}

fn typed_json_value(value: &Value, schema: &DoweType) -> Result<Value, StoreActionError> {
    match schema {
        DoweType::Unknown => Ok(value.clone()),
        DoweType::Null => {
            if value.is_null() {
                Ok(Value::Null)
            } else {
                Err(StoreActionError::invalid_body(
                    "Request body does not match declared type",
                ))
            }
        }
        DoweType::Bool => value.as_bool().map(Value::Bool).ok_or_else(|| {
            StoreActionError::invalid_body("Request body does not match declared type")
        }),
        DoweType::Number => {
            if value.is_number() {
                Ok(value.clone())
            } else {
                Err(StoreActionError::invalid_body(
                    "Request body does not match declared type",
                ))
            }
        }
        DoweType::String => value
            .as_str()
            .map(|value| Value::String(value.to_string()))
            .ok_or_else(|| {
                StoreActionError::invalid_body("Request body does not match declared type")
            }),
        DoweType::Array(item) => {
            let Some(values) = value.as_array() else {
                return Err(StoreActionError::invalid_body(
                    "Request body does not match declared type",
                ));
            };
            values
                .iter()
                .map(|value| typed_json_value(value, item))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array)
        }
        DoweType::Object(fields) => {
            let Some(values) = value.as_object() else {
                return Err(StoreActionError::invalid_body(
                    "Request body does not match declared type",
                ));
            };
            let mut output = Map::new();
            for field in fields {
                match values.get(&field.name) {
                    Some(Value::Null) if field.optional => {
                        output.insert(field.name.clone(), Value::Null);
                    }
                    Some(value) => {
                        output.insert(field.name.clone(), typed_json_value(value, &field.value)?);
                    }
                    None if field.optional => {}
                    None => {
                        return Err(StoreActionError::invalid_body(
                            "Request body does not match declared type",
                        ));
                    }
                }
            }
            Ok(Value::Object(output))
        }
    }
}

impl ResolvedValue {
    fn into_json(self) -> Option<Value> {
        match self {
            ResolvedValue::Json(value) => Some(value),
            ResolvedValue::Missing => None,
        }
    }
}

fn validate_required_fields(
    record: &StoreRecord,
    fields: &[String],
) -> Result<(), StoreActionError> {
    for field in fields {
        let valid = record.get(field).is_some_and(|value| {
            value
                .to_json()
                .as_str()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
        });
        if !valid {
            return Err(StoreActionError::invalid_body(
                "Required fields must be non-empty strings",
            ));
        }
    }
    Ok(())
}

fn record_matches(record: &StoreRecord, field: &str, expected: &StoreValue) -> bool {
    record
        .get(field)
        .is_some_and(|value| value.comparable_text() == expected.comparable_text())
}

fn changed_json(changed: usize) -> Value {
    let mut output = Map::new();
    output.insert("changed".to_string(), Value::Number(changed.into()));
    Value::Object(output)
}

fn kv_set_json(key: &str) -> Value {
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(true));
    output.insert("key".to_string(), Value::String(key.to_string()));
    Value::Object(output)
}

fn kv_delete_json(deleted: bool) -> Value {
    let mut output = Map::new();
    output.insert("deleted".to_string(), Value::Bool(deleted));
    Value::Object(output)
}

fn kv_clear_json(cleared: usize) -> Value {
    let mut output = Map::new();
    output.insert("cleared".to_string(), Value::Number(cleared.into()));
    Value::Object(output)
}

fn log_json_text(value: Value) -> String {
    match value {
        Value::String(value) => value,
        value => value.to_string(),
    }
}

fn number_json(value: &str) -> Value {
    value
        .parse::<i64>()
        .map(|value| Value::Number(value.into()))
        .unwrap_or_else(|_| Value::String(value.to_string()))
}

fn read_json_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in path.split('.') {
        current = current.as_object()?.get(part)?;
    }
    Some(current)
}

fn status_from_u16(status: u16) -> StatusCode {
    StatusCode::from_u16(status).unwrap_or(StatusCode::OK)
}

fn timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    seconds.to_string()
}

fn json_error(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    let mut error = Map::new();
    error.insert("code".to_string(), Value::String(code.to_string()));
    error.insert("message".to_string(), Value::String(message.to_string()));
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(false));
    output.insert("error".to_string(), Value::Object(error));
    json_response(status, Value::Object(output))
}

fn execute_store_transaction(
    root: &Path,
    transaction: &StoreTransactionEndpoint,
) -> dowe_store::StoreResult<Value> {
    init_database(root, &transaction.database)?;
    let database = open_database(root, &transaction.database)?;
    let mut tx = database.transaction();
    let mut bindings = std::collections::BTreeMap::<String, StoreRecord>::new();

    for operation in &transaction.operations {
        match operation {
            StoreTransactionOperation::Insert {
                binding,
                table,
                value,
            } => {
                let record = tx.insert(table, literal_record(value))?;
                bindings.insert(binding.clone(), record);
            }
        }
    }

    let committed = tx.commit()?;
    if let Some(binding) = &transaction.return_binding
        && let Some(record) = bindings.get(binding)
    {
        return Ok(record_json(record));
    }
    Ok(Value::Array(committed.iter().map(record_json).collect()))
}

fn literal_record(value: &StoreLiteral) -> StoreRecord {
    match value {
        StoreLiteral::Object(entries) => entries
            .iter()
            .map(|(key, value)| (key.clone(), literal_value(value)))
            .collect(),
        _ => StoreRecord::new(),
    }
}

fn literal_value(value: &StoreLiteral) -> StoreValue {
    match value {
        StoreLiteral::Null => StoreValue::Null,
        StoreLiteral::Bool(value) => StoreValue::Bool(*value),
        StoreLiteral::Number(value) => value
            .parse::<i64>()
            .map(StoreValue::Int)
            .unwrap_or_else(|_| StoreValue::Decimal(value.clone())),
        StoreLiteral::String(value) | StoreLiteral::Reference(value) => {
            StoreValue::String(value.clone())
        }
        StoreLiteral::Array(values) => StoreValue::Json(Value::Array(
            values
                .iter()
                .map(|value| literal_value(value).to_json())
                .collect(),
        )),
        StoreLiteral::Object(entries) => StoreValue::Json(Value::Object(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), literal_value(value).to_json()))
                .collect(),
        )),
    }
}

fn record_json(record: &StoreRecord) -> Value {
    Value::Object(
        record
            .iter()
            .map(|(key, value)| (key.clone(), value.to_json()))
            .collect(),
    )
}

fn json_response(status: StatusCode, value: Value) -> Response {
    (
        status,
        [(CONTENT_TYPE, "application/json; charset=utf-8")],
        value.to_string(),
    )
        .into_response()
}

fn store_error_response(error: dowe_store::StoreError) -> Response {
    let status = match error {
        dowe_store::StoreError::Authentication(_) => StatusCode::UNAUTHORIZED,
        dowe_store::StoreError::Authorization(_) => StatusCode::FORBIDDEN,
        dowe_store::StoreError::InvalidName(_) | dowe_store::StoreError::InvalidQuery(_) => {
            StatusCode::BAD_REQUEST
        }
        dowe_store::StoreError::AlreadyExists(_)
        | dowe_store::StoreError::TransactionConflict(_) => StatusCode::CONFLICT,
        dowe_store::StoreError::NotFound(_) => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    text_response(status, error.to_string())
}

pub(crate) fn chunk_response(web: &WebOutput, path: &str) -> Option<Response> {
    let prefix = "/chunks/";
    let chunk_path = path.strip_prefix(prefix)?;
    let relative = std::path::Path::new("web/chunks").join(chunk_path);
    if let Some(chunk) = web
        .translation_chunks
        .iter()
        .find(|chunk| chunk.relative_path == relative)
    {
        return Some(javascript_response(chunk.content.clone()));
    }
    let chunk = web
        .chunks
        .iter()
        .find(|chunk| chunk.relative_path == relative || chunk.css_relative_path == relative)?;
    let content_type = if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else {
        "application/javascript; charset=utf-8"
    };
    let content = if path.ends_with(".css") {
        chunk.css_content.clone()
    } else {
        chunk.content.clone()
    };

    Some((StatusCode::OK, [(CONTENT_TYPE, content_type)], content).into_response())
}

pub(crate) fn design_css_response(project: &CompiledProject, relative_path: &str) -> Response {
    let path = project.root.join(".dowe").join(relative_path);
    match fs::read_to_string(path) {
        Ok(css) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/css; charset=utf-8")],
            css,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(crate) fn font_response(project: &CompiledProject, path: &str) -> Option<Response> {
    let font_path = path.strip_prefix("/fonts/")?;
    let relative = Path::new(font_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Some(StatusCode::NOT_FOUND.into_response());
    }

    let path = project.root.join(".dowe/fonts").join(relative);
    let Ok(content) = fs::read(path) else {
        return Some(StatusCode::NOT_FOUND.into_response());
    };

    Some(
        (
            StatusCode::OK,
            [
                (CONTENT_TYPE, "font/ttf"),
                (CACHE_CONTROL, "public, max-age=31536000"),
            ],
            content,
        )
            .into_response(),
    )
}

fn render_page(page: &ViewPage) -> Response {
    Html(inject_dev_client(&page.html_document)).into_response()
}

fn dev_client_response() -> Response {
    javascript_response(dev_client_script())
}

pub(crate) fn javascript_response(content: String) -> Response {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        content,
    )
        .into_response()
}

pub(crate) fn json_response_text(content: String) -> Response {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json; charset=utf-8")],
        content,
    )
        .into_response()
}

fn inject_dev_client(html: &str) -> String {
    let script = r#"<script type="module" src="/_dowe/dev/client.js"></script>"#;
    if html.contains(script) {
        return html.to_string();
    }

    if let Some(index) = html.rfind("</body>") {
        let mut output = String::with_capacity(html.len() + script.len());
        output.push_str(&html[..index]);
        output.push_str(script);
        output.push_str(&html[index..]);
        output
    } else {
        format!("{html}{script}")
    }
}

fn dev_client_script() -> String {
    r#"const protocol=location.protocol==="https:"?"wss":"ws";let active=true;function connect(){if(!active)return;const socket=new WebSocket(`${protocol}://${location.host}/_dowe/dev/ws`);socket.onmessage=(event)=>{try{const message=JSON.parse(event.data);if(message.type==="reload")location.reload();if(message.type==="shutdown")active=false;}catch(error){}};socket.onclose=()=>{if(active)setTimeout(connect,250);};}connect();"#
        .to_string()
}
