fn response_for_request(database: &Database, request: DatabaseRequest) -> DatabaseResponse {
    let id = request.id;
    match execute_request(database, request) {
        Ok(data) => DatabaseResponse {
            id,
            ok: true,
            data: Some(data),
            error: None,
        },
        Err(error) => DatabaseResponse {
            id,
            ok: false,
            data: None,
            error: Some(DatabaseRemoteError {
                category: error_category(&error).to_string(),
                message: error.to_string(),
            }),
        },
    }
}

fn execute_request(database: &Database, request: DatabaseRequest) -> StoreResult<Value> {
    match request.operation.as_str() {
        "list" => {
            let table = required_table(&request)?;
            let records = database.records(table)?;
            Ok(Value::Array(records.iter().map(record_to_json).collect()))
        }
        "read" => {
            let table = required_table(&request)?;
            let filters = filter_values(&request.filters)?;
            let record = database
                .records(table)?
                .into_iter()
                .find(|record| record_matches(record, &filters));
            if record.is_none() && request.required {
                return Err(StoreError::NotFound("record was not found".to_string()));
            }
            Ok(record.as_ref().map(record_to_json).unwrap_or(Value::Null))
        }
        "insert" => {
            let table = required_table(&request)?.to_string();
            let value = request
                .value
                .ok_or_else(|| StoreError::InvalidQuery("insert requires value".to_string()))?;
            let record = json_record(value)?;
            Ok(record_to_json(&database.insert(&table, record)?))
        }
        "update" => {
            let table = required_table(&request)?.to_string();
            let filters = filter_values(&request.filters)?;
            let patch =
                json_record(request.patch.ok_or_else(|| {
                    StoreError::InvalidQuery("update requires patch".to_string())
                })?)?;
            let records = database.records(&table)?;
            let mut changed = 0usize;
            for record in records
                .into_iter()
                .filter(|record| record_matches(record, &filters))
            {
                let Some(id) = record.get("id") else {
                    continue;
                };
                changed += database.update(&table, "id", id, patch.clone())?;
            }
            if changed == 0 && request.required {
                return Err(StoreError::NotFound("record was not found".to_string()));
            }
            Ok(json!({ "changed": changed }))
        }
        "delete" => {
            let table = required_table(&request)?;
            let filters = filter_values(&request.filters)?;
            let records = database.records(table)?;
            let mut changed = 0usize;
            for record in records
                .into_iter()
                .filter(|record| record_matches(record, &filters))
            {
                let Some(id) = record.get("id") else {
                    continue;
                };
                changed += database.delete(table, "id", id)?;
            }
            if changed == 0 && request.required {
                return Err(StoreError::NotFound("record was not found".to_string()));
            }
            Ok(json!({ "changed": changed }))
        }
        "query" => {
            if let Some(query) = &request.query {
                return database.query_portable_json(query, &request.params);
            }
            let sql = request
                .sql
                .as_deref()
                .ok_or_else(|| StoreError::InvalidQuery("query requires sql".to_string()))?;
            database.query_json(&bind_query_params(sql, &request.params)?)
        }
        "inspect" => serde_json::to_value(database.inspect()?)
            .map_err(|error| StoreError::Remote(error.to_string())),
        "transaction" => {
            if request.operations.is_empty() {
                return Err(StoreError::InvalidQuery(
                    "transaction requires at least one operation".to_string(),
                ));
            }
            let mut transaction = database.transaction();
            for operation in request.operations {
                transaction.insert(&operation.table, json_record(operation.value)?)?;
            }
            Ok(Value::Array(
                transaction.commit()?.iter().map(record_to_json).collect(),
            ))
        }
        operation => Err(StoreError::InvalidQuery(format!(
            "unsupported remote Database operation `{operation}`"
        ))),
    }
}

