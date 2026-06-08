use crate::auth::verify_user;
use crate::engine::{DatabaseInspection, StoreRecord, open_database};
use crate::error::{StoreError, StoreResult};
use crate::names::validate_field_name;
use crate::value::{StoreValue, record_to_json};
use axum::Json;
use axum::Router;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteStoreConfig {
    pub host: String,
    pub database: String,
    pub user: String,
    pub credential: String,
}

#[derive(Clone)]
pub struct RemoteStoreClient {
    config: RemoteStoreConfig,
    client: reqwest::Client,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreServerConfig {
    pub root: PathBuf,
    pub host: String,
    pub port: u16,
}

pub struct RunningStoreServer {
    pub addr: std::net::SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<StoreResult<()>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteStoreRequest {
    pub operation: String,
    #[serde(default)]
    pub table: Option<String>,
    #[serde(default)]
    pub field: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub patch: Option<Value>,
    #[serde(default)]
    pub required: Option<bool>,
    #[serde(default)]
    pub sql: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoteStoreResponse {
    ok: bool,
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    error: Option<RemoteStoreError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoteStoreError {
    category: String,
    message: String,
}

#[derive(Clone)]
struct StoreServerState {
    root: PathBuf,
}

impl Default for StoreServerConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            host: "127.0.0.1".to_string(),
            port: 4147,
        }
    }
}

impl RemoteStoreClient {
    pub fn new(config: RemoteStoreConfig) -> StoreResult<Self> {
        if config.host.trim().is_empty() {
            return Err(StoreError::Remote("remote Store host is empty".to_string()));
        }
        if config.user.trim().is_empty() {
            return Err(StoreError::Authentication(
                "remote Store user is empty".to_string(),
            ));
        }
        if config.credential.is_empty() {
            return Err(StoreError::Authentication(
                "remote Store credential is empty".to_string(),
            ));
        }
        Ok(Self {
            config,
            client: reqwest::Client::new(),
        })
    }

    pub async fn list(&self, table: &str) -> StoreResult<Value> {
        self.send(RemoteStoreRequest {
            operation: "list".to_string(),
            table: Some(table.to_string()),
            field: None,
            value: None,
            patch: None,
            required: None,
            sql: None,
        })
        .await
    }

    pub async fn read(
        &self,
        table: &str,
        field: &str,
        value: Value,
        required: bool,
    ) -> StoreResult<Value> {
        self.send(RemoteStoreRequest {
            operation: "read".to_string(),
            table: Some(table.to_string()),
            field: Some(field.to_string()),
            value: Some(value),
            patch: None,
            required: Some(required),
            sql: None,
        })
        .await
    }

    pub async fn insert(&self, table: &str, value: Value) -> StoreResult<Value> {
        self.send(RemoteStoreRequest {
            operation: "insert".to_string(),
            table: Some(table.to_string()),
            field: None,
            value: Some(value),
            patch: None,
            required: None,
            sql: None,
        })
        .await
    }

    pub async fn update(
        &self,
        table: &str,
        field: &str,
        value: Value,
        patch: Value,
        required: bool,
    ) -> StoreResult<Value> {
        self.send(RemoteStoreRequest {
            operation: "update".to_string(),
            table: Some(table.to_string()),
            field: Some(field.to_string()),
            value: Some(value),
            patch: Some(patch),
            required: Some(required),
            sql: None,
        })
        .await
    }

    pub async fn delete(
        &self,
        table: &str,
        field: &str,
        value: Value,
        required: bool,
    ) -> StoreResult<Value> {
        self.send(RemoteStoreRequest {
            operation: "delete".to_string(),
            table: Some(table.to_string()),
            field: Some(field.to_string()),
            value: Some(value),
            patch: None,
            required: Some(required),
            sql: None,
        })
        .await
    }

    pub async fn query(&self, sql: &str) -> StoreResult<Value> {
        self.send(RemoteStoreRequest {
            operation: "query".to_string(),
            table: None,
            field: None,
            value: None,
            patch: None,
            required: None,
            sql: Some(sql.to_string()),
        })
        .await
    }

    pub async fn inspect(&self) -> StoreResult<Value> {
        self.send(RemoteStoreRequest {
            operation: "inspect".to_string(),
            table: None,
            field: None,
            value: None,
            patch: None,
            required: None,
            sql: None,
        })
        .await
    }

    async fn send(&self, request: RemoteStoreRequest) -> StoreResult<Value> {
        let url = format!(
            "{}/v1/databases/{}/operation",
            self.config.host.trim_end_matches('/'),
            self.config.database
        );
        let response = self
            .client
            .post(url)
            .timeout(Duration::from_secs(30))
            .header("X-Dowe-Store-User", &self.config.user)
            .bearer_auth(&self.config.credential)
            .json(&request)
            .send()
            .await
            .map_err(|error| StoreError::Remote(error.to_string()))?;
        let status = response.status();
        let body = response
            .json::<RemoteStoreResponse>()
            .await
            .map_err(|error| StoreError::Remote(error.to_string()))?;
        if body.ok {
            return Ok(body.data.unwrap_or(Value::Null));
        }
        let Some(error) = body.error else {
            return Err(StoreError::Remote(format!(
                "remote Store returned HTTP {status} without error"
            )));
        };
        Err(remote_error(&error.category, error.message))
    }
}

pub async fn start_store_server(config: StoreServerConfig) -> StoreResult<RunningStoreServer> {
    let listener = TcpListener::bind((config.host.as_str(), config.port))
        .await
        .map_err(|error| StoreError::Remote(error.to_string()))?;
    let addr = listener
        .local_addr()
        .map_err(|error| StoreError::Remote(error.to_string()))?;
    let router = Router::new()
        .route(
            "/v1/databases/{database}/operation",
            post(operation_handler),
        )
        .with_state(StoreServerState { root: config.root });
    let (shutdown, signal) = oneshot::channel();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = signal.await;
            })
            .await
            .map_err(|error| StoreError::Remote(error.to_string()))
    });
    Ok(RunningStoreServer {
        addr,
        shutdown: Some(shutdown),
        handle,
    })
}

pub async fn serve_store_server(config: StoreServerConfig) -> StoreResult<()> {
    let server = start_store_server(config).await?;
    server.wait().await
}

impl RunningStoreServer {
    pub async fn shutdown(mut self) -> StoreResult<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.handle
            .await
            .map_err(|error| StoreError::Remote(error.to_string()))?
    }

    pub async fn wait(mut self) -> StoreResult<()> {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(|error| StoreError::Remote(error.to_string()))?;
                self.shutdown().await
            }
            result = &mut self.handle => result.map_err(|error| StoreError::Remote(error.to_string()))?,
        }
    }
}

async fn operation_handler(
    State(state): State<StoreServerState>,
    AxumPath(database): AxumPath<String>,
    headers: HeaderMap,
    Json(request): Json<RemoteStoreRequest>,
) -> Response {
    match authorize(&state.root, &database, &headers)
        .and_then(|_| execute(&state.root, &database, request))
    {
        Ok(value) => json_status(StatusCode::OK, success(value)),
        Err(error) => store_error_response(error),
    }
}

fn authorize(root: &Path, database: &str, headers: &HeaderMap) -> StoreResult<()> {
    let user = headers
        .get("X-Dowe-Store-User")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| StoreError::Authentication("Store user is required".to_string()))?;
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(parse_bearer)
        .ok_or_else(|| StoreError::Authentication("Store bearer token is required".to_string()))?;
    verify_user(root, database, user, &token)
}

fn execute(root: &Path, database: &str, request: RemoteStoreRequest) -> StoreResult<Value> {
    let db = open_database(root, database)?;
    match request.operation.as_str() {
        "list" => {
            let table = required_text(request.table, "table")?;
            let records = db.records(&table)?;
            Ok(Value::Array(records.iter().map(record_to_json).collect()))
        }
        "read" => {
            let table = required_text(request.table, "table")?;
            let field = required_text(request.field, "field")?;
            validate_field_name(&field)?;
            let expected = StoreValue::from_json(required_value(request.value, "value")?);
            let record = db
                .records(&table)?
                .into_iter()
                .find(|record| record_matches(record, &field, &expected));
            if record.is_none() && request.required.unwrap_or(false) {
                return Err(StoreError::NotFound("record not found".to_string()));
            }
            Ok(record.as_ref().map(record_to_json).unwrap_or(Value::Null))
        }
        "insert" => {
            let table = required_text(request.table, "table")?;
            let record = json_record(required_value(request.value, "value")?)?;
            Ok(record_to_json(&db.insert(&table, record)?))
        }
        "update" => {
            let table = required_text(request.table, "table")?;
            let field = required_text(request.field, "field")?;
            validate_field_name(&field)?;
            let expected = StoreValue::from_json(required_value(request.value, "value")?);
            let patch = json_record(required_value(request.patch, "patch")?)?;
            let changed = db.update(&table, &field, &expected, patch)?;
            if changed == 0 && request.required.unwrap_or(false) {
                return Err(StoreError::NotFound("record not found".to_string()));
            }
            Ok(changed_json(changed))
        }
        "delete" => {
            let table = required_text(request.table, "table")?;
            let field = required_text(request.field, "field")?;
            validate_field_name(&field)?;
            let expected = StoreValue::from_json(required_value(request.value, "value")?);
            let changed = db.delete(&table, &field, &expected)?;
            if changed == 0 && request.required.unwrap_or(false) {
                return Err(StoreError::NotFound("record not found".to_string()));
            }
            Ok(changed_json(changed))
        }
        "query" => db.query_json(&required_text(request.sql, "sql")?),
        "inspect" => inspect_json(&db.inspect()?),
        value => Err(StoreError::InvalidQuery(format!(
            "unsupported remote Store operation `{value}`"
        ))),
    }
}

fn parse_bearer(value: &str) -> Option<String> {
    let mut parts = value.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;
    if parts.next().is_some() || token.is_empty() || !scheme.eq_ignore_ascii_case("Bearer") {
        return None;
    }
    Some(token.to_string())
}

fn required_text(value: Option<String>, name: &str) -> StoreResult<String> {
    value
        .filter(|value| !value.is_empty())
        .ok_or_else(|| StoreError::InvalidQuery(format!("remote Store request missing `{name}`")))
}

fn required_value(value: Option<Value>, name: &str) -> StoreResult<Value> {
    value.ok_or_else(|| StoreError::InvalidQuery(format!("remote Store request missing `{name}`")))
}

fn json_record(value: Value) -> StoreResult<StoreRecord> {
    let Some(object) = value.as_object() else {
        return Err(StoreError::InvalidQuery(
            "remote Store record must be a JSON object".to_string(),
        ));
    };
    let mut record = BTreeMap::new();
    for (key, value) in object {
        validate_field_name(key)?;
        record.insert(key.clone(), StoreValue::from_json(value.clone()));
    }
    Ok(record)
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

fn inspect_json(inspection: &DatabaseInspection) -> StoreResult<Value> {
    let mut root = Map::new();
    root.insert(
        "databaseId".to_string(),
        Value::String(inspection.database_id.clone()),
    );
    root.insert("name".to_string(), Value::String(inspection.name.clone()));
    root.insert(
        "formatVersion".to_string(),
        Value::Number(inspection.format_version.into()),
    );
    root.insert(
        "tables".to_string(),
        Value::Array(
            inspection
                .tables
                .iter()
                .map(|table| {
                    let mut value = Map::new();
                    value.insert("name".to_string(), Value::String(table.name.clone()));
                    value.insert("records".to_string(), Value::Number(table.records.into()));
                    value.insert(
                        "indexes".to_string(),
                        Value::Array(
                            table
                                .indexes
                                .iter()
                                .map(|index| Value::String(index.clone()))
                                .collect(),
                        ),
                    );
                    Value::Object(value)
                })
                .collect(),
        ),
    );
    Ok(Value::Object(root))
}

fn success(data: Value) -> RemoteStoreResponse {
    RemoteStoreResponse {
        ok: true,
        data: Some(data),
        error: None,
    }
}

fn failure(error: &StoreError) -> RemoteStoreResponse {
    RemoteStoreResponse {
        ok: false,
        data: None,
        error: Some(RemoteStoreError {
            category: error.category().to_string(),
            message: error.message().to_string(),
        }),
    }
}

fn json_status(status: StatusCode, value: RemoteStoreResponse) -> Response {
    (status, Json(value)).into_response()
}

fn store_error_response(error: StoreError) -> Response {
    let status = match error {
        StoreError::Authentication(_) => StatusCode::UNAUTHORIZED,
        StoreError::Authorization(_) => StatusCode::FORBIDDEN,
        StoreError::InvalidName(_) | StoreError::InvalidQuery(_) => StatusCode::BAD_REQUEST,
        StoreError::AlreadyExists(_) | StoreError::TransactionConflict(_) => StatusCode::CONFLICT,
        StoreError::NotFound(_) => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    json_status(status, failure(&error))
}

fn remote_error(category: &str, message: String) -> StoreError {
    match category {
        "NotFound" => StoreError::NotFound(message),
        "AlreadyExists" => StoreError::AlreadyExists(message),
        "InvalidName" => StoreError::InvalidName(message),
        "InvalidUlid" => StoreError::InvalidUlid(message),
        "InvalidQuery" => StoreError::InvalidQuery(message),
        "Authentication" => StoreError::Authentication(message),
        "Authorization" => StoreError::Authorization(message),
        "TypeError" => StoreError::TypeError(message),
        "TransactionConflict" => StoreError::TransactionConflict(message),
        "DurabilityError" => StoreError::DurabilityError(message),
        "Corruption" => StoreError::Corruption(message),
        "UnsupportedFormat" => StoreError::UnsupportedFormat(message),
        "Io" => StoreError::Io(message),
        "Remote" => StoreError::Remote(message),
        value => StoreError::Remote(format!("unknown remote Store error `{value}`: {message}")),
    }
}
