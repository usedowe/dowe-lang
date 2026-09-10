fn identifier(value: &str) -> String {
    value
        .split('.')
        .map(|part| format!("\"{part}\""))
        .collect::<Vec<_>>()
        .join(".")
}

fn generates_record_id(table: &str) -> bool {
    !table.starts_with("_dowe_")
}

fn d1_value(value: Value) -> StoreResult<Value> {
    match value {
        Value::Bool(value) => Ok(Value::from(if value { 1 } else { 0 })),
        Value::Null | Value::Number(_) | Value::String(_) => Ok(value),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(&value)
            .map(Value::String)
            .map_err(Into::into),
    }
}

fn decode_bool(value: Value) -> Value {
    match value {
        Value::Number(value) => Value::Bool(value.as_i64().unwrap_or_default() != 0),
        Value::String(value) if value == "0" => Value::Bool(false),
        Value::String(value) if value == "1" => Value::Bool(true),
        value => value,
    }
}

fn decode_json(value: Value) -> StoreResult<Value> {
    match value {
        Value::String(value) => serde_json::from_str(&value).map_err(Into::into),
        value => Ok(value),
    }
}

fn d1_error(message: String) -> StoreError {
    let normalized = message.to_ascii_lowercase();
    if normalized.contains("unique constraint") {
        StoreError::AlreadyExists(message)
    } else if normalized.contains("no such table")
        || normalized.contains("no such column")
        || normalized.contains("syntax error")
    {
        StoreError::InvalidQuery(message)
    } else {
        StoreError::Remote(message)
    }
}
