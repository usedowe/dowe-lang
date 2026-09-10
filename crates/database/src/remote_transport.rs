fn request(operation: &str, table: Option<&str>) -> DatabaseRequest {
    DatabaseRequest {
        id: 0,
        operation: operation.to_string(),
        table: table.map(str::to_string),
        filters: Vec::new(),
        value: None,
        patch: None,
        required: false,
        sql: None,
        params: Vec::new(),
        query: None,
        operations: Vec::new(),
    }
}

async fn connect(config: &DoweDatabaseConfig) -> StoreResult<ClientSocket> {
    let url = websocket_url(config)?;
    let mut request = url
        .into_client_request()
        .map_err(|error| StoreError::Remote(error.to_string()))?;
    request.headers_mut().insert(
        "X-Dowe-Database-Account",
        HeaderValue::from_str(&config.account)
            .map_err(|error| StoreError::Authentication(error.to_string()))?,
    );
    request.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", config.secret))
            .map_err(|error| StoreError::Authentication(error.to_string()))?,
    );
    connect_async(request)
        .await
        .map(|(socket, _)| socket)
        .map_err(websocket_error)
}

fn websocket_error(error: tokio_tungstenite::tungstenite::Error) -> StoreError {
    match &error {
        tokio_tungstenite::tungstenite::Error::Http(response)
            if response.status() == StatusCode::UNAUTHORIZED =>
        {
            StoreError::Authentication("Dowe Database rejected the account or secret".to_string())
        }
        tokio_tungstenite::tungstenite::Error::Http(response)
            if response.status() == StatusCode::FORBIDDEN =>
        {
            StoreError::Authorization(
                "Dowe Database account cannot access this database".to_string(),
            )
        }
        _ => StoreError::Remote(error.to_string()),
    }
}

async fn exchange(socket: &mut ClientSocket, request: &DatabaseRequest) -> StoreResult<Value> {
    let payload =
        serde_json::to_string(request).map_err(|error| StoreError::Remote(error.to_string()))?;
    socket
        .send(TungsteniteMessage::Text(payload.into()))
        .await
        .map_err(|error| StoreError::Remote(error.to_string()))?;
    while let Some(message) = socket.next().await {
        let message = message.map_err(|error| StoreError::Remote(error.to_string()))?;
        match message {
            TungsteniteMessage::Text(text) => {
                let response = serde_json::from_str::<DatabaseResponse>(&text)
                    .map_err(|error| StoreError::Remote(error.to_string()))?;
                if response.id != request.id {
                    continue;
                }
                if response.ok {
                    return Ok(response.data.unwrap_or(Value::Null));
                }
                let error = response.error.unwrap_or(DatabaseRemoteError {
                    category: "Remote".to_string(),
                    message: "Dowe Database returned an unknown error".to_string(),
                });
                return Err(remote_error(error));
            }
            TungsteniteMessage::Close(_) => {
                return Err(StoreError::Remote(
                    "Dowe Database closed the WebSocket".to_string(),
                ));
            }
            _ => {}
        }
    }
    Err(StoreError::Remote(
        "Dowe Database WebSocket ended".to_string(),
    ))
}

