fn typed_json_value(value: &Value, schema: &DoweType) -> Result<Value, StoreActionError> {
    match schema {
        DoweType::Unknown => Ok(value.clone()),
        DoweType::Null => {
            if value.is_null() {
                Ok(Value::Null)
            } else {
                Err(StoreActionError::invalid_body(
                    "Request body does not match declared type",
                ))
            }
        }
        DoweType::Bool => value.as_bool().map(Value::Bool).ok_or_else(|| {
            StoreActionError::invalid_body("Request body does not match declared type")
        }),
        DoweType::Number => {
            if value.is_number() {
                Ok(value.clone())
            } else {
                Err(StoreActionError::invalid_body(
                    "Request body does not match declared type",
                ))
            }
        }
        DoweType::String => value
            .as_str()
            .map(|value| Value::String(value.to_string()))
            .ok_or_else(|| {
                StoreActionError::invalid_body("Request body does not match declared type")
            }),
        DoweType::Array(item) => {
            let Some(values) = value.as_array() else {
                return Err(StoreActionError::invalid_body(
                    "Request body does not match declared type",
                ));
            };
            values
                .iter()
                .map(|value| typed_json_value(value, item))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array)
        }
        DoweType::Object(fields) => {
            let Some(values) = value.as_object() else {
                return Err(StoreActionError::invalid_body(
                    "Request body does not match declared type",
                ));
            };
            let mut output = Map::new();
            for field in fields {
                match values.get(&field.name) {
                    Some(Value::Null) if field.optional => {
                        output.insert(field.name.clone(), Value::Null);
                    }
                    Some(value) => {
                        output.insert(field.name.clone(), typed_json_value(value, &field.value)?);
                    }
                    None if field.optional => {}
                    None => {
                        return Err(StoreActionError::invalid_body(
                            "Request body does not match declared type",
                        ));
                    }
                }
            }
            Ok(Value::Object(output))
        }
    }
}

impl ResolvedValue {
    fn into_json(self) -> Option<Value> {
        match self {
            ResolvedValue::Json(value) => Some(value),
            ResolvedValue::Missing => None,
        }
    }
}

fn validate_required_fields(
    record: &StoreRecord,
    fields: &[String],
) -> Result<(), StoreActionError> {
    for field in fields {
        let valid = record.get(field).is_some_and(|value| {
            value
                .to_json()
                .as_str()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
        });
        if !valid {
            return Err(StoreActionError::invalid_body(
                "Required fields must be non-empty strings",
            ));
        }
    }
    Ok(())
}

fn record_matches(record: &StoreRecord, field: &str, expected: &StoreValue) -> bool {
    record
        .get(field)
        .is_some_and(|value| value.comparable_text() == expected.comparable_text())
}

fn changed_json(changed: usize) -> Value {
    let mut output = Map::new();
    output.insert("changed".to_string(), Value::Number(changed.into()));
    Value::Object(output)
}

fn kv_set_json(key: &str) -> Value {
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(true));
    output.insert("key".to_string(), Value::String(key.to_string()));
    Value::Object(output)
}

fn kv_delete_json(deleted: bool) -> Value {
    let mut output = Map::new();
    output.insert("deleted".to_string(), Value::Bool(deleted));
    Value::Object(output)
}

fn kv_clear_json(cleared: usize) -> Value {
    let mut output = Map::new();
    output.insert("cleared".to_string(), Value::Number(cleared.into()));
    Value::Object(output)
}

fn log_json_text(value: Value) -> String {
    match value {
        Value::String(value) => value,
        value => value.to_string(),
    }
}

fn number_json(value: &str) -> Value {
    value
        .parse::<i64>()
        .map(|value| Value::Number(value.into()))
        .unwrap_or_else(|_| Value::String(value.to_string()))
}

fn read_json_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in path.split('.') {
        current = current.as_object()?.get(part)?;
    }
    Some(current)
}

fn status_from_u16(status: u16) -> StatusCode {
    StatusCode::from_u16(status).unwrap_or(StatusCode::OK)
}

fn timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    seconds.to_string()
}

fn json_error(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    let mut error = Map::new();
    error.insert("code".to_string(), Value::String(code.to_string()));
    error.insert("message".to_string(), Value::String(message.to_string()));
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(false));
    output.insert("error".to_string(), Value::Object(error));
    json_response(status, Value::Object(output))
}

fn execute_store_transaction(
    root: &Path,
    transaction: &StoreTransactionEndpoint,
) -> dowe_store::StoreResult<Value> {
    init_database(root, &transaction.database)?;
    let database = open_database(root, &transaction.database)?;
    let mut tx = database.transaction();
    let mut bindings = std::collections::BTreeMap::<String, StoreRecord>::new();

    for operation in &transaction.operations {
        match operation {
            StoreTransactionOperation::Insert {
                binding,
                table,
                value,
            } => {
                let record = tx.insert(table, literal_record(value))?;
                bindings.insert(binding.clone(), record);
            }
        }
    }

    let committed = tx.commit()?;
    if let Some(binding) = &transaction.return_binding
        && let Some(record) = bindings.get(binding)
    {
        return Ok(record_json(record));
    }
    Ok(Value::Array(committed.iter().map(record_json).collect()))
}

fn literal_record(value: &StoreLiteral) -> StoreRecord {
    match value {
        StoreLiteral::Object(entries) => entries
            .iter()
            .map(|(key, value)| (key.clone(), literal_value(value)))
            .collect(),
        _ => StoreRecord::new(),
    }
}

fn literal_value(value: &StoreLiteral) -> StoreValue {
    match value {
        StoreLiteral::Null => StoreValue::Null,
        StoreLiteral::Bool(value) => StoreValue::Bool(*value),
        StoreLiteral::Number(value) => value
            .parse::<i64>()
            .map(StoreValue::Int)
            .unwrap_or_else(|_| StoreValue::Decimal(value.clone())),
        StoreLiteral::String(value) | StoreLiteral::Reference(value) => {
            StoreValue::String(value.clone())
        }
        StoreLiteral::Array(values) => StoreValue::Json(Value::Array(
            values
                .iter()
                .map(|value| literal_value(value).to_json())
                .collect(),
        )),
        StoreLiteral::Object(entries) => StoreValue::Json(Value::Object(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), literal_value(value).to_json()))
                .collect(),
        )),
    }
}

fn record_json(record: &StoreRecord) -> Value {
    Value::Object(
        record
            .iter()
            .map(|(key, value)| (key.clone(), value.to_json()))
            .collect(),
    )
}

fn json_response(status: StatusCode, value: Value) -> Response {
    (
        status,
        [(CONTENT_TYPE, "application/json; charset=utf-8")],
        value.to_string(),
    )
        .into_response()
}

fn store_error_response(error: dowe_store::StoreError) -> Response {
    let status = match error {
        dowe_store::StoreError::Authentication(_) => StatusCode::UNAUTHORIZED,
        dowe_store::StoreError::Authorization(_) => StatusCode::FORBIDDEN,
        dowe_store::StoreError::InvalidName(_) | dowe_store::StoreError::InvalidQuery(_) => {
            StatusCode::BAD_REQUEST
        }
        dowe_store::StoreError::AlreadyExists(_)
        | dowe_store::StoreError::TransactionConflict(_) => StatusCode::CONFLICT,
        dowe_store::StoreError::NotFound(_) => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    text_response(status, error.to_string())
}
