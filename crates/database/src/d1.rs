use crate::error::{StoreError, StoreResult};
use crate::names::{validate_field_name, validate_table_name};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct D1Config {
    pub account: String,
    pub database: String,
    pub secret: String,
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
            },
            client: reqwest::Client::new(),
            endpoint_override: Some(endpoint),
        }
    }

    pub async fn list(&self, table: &str) -> StoreResult<Value> {
        validate_table_name(table)?;
        let sql = format!("SELECT * FROM {}", identifier(table));
        let execution = self.execute(&sql, &[]).await?;
        Ok(Value::Array(execution.rows))
    }

    pub async fn read(
        &self,
        table: &str,
        filters: &[(String, Value)],
        required: bool,
    ) -> StoreResult<Value> {
        validate_table_name(table)?;
        let (where_sql, params) = filters_sql(filters, 1)?;
        let sql = format!(
            "SELECT * FROM {} WHERE {where_sql} LIMIT 1",
            identifier(table)
        );
        let execution = self.execute(&sql, &params).await?;
        let value = execution.rows.into_iter().next().unwrap_or(Value::Null);
        if value.is_null() && required {
            return Err(StoreError::NotFound("D1 record was not found".to_string()));
        }
        Ok(value)
    }

    pub async fn insert(&self, table: &str, value: Value) -> StoreResult<Value> {
        validate_table_name(table)?;
        let Value::Object(mut record) = value else {
            return Err(StoreError::InvalidQuery(
                "D1 insert value must be an object".to_string(),
            ));
        };
        if !record.contains_key("id") {
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
            .map(|(_, value)| d1_value(value))
            .collect::<StoreResult<Vec<_>>>()?;
        let sql = format!(
            "INSERT INTO {} ({fields}) VALUES ({placeholders}) RETURNING *",
            identifier(table)
        );
        let execution = self.execute(&sql, &params).await?;
        execution.rows.into_iter().next().ok_or_else(|| {
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
            params.push(d1_value(value)?);
        }
        let (where_sql, filter_params) = filters_sql(filters, params.len() + 1)?;
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
        let (where_sql, params) = filters_sql(filters, 1)?;
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
        Ok(Value::Array(execution.rows))
    }

    pub async fn execute_batch(&self, sql: &str) -> StoreResult<()> {
        for statement in sql.split(';').map(str::trim).filter(|sql| !sql.is_empty()) {
            self.execute(statement, &[]).await?;
        }
        Ok(())
    }

    async fn execute(&self, sql: &str, params: &[Value]) -> StoreResult<D1Execution> {
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
            .json(&D1QueryRequest { sql, params })
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
        let result = envelope.result.into_iter().next().ok_or_else(|| {
            StoreError::Remote("D1 response did not include a query result".to_string())
        })?;
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
    }
}

fn filters_sql(
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
        params.push(d1_value(value.clone())?);
    }
    Ok((clauses.join(" AND "), params))
}

fn identifier(value: &str) -> String {
    value
        .split('.')
        .map(|part| format!("\"{part}\""))
        .collect::<Vec<_>>()
        .join(".")
}

fn d1_value(value: Value) -> StoreResult<Value> {
    match value {
        Value::Bool(value) => Ok(Value::from(if value { 1 } else { 0 })),
        Value::Null | Value::Number(_) | Value::String(_) => Ok(value),
        Value::Array(_) | Value::Object(_) => Err(StoreError::InvalidQuery(
            "D1 parameters must be null, numbers, booleans, or strings".to_string(),
        )),
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
    use super::{D1Client, d1_error};
    use crate::StoreError;
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
