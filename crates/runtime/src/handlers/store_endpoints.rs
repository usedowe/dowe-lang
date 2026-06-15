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
) -> dowe_store::StoreResult<Value> {
    let record = literal_record(value);
    if let Some(client) = remote_client_for_connection(project, connection)? {
        return client.insert(table, record_json(&record)).await;
    }
    init_database(&project.root, &connection.database)?;
    let database = open_database(&project.root, &connection.database)?;
    Ok(record_json(&database.insert(table, record)?))
}

async fn execute_store_query(
    project: &CompiledProject,
    connection: &StoreConnection,
    sql: &str,
) -> dowe_store::StoreResult<Value> {
    if let Some(client) = remote_client_for_connection(project, connection)? {
        return client.query(sql).await;
    }
    init_database(&project.root, &connection.database)?;
    let database = open_database(&project.root, &connection.database)?;
    database.query_json(sql)
}

fn remote_client_for_connection(
    project: &CompiledProject,
    connection: &StoreConnection,
) -> dowe_store::StoreResult<Option<RemoteStoreClient>> {
    let Some(remote) = &connection.remote else {
        return Ok(None);
    };
    let credential = match &remote.credential {
        StoreCredential::Token(value) | StoreCredential::Password(value) => {
            connection_value(project, value)?
        }
    };
    Ok(Some(RemoteStoreClient::new(RemoteStoreConfig {
        host: connection_value(project, &remote.host)?,
        database: connection.database.clone(),
        user: connection_value(project, &remote.user)?,
        credential,
    })?))
}

fn connection_value(
    project: &CompiledProject,
    value: &StoreConnectionValue,
) -> dowe_store::StoreResult<String> {
    match value {
        StoreConnectionValue::Static(value) => Ok(value.clone()),
        StoreConnectionValue::Environment(name) => project
            .environment_config
            .variable(name)
            .and_then(|variable| variable.resolved_value.clone())
            .ok_or_else(|| {
                dowe_store::StoreError::Remote(format!(
                    "Store environment variable `{name}` is not configured"
                ))
            }),
    }
}

async fn execute_store_action_json(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &StoreActionJsonEndpoint,
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

async fn execute_kv_action_json(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    response: &KvActionJsonEndpoint,
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
