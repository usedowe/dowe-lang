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
