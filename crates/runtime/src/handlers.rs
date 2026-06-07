use crate::server::DevRuntimeState;
use crate::server_actions::{
    execute_resolved_log, execute_server_action, execute_server_action_with_resolver,
};
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::header::{
    ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, CACHE_CONTROL, CONTENT_TYPE,
    ORIGIN, VARY,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use dowe_compiler::{
    CompiledProject, CorsConfig, DoweType, EndpointBehavior, HttpMethod, ServerConfig,
    ServerMiddleware, ServerMiddlewareResponseBody, ServerMiddlewareStatement, ServerSecret,
    ServerStatement, ServerStoreStatement, StoreActionJsonEndpoint, StoreFilter, StoreLiteral,
    StoreTransactionEndpoint, StoreTransactionOperation, ViewPage, WebOutput, WebSocketHandlers,
    normalize_cors_method, normalize_http_header_name,
};
use dowe_crypto::{
    JwtValidationOptions, decrypt_jwe_dir_a256gcm, encrypt_jwe_dir_a256gcm, sign_jws_hs256,
    verify_jws_hs256,
};
use dowe_store::{Database, StoreRecord, StoreValue, init_database, open_database};
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
        );
    }

    if method == Method::GET {
        desktop_static_response(&project, uri.path())
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub(crate) fn server_response(
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
                    EndpointBehavior::StoreActionJson(_)
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
                    EndpointBehavior::StoreInsertJson(insert) => {
                        match init_database(&project.root, &insert.database)
                            .and_then(|_| open_database(&project.root, &insert.database))
                            .and_then(|database| {
                                database.insert(&insert.table, literal_record(&insert.value))
                            }) {
                            Ok(record) => json_response(StatusCode::OK, record_json(&record)),
                            Err(error) => store_error_response(error),
                        }
                    }
                    EndpointBehavior::StoreQueryJson(query) => {
                        match init_database(&project.root, &query.database)
                            .and_then(|_| open_database(&project.root, &query.database))
                            .and_then(|database| database.query_json(&query.sql))
                        {
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
                    EndpointBehavior::StoreActionJson(response) => execute_store_action_json(
                        &project.root,
                        &matched.endpoint.action,
                        &response,
                        &matched.params,
                        &body,
                    ),
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

pub async fn websocket_handler(
    State(state): State<DevRuntimeState>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let project = state.project.read().await;
    let Some(route) = project.backend.find_websocket("/ws") else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let handlers = route.handlers;

    websocket_response(upgrade, handlers)
}

pub async fn desktop_websocket_handler(
    State(state): State<DevRuntimeState>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let project = state.project.read().await;
    let Some(server) = &project.desktop_server else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(route) = server.find_websocket("/ws") else {
        return StatusCode::NOT_FOUND.into_response();
    };
    websocket_response(upgrade, route.handlers)
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

async fn handle_websocket(mut socket: WebSocket, handlers: WebSocketHandlers) {
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
            Ok(Message::Text(_)) | Ok(Message::Binary(_)) => {
                execute_server_action(&handlers.message);
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }

    if !closed {
        execute_server_action(&handlers.close);
    }
}

fn websocket_response(upgrade: WebSocketUpgrade, handlers: WebSocketHandlers) -> Response {
    upgrade
        .on_upgrade(move |socket| handle_websocket(socket, handlers))
        .into_response()
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

fn execute_store_action_json(
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &StoreActionJsonEndpoint,
    params: &HashMap<String, String>,
    body: &Bytes,
) -> Response {
    let mut context = StoreActionContext {
        root,
        params,
        body,
        request_body: None,
        bindings: HashMap::new(),
        handles: HashMap::new(),
        handle_databases: HashMap::new(),
    };
    match context
        .execute(action)
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

struct StoreActionContext<'a> {
    root: &'a Path,
    params: &'a HashMap<String, String>,
    body: &'a Bytes,
    request_body: Option<Value>,
    bindings: HashMap<String, Value>,
    handles: HashMap<String, Database>,
    handle_databases: HashMap<String, String>,
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
}

impl<'a> StoreActionContext<'a> {
    fn execute(&mut self, action: &dowe_compiler::ServerAction) -> Result<(), StoreActionError> {
        for statement in &action.statements {
            match statement {
                ServerStatement::Log(log) => execute_resolved_log(log, |reference| {
                    self.resolve_reference(reference)
                        .into_json()
                        .map(log_json_text)
                }),
                ServerStatement::RequestJson { binding, schema } => {
                    let value = serde_json::from_slice::<Value>(self.body).map_err(|_| {
                        StoreActionError {
                            status: StatusCode::BAD_REQUEST,
                            code: "invalid_json",
                            message: "Invalid JSON body",
                        }
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
                ServerStatement::Store(statement) => self.execute_store(statement)?,
            }
        }
        Ok(())
    }

    fn execute_store(&mut self, statement: &ServerStoreStatement) -> Result<(), StoreActionError> {
        match statement {
            ServerStoreStatement::Handle { binding, database } => {
                init_database(self.root, database).map_err(|_| StoreActionError::store())?;
                let database =
                    open_database(self.root, database).map_err(|_| StoreActionError::store())?;
                let database_name = database.metadata().name.clone();
                self.handles.insert(binding.clone(), database);
                self.handle_databases.insert(binding.clone(), database_name);
            }
            ServerStoreStatement::List {
                binding,
                handle,
                table,
            } => {
                let records = self
                    .database(handle)?
                    .records(table)
                    .map_err(|_| StoreActionError::store())?;
                self.bindings.insert(
                    binding.clone(),
                    Value::Array(records.iter().map(record_json).collect()),
                );
            }
            ServerStoreStatement::Read {
                binding,
                handle,
                table,
                filter,
                required,
            } => {
                let expected = self.filter_value(filter)?;
                let record = self
                    .database(handle)?
                    .records(table)
                    .map_err(|_| StoreActionError::store())?
                    .into_iter()
                    .find(|record| record_matches(record, &filter.field, &expected));
                if record.is_none() && *required {
                    return Err(StoreActionError::not_found("Record not found"));
                }
                self.bindings.insert(
                    binding.clone(),
                    record.as_ref().map(record_json).unwrap_or(Value::Null),
                );
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
                let inserted = self
                    .database(handle)?
                    .insert(table, record)
                    .map_err(|_| StoreActionError::store())?;
                self.bindings
                    .insert(binding.clone(), record_json(&inserted));
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
                let changed = self
                    .database(handle)?
                    .update(table, &filter.field, &expected, patch)
                    .map_err(|_| StoreActionError::store())?;
                if changed == 0 && *required {
                    return Err(StoreActionError::not_found("Record not found"));
                }
                self.bindings.insert(binding.clone(), changed_json(changed));
            }
            ServerStoreStatement::Delete {
                binding,
                handle,
                table,
                filter,
                required,
            } => {
                let expected = self.filter_value(filter)?;
                let changed = self
                    .database(handle)?
                    .delete(table, &filter.field, &expected)
                    .map_err(|_| StoreActionError::store())?;
                if changed == 0 && *required {
                    return Err(StoreActionError::not_found("Record not found"));
                }
                self.bindings.insert(binding.clone(), changed_json(changed));
            }
            ServerStoreStatement::Query {
                binding,
                handle,
                sql,
            } => {
                let value = self
                    .database(handle)?
                    .query_json(sql)
                    .map_err(|_| StoreActionError::store())?;
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Transaction {
                binding,
                handle,
                operations,
                return_binding,
            } => {
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

    fn database(&self, handle: &str) -> Result<&Database, StoreActionError> {
        self.handles.get(handle).ok_or_else(StoreActionError::store)
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
