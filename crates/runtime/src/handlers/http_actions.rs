async fn execute_http_action_json(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &HttpActionJsonEndpoint,
    params: &HashMap<String, String>,
    body: &Bytes,
    raw_query: Option<&str>,
    headers: &HeaderMap,
    request_context: &HashMap<String, Value>,
    cache_mode: CacheRuntimeMode,
) -> Response {
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        raw_query,
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
    raw_query: Option<&str>,
    headers: &HeaderMap,
    cache_mode: CacheRuntimeMode,
) -> Response {
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        raw_query,
        headers: Some(headers),
        request_context: None,
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

async fn execute_http_reverse_proxy(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &HttpReverseProxyEndpoint,
    params: &HashMap<String, String>,
    body: &Bytes,
    raw_query: Option<&str>,
    headers: &HeaderMap,
    method: &HttpMethod,
    path: &str,
    cache_mode: CacheRuntimeMode,
) -> Response {
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        raw_query,
        headers: Some(headers),
        request_context: None,
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
        if matches!(
            statement,
            ServerStatement::Task(job)
                if matches!(job.timing, dowe_compiler::ServerTaskTiming::ResponseHeaders)
        ) {
            continue;
        }
        if let Err(error) = context.execute_statement(statement).await {
            return json_error(error.status, error.code, error.message);
        }
    }
    let state = response
        .state
        .as_deref()
        .and_then(|reference| context.resolve_reference(reference).into_json())
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "ready".to_string());
    if state == "loading" {
        return reverse_proxy_fallback(&context, response.loading_url.as_deref(), "loading");
    }
    if state != "ready" {
        return reverse_proxy_fallback(&context, response.error_url.as_deref(), "error");
    }
    let upstreams = context
        .resolve_reference(&response.upstream)
        .into_json()
        .unwrap_or(Value::Null);
    let pool_key = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or(&response.upstream);
    let Some(upstream) = select_reverse_proxy_upstream(&upstreams, response.strategy, pool_key)
    else {
        return reverse_proxy_fallback(&context, response.error_url.as_deref(), "error");
    };
    let started = Instant::now();
    let (response, status, bytes_out) =
        match reverse_proxy_request(&upstream, method, path, raw_query, headers, body).await {
            ReverseProxyRequestOutcome::Upstream {
                response,
                status,
                bytes_out,
            } => (response, status, bytes_out),
            ReverseProxyRequestOutcome::Local(response) => return response,
        };
    for statement in &action.statements {
        let ServerStatement::Task(job) = statement else {
            continue;
        };
        if !matches!(job.timing, dowe_compiler::ServerTaskTiming::ResponseHeaders) {
            continue;
        }
        let Ok(value) = context.evaluate(&job.args).map(ResolvedValue::into_json) else {
            continue;
        };
        let Some(mut args) = value else {
            continue;
        };
        enrich_reverse_proxy_telemetry(
            &mut args,
            status,
            method.as_str(),
            path,
            started.elapsed().as_secs_f64() * 1000.0,
            body.len() as u64,
            bytes_out,
        );
        crate::background_jobs::launch_task_with_args(root, job, args, cache_mode);
    }
    response
}

enum ReverseProxyRequestOutcome {
    Local(Response),
    Upstream {
        response: Response,
        status: u64,
        bytes_out: u64,
    },
}

static REVERSE_PROXY_POOL_COUNTERS: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();

fn select_reverse_proxy_upstream(
    value: &Value,
    strategy: ReverseProxyStrategy,
    pool_key: &str,
) -> Option<String> {
    let upstreams = match value {
        Value::String(url) => vec![url.clone()],
        Value::Array(values) => values
            .iter()
            .filter_map(|value| match value {
                Value::String(url) => Some(url.clone()),
                Value::Object(server)
                    if server.get("enabled").and_then(Value::as_bool) != Some(false)
                        && server
                            .get("status")
                            .and_then(Value::as_str)
                            .is_none_or(|status| status == "ready") =>
                {
                    server
                        .get("url")
                        .or_else(|| server.get("upstreamUrl"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                }
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };
    if upstreams.is_empty() {
        return None;
    }
    let index = match strategy {
        ReverseProxyStrategy::Single => 0,
        ReverseProxyStrategy::RoundRobin => {
            let counters = REVERSE_PROXY_POOL_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()));
            let mut counters = counters.lock().unwrap_or_else(|error| error.into_inner());
            let counter = counters.entry(pool_key.to_string()).or_default();
            let index = *counter as usize % upstreams.len();
            *counter = counter.wrapping_add(1);
            index
        }
    };
    upstreams.get(index).cloned()
}

fn reverse_proxy_fallback(
    context: &StoreActionContext<'_>,
    reference: Option<&str>,
    _state: &str,
) -> Response {
    let url = reference
        .and_then(|reference| context.resolve_reference(reference).into_json())
        .and_then(|value| value.as_str().map(str::to_string));
    let Some(url) = url else {
        return json_error(
            StatusCode::BAD_GATEWAY,
            "reverse_proxy_route_missing",
            "Reverse proxy route has no available upstream or fallback URL",
        );
    };
    reverse_proxy_redirect(&url)
}

fn reverse_proxy_redirect(url: &str) -> Response {
    let Ok(parsed) = reqwest::Url::parse(&url) else {
        return reverse_proxy_invalid_upstream();
    };
    if !matches!(parsed.scheme(), "http" | "https")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.host_str().is_none()
    {
        return reverse_proxy_invalid_upstream();
    }
    let mut response = StatusCode::TEMPORARY_REDIRECT.into_response();
    if let Ok(value) = HeaderValue::from_str(&url) {
        response.headers_mut().insert(LOCATION, value);
    }
    response
}

fn enrich_reverse_proxy_telemetry(
    args: &mut Value,
    status: u64,
    method: &str,
    path: &str,
    latency_ms: f64,
    bytes_in: u64,
    bytes_out: u64,
) {
    let Some(event) = args.get_mut("event").and_then(Value::as_object_mut) else {
        return;
    };
    event.insert("status".to_string(), Value::Number(status.into()));
    event.insert("method".to_string(), Value::String(method.to_string()));
    event.insert("path".to_string(), Value::String(path.to_string()));
    if let Some(latency) = serde_json::Number::from_f64(latency_ms) {
        event.insert("latencyMs".to_string(), Value::Number(latency));
    }
    event.insert("bytesIn".to_string(), Value::Number(bytes_in.into()));
    event.insert("bytesOut".to_string(), Value::Number(bytes_out.into()));
}

async fn reverse_proxy_request(
    upstream: &str,
    method: &HttpMethod,
    path: &str,
    raw_query: Option<&str>,
    headers: &HeaderMap,
    body: &Bytes,
) -> ReverseProxyRequestOutcome {
    let target = format!("{}{}", upstream.trim_end_matches('/'), path);
    let Ok(mut url) = reqwest::Url::parse(&target) else {
        return ReverseProxyRequestOutcome::Local(reverse_proxy_invalid_upstream());
    };
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.host_str().is_none()
        || url.fragment().is_some()
    {
        return ReverseProxyRequestOutcome::Local(reverse_proxy_invalid_upstream());
    }
    url.set_query(raw_query);
    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(30))
        .build()
    {
        Ok(client) => client,
        Err(_) => return ReverseProxyRequestOutcome::Local(reverse_proxy_bad_gateway()),
    };
    let request_method = match reqwest::Method::from_bytes(method.as_str().as_bytes()) {
        Ok(method) => method,
        Err(_) => {
            return ReverseProxyRequestOutcome::Local(
                StatusCode::METHOD_NOT_ALLOWED.into_response(),
            );
        }
    };
    let mut request = client.request(request_method, url).body(body.clone());
    for (name, value) in headers {
        if reverse_proxy_hop_header(name.as_str()) {
            continue;
        }
        request = request.header(name.as_str(), value.as_bytes());
    }
    if let Some(host) = headers.get("host").and_then(|value| value.to_str().ok()) {
        request = request.header("x-forwarded-host", host);
    }
    let upstream = match request.send().await {
        Ok(response) => response,
        Err(_) => return ReverseProxyRequestOutcome::Local(reverse_proxy_bad_gateway()),
    };
    let status = status_from_reqwest(upstream.status());
    let bytes_out = upstream
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let response_headers = upstream.headers().clone();
    let mut response = Response::new(Body::from_stream(upstream.bytes_stream()));
    *response.status_mut() = status;
    for (name, value) in &response_headers {
        if reverse_proxy_hop_header(name.as_str()) {
            continue;
        }
        let Ok(name) = HeaderName::from_bytes(name.as_str().as_bytes()) else {
            continue;
        };
        let Ok(value) = HeaderValue::from_bytes(value.as_bytes()) else {
            continue;
        };
        response.headers_mut().append(name, value);
    }
    ReverseProxyRequestOutcome::Upstream {
        response,
        status: u64::from(status.as_u16()),
        bytes_out,
    }
}

fn reverse_proxy_hop_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
            | "host"
            | "content-length"
    )
}

fn reverse_proxy_invalid_upstream() -> Response {
    json_error(
        StatusCode::BAD_GATEWAY,
        "reverse_proxy_upstream_invalid",
        "Reverse proxy upstream URL is invalid",
    )
}

fn reverse_proxy_bad_gateway() -> Response {
    json_error(
        StatusCode::BAD_GATEWAY,
        "reverse_proxy_unavailable",
        "Reverse proxy upstream is unavailable",
    )
}

async fn execute_http_bytes(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &HttpBytesEndpoint,
    params: &HashMap<String, String>,
    body: &Bytes,
    raw_query: Option<&str>,
    headers: &HeaderMap,
    cache_mode: CacheRuntimeMode,
) -> Response {
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        raw_query,
        headers: Some(headers),
        request_context: None,
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
    match context.execute(action).await {
        Ok(()) => match context.bytes_results.remove(&response.binding) {
            Some(body) => bytes_endpoint_response(&context, response, body),
            None => match context.http_results.remove(&response.binding) {
                Some(result) => http_result_response(result).await,
                None => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "invalid_response",
                    "Byte response binding is missing",
                ),
            },
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
    raw_query: Option<&str>,
    headers: &HeaderMap,
    cache_mode: CacheRuntimeMode,
) -> Response {
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        raw_query,
        headers: Some(headers),
        request_context: None,
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

fn bytes_endpoint_response(
    context: &StoreActionContext<'_>,
    endpoint: &HttpBytesEndpoint,
    body: Bytes,
) -> Response {
    let mut response = body_response(
        status_from_u16(endpoint.status),
        endpoint.content_type.clone(),
        body,
    );
    for header in &endpoint.headers {
        if let Some(value) = context
            .evaluate(&header.value)
            .ok()
            .and_then(ResolvedValue::into_json)
        {
            insert_dynamic_header(&mut response, &header.name, &json_text(&value));
        }
    }
    for cookie in &endpoint.cookies {
        if let Some(value) = context
            .evaluate(&cookie.value)
            .ok()
            .and_then(ResolvedValue::into_json)
        {
            append_header(
                &mut response,
                "set-cookie",
                &cookie_header(cookie, &json_text(&value)),
            );
        }
    }
    response
}

fn cookie_header(cookie: &ResponseCookie, value: &str) -> String {
    let mut output = format!("{}={}", cookie.name, cookie_value(value));
    if let Some(path) = &cookie.path {
        output.push_str("; Path=");
        output.push_str(path);
    }
    if let Some(max_age) = cookie.max_age {
        output.push_str("; Max-Age=");
        output.push_str(&max_age.to_string());
    }
    if let Some(same_site) = &cookie.same_site {
        output.push_str("; SameSite=");
        output.push_str(same_site);
    }
    if cookie.http_only {
        output.push_str("; HttpOnly");
    }
    if cookie.secure {
        output.push_str("; Secure");
    }
    output
}

fn cookie_value(value: &str) -> String {
    value
        .chars()
        .filter(|value| {
            value.is_ascii()
                && !matches!(
                    value,
                    '\u{0}'..='\u{20}' | '\u{7f}' | '"' | ',' | ';' | '\\'
                )
        })
        .collect()
}

fn json_text(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Null => String::new(),
        _ => value.to_string(),
    }
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

fn response_location(headers: &reqwest::header::HeaderMap) -> Option<String> {
    headers
        .get("location")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

fn response_headers_json(headers: &reqwest::header::HeaderMap) -> Value {
    let mut output = Map::new();
    for (name, value) in headers {
        let name = name.as_str();
        if name.eq_ignore_ascii_case("set-cookie") {
            continue;
        }
        if let Ok(value) = value.to_str() {
            output.insert(name.to_string(), Value::String(value.to_string()));
        }
    }
    Value::Object(output)
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
    url: String,
    redirected: bool,
    headers: Value,
    location: Option<String>,
) -> Value {
    let mut output = Map::new();
    output.insert(
        "status".to_string(),
        Value::Number(u64::from(status.as_u16()).into()),
    );
    output.insert("ok".to_string(), Value::Bool(status.is_success()));
    output.insert("url".to_string(), Value::String(url));
    output.insert("redirected".to_string(), Value::Bool(redirected));
    output.insert("headers".to_string(), headers);
    if let Some(content_type) = content_type {
        output.insert("contentType".to_string(), Value::String(content_type));
    }
    if let Some(location) = location {
        output.insert("location".to_string(), Value::String(location));
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

#[cfg(test)]
mod reverse_proxy_tests {
    use super::*;
    use axum::Router;
    use axum::body::to_bytes;

    async fn echo(method: Method, uri: Uri, headers: HeaderMap, body: Bytes) -> Response {
        let mut response = json_response(
            StatusCode::CREATED,
            json!({
                "method": method.as_str(),
                "path": uri.path(),
                "query": uri.query(),
                "body": String::from_utf8_lossy(&body),
                "forwardedHost": headers
                    .get("x-forwarded-host")
                    .and_then(|value| value.to_str().ok()),
                "requestHeader": headers
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok()),
                "connectionForwarded": headers.contains_key("connection"),
            }),
        );
        response
            .headers_mut()
            .insert("x-upstream", HeaderValue::from_static("ready"));
        response
            .headers_mut()
            .insert("connection", HeaderValue::from_static("close"));
        response
    }

    async fn reverse_proxy_outcome(
        status: StatusCode,
        content_length: Option<&'static str>,
    ) -> ReverseProxyRequestOutcome {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener");
        let address = listener.local_addr().expect("address");
        let server = tokio::spawn(async move {
            let upstream = Router::new().fallback(move || async move {
                let body = match content_length {
                    Some(_) => Body::from("upstream"),
                    None => Body::from_stream(futures_util::stream::once(async {
                        Ok::<_, std::io::Error>(Bytes::from_static(b"upstream"))
                    })),
                };
                let mut response = Response::new(body);
                *response.status_mut() = status;
                if let Some(content_length) = content_length {
                    response
                        .headers_mut()
                        .insert("content-length", HeaderValue::from_static(content_length));
                }
                response
            });
            axum::serve(listener, upstream).await.expect("upstream");
        });
        let outcome = reverse_proxy_request(
            &format!("http://{address}"),
            &HttpMethod::Get,
            "/status",
            None,
            &HeaderMap::new(),
            &Bytes::new(),
        )
        .await;
        server.abort();
        outcome
    }

    #[tokio::test]
    async fn reverse_proxy_preserves_request_and_filters_hop_headers() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener");
        let addr = listener.local_addr().expect("address");
        let server = tokio::spawn(async move {
            axum::serve(listener, Router::new().fallback(echo))
                .await
                .expect("upstream");
        });
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("app.dowe.cloud"));
        headers.insert("x-request-id", HeaderValue::from_static("req_1"));
        headers.insert("connection", HeaderValue::from_static("keep-alive"));

        let outcome = reverse_proxy_request(
            &format!("http://{addr}"),
            &HttpMethod::Post,
            "/api/items",
            Some("page=2"),
            &headers,
            &Bytes::from_static(b"payload"),
        )
        .await;
        let ReverseProxyRequestOutcome::Upstream {
            response,
            status,
            bytes_out,
        } = outcome
        else {
            panic!("expected upstream headers");
        };

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(status, 201);
        assert!(bytes_out > 0);
        assert_eq!(
            response.headers().get("x-upstream"),
            Some(&HeaderValue::from_static("ready"))
        );
        assert!(!response.headers().contains_key("connection"));
        let body = to_bytes(response.into_body(), 4096).await.expect("body");
        let body: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(body["method"], "POST");
        assert_eq!(body["path"], "/api/items");
        assert_eq!(body["query"], "page=2");
        assert_eq!(body["body"], "payload");
        assert_eq!(body["forwardedHost"], "app.dowe.cloud");
        assert_eq!(body["requestHeader"], "req_1");
        assert_eq!(body["connectionForwarded"], false);
        server.abort();
    }

    #[tokio::test]
    async fn reverse_proxy_treats_upstream_error_headers_as_real_upstream_responses() {
        for status in [StatusCode::NOT_FOUND, StatusCode::INTERNAL_SERVER_ERROR] {
            let outcome = reverse_proxy_outcome(status, Some("8")).await;
            let ReverseProxyRequestOutcome::Upstream {
                response,
                status: observed_status,
                bytes_out,
            } = outcome
            else {
                panic!("expected upstream headers");
            };

            assert_eq!(response.status(), status);
            assert_eq!(observed_status, u64::from(status.as_u16()));
            assert_eq!(bytes_out, 8);
        }
    }

    #[tokio::test]
    async fn reverse_proxy_uses_zero_bytes_out_without_an_upstream_content_length() {
        let outcome = reverse_proxy_outcome(StatusCode::OK, None).await;
        let ReverseProxyRequestOutcome::Upstream { bytes_out, .. } = outcome else {
            panic!("expected upstream headers");
        };

        assert_eq!(bytes_out, 0);
    }

    #[tokio::test]
    async fn reverse_proxy_treats_invalid_and_unreachable_upstreams_as_local_failures() {
        let invalid = reverse_proxy_request(
            "file:///tmp/proxy",
            &HttpMethod::Get,
            "/status",
            None,
            &HeaderMap::new(),
            &Bytes::new(),
        )
        .await;
        assert!(matches!(
            invalid,
            ReverseProxyRequestOutcome::Local(response) if response.status() == StatusCode::BAD_GATEWAY
        ));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener");
        let address = listener.local_addr().expect("address");
        drop(listener);
        let unreachable = reverse_proxy_request(
            &format!("http://{address}"),
            &HttpMethod::Get,
            "/status",
            None,
            &HeaderMap::new(),
            &Bytes::new(),
        )
        .await;
        assert!(matches!(
            unreachable,
            ReverseProxyRequestOutcome::Local(response) if response.status() == StatusCode::BAD_GATEWAY
        ));
    }

    #[test]
    fn reverse_proxy_round_robin_filters_unavailable_runtimes() {
        let pool = json!([
            { "upstreamUrl": "http://runtime-a:8080", "status": "ready" },
            { "upstreamUrl": "http://runtime-b:8080", "status": "loading" },
            { "url": "http://runtime-c:8080", "status": "ready", "enabled": true },
            { "url": "http://runtime-d:8080", "status": "ready", "enabled": false }
        ]);
        let key = "round-robin-test.dowe.cloud";

        assert_eq!(
            select_reverse_proxy_upstream(&pool, ReverseProxyStrategy::RoundRobin, key).as_deref(),
            Some("http://runtime-a:8080")
        );
        assert_eq!(
            select_reverse_proxy_upstream(&pool, ReverseProxyStrategy::RoundRobin, key).as_deref(),
            Some("http://runtime-c:8080")
        );
        assert_eq!(
            select_reverse_proxy_upstream(&pool, ReverseProxyStrategy::RoundRobin, key).as_deref(),
            Some("http://runtime-a:8080")
        );
    }

    #[test]
    fn reverse_proxy_uses_temporary_redirect_for_state_fallbacks() {
        let response = reverse_proxy_redirect("https://cloud.dowe.dev/loading");

        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            response.headers().get(LOCATION),
            Some(&HeaderValue::from_static("https://cloud.dowe.dev/loading"))
        );
        assert_eq!(
            reverse_proxy_redirect("file:///tmp/loading").status(),
            StatusCode::BAD_GATEWAY
        );
    }

    #[test]
    fn reverse_proxy_enriches_dowe_task_telemetry() {
        let mut args = json!({ "event": { "projectId": "project_1" } });
        enrich_reverse_proxy_telemetry(&mut args, 201, "POST", "/api/items", 4.5, 7, 19);

        assert_eq!(args["event"]["projectId"], "project_1");
        assert_eq!(args["event"]["status"], 201);
        assert_eq!(args["event"]["method"], "POST");
        assert_eq!(args["event"]["path"], "/api/items");
        assert_eq!(args["event"]["latencyMs"], 4.5);
        assert_eq!(args["event"]["bytesIn"], 7);
        assert_eq!(args["event"]["bytesOut"], 19);
    }
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
