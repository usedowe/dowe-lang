impl RunningDatabaseService {
    pub async fn shutdown(mut self) -> StoreResult<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        match tokio::time::timeout(Duration::from_secs(2), &mut self.handle).await {
            Ok(result) => result.map_err(|error| StoreError::Remote(error.to_string()))?,
            Err(_) => {
                self.handle.abort();
                let _ = self.handle.await;
                Ok(())
            }
        }
    }

    pub async fn wait(mut self) -> StoreResult<()> {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(|error| StoreError::Remote(error.to_string()))?;
                self.shutdown().await
            }
            result = &mut self.handle => {
                result.map_err(|error| StoreError::Remote(error.to_string()))?
            }
        }
    }
}

pub async fn start_database_service(
    config: DatabaseServiceConfig,
) -> StoreResult<RunningDatabaseService> {
    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .map_err(|error| StoreError::Remote(error.to_string()))?;
    let addr = listener
        .local_addr()
        .map_err(|error| StoreError::Remote(error.to_string()))?;
    let state = DatabaseServiceState { root: config.root };
    let router = build_database_service_router(state);
    let (shutdown, signal) = oneshot::channel();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = signal.await;
            })
            .await
            .map_err(|error| StoreError::Remote(error.to_string()))
    });
    Ok(RunningDatabaseService {
        addr,
        shutdown: Some(shutdown),
        handle,
    })
}

pub async fn serve_database_service(config: DatabaseServiceConfig) -> StoreResult<()> {
    start_database_service(config).await?.wait().await
}

pub fn database_service_router(root: PathBuf) -> Router {
    build_database_service_router(DatabaseServiceState { root })
}

fn build_database_service_router(state: DatabaseServiceState) -> Router {
    Router::new()
        .route("/v1/databases/{database}", get(database_upgrade))
        .with_state(state)
}

async fn database_upgrade(
    State(state): State<DatabaseServiceState>,
    AxumPath(database): AxumPath<String>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let account = headers
        .get("X-Dowe-Database-Account")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let secret = bearer_secret(&headers).unwrap_or_default();
    match verify_account(&state.root, &database, account, secret) {
        Ok(()) => match open_database(&state.root, &database) {
            Ok(database) => upgrade
                .on_upgrade(move |socket| handle_socket(socket, database))
                .into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        Err(StoreError::Authorization(_)) => StatusCode::FORBIDDEN.into_response(),
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn handle_socket(mut socket: WebSocket, database: Database) {
    while let Some(message) = socket.recv().await {
        let Ok(message) = message else {
            return;
        };
        let AxumMessage::Text(text) = message else {
            if matches!(message, AxumMessage::Close(_)) {
                return;
            }
            continue;
        };
        let response = match serde_json::from_str::<DatabaseRequest>(&text) {
            Ok(request) => {
                let id = request.id;
                let database = database.clone();
                tokio::task::spawn_blocking(move || response_for_request(&database, request))
                    .await
                    .unwrap_or_else(|error| DatabaseResponse {
                        id,
                        ok: false,
                        data: None,
                        error: Some(DatabaseRemoteError {
                            category: "Remote".to_string(),
                            message: error.to_string(),
                        }),
                    })
            }
            Err(error) => DatabaseResponse {
                id: 0,
                ok: false,
                data: None,
                error: Some(DatabaseRemoteError {
                    category: "InvalidQuery".to_string(),
                    message: error.to_string(),
                }),
            },
        };
        let Ok(text) = serde_json::to_string(&response) else {
            return;
        };
        if socket.send(AxumMessage::Text(text.into())).await.is_err() {
            return;
        }
    }
}

