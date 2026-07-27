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
) -> dowe_database::StoreResult<Value> {
    let record = literal_record(value);
    if let Some(client) = remote_client_for_connection(project, connection)? {
        return match client {
            StoreEndpointClient::Dowe(client) => client.insert(table, record_json(&record)).await,
            StoreEndpointClient::D1(client) => client.insert(table, record_json(&record)).await,
            StoreEndpointClient::Postgres(client) => {
                client.insert(table, record_json(&record)).await
            }
        };
    }
    init_database(&project.root, &connection.database)?;
    let database = open_database(&project.root, &connection.database)?;
    Ok(record_json(&database.insert(table, record)?))
}

async fn execute_store_query(
    project: &CompiledProject,
    connection: &StoreConnection,
    sql: &str,
) -> dowe_database::StoreResult<Value> {
    if let Some(client) = remote_client_for_connection(project, connection)? {
        return match client {
            StoreEndpointClient::Dowe(client) => client.query(sql).await,
            StoreEndpointClient::D1(client) => client.query(sql).await,
            StoreEndpointClient::Postgres(client) => client.query(sql).await,
        };
    }
    init_database(&project.root, &connection.database)?;
    let database = open_database(&project.root, &connection.database)?;
    database.query_json(sql)
}

fn remote_client_for_connection(
    project: &CompiledProject,
    connection: &StoreConnection,
) -> dowe_database::StoreResult<Option<StoreEndpointClient>> {
    if project.local_databases {
        return Ok(None);
    }
    configured_database_client(project, connection).map(Some)
}

type StoreEndpointClient = ConfiguredDatabaseClient;

async fn execute_store_action_json(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &StoreActionJsonEndpoint,
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

async fn execute_kv_action_json(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &KvActionJsonEndpoint,
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

async fn execute_vector_action_json(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &VectorActionJsonEndpoint,
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
        handle_databases: HashMap::new(),
        cache_mode,
    };
    match context
        .execute(action)
        .await
        .and_then(|_| context.evaluate(&response.value))
    {
        Ok(ResolvedValue::Json(value)) => {
            json_response(status_from_u16(response.status), value)
        }
        Ok(ResolvedValue::Missing) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "invalid_response",
            "Response value is missing",
        ),
        Err(error) => json_error(error.status, error.code, error.message),
    }
}
