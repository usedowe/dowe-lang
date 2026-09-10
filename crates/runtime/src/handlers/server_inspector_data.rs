fn inspector_execute_error(status: StatusCode, message: &str) -> Response {
    no_store((status, axum::Json(json!({ "error": message }))).into_response())
}

fn inspector_query_string(query: &Map<String, Value>) -> String {
    query
        .iter()
        .filter_map(|(name, value)| {
            let value = match value {
                Value::String(value) => value.clone(),
                Value::Null => return None,
                value => value.to_string(),
            };
            Some(format!(
                "{}={}",
                inspector_url_encode(name),
                inspector_url_encode(&value)
            ))
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn inspector_url_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

fn inspector_response_headers(headers: &HeaderMap) -> Map<String, Value> {
    headers
        .iter()
        .filter(|(name, _)| {
            !matches!(
                name.as_str().to_ascii_lowercase().as_str(),
                "set-cookie" | "authorization" | "proxy-authorization" | "cookie"
            )
        })
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), Value::String(value.to_string())))
        })
        .collect()
}

pub(crate) async fn server_inspector_selection(
    State(state): State<DevRuntimeState>,
    body: Bytes,
) -> Response {
    if body.len() > 16 * 1024 {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let project = state.project.read().await;
    if project.server_inspector.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(id) = serde_json::from_slice::<Value>(&body)
        .ok()
        .and_then(|value| value.get("id").and_then(Value::as_str).map(str::to_string))
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let manifest = project.server_inspector.as_ref().expect("checked above");
    let known = manifest.nodes.iter().any(|node| node.id == id)
        || manifest.routes.iter().any(|route| route.id == id)
        || manifest.websockets.iter().any(|route| route.id == id)
        || manifest.jobs.iter().any(|job| job.id == id);
    if !known {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let selection_path = project
        .root
        .join(".dowe")
        .join("dev")
        .join("server-inspector-selection.json");
    if let Some(parent) = selection_path.parent()
        && fs::create_dir_all(parent).is_ok()
    {
        let _ = fs::write(selection_path, json!({ "id": id }).to_string());
    }
    no_store(StatusCode::NO_CONTENT.into_response())
}

fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    response
}

fn inspect_databases(root: &Path, query: &ServerInspectorDataQuery) -> Value {
    if let Some(name) = query.name.as_deref() {
        let Ok(database) = dowe_database::open_database(root, name) else {
            return inspector_data_error("database", "Database was not found");
        };
        let Ok(inspection) = database.inspect() else {
            return inspector_data_error("database", "Database metadata is unavailable");
        };
        if let Some(table) = query.table.as_deref() {
            let Some(table_info) = inspection.tables.iter().find(|item| item.name == table) else {
                return inspector_data_error("database", "Table was not found");
            };
            let limit = inspector_limit(query);
            let sql = format!("SELECT * FROM {} LIMIT {limit}", table_info.name);
            let Ok(rows) = database.query_json(&sql) else {
                return inspector_data_error("database", "Table records are unavailable");
            };
            let rows = sanitize_inspector_value(rows);
            return json!({
                "kind": "database",
                "name": name,
                "table": table,
                "columns": inspector_columns(&rows),
                "rows": rows,
                "total": table_info.records,
                "readOnly": true,
                "limit": limit,
            });
        }
        return json!({ "kind": "database", "name": name, "item": inspection, "readOnly": true });
    }

    let mut items = Vec::new();
    if let Ok(names) = dowe_database::list_databases(root) {
        for metadata in names.into_iter().take(100) {
            if let Ok(database) = dowe_database::open_database(root, &metadata.name)
                && let Ok(inspection) = database.inspect()
            {
                items.push(json!(inspection));
            }
        }
    }
    json!({ "kind": "database", "items": items, "readOnly": true, "limit": 100 })
}

fn inspect_caches(root: &Path, query: &ServerInspectorDataQuery) -> Value {
    if let Some(name) = query.name.as_deref() {
        let Ok(database) = dowe_cache::open_database(root, name, false) else {
            return inspector_data_error("cache", "Cache was not found");
        };
        let Ok(mut inspection) = database.inspect() else {
            return inspector_data_error("cache", "Cache metadata is unavailable");
        };
        inspection.keys.truncate(200);
        inspection.keys = inspection
            .keys
            .into_iter()
            .map(sanitize_runtime_key)
            .collect();
        if let Some(key) = query.key.as_deref() {
            let Ok(value) = database.get(key) else {
                return inspector_data_error("cache", "Cache value is unavailable");
            };
            return json!({
                "kind": "cache",
                "name": name,
                "key": sanitize_runtime_key(key.to_string()),
                "exists": value.is_some(),
                "value": value.map(sanitize_inspector_value),
                "readOnly": true,
            });
        }
        return json!({ "kind": "cache", "name": name, "item": cache_inspection_json(&inspection), "readOnly": true });
    }

    let mut items = Vec::new();
    if let Ok(names) = dowe_cache::list_databases(root) {
        for name in names.into_iter().take(100) {
            if let Ok(database) = dowe_cache::open_database(root, &name, false)
                && let Ok(mut inspection) = database.inspect()
            {
                inspection.keys.truncate(200);
                inspection.keys = inspection
                    .keys
                    .into_iter()
                    .map(sanitize_runtime_key)
                    .collect();
                items.push(cache_inspection_json(&inspection));
            }
        }
    }
    json!({ "kind": "cache", "items": items, "readOnly": true, "limit": 100, "keyLimit": 200 })
}

fn sanitize_runtime_key(key: String) -> String {
    let lower = key.to_ascii_lowercase();
    if [
        "secret",
        "token",
        "password",
        "authorization",
        "cookie",
        "credential",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        "[redacted]".to_string()
    } else {
        key
    }
}

fn inspect_vectors(root: &Path, query: &ServerInspectorDataQuery) -> Value {
    if let Some(name) = query.name.as_deref() {
        let Ok(database) = dowe_vector::open_database(root, name, false) else {
            return inspector_data_error("vector", "Vector database was not found");
        };
        let Ok(inspection) = database.inspect() else {
            return inspector_data_error("vector", "Vector metadata is unavailable");
        };
        if let Some(id) = query.id.as_deref() {
            let Ok(embedding) = database.read(id) else {
                return inspector_data_error("vector", "Embedding is unavailable");
            };
            return json!({
                "kind": "vector",
                "name": name,
                "id": id,
                "item": embedding.map(vector_item_json),
                "readOnly": true,
            });
        }
        let Ok(embeddings) = database.list(inspector_limit(query), None) else {
            return inspector_data_error("vector", "Embeddings are unavailable");
        };
        let rows = embeddings
            .into_iter()
            .map(vector_item_json)
            .collect::<Vec<_>>();
        return json!({
            "kind": "vector",
            "name": name,
            "item": inspection,
            "rows": rows,
            "readOnly": true,
            "limit": inspector_limit(query),
        });
    }

    let mut items = Vec::new();
    if let Ok(names) = dowe_vector::list_databases(root) {
        for name in names.into_iter().take(100) {
            if let Ok(database) = dowe_vector::open_database(root, &name, false)
                && let Ok(inspection) = database.inspect()
            {
                items.push(json!(inspection));
            }
        }
    }
    json!({ "kind": "vector", "items": items, "readOnly": true, "limit": 100 })
}

fn inspect_queues(root: &Path, query: &ServerInspectorDataQuery) -> Value {
    if let Some(name) = query.name.as_deref() {
        let Ok(queue) = dowe_queue::open_namespace(root, name) else {
            return inspector_data_error("queue", "Queue namespace was not found");
        };
        let Ok(inspection) = queue.inspect() else {
            return inspector_data_error("queue", "Queue metadata is unavailable");
        };
        if let Some(queue_name) = query.queue.as_deref() {
            let Ok(messages) = queue.inspect_messages(queue_name, inspector_limit(query)) else {
                return inspector_data_error("queue", "Queue messages are unavailable");
            };
            let rows = messages
                .into_iter()
                .map(|message| {
                    json!({
                        "id": message.id,
                        "topic": message.topic,
                        "value": sanitize_inspector_value(message.value),
                        "publishedAt": message.published_at,
                        "redelivered": message.redelivered,
                    })
                })
                .collect::<Vec<_>>();
            return json!({
                "kind": "queue",
                "name": name,
                "queue": queue_name,
                "rows": rows,
                "readOnly": true,
                "limit": inspector_limit(query),
            });
        }
        return json!({ "kind": "queue", "name": name, "item": inspection, "readOnly": true });
    }

    let mut items = Vec::new();
    if let Ok(names) = dowe_queue::list_namespaces(root) {
        for name in names.into_iter().take(100) {
            if let Ok(queue) = dowe_queue::open_namespace(root, &name)
                && let Ok(inspection) = queue.inspect()
            {
                items.push(json!(inspection));
            }
        }
    }
    json!({ "kind": "queue", "items": items, "readOnly": true, "limit": 100 })
}

fn inspector_limit(query: &ServerInspectorDataQuery) -> usize {
    query.limit.unwrap_or(100).clamp(1, 100)
}

fn inspector_data_error(kind: &str, message: &str) -> Value {
    json!({ "kind": kind, "error": message, "readOnly": true })
}

fn cache_inspection_json(inspection: &dowe_cache::KvInspection) -> Value {
    json!({
        "name": inspection.name,
        "persistent": inspection.persistent,
        "memoryKeys": inspection.memory_keys,
        "persistedKeys": inspection.persisted_keys,
        "keys": inspection.keys,
    })
}

fn vector_item_json(embedding: dowe_vector::Embedding) -> Value {
    let dimensions = embedding.vector.len();
    json!({
        "id": embedding.id,
        "dimensions": dimensions,
        "vector": embedding.vector.into_iter().take(64).collect::<Vec<_>>(),
        "vectorTruncated": dimensions > 64,
        "metadata": sanitize_inspector_value(embedding.metadata),
    })
}

fn inspector_columns(rows: &Value) -> Vec<String> {
    let mut columns = std::collections::BTreeSet::new();
    if let Value::Array(rows) = rows {
        for row in rows.iter().take(100) {
            if let Value::Object(row) = row {
                columns.extend(row.keys().cloned());
            }
        }
    }
    columns.into_iter().collect()
}

fn sanitize_inspector_value(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .take(100)
                .map(|(key, value)| {
                    let value = if inspector_sensitive_key(&key) {
                        Value::String("[redacted]".to_string())
                    } else {
                        sanitize_inspector_value(value)
                    };
                    (key, value)
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .take(100)
                .map(sanitize_inspector_value)
                .collect(),
        ),
        Value::String(value) => {
            let truncated = value.chars().take(4096).collect::<String>();
            if truncated.len() < value.len() {
                Value::String(format!("{truncated}…"))
            } else {
                Value::String(value)
            }
        }
        other => other,
    }
}

fn inspector_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "secret",
        "token",
        "password",
        "authorization",
        "cookie",
        "credential",
        "private_key",
        "request_body",
        "response_body",
        "hash",
    ]
    .iter()
    .any(|needle| key.contains(needle))
}

fn safe_source_path(root: &Path, relative: &str) -> Option<PathBuf> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return None;
    }
    let path = root.join(relative_path);
    path.starts_with(root).then_some(path)
}


