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

fn record_matches_all(record: &StoreRecord, filters: &[(String, StoreValue)]) -> bool {
    filters
        .iter()
        .all(|(field, expected)| record_matches(record, field, expected))
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
        .parse::<serde_json::Number>()
        .map(Value::Number)
        .unwrap_or_else(|_| Value::String(value.to_string()))
}

fn read_json_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in path.split('.') {
        current = if let Some(object) = current.as_object() {
            object.get(part)?
        } else if let Some(array) = current.as_array() {
            array.get(part.parse::<usize>().ok()?)?
        } else {
            return None;
        };
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

async fn execute_store_transaction(
    project: &CompiledProject,
    transaction: &StoreTransactionEndpoint,
) -> dowe_database::StoreResult<Value> {
    if transaction.rollback {
        return Ok(Value::Null);
    }
    if transaction.operations.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    if let Some(client) = remote_client_for_connection(project, &transaction.connection)? {
        return match client {
            StoreEndpointClient::Dowe(client) => {
                let value = client
                    .transaction(&transaction_insert_requests(&transaction.operations))
                    .await?;
                transaction_result(value, transaction)
            }
            StoreEndpointClient::D1(client) => {
                let value = client
                    .transaction(&transaction_insert_requests(&transaction.operations))
                    .await?;
                transaction_result(value, transaction)
            }
            StoreEndpointClient::Postgres(client) => {
                let value = client
                    .transaction(&transaction_insert_requests(&transaction.operations))
                    .await?;
                transaction_result(value, transaction)
            }
        };
    }
    init_database(&project.root, &transaction.connection.database)?;
    let database = open_database(&project.root, &transaction.connection.database)?;
    execute_local_store_transaction(&database, transaction)
}

fn execute_local_store_transaction(
    database: &Database,
    transaction: &StoreTransactionEndpoint,
) -> dowe_database::StoreResult<Value> {
    if transaction.rollback {
        return Ok(Value::Null);
    }
    let mut tx = database.transaction();

    for operation in &transaction.operations {
        match operation {
            StoreTransactionOperation::Insert {
                binding,
                table,
                value,
            } => {
                let _ = binding;
                tx.insert(table, literal_record(value))?;
            }
        }
    }

    let committed = tx.commit()?;
    transaction_result(
        Value::Array(committed.iter().map(record_json).collect()),
        transaction,
    )
}

fn transaction_insert_requests(
    operations: &[StoreTransactionOperation],
) -> Vec<DatabaseTransactionInsert> {
    operations
        .iter()
        .map(|operation| match operation {
            StoreTransactionOperation::Insert { table, value, .. } => DatabaseTransactionInsert {
                table: table.clone(),
                value: record_json(&literal_record(value)),
            },
        })
        .collect()
}

fn transaction_result(
    committed: Value,
    transaction: &StoreTransactionEndpoint,
) -> dowe_database::StoreResult<Value> {
    if let Some(return_binding) = &transaction.return_binding {
        let index = transaction
            .operations
            .iter()
            .position(|operation| match operation {
                StoreTransactionOperation::Insert { binding, .. } => binding == return_binding,
            })
            .ok_or_else(|| {
                dowe_database::StoreError::InvalidQuery(format!(
                    "transaction return binding `{return_binding}` is missing"
                ))
            })?;
        return committed
            .as_array()
            .and_then(|records| records.get(index))
            .cloned()
            .ok_or_else(|| {
                dowe_database::StoreError::DurabilityError(
                    "Database transaction returned an incomplete result".to_string(),
                )
            });
    }
    Ok(committed)
}

#[cfg(test)]
mod runtime_svg_catalog_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn reads_numeric_array_segments_from_query_results() {
        let value = serde_json::json!({
            "totals": [
                { "total": 67 }
            ]
        });

        assert_eq!(
            read_json_path(&value, "totals.0.total").and_then(Value::as_u64),
            Some(67)
        );
        assert!(read_json_path(&value, "totals.first.total").is_none());
    }

    #[test]
    fn rollback_discards_staged_local_database_inserts() {
        let root = tempdir().expect("root");
        init_database(root.path(), "app").expect("database");
        let database = open_database(root.path(), "app").expect("open");
        let transaction = StoreTransactionEndpoint {
            connection: StoreConnection {
                binding: "db".to_string(),
                provider: dowe_compiler::DatabaseProvider::Dowe,
                database: "app".to_string(),
                host: None,
                port: None,
                account: None,
                secret: None,
                entities: Vec::new(),
                seeders: Vec::new(),
            },
            operations: vec![StoreTransactionOperation::Insert {
                binding: "user".to_string(),
                table: "users".to_string(),
                value: StoreLiteral::Object(vec![(
                    "name".to_string(),
                    StoreLiteral::String("Ana".to_string()),
                )]),
            }],
            return_binding: None,
            rollback: true,
        };

        assert_eq!(
            execute_local_store_transaction(&database, &transaction).expect("rollback"),
            Value::Null
        );
        assert!(database.records("users").expect("records").is_empty());
    }
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

fn store_error_response(error: dowe_database::StoreError) -> Response {
    let status = match error {
        dowe_database::StoreError::Authentication(_) => StatusCode::UNAUTHORIZED,
        dowe_database::StoreError::Authorization(_) => StatusCode::FORBIDDEN,
        dowe_database::StoreError::InvalidName(_) | dowe_database::StoreError::InvalidQuery(_) => {
            StatusCode::BAD_REQUEST
        }
        dowe_database::StoreError::AlreadyExists(_)
        | dowe_database::StoreError::TransactionConflict(_) => StatusCode::CONFLICT,
        dowe_database::StoreError::NotFound(_) => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    text_response(status, error.to_string())
}
