async fn verify_opaque_session(
    project: &CompiledProject,
    root: &Path,
    params: &HashMap<String, String>,
    body: &Bytes,
    raw_query: Option<&str>,
    headers: &HeaderMap,
    cache_mode: CacheRuntimeMode,
    cache: &CacheConnection,
    database: &StoreConnection,
    token: Option<&str>,
    max_age_seconds: u64,
) -> Result<Value, StoreActionError> {
    let Some(token) = token else {
        return Ok(invalid_session_value());
    };
    if dowe_id::validate_ulid(token).is_err() || !session_is_fresh(token, max_age_seconds) {
        return Ok(invalid_session_value());
    }

    let cache_key = format!("session:{token}");
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
    context
        .execute_kv(&ServerKvStatement::Handle {
            connection: cache.clone(),
        })
        .await?;
    context
        .execute_kv(&ServerKvStatement::Get {
            binding: "cachedSession".to_string(),
            handle: cache.binding.clone(),
            key: StoreLiteral::String(cache_key.clone()),
            required: false,
        })
        .await?;
    if let Some(value) = context
        .bindings
        .get("cachedSession")
        .and_then(|value| valid_session_value(token, value))
    {
        return Ok(value);
    }

    context
        .execute_store(&ServerStoreStatement::Handle {
            connection: database.clone(),
        })
        .await?;
    context
        .execute_store(&ServerStoreStatement::Read {
            binding: "persistedSession".to_string(),
            handle: database.binding.clone(),
            table: "sessions".to_string(),
            filter: StoreFilter {
                field: "id".to_string(),
                value: StoreLiteral::String(token.to_string()),
                additional: Vec::new(),
            },
            required: false,
        })
        .await?;
    let Some(value) = context
        .bindings
        .get("persistedSession")
        .and_then(|value| valid_session_value(token, value))
    else {
        return Ok(invalid_session_value());
    };
    let user_id = value
        .get("userId")
        .and_then(Value::as_str)
        .ok_or_else(StoreActionError::store)?;
    context
        .execute_kv(&ServerKvStatement::Set {
            binding: "rehydratedSession".to_string(),
            handle: cache.binding.clone(),
            key: StoreLiteral::String(cache_key),
            value: StoreLiteral::Object(vec![
                ("id".to_string(), StoreLiteral::String(token.to_string())),
                (
                    "userId".to_string(),
                    StoreLiteral::String(user_id.to_string()),
                ),
            ]),
        })
        .await?;
    Ok(value)
}

fn session_is_fresh(token: &str, max_age_seconds: u64) -> bool {
    let Ok(created_millis) = dowe_id::ulid_timestamp_millis(token) else {
        return false;
    };
    let now_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default();
    now_millis >= created_millis
        && now_millis.saturating_sub(created_millis) <= max_age_seconds.saturating_mul(1000)
}

fn valid_session_value(token: &str, value: &Value) -> Option<Value> {
    let object = value.as_object()?;
    let id = object.get("id").and_then(Value::as_str)?;
    let user_id = object.get("userId").and_then(Value::as_str)?;
    if id != token || user_id.is_empty() {
        return None;
    }
    let mut output = Map::new();
    output.insert("valid".to_string(), Value::Bool(true));
    output.insert("id".to_string(), Value::String(id.to_string()));
    output.insert("userId".to_string(), Value::String(user_id.to_string()));
    Some(Value::Object(output))
}

fn invalid_session_value() -> Value {
    let mut output = Map::new();
    output.insert("valid".to_string(), Value::Bool(false));
    Value::Object(output)
}
