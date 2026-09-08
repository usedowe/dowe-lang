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
