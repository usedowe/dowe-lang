fn websocket_url(config: &DoweDatabaseConfig) -> StoreResult<String> {
    let host = config.host.trim().trim_end_matches('/');
    let (scheme, authority) = if let Some(value) = host.strip_prefix("https://") {
        ("wss", value)
    } else if let Some(value) = host.strip_prefix("http://") {
        ("ws", value)
    } else if let Some(value) = host.strip_prefix("wss://") {
        ("wss", value)
    } else if let Some(value) = host.strip_prefix("ws://") {
        ("ws", value)
    } else if is_loopback(host) {
        ("ws", host)
    } else {
        ("wss", host)
    };
    let authority = authority.split('/').next().unwrap_or(authority);
    let authority = if authority.contains(':') {
        authority.to_string()
    } else {
        format!("{authority}:{}", config.port)
    };
    if scheme == "ws" && !is_loopback(authority.split(':').next().unwrap_or_default()) {
        return Err(StoreError::Remote(
            "remote Dowe Database connections require `wss://`".to_string(),
        ));
    }
    Ok(format!(
        "{scheme}://{authority}/v1/databases/{}",
        config.database
    ))
}

fn bearer_secret(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
}

fn required_table(request: &DatabaseRequest) -> StoreResult<&str> {
    request
        .table
        .as_deref()
        .ok_or_else(|| StoreError::InvalidQuery("operation requires table".to_string()))
}

fn filter_values(filters: &[(String, Value)]) -> StoreResult<Vec<(String, StoreValue)>> {
    if filters.is_empty() {
        return Err(StoreError::InvalidQuery(
            "operation requires equality filters".to_string(),
        ));
    }
    filters
        .iter()
        .map(|(field, value)| {
            validate_field_name(field)?;
            Ok((field.clone(), StoreValue::from_json(value.clone())))
        })
        .collect()
}

fn record_matches(record: &StoreRecord, filters: &[(String, StoreValue)]) -> bool {
    filters.iter().all(|(field, expected)| {
        record
            .get(field)
            .is_some_and(|value| value.comparable_text() == expected.comparable_text())
    })
}

fn json_record(value: Value) -> StoreResult<StoreRecord> {
    let Value::Object(value) = value else {
        return Err(StoreError::InvalidQuery(
            "record value must be an object".to_string(),
        ));
    };
    value
        .into_iter()
        .map(|(key, value)| {
            validate_field_name(&key)?;
            Ok((key, StoreValue::from_json(value)))
        })
        .collect::<StoreResult<BTreeMap<_, _>>>()
}

fn remote_error(error: DatabaseRemoteError) -> StoreError {
    match error.category.as_str() {
        "Authentication" => StoreError::Authentication(error.message),
        "Authorization" => StoreError::Authorization(error.message),
        "NotFound" => StoreError::NotFound(error.message),
        "AlreadyExists" => StoreError::AlreadyExists(error.message),
        "InvalidName" => StoreError::InvalidName(error.message),
        "InvalidUlid" => StoreError::InvalidUlid(error.message),
        "InvalidQuery" => StoreError::InvalidQuery(error.message),
        "TypeError" => StoreError::TypeError(error.message),
        "TransactionConflict" => StoreError::TransactionConflict(error.message),
        "DurabilityError" => StoreError::DurabilityError(error.message),
        "Corruption" => StoreError::Corruption(error.message),
        "UnsupportedFormat" => StoreError::UnsupportedFormat(error.message),
        _ => StoreError::Remote(error.message),
    }
}

fn error_category(error: &StoreError) -> &'static str {
    match error {
        StoreError::NotFound(_) => "NotFound",
        StoreError::AlreadyExists(_) => "AlreadyExists",
        StoreError::InvalidName(_) => "InvalidName",
        StoreError::InvalidUlid(_) => "InvalidUlid",
        StoreError::InvalidQuery(_) => "InvalidQuery",
        StoreError::TypeError(_) => "TypeError",
        StoreError::TransactionConflict(_) => "TransactionConflict",
        StoreError::DurabilityError(_) => "DurabilityError",
        StoreError::Corruption(_) => "Corruption",
        StoreError::UnsupportedFormat(_) => "UnsupportedFormat",
        StoreError::Authentication(_) => "Authentication",
        StoreError::Authorization(_) => "Authorization",
        StoreError::Remote(_) | StoreError::Io(_) => "Remote",
    }
}

fn is_transport_error(error: &StoreError) -> bool {
    matches!(error, StoreError::Remote(_))
}

fn is_loopback(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}
