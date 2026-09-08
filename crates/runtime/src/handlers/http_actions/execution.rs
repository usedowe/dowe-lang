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

