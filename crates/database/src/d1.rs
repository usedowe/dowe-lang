use crate::error::{StoreError, StoreResult};
use crate::names::{validate_field_name, validate_table_name};
use crate::remote::DatabaseTransactionInsert;
use dowe_database_query::{QueryDialect, QueryProjectionValue, SelectQuery, render_select};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct D1Config {
    pub account: String,
    pub database: String,
    pub secret: String,
    pub schema: Vec<D1TableSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct D1TableSchema {
    pub table: String,
    pub bool_fields: Vec<String>,
    pub json_fields: Vec<String>,
}

#[derive(Clone)]
pub struct D1Client {
    config: D1Config,
    client: reqwest::Client,
    endpoint_override: Option<String>,
}

#[derive(Debug, Serialize)]
struct D1QueryRequest<'a> {
    sql: &'a str,
    params: &'a [Value],
}

#[derive(Debug, Serialize)]
struct D1BatchRequest<'a> {
    batch: &'a [D1BatchStatement],
}

#[derive(Debug, Serialize)]
struct D1BatchStatement {
    sql: String,
    params: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct D1Envelope {
    success: bool,
    #[serde(default)]
    result: Vec<D1QueryResult>,
    #[serde(default)]
    errors: Vec<D1Error>,
}

#[derive(Debug, Deserialize)]
struct D1QueryResult {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    results: Vec<Value>,
    #[serde(default)]
    meta: D1Meta,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct D1Meta {
    #[serde(default)]
    changes: usize,
}

#[derive(Debug, Deserialize)]
struct D1Error {
    #[serde(default)]
    message: String,
}

struct D1Execution {
    rows: Vec<Value>,
    changes: usize,
}

impl D1Client {
    pub fn new(config: D1Config) -> StoreResult<Self> {
        if config.account.trim().is_empty() {
            return Err(StoreError::Remote("D1 account is empty".to_string()));
        }
        if config.database.trim().is_empty() {
            return Err(StoreError::Remote("D1 database ID is empty".to_string()));
        }
        if config.secret.is_empty() {
            return Err(StoreError::Authentication("D1 secret is empty".to_string()));
        }
        Ok(Self {
            config,
            client: reqwest::Client::new(),
            endpoint_override: None,
        })
    }

    #[cfg(test)]
    fn for_endpoint(endpoint: String, secret: String) -> Self {
        Self {
            config: D1Config {
                account: "test-account".to_string(),
                database: "test-database".to_string(),
                secret,
                schema: Vec::new(),
            },
            client: reqwest::Client::new(),
            endpoint_override: Some(endpoint),
        }
    }

    pub async fn list(&self, table: &str) -> StoreResult<Value> {
        validate_table_name(table)?;
        let sql = format!("SELECT * FROM {}", identifier(table));
        let execution = self.execute(&sql, &[]).await?;
        Ok(Value::Array(self.decode_rows(Some(table), execution.rows)?))
    }

    pub async fn read(
        &self,
        table: &str,
        filters: &[(String, Value)],
        required: bool,
    ) -> StoreResult<Value> {
        validate_table_name(table)?;
        let (where_sql, params) = self.filters_sql(table, filters, 1)?;
        let sql = format!(
            "SELECT * FROM {} WHERE {where_sql} LIMIT 1",
            identifier(table)
        );
        let execution = self.execute(&sql, &params).await?;
        let value = self
            .decode_rows(Some(table), execution.rows)?
            .into_iter()
            .next()
            .unwrap_or(Value::Null);
        if value.is_null() && required {
            return Err(StoreError::NotFound("D1 record was not found".to_string()));
        }
        Ok(value)
    }

    pub async fn insert(&self, table: &str, value: Value) -> StoreResult<Value> {
        validate_table_name(table)?;
        let statement = self.insert_statement(table, value)?;
        let execution = self.execute(&statement.sql, &statement.params).await?;
        self.decode_rows(Some(table), execution.rows)?
            .into_iter()
            .next()
            .ok_or_else(|| {
                StoreError::Remote("D1 insert did not return the created record".to_string())
            })
    }

    pub async fn update(
        &self,
        table: &str,
        filters: &[(String, Value)],
        patch: Value,
        required: bool,
    ) -> StoreResult<Value> {
        validate_table_name(table)?;
        let Value::Object(patch) = patch else {
            return Err(StoreError::InvalidQuery(
                "D1 update value must be an object".to_string(),
            ));
        };
        if patch.is_empty() {
            return Ok(json!({ "changed": 0 }));
        }
        let mut entries = patch.into_iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        let mut params = Vec::with_capacity(entries.len() + filters.len());
        let mut assignments = Vec::with_capacity(entries.len());
        for (index, (field, value)) in entries.into_iter().enumerate() {
            validate_field_name(&field)?;
            assignments.push(format!("{} = ?{}", identifier(&field), index + 1));
            params.push(self.encode_value(table, &field, value)?);
        }
        let (where_sql, filter_params) = self.filters_sql(table, filters, params.len() + 1)?;
        params.extend(filter_params);
        let sql = format!(
            "UPDATE {} SET {} WHERE {where_sql}",
            identifier(table),
            assignments.join(", ")
        );
        let execution = self.execute(&sql, &params).await?;
        if execution.changes == 0 && required {
            return Err(StoreError::NotFound("D1 record was not found".to_string()));
        }
        Ok(json!({ "changed": execution.changes }))
    }

    pub async fn delete(
        &self,
        table: &str,
        filters: &[(String, Value)],
        required: bool,
    ) -> StoreResult<Value> {
        validate_table_name(table)?;
        let (where_sql, params) = self.filters_sql(table, filters, 1)?;
        let sql = format!("DELETE FROM {} WHERE {where_sql}", identifier(table));
        let execution = self.execute(&sql, &params).await?;
        if execution.changes == 0 && required {
            return Err(StoreError::NotFound("D1 record was not found".to_string()));
        }
        Ok(json!({ "changed": execution.changes }))
    }

    pub async fn query(&self, sql: &str) -> StoreResult<Value> {
        self.query_with_params(sql, &[]).await
    }

    pub async fn query_with_params(&self, sql: &str, params: &[Value]) -> StoreResult<Value> {
        let params = params
            .iter()
            .cloned()
            .map(d1_value)
            .collect::<StoreResult<Vec<_>>>()?;
        let execution = self.execute(sql, &params).await?;
        Ok(Value::Array(self.decode_rows(None, execution.rows)?))
    }

    pub async fn query_select(&self, query: &SelectQuery, params: &[Value]) -> StoreResult<Value> {
        let params = params
            .iter()
            .cloned()
            .map(d1_value)
            .collect::<StoreResult<Vec<_>>>()?;
        let execution = self
            .execute(&render_select(query, QueryDialect::D1), &params)
            .await?;
        Ok(Value::Array(
            self.decode_select_rows(query, execution.rows)?,
        ))
    }

    pub async fn execute_batch(&self, sql: &str) -> StoreResult<()> {
        let statements = sql
            .split(';')
            .map(str::trim)
            .filter(|sql| !sql.is_empty())
            .map(|sql| D1BatchStatement {
                sql: sql.to_string(),
                params: Vec::new(),
            })
            .collect::<Vec<_>>();
        self.execute_atomic_batch(&statements).await?;
        Ok(())
    }

    pub async fn transaction(
        &self,
        operations: &[DatabaseTransactionInsert],
    ) -> StoreResult<Value> {
        if operations.is_empty() {
            return Err(StoreError::InvalidQuery(
                "D1 transaction requires at least one operation".to_string(),
            ));
        }
        let statements = operations
            .iter()
            .map(|operation| self.insert_statement(&operation.table, operation.value.clone()))
            .collect::<StoreResult<Vec<_>>>()?;
        let executions = self.execute_atomic_batch(&statements).await?;
        let mut values = Vec::with_capacity(executions.len());
        for (operation, execution) in operations.iter().zip(executions) {
            let value = self
                .decode_rows(Some(&operation.table), execution.rows)?
                .into_iter()
                .next()
                .ok_or_else(|| {
                    StoreError::Remote(
                        "D1 transaction insert did not return the created record".to_string(),
                    )
                })?;
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    async fn execute(&self, sql: &str, params: &[Value]) -> StoreResult<D1Execution> {
        let mut executions = self.send(&D1QueryRequest { sql, params }).await?;
        executions.pop().ok_or_else(|| {
            StoreError::Remote("D1 response did not include a query result".to_string())
        })
    }

    async fn execute_atomic_batch(
        &self,
        statements: &[D1BatchStatement],
    ) -> StoreResult<Vec<D1Execution>> {
        if statements.is_empty() {
            return Ok(Vec::new());
        }
        self.send(&D1BatchRequest { batch: statements }).await
    }

    async fn send<T>(&self, body: &T) -> StoreResult<Vec<D1Execution>>
    where
        T: Serialize + ?Sized,
    {
        let response = self
            .client
            .post(self.endpoint_override.clone().unwrap_or_else(|| {
                format!(
                    "https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/query",
                    self.config.account, self.config.database
                )
            }))
            .timeout(Duration::from_secs(30))
            .bearer_auth(&self.config.secret)
            .json(body)
            .send()
            .await
            .map_err(|error| {
                if error.is_timeout() {
                    StoreError::Remote("D1 request timed out".to_string())
                } else {
                    StoreError::Remote("D1 HTTP request failed".to_string())
                }
            })?;
        let status = response.status();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(StoreError::Authentication(format!(
                "D1 rejected the request with HTTP {status}"
            )));
        }
        let envelope = response
            .json::<D1Envelope>()
            .await
            .map_err(|_| StoreError::Remote("D1 returned an invalid response".to_string()))?;
        if !status.is_success() || !envelope.success {
            let message = envelope
                .errors
                .into_iter()
                .map(|error| error.message)
                .find(|message| !message.is_empty())
                .unwrap_or_else(|| format!("D1 query failed with HTTP {status}"));
            return Err(d1_error(message));
        }
        if envelope.result.is_empty() {
            return Err(StoreError::Remote(
                "D1 response did not include a query result".to_string(),
            ));
        }
        envelope
            .result
            .into_iter()
            .map(|result| {
                if !result.success {
                    return Err(d1_error(
                        result
                            .error
                            .unwrap_or_else(|| "D1 query failed".to_string()),
                    ));
                }
                Ok(D1Execution {
                    rows: result.results,
                    changes: result.meta.changes,
                })
            })
            .collect()
    }

    fn insert_statement(&self, table: &str, value: Value) -> StoreResult<D1BatchStatement> {
        validate_table_name(table)?;
        let Value::Object(mut record) = value else {
            return Err(StoreError::InvalidQuery(
                "D1 insert value must be an object".to_string(),
            ));
        };
        if !record.contains_key("id") && generates_record_id(table) {
            record.insert("id".to_string(), Value::String(dowe_id::generate_ulid()));
        }
        let mut entries = record.into_iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        for (field, _) in &entries {
            validate_field_name(field)?;
        }
        let fields = entries
            .iter()
            .map(|(field, _)| identifier(field))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = (1..=entries.len())
            .map(|index| format!("?{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let params = entries
            .into_iter()
            .map(|(field, value)| self.encode_value(table, &field, value))
            .collect::<StoreResult<Vec<_>>>()?;
        Ok(D1BatchStatement {
            sql: format!(
                "INSERT INTO {} ({fields}) VALUES ({placeholders}) RETURNING *",
                identifier(table)
            ),
            params,
        })
    }

    fn filters_sql(
        &self,
        table: &str,
        filters: &[(String, Value)],
        first_parameter: usize,
    ) -> StoreResult<(String, Vec<Value>)> {
        if filters.is_empty() {
            return Err(StoreError::InvalidQuery(
                "D1 mutation filter cannot be empty".to_string(),
            ));
        }
        let mut clauses = Vec::with_capacity(filters.len());
        let mut params = Vec::with_capacity(filters.len());
        for (offset, (field, value)) in filters.iter().enumerate() {
            validate_field_name(field)?;
            clauses.push(format!(
                "{} = ?{}",
                identifier(field),
                first_parameter + offset
            ));
            params.push(self.encode_value(table, field, value.clone())?);
        }
        Ok((clauses.join(" AND "), params))
    }

    fn encode_value(&self, table: &str, field: &str, value: Value) -> StoreResult<Value> {
        if self.field_kind(Some(table), field) == Some(D1FieldKind::Json) {
            return serde_json::to_string(&value)
                .map(Value::String)
                .map_err(Into::into);
        }
        d1_value(value)
    }

    fn decode_rows(&self, table: Option<&str>, rows: Vec<Value>) -> StoreResult<Vec<Value>> {
        rows.into_iter()
            .map(|row| {
                let Value::Object(record) = row else {
                    return Ok(row);
                };
                record
                    .into_iter()
                    .map(|(field, value)| {
                        let value = match self.field_kind(table, &field) {
                            Some(D1FieldKind::Bool) => decode_bool(value),
                            Some(D1FieldKind::Json) => decode_json(value)?,
                            None => value,
                        };
                        Ok((field, value))
                    })
                    .collect::<StoreResult<Map<_, _>>>()
                    .map(Value::Object)
            })
            .collect()
    }

    fn decode_select_rows(&self, query: &SelectQuery, rows: Vec<Value>) -> StoreResult<Vec<Value>> {
        if query.projections.len() == 1
            && matches!(query.projections[0].value, QueryProjectionValue::Wildcard)
        {
            return self.decode_rows(Some(&query.source.table), rows);
        }
        rows.into_iter()
            .map(|row| {
                let Value::Object(mut record) = row else {
                    return Ok(row);
                };
                for projection in &query.projections {
                    let QueryProjectionValue::Identifier(identifier) = &projection.value else {
                        continue;
                    };
                    let Some(field) = identifier.parts.last() else {
                        continue;
                    };
                    let Some(output) = projection.output_name() else {
                        continue;
                    };
                    let table = identifier
                        .parts
                        .first()
                        .filter(|_| identifier.parts.len() > 1)
                        .map(String::as_str)
                        .unwrap_or(&query.source.table);
                    let Some(value) = record.remove(output) else {
                        continue;
                    };
                    let value = match self.field_kind(Some(table), field) {
                        Some(D1FieldKind::Bool) => decode_bool(value),
                        Some(D1FieldKind::Json) => decode_json(value)?,
                        None => value,
                    };
                    record.insert(output.to_string(), value);
                }
                Ok(Value::Object(record))
            })
            .collect()
    }

    fn field_kind(&self, table: Option<&str>, field: &str) -> Option<D1FieldKind> {
        let mut kinds = self
            .config
            .schema
            .iter()
            .filter(|schema| table.is_none_or(|table| schema.table == table))
            .filter_map(|schema| {
                if schema
                    .bool_fields
                    .iter()
                    .any(|candidate| candidate == field)
                {
                    Some(D1FieldKind::Bool)
                } else if schema
                    .json_fields
                    .iter()
                    .any(|candidate| candidate == field)
                {
                    Some(D1FieldKind::Json)
                } else {
                    None
                }
            });
        let first = kinds.next()?;
        kinds.all(|kind| kind == first).then_some(first)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum D1FieldKind {
    Bool,
    Json,
}

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

#[cfg(test)]
mod tests {
    use super::{D1Client, D1TableSchema, d1_error};
    use crate::{DatabaseTransactionInsert, StoreError};
    use axum::Router;
    use axum::extract::State;
    use axum::http::HeaderMap;
    use axum::routing::post;
    use serde_json::{Value, json};
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn sends_prepared_d1_queries_with_compound_filters() {
        let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
        let state = requests.clone();
        let router = Router::new()
            .route(
                "/query",
                post(
                    |State(requests): State<Arc<Mutex<Vec<Value>>>>,
                     headers: HeaderMap,
                     axum::Json(body): axum::Json<Value>| async move {
                        assert_eq!(
                            headers
                                .get("authorization")
                                .and_then(|value| value.to_str().ok()),
                            Some("Bearer secret")
                        );
                        requests.lock().expect("requests").push(body);
                        axum::Json(json!({
                            "success": true,
                            "result": [{
                                "success": true,
                                "results": [],
                                "meta": { "changes": 1 }
                            }],
                            "errors": []
                        }))
                    },
                ),
            )
            .with_state(state);
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
        let addr = listener.local_addr().expect("address");
        let server = tokio::spawn(async move { axum::serve(listener, router).await });
        let client = D1Client::for_endpoint(format!("http://{addr}/query"), "secret".to_string());

        let updated = client
            .update(
                "blogs",
                &[
                    ("id".to_string(), json!("blog-1")),
                    ("ownerId".to_string(), json!("user-1")),
                ],
                json!({ "title": "Updated" }),
                true,
            )
            .await
            .expect("update");

        assert_eq!(updated, json!({ "changed": 1 }));
        let requests = requests.lock().expect("requests");
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0]["sql"],
            "UPDATE \"blogs\" SET \"title\" = ?1 WHERE \"id\" = ?2 AND \"ownerId\" = ?3"
        );
        assert_eq!(
            requests[0]["params"],
            json!(["Updated", "blog-1", "user-1"])
        );
        drop(requests);
        drop(client);
        server.abort();
        let _ = server.await;
    }

    #[tokio::test]
    async fn sends_bound_parameters_for_d1_query() {
        let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
        let state = requests.clone();
        let router = Router::new()
            .route(
                "/query",
                post(
                    |State(requests): State<Arc<Mutex<Vec<Value>>>>,
                     axum::Json(body): axum::Json<Value>| async move {
                        requests.lock().expect("requests").push(body);
                        axum::Json(json!({
                            "success": true,
                            "result": [{
                                "success": true,
                                "results": [{ "name": "alt-arrow-down" }],
                                "meta": { "changes": 0 }
                            }],
                            "errors": []
                        }))
                    },
                ),
            )
            .with_state(state);
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
        let addr = listener.local_addr().expect("address");
        let server = tokio::spawn(async move { axum::serve(listener, router).await });
        let client = D1Client::for_endpoint(format!("http://{addr}/query"), "secret".to_string());

        let rows = client
            .query_with_params(
                "SELECT name FROM icons WHERE category = ?1 AND style = ?2",
                &[json!("arrows"), json!("linear")],
            )
            .await
            .expect("rows");

        assert_eq!(rows[0]["name"], "alt-arrow-down");
        let requests = requests.lock().expect("requests");
        assert_eq!(requests[0]["params"], json!(["arrows", "linear"]));
        drop(requests);
        drop(client);
        server.abort();
        let _ = server.await;
    }

    #[tokio::test]
    async fn sends_atomic_batches_and_decodes_entity_values() {
        let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
        let state = requests.clone();
        let router = Router::new()
            .route(
                "/query",
                post(
                    |State(requests): State<Arc<Mutex<Vec<Value>>>>,
                     axum::Json(body): axum::Json<Value>| async move {
                        requests.lock().expect("requests").push(body);
                        axum::Json(json!({
                            "success": true,
                            "result": [
                                {
                                    "success": true,
                                    "results": [{
                                        "id": "blog-1",
                                        "published": 1,
                                        "metadata": "{\"tags\":[\"dowe\"]}"
                                    }],
                                    "meta": { "changes": 1 }
                                },
                                {
                                    "success": true,
                                    "results": [{ "fingerprint": "seed-1" }],
                                    "meta": { "changes": 1 }
                                }
                            ],
                            "errors": []
                        }))
                    },
                ),
            )
            .with_state(state);
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
        let addr = listener.local_addr().expect("address");
        let server = tokio::spawn(async move { axum::serve(listener, router).await });
        let mut client =
            D1Client::for_endpoint(format!("http://{addr}/query"), "secret".to_string());
        client.config.schema.push(D1TableSchema {
            table: "blogs".to_string(),
            bool_fields: vec!["published".to_string()],
            json_fields: vec!["metadata".to_string()],
        });
        let result = client
            .transaction(&[
                DatabaseTransactionInsert {
                    table: "blogs".to_string(),
                    value: json!({
                        "id": "blog-1",
                        "published": true,
                        "metadata": { "tags": ["dowe"] }
                    }),
                },
                DatabaseTransactionInsert {
                    table: "_dowe_seeders".to_string(),
                    value: json!({ "fingerprint": "seed-1" }),
                },
            ])
            .await
            .expect("transaction");

        assert_eq!(result[0]["published"], json!(true));
        assert_eq!(result[0]["metadata"], json!({ "tags": ["dowe"] }));
        let requests = requests.lock().expect("requests");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["batch"].as_array().map(Vec::len), Some(2));
        let first_params = requests[0]["batch"][0]["params"]
            .as_array()
            .expect("params");
        assert!(first_params.contains(&json!(1)));
        assert!(first_params.contains(&json!("{\"tags\":[\"dowe\"]}")));
        drop(requests);
        drop(client);
        server.abort();
        let _ = server.await;
    }

    #[test]
    fn classifies_d1_constraint_and_query_errors() {
        assert!(matches!(
            d1_error("UNIQUE constraint failed: users.email".to_string()),
            StoreError::AlreadyExists(_)
        ));
        assert!(matches!(
            d1_error("no such table: blogs".to_string()),
            StoreError::InvalidQuery(_)
        ));
    }
}
