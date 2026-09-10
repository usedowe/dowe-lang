use crate::auth::verify_account;
use crate::engine::{Database, StoreRecord, open_database};
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
use dowe_database_query::SelectQuery;
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
    #[serde(default)]
    pub query: Option<SelectQuery>,
    #[serde(default)]
    pub operations: Vec<DatabaseTransactionInsert>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseTransactionInsert {
    pub table: String,
    pub value: Value,
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

