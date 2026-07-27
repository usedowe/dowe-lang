use crate::auth::verify_account;
use crate::engine::{StoreRecord, open_database};
use crate::error::{StoreError, StoreResult};
use crate::names::validate_field_name;
use crate::query::bind_query_params;
use crate::value::{StoreValue, record_to_json};
use axum::Router;
use axum::extract::ws::{Message as AxumMessage, WebSocket};
use axum::extract::{Path as AxumPath, State, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::Message as TungsteniteMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

type ClientSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoweDatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub account: String,
    pub secret: String,
}

#[derive(Clone)]
pub struct DoweDatabaseClient {
    config: DoweDatabaseConfig,
    socket: Arc<Mutex<Option<ClientSocket>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseServiceConfig {
    pub root: PathBuf,
    pub host: String,
    pub port: u16,
}

pub struct RunningDatabaseService {
    pub addr: std::net::SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<StoreResult<()>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseRequest {
    pub id: u64,
    pub operation: String,
    #[serde(default)]
    pub table: Option<String>,
    #[serde(default)]
    pub filters: Vec<(String, Value)>,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub patch: Option<Value>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub sql: Option<String>,
    #[serde(default)]
    pub params: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseResponse {
    id: u64,
    ok: bool,
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    error: Option<DatabaseRemoteError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseRemoteError {
    category: String,
    message: String,
}

#[derive(Clone)]
struct DatabaseServiceState {
    root: PathBuf,
}

impl Default for DatabaseServiceConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            host: "127.0.0.1".to_string(),
            port: 4147,
        }
    }
}

impl DoweDatabaseClient {
    pub fn new(config: DoweDatabaseConfig) -> StoreResult<Self> {
        if config.host.trim().is_empty() {
            return Err(StoreError::Remote(
                "Dowe Database host is empty".to_string(),
            ));
        }
        if config.port == 0 {
            return Err(StoreError::Remote(
                "Dowe Database port must be greater than zero".to_string(),
            ));
        }
        if config.account.trim().is_empty() {
            return Err(StoreError::Authentication(
                "Dowe Database account is empty".to_string(),
            ));
        }
        if config.secret.is_empty() {
            return Err(StoreError::Authentication(
                "Dowe Database secret is empty".to_string(),
            ));
        }
        Ok(Self {
            config,
            socket: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn list(&self, table: &str) -> StoreResult<Value> {
        self.send(request("list", Some(table))).await
    }

    pub async fn read(
        &self,
        table: &str,
        filters: Vec<(String, Value)>,
        required: bool,
    ) -> StoreResult<Value> {
        let mut request = request("read", Some(table));
        request.filters = filters;
        request.required = required;
        self.send(request).await
    }

    pub async fn insert(&self, table: &str, value: Value) -> StoreResult<Value> {
        let mut request = request("insert", Some(table));
        request.value = Some(value);
        self.send(request).await
    }

    pub async fn update(
        &self,
        table: &str,
        filters: Vec<(String, Value)>,
        patch: Value,
        required: bool,
    ) -> StoreResult<Value> {
        let mut request = request("update", Some(table));
        request.filters = filters;
        request.patch = Some(patch);
        request.required = required;
        self.send(request).await
    }

    pub async fn delete(
        &self,
        table: &str,
        filters: Vec<(String, Value)>,
        required: bool,
    ) -> StoreResult<Value> {
        let mut request = request("delete", Some(table));
        request.filters = filters;
        request.required = required;
        self.send(request).await
    }

    pub async fn query(&self, sql: &str) -> StoreResult<Value> {
        self.query_with_params(sql, &[]).await
    }

    pub async fn query_with_params(&self, sql: &str, params: &[Value]) -> StoreResult<Value> {
        let mut request = request("query", None);
        request.sql = Some(sql.to_string());
        request.params = params.to_vec();
        self.send(request).await
    }

    pub async fn inspect(&self) -> StoreResult<Value> {
        self.send(request("inspect", None)).await
    }

    async fn send(&self, mut request: DatabaseRequest) -> StoreResult<Value> {
        request.id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let mut socket = self.socket.lock().await;
        for attempt in 0..2 {
            if socket.is_none() {
                *socket = Some(connect(&self.config).await?);
            }
            let result = tokio::time::timeout(
                Duration::from_secs(30),
                exchange(socket.as_mut().expect("connected"), &request),
            )
            .await
            .map_err(|_| StoreError::Remote("Dowe Database request timed out".to_string()))?;
            match result {
                Ok(value) => return Ok(value),
                Err(error) if attempt == 0 && is_transport_error(&error) => {
                    *socket = None;
                }
                Err(error) => return Err(error),
            }
        }
        Err(StoreError::Remote(
            "Dowe Database WebSocket exchange failed".to_string(),
        ))
    }
}

impl RunningDatabaseService {
    pub async fn shutdown(mut self) -> StoreResult<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        match tokio::time::timeout(Duration::from_secs(2), &mut self.handle).await {
            Ok(result) => result.map_err(|error| StoreError::Remote(error.to_string()))?,
            Err(_) => {
                self.handle.abort();
                let _ = self.handle.await;
                Ok(())
            }
        }
    }

    pub async fn wait(mut self) -> StoreResult<()> {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(|error| StoreError::Remote(error.to_string()))?;
                self.shutdown().await
            }
            result = &mut self.handle => {
                result.map_err(|error| StoreError::Remote(error.to_string()))?
            }
        }
    }
}

pub async fn start_database_service(
    config: DatabaseServiceConfig,
) -> StoreResult<RunningDatabaseService> {
    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .map_err(|error| StoreError::Remote(error.to_string()))?;
    let addr = listener
        .local_addr()
        .map_err(|error| StoreError::Remote(error.to_string()))?;
    let state = DatabaseServiceState { root: config.root };
    let router = build_database_service_router(state);
    let (shutdown, signal) = oneshot::channel();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = signal.await;
            })
            .await
            .map_err(|error| StoreError::Remote(error.to_string()))
    });
    Ok(RunningDatabaseService {
        addr,
        shutdown: Some(shutdown),
        handle,
    })
}

pub async fn serve_database_service(config: DatabaseServiceConfig) -> StoreResult<()> {
    start_database_service(config).await?.wait().await
}

pub fn database_service_router(root: PathBuf) -> Router {
    build_database_service_router(DatabaseServiceState { root })
}

fn build_database_service_router(state: DatabaseServiceState) -> Router {
    Router::new()
        .route("/v1/databases/{database}", get(database_upgrade))
        .with_state(state)
}

async fn database_upgrade(
    State(state): State<DatabaseServiceState>,
    AxumPath(database): AxumPath<String>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let account = headers
        .get("X-Dowe-Database-Account")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let secret = bearer_secret(&headers).unwrap_or_default();
    match verify_account(&state.root, &database, account, secret) {
        Ok(()) => upgrade
            .on_upgrade(move |socket| handle_socket(socket, state.root, database))
            .into_response(),
        Err(StoreError::Authorization(_)) => StatusCode::FORBIDDEN.into_response(),
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn handle_socket(mut socket: WebSocket, root: PathBuf, database: String) {
    while let Some(message) = socket.recv().await {
        let Ok(message) = message else {
            return;
        };
        let AxumMessage::Text(text) = message else {
            if matches!(message, AxumMessage::Close(_)) {
                return;
            }
            continue;
        };
        let response = match serde_json::from_str::<DatabaseRequest>(&text) {
            Ok(request) => {
                let id = request.id;
                let root = root.clone();
                let database = database.clone();
                tokio::task::spawn_blocking(move || response_for_request(&root, &database, request))
                    .await
                    .unwrap_or_else(|error| DatabaseResponse {
                        id,
                        ok: false,
                        data: None,
                        error: Some(DatabaseRemoteError {
                            category: "Remote".to_string(),
                            message: error.to_string(),
                        }),
                    })
            }
            Err(error) => DatabaseResponse {
                id: 0,
                ok: false,
                data: None,
                error: Some(DatabaseRemoteError {
                    category: "InvalidQuery".to_string(),
                    message: error.to_string(),
                }),
            },
        };
        let Ok(text) = serde_json::to_string(&response) else {
            return;
        };
        if socket.send(AxumMessage::Text(text.into())).await.is_err() {
            return;
        }
    }
}

fn response_for_request(
    root: &std::path::Path,
    database_name: &str,
    request: DatabaseRequest,
) -> DatabaseResponse {
    let id = request.id;
    match execute_request(root, database_name, request) {
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

fn execute_request(
    root: &std::path::Path,
    database_name: &str,
    request: DatabaseRequest,
) -> StoreResult<Value> {
    let database = open_database(root, database_name)?;
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
            let sql = request
                .sql
                .as_deref()
                .ok_or_else(|| StoreError::InvalidQuery("query requires sql".to_string()))?;
            database.query_json(&bind_query_params(sql, &request.params)?)
        }
        "inspect" => serde_json::to_value(database.inspect()?)
            .map_err(|error| StoreError::Remote(error.to_string())),
        operation => Err(StoreError::InvalidQuery(format!(
            "unsupported remote Database operation `{operation}`"
        ))),
    }
}

fn request(operation: &str, table: Option<&str>) -> DatabaseRequest {
    DatabaseRequest {
        id: 0,
        operation: operation.to_string(),
        table: table.map(str::to_string),
        filters: Vec::new(),
        value: None,
        patch: None,
        required: false,
        sql: None,
        params: Vec::new(),
    }
}

async fn connect(config: &DoweDatabaseConfig) -> StoreResult<ClientSocket> {
    let url = websocket_url(config)?;
    let mut request = url
        .into_client_request()
        .map_err(|error| StoreError::Remote(error.to_string()))?;
    request.headers_mut().insert(
        "X-Dowe-Database-Account",
        HeaderValue::from_str(&config.account)
            .map_err(|error| StoreError::Authentication(error.to_string()))?,
    );
    request.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", config.secret))
            .map_err(|error| StoreError::Authentication(error.to_string()))?,
    );
    connect_async(request)
        .await
        .map(|(socket, _)| socket)
        .map_err(websocket_error)
}

fn websocket_error(error: tokio_tungstenite::tungstenite::Error) -> StoreError {
    match &error {
        tokio_tungstenite::tungstenite::Error::Http(response)
            if response.status() == StatusCode::UNAUTHORIZED =>
        {
            StoreError::Authentication("Dowe Database rejected the account or secret".to_string())
        }
        tokio_tungstenite::tungstenite::Error::Http(response)
            if response.status() == StatusCode::FORBIDDEN =>
        {
            StoreError::Authorization(
                "Dowe Database account cannot access this database".to_string(),
            )
        }
        _ => StoreError::Remote(error.to_string()),
    }
}

async fn exchange(socket: &mut ClientSocket, request: &DatabaseRequest) -> StoreResult<Value> {
    let payload =
        serde_json::to_string(request).map_err(|error| StoreError::Remote(error.to_string()))?;
    socket
        .send(TungsteniteMessage::Text(payload.into()))
        .await
        .map_err(|error| StoreError::Remote(error.to_string()))?;
    while let Some(message) = socket.next().await {
        let message = message.map_err(|error| StoreError::Remote(error.to_string()))?;
        match message {
            TungsteniteMessage::Text(text) => {
                let response = serde_json::from_str::<DatabaseResponse>(&text)
                    .map_err(|error| StoreError::Remote(error.to_string()))?;
                if response.id != request.id {
                    continue;
                }
                if response.ok {
                    return Ok(response.data.unwrap_or(Value::Null));
                }
                let error = response.error.unwrap_or(DatabaseRemoteError {
                    category: "Remote".to_string(),
                    message: "Dowe Database returned an unknown error".to_string(),
                });
                return Err(remote_error(error));
            }
            TungsteniteMessage::Close(_) => {
                return Err(StoreError::Remote(
                    "Dowe Database closed the WebSocket".to_string(),
                ));
            }
            _ => {}
        }
    }
    Err(StoreError::Remote(
        "Dowe Database WebSocket ended".to_string(),
    ))
}

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
