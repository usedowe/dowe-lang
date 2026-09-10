#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct ServerInspectorDataQuery {
    name: Option<String>,
    table: Option<String>,
    key: Option<String>,
    queue: Option<String>,
    id: Option<String>,
    limit: Option<usize>,
}

pub(crate) async fn server_inspector_index(State(state): State<DevRuntimeState>) -> Response {
    let project = state.project.read().await;
    if project.server_inspector.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }
    no_store(Html(server_inspector_html()).into_response())
}

pub(crate) async fn server_inspector_manifest(State(state): State<DevRuntimeState>) -> Response {
    let project = state.project.read().await;
    let Some(manifest) = project.server_inspector.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    no_store(axum::Json(manifest).into_response())
}

pub(crate) async fn server_inspector_source(
    State(state): State<DevRuntimeState>,
    InspectorPath(id): InspectorPath<String>,
) -> Response {
    let project = state.project.read().await;
    let Some(manifest) = project.server_inspector.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let source = manifest
        .nodes
        .iter()
        .find(|node| node.id == id)
        .and_then(|node| node.source.clone())
        .or_else(|| {
            manifest
                .routes
                .iter()
                .find(|route| route.id == id)
                .and_then(|route| route.source.clone())
        })
        .or_else(|| {
            manifest
                .websockets
                .iter()
                .find(|route| route.id == id)
                .and_then(|route| route.source.clone())
        })
        .or_else(|| {
            manifest
                .jobs
                .iter()
                .find(|job| job.id == id)
                .and_then(|job| job.source.clone())
        });
    let Some(source) = source else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(path) = safe_source_path(&project.root, &source.path) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Ok(content) = fs::read_to_string(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let lines = content.lines().collect::<Vec<_>>();
    let start = source.line.max(1).min(lines.len().max(1));
    let end = source
        .end_line
        .max(start)
        .min((start + 200).min(lines.len().max(1)));
    let code = lines
        .get(start.saturating_sub(1)..end)
        .unwrap_or_default()
        .join("\n");
    no_store(
        axum::Json(json!({
            "id": id,
            "path": source.path,
            "startLine": start,
            "endLine": end,
            "code": code,
        }))
        .into_response(),
    )
}

pub(crate) async fn server_inspector_data(
    State(state): State<DevRuntimeState>,
    InspectorPath(kind): InspectorPath<String>,
    Query(query): Query<ServerInspectorDataQuery>,
) -> Response {
    let project = state.project.read().await;
    if project.server_inspector.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let root = project.root.clone();
    let payload = match kind.as_str() {
        "database" | "databases" => inspect_databases(&root, &query),
        "cache" | "caches" => inspect_caches(&root, &query),
        "vector" | "vectors" => inspect_vectors(&root, &query),
        "queue" | "queues" => inspect_queues(&root, &query),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    no_store(axum::Json(payload).into_response())
}

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct ServerInspectorExecuteRequest {
    id: String,
    method: String,
    path: String,
    #[serde(default)]
    query: Map<String, Value>,
    #[serde(default)]
    headers: Map<String, Value>,
    body: Option<Value>,
}

pub(crate) async fn server_inspector_execute(
    State(state): State<DevRuntimeState>,
    body: Bytes,
) -> Response {
    if body.len() > 512 * 1024 {
        return inspector_execute_error(StatusCode::PAYLOAD_TOO_LARGE, "Request is too large");
    }
    let Ok(request) = serde_json::from_slice::<ServerInspectorExecuteRequest>(&body) else {
        return inspector_execute_error(StatusCode::BAD_REQUEST, "Invalid execute request");
    };
    if request.path.len() > 4096
        || !request.path.starts_with('/')
        || request.path.contains(['\r', '\n'])
        || request.id.is_empty()
    {
        return inspector_execute_error(StatusCode::BAD_REQUEST, "Invalid endpoint path");
    }
    let Ok(method) = Method::from_bytes(request.method.as_bytes()) else {
        return inspector_execute_error(StatusCode::BAD_REQUEST, "Invalid endpoint method");
    };
    let project = state.project.read().await;
    let Some(manifest) = project.server_inspector.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(route) = manifest.routes.iter().find(|route| route.id == request.id) else {
        return inspector_execute_error(StatusCode::BAD_REQUEST, "Endpoint is not in the manifest");
    };
    if route.method != method.as_str() {
        return inspector_execute_error(StatusCode::BAD_REQUEST, "Endpoint method does not match");
    }
    let Ok(http_method) = HttpMethod::from_str(method.as_str()) else {
        return inspector_execute_error(
            StatusCode::METHOD_NOT_ALLOWED,
            "Unsupported endpoint method",
        );
    };
    let Some(matched) = project.backend.find_endpoint(&http_method, &request.path) else {
        return inspector_execute_error(StatusCode::NOT_FOUND, "Endpoint path did not match");
    };
    if matched.endpoint.path != route.path || matched.endpoint.method.as_str() != route.method {
        return inspector_execute_error(StatusCode::BAD_REQUEST, "Endpoint path is not allowed");
    }
    let mut headers = HeaderMap::new();
    for (name, value) in request.headers {
        let Ok(name) = HeaderName::from_bytes(name.as_bytes()) else {
            return inspector_execute_error(StatusCode::BAD_REQUEST, "Invalid request header");
        };
        let Some(value) = value.as_str() else {
            return inspector_execute_error(
                StatusCode::BAD_REQUEST,
                "Request headers must be strings",
            );
        };
        if value.len() > 16 * 1024 {
            return inspector_execute_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "Request header is too large",
            );
        }
        let Ok(value) = HeaderValue::from_str(value) else {
            return inspector_execute_error(
                StatusCode::BAD_REQUEST,
                "Invalid request header value",
            );
        };
        headers.insert(name, value);
    }
    let query = inspector_query_string(&request.query);
    let raw_query = (!query.is_empty()).then_some(query.as_str());
    let request_body = match request.body {
        None | Some(Value::Null) => Bytes::new(),
        Some(Value::String(value)) => Bytes::from(value),
        Some(value) => match serde_json::to_vec(&value) {
            Ok(value) if value.len() <= 512 * 1024 => Bytes::from(value),
            Ok(_) => {
                return inspector_execute_error(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "Request body is too large",
                );
            }
            Err(_) => {
                return inspector_execute_error(
                    StatusCode::BAD_REQUEST,
                    "Request body is not serializable",
                );
            }
        },
    };
    let response = server_response(
        &project,
        &project.backend,
        &state.dev_origins,
        state.cache_mode,
        method,
        &request.path,
        raw_query,
        headers,
        request_body,
    )
    .await;
    let status = response.status();
    let response_headers = inspector_response_headers(response.headers());
    let (body, truncated) = match axum::body::to_bytes(response.into_body(), 256 * 1024).await {
        Ok(body) => (body, false),
        Err(_) => (
            Bytes::from_static(b"Response body exceeded the inspector limit"),
            true,
        ),
    };
    let body_text = String::from_utf8_lossy(&body).to_string();
    no_store(
        axum::Json(json!({
            "status": status.as_u16(),
            "statusText": status.canonical_reason().unwrap_or(""),
            "headers": response_headers,
            "body": body_text,
            "truncated": truncated,
        }))
        .into_response(),
    )
}


