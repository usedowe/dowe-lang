use super::*;
use super::transport;

pub(crate) fn backend_router(
    state: DevRuntimeState,
    websocket_paths: Vec<String>,
    cache_service: bool,
    database_service: bool,
    vector_service: bool,
    queue_service: bool,
    project_root: std::path::PathBuf,
) -> Router {
    let mut router = Router::new()
        .route(
            "/_dowe/notifications/installations",
            post(notification_installation_handler),
        )
        .route(
            "/_dowe/notifications/installations/{id}",
            delete(notification_revoke_handler),
        )
        .route("/_dowe/dev/ws", get(dev_websocket_handler))
        .route("/_dowe/dev/server", get(server_inspector_index))
        .route("/_dowe/dev/server/", get(server_inspector_index))
        .route(
            "/_dowe/dev/server/manifest.json",
            get(server_inspector_manifest),
        )
        .route(
            "/_dowe/dev/server/source/{id}",
            get(server_inspector_source),
        )
        .route("/_dowe/dev/server/data/{kind}", get(server_inspector_data))
        .route(
            "/_dowe/dev/server/execute",
            axum::routing::post(server_inspector_execute),
        )
        .route("/_dowe/dev/server/events", get(dev_websocket_handler))
        .route(
            "/_dowe/dev/server/selection",
            axum::routing::post(server_inspector_selection),
        );
    if cache_service {
        router = router.route("/v1/caches/{name}", get(cache_service_handler));
    }
    if vector_service {
        router = router.route("/v1/vectors/{name}", get(vector_service_handler));
    }
    if queue_service {
        router = router.route("/v1/queues/{name}", get(queue_service_handler));
    }
    for path in websocket_paths {
        let websocket_path = path.clone();
        router = router.route(
            &path,
            get(
                move |State(state): State<DevRuntimeState>,
                      upgrade: WebSocketUpgrade,
                      uri: axum::http::Uri,
                      headers: axum::http::HeaderMap| {
                    let path = websocket_path.clone();
                    async move {
                        backend_declared_websocket_handler(state, upgrade, uri, headers, path).await
                    }
                },
            ),
        );
    }
    let router = router.fallback(backend_handler).with_state(state);
    if database_service {
        router.merge(dowe_database::database_service_router(project_root))
    } else {
        router
    }
}

pub(crate) fn views_router(state: DevRuntimeState) -> Router {
    Router::new()
        .route("/_dowe/dev/ws", get(dev_websocket_handler))
        .route("/_dowe/dev/ipc", post(dev_native_ipc_handler))
        .fallback(views_handler)
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct NotificationInstallationRequest {
    id: String,
    platform: dowe_notifications::NotificationPlatform,
    provider: dowe_notifications::NotificationProvider,
    token: String,
    #[serde(default)]
    preferences: std::collections::BTreeMap<String, bool>,
}

async fn notification_installation_handler(
    State(state): State<DevRuntimeState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let Some(subject) = notification_auth_subject(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"authentication_required"})),
        )
            .into_response();
    };
    let request = match serde_json::from_slice::<NotificationInstallationRequest>(&body) {
        Ok(request) => request,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error":"invalid_installation", "message":error.to_string()})),
            )
                .into_response();
        }
    };
    let project = state.project.read().await.clone();
    let namespace = transport::notification_namespace(&project.root);
    let environment = match state.cache_mode {
        crate::handlers::CacheRuntimeMode::Local => "development",
        crate::handlers::CacheRuntimeMode::Production => "production",
    };
    let tenant = headers
        .get("x-dowe-tenant")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string();
    let installation = dowe_notifications::Installation {
        id: request.id,
        app: namespace.clone(),
        environment: std::env::var("DOWE_NOTIFICATION_ENVIRONMENT")
            .unwrap_or_else(|_| environment.to_string()),
        tenant,
        user: subject,
        platform: request.platform,
        provider: request.provider,
        token: request.token,
        registration_version: 0,
        active: true,
        preferences: request.preferences,
        updated_at: unix_seconds(),
    };
    let store = match dowe_notifications::NotificationStore::open(&project.root, &namespace) {
        Ok(store) => store,
        Err(error) => return notification_error_response(error),
    };
    let subject = installation.user.clone();
    match crate::register_notification_installation(&store, &subject, installation) {
        Ok(installation) => (StatusCode::CREATED, Json(json!(installation))).into_response(),
        Err(error) => notification_error_response(error),
    }
}

async fn notification_revoke_handler(
    State(state): State<DevRuntimeState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let Some(subject) = notification_auth_subject(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"authentication_required"})),
        )
            .into_response();
    };
    let project = state.project.read().await.clone();
    let namespace = transport::notification_namespace(&project.root);
    let store = match dowe_notifications::NotificationStore::open(&project.root, &namespace) {
        Ok(store) => store,
        Err(error) => return notification_error_response(error),
    };
    let installation = match store.installation(&id) {
        Ok(value) => value,
        Err(error) => return notification_error_response(error),
    };
    if installation.user != subject {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error":"installation_owner_mismatch"})),
        )
            .into_response();
    }
    match store.revoke(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => notification_error_response(error),
    }
}

fn notification_auth_subject(headers: &HeaderMap) -> Option<String> {
    let secret = std::env::var("DOWE_NOTIFICATION_JWT_SECRET").ok()?;
    let authorization = headers.get("authorization")?.to_str().ok()?;
    let token = authorization.strip_prefix("Bearer ")?;
    let claims = dowe_crypto::verify_jws_hs256(
        token,
        &secret,
        &dowe_crypto::JwtValidationOptions::default(),
    )
    .ok()?;
    claims
        .get("sub")?
        .as_str()
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn notification_error_response(error: dowe_notifications::NotificationError) -> Response {
    let status = match error {
        dowe_notifications::NotificationError::Unauthorized => StatusCode::FORBIDDEN,
        dowe_notifications::NotificationError::NotFound => StatusCode::NOT_FOUND,
        dowe_notifications::NotificationError::InvalidPayload(_)
        | dowe_notifications::NotificationError::PayloadTooLarge(_)
        | dowe_notifications::NotificationError::IdempotencyConflict => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, Json(json!({"error":error.to_string()}))).into_response()
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[derive(Debug, Deserialize)]
struct DevNativeIpcRequest {
    function: String,
    #[serde(default)]
    args: Value,
}

async fn dev_native_ipc_handler(
    State(state): State<DevRuntimeState>,
    Json(request): Json<DevNativeIpcRequest>,
) -> Json<Value> {
    let project = state.project.read().await.clone();
    match crate::invoke_native_function(
        &project,
        NativeIpcTarget::Desktop,
        &request.function,
        request.args,
    )
    .await
    {
        Ok(value) => Json(json!({ "ok": true, "data": value })),
        Err(error) => Json(json!({ "ok": false, "data": null, "error": error.to_string() })),
    }
}

pub(crate) fn desktop_router(state: DevRuntimeState, websocket_paths: Vec<String>) -> Router {
    let mut router = Router::new().route("/_dowe/dev/ws", get(dev_websocket_handler));
    for path in websocket_paths {
        let websocket_path = path.clone();
        router = router.route(
            &path,
            get(
                move |State(state): State<DevRuntimeState>,
                      upgrade: WebSocketUpgrade,
                      uri: axum::http::Uri,
                      headers: axum::http::HeaderMap| {
                    let path = websocket_path.clone();
                    async move {
                        desktop_declared_websocket_handler(state, upgrade, uri, headers, path).await
                    }
                },
            ),
        );
    }
    router.fallback(desktop_handler).with_state(state)
}

pub(crate) fn production_router(
    state: DevRuntimeState,
    websocket_paths: Vec<String>,
    cache_service: bool,
    database_service: bool,
    vector_service: bool,
    queue_service: bool,
    project_root: std::path::PathBuf,
    access: Option<ProductionAccess>,
) -> Router {
    let mut router = Router::new()
        .route(
            "/_dowe/notifications/installations",
            post(notification_installation_handler),
        )
        .route(
            "/_dowe/notifications/installations/{id}",
            delete(notification_revoke_handler),
        );
    if cache_service {
        router = router.route("/v1/caches/{name}", get(cache_service_handler));
    }
    if vector_service {
        router = router.route("/v1/vectors/{name}", get(vector_service_handler));
    }
    if queue_service {
        router = router.route("/v1/queues/{name}", get(queue_service_handler));
    }
    for path in websocket_paths {
        let websocket_path = path.clone();
        router = router.route(
            &path,
            get(
                move |State(state): State<DevRuntimeState>,
                      upgrade: WebSocketUpgrade,
                      uri: axum::http::Uri,
                      headers: axum::http::HeaderMap| {
                    let path = websocket_path.clone();
                    async move {
                        production_declared_websocket_handler(state, upgrade, uri, headers, path)
                            .await
                    }
                },
            ),
        );
    }
    let router = router.fallback(production_handler).with_state(state);
    let router = if database_service {
        router.merge(dowe_database::database_service_router(project_root))
    } else {
        router
    };
    let router = router.layer(CompressionLayer::new().br(true).gzip(true));
    if let Some(access) = access {
        router.layer(middleware::from_fn_with_state(
            access,
            crate::production_access::require_production_access,
        ))
    } else {
        router
    }
}

async fn cache_service_handler(
    State(state): State<DevRuntimeState>,
    AxumPath(name): AxumPath<String>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let project = state.project.read().await;
    dowe_cache::cache_service_upgrade(project.root.clone(), name, headers, upgrade).await
}

async fn vector_service_handler(
    State(state): State<DevRuntimeState>,
    AxumPath(name): AxumPath<String>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let project = state.project.read().await;
    dowe_vector::vector_service_upgrade(project.root.clone(), name, headers, upgrade).await
}

async fn queue_service_handler(
    State(state): State<DevRuntimeState>,
    AxumPath(name): AxumPath<String>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let project = state.project.read().await;
    dowe_queue::queue_service_upgrade(project.root.clone(), name, headers, upgrade).await
}
