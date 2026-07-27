use crate::auth::verify_account;
use crate::engine::open_database;
use crate::error::{KvError, KvResult};
use crate::protocol::{CacheRequest, CacheResponse};
use axum::Router;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path as AxumPath, State, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheServerConfig {
    pub root: PathBuf,
    pub host: String,
    pub port: u16,
}

pub struct RunningCacheServer {
    pub addr: std::net::SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<KvResult<()>>,
}

#[derive(Clone)]
struct CacheServerState {
    root: PathBuf,
}

impl Default for CacheServerConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            host: "127.0.0.1".to_string(),
            port: 4148,
        }
    }
}

pub async fn start_cache_server(config: CacheServerConfig) -> KvResult<RunningCacheServer> {
    let listener = TcpListener::bind((config.host.as_str(), config.port))
        .await
        .map_err(|_| KvError::Remote("Dowe Cache could not bind its listener".to_string()))?;
    let addr = listener
        .local_addr()
        .map_err(|_| KvError::Remote("Dowe Cache listener is unavailable".to_string()))?;
    let router = Router::new()
        .route("/v1/caches/{name}", get(service_handler))
        .with_state(CacheServerState { root: config.root });
    let (shutdown, signal) = oneshot::channel();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = signal.await;
            })
            .await
            .map_err(|_| KvError::Remote("Dowe Cache server failed".to_string()))
    });
    Ok(RunningCacheServer {
        addr,
        shutdown: Some(shutdown),
        handle,
    })
}

pub async fn serve_cache_server(config: CacheServerConfig) -> KvResult<()> {
    start_cache_server(config).await?.wait().await
}

pub async fn cache_service_upgrade(
    root: PathBuf,
    name: String,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    match authorize(&root, &name, &headers) {
        Ok(()) => upgrade
            .on_upgrade(move |socket| serve_socket(socket, root, name))
            .into_response(),
        Err(error) => error_response(error),
    }
}

impl RunningCacheServer {
    pub async fn shutdown(mut self) -> KvResult<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.handle
            .await
            .map_err(|_| KvError::Remote("Dowe Cache server task failed".to_string()))?
    }

    pub async fn wait(mut self) -> KvResult<()> {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(|_| KvError::Remote("Dowe Cache signal failed".to_string()))?;
                self.shutdown().await
            }
            result = &mut self.handle => {
                result.map_err(|_| KvError::Remote("Dowe Cache server task failed".to_string()))?
            },
        }
    }
}

async fn service_handler(
    State(state): State<CacheServerState>,
    AxumPath(name): AxumPath<String>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    cache_service_upgrade(state.root, name, headers, upgrade).await
}

async fn serve_socket(mut socket: WebSocket, root: PathBuf, name: String) {
    while let Some(message) = socket.recv().await {
        match message {
            Ok(Message::Text(text)) => {
                let response = handle_text(&root, &name, text.as_str());
                if socket
                    .send(Message::Text(response_text(&response).into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Ok(Message::Binary(bytes)) => {
                let response = handle_bytes(&root, &name, &bytes);
                if socket
                    .send(Message::Text(response_text(&response).into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Ok(Message::Ping(payload)) => {
                if socket.send(Message::Pong(payload)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(Message::Pong(_)) => {}
        }
    }
}

fn handle_text(root: &Path, name: &str, text: &str) -> CacheResponse {
    match serde_json::from_str::<CacheRequest>(text) {
        Ok(request) => execute(root, name, request),
        Err(_) => CacheResponse::failure(
            String::new(),
            &KvError::InvalidRequest("Dowe Cache request is invalid JSON".to_string()),
        ),
    }
}

fn handle_bytes(root: &Path, name: &str, bytes: &[u8]) -> CacheResponse {
    match serde_json::from_slice::<CacheRequest>(bytes) {
        Ok(request) => execute(root, name, request),
        Err(_) => CacheResponse::failure(
            String::new(),
            &KvError::InvalidRequest("Dowe Cache request is invalid JSON".to_string()),
        ),
    }
}

fn response_text(response: &CacheResponse) -> String {
    serde_json::to_string(response).unwrap_or_else(|_| {
        "{\"id\":\"\",\"ok\":false,\"error\":{\"category\":\"InvalidRequest\",\"message\":\"Dowe Cache response serialization failed\"}}".to_string()
    })
}

fn execute(root: &Path, name: &str, request: CacheRequest) -> CacheResponse {
    let id = request.id.clone();
    let result = execute_operation(root, name, request);
    match result {
        Ok(value) => CacheResponse::success(id, value),
        Err(error) => CacheResponse::failure(id, &error),
    }
}

fn execute_operation(root: &Path, name: &str, request: CacheRequest) -> KvResult<Value> {
    if request.id.is_empty() {
        return Err(KvError::InvalidRequest(
            "Dowe Cache request ID is required".to_string(),
        ));
    }
    let cache = open_database(root, name, true)?;
    match request.operation.as_str() {
        "get" => {
            let key = required_key(request.key)?;
            let value = cache.get(&key)?;
            if value.is_none() && request.required {
                return Err(KvError::NotFound("Cache key was not found".to_string()));
            }
            Ok(value.unwrap_or(Value::Null))
        }
        "set" => {
            let key = required_key(request.key)?;
            let value = request
                .value
                .ok_or_else(|| KvError::InvalidRequest("Cache set requires `value`".to_string()))?;
            cache.set(&key, value)?;
            Ok(set_json(&key))
        }
        "delete" => {
            let key = required_key(request.key)?;
            Ok(delete_json(cache.delete(&key)?))
        }
        "keys" => Ok(Value::Array(
            cache
                .keys(request.prefix.as_deref())?
                .into_iter()
                .map(Value::String)
                .collect(),
        )),
        "clear" => Ok(clear_json(cache.clear()?)),
        operation => Err(KvError::InvalidRequest(format!(
            "unsupported Cache operation `{operation}`"
        ))),
    }
}

fn authorize(root: &Path, name: &str, headers: &HeaderMap) -> KvResult<()> {
    let account = headers
        .get("X-Dowe-Cache-Account")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| KvError::Authentication("Cache account is required".to_string()))?;
    let secret = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(parse_bearer)
        .ok_or_else(|| KvError::Authentication("Cache bearer secret is required".to_string()))?;
    verify_account(root, name, account, secret)
}

fn parse_bearer(value: &str) -> Option<&str> {
    let mut parts = value.split_whitespace();
    let scheme = parts.next()?;
    let secret = parts.next()?;
    if parts.next().is_some() || secret.is_empty() || !scheme.eq_ignore_ascii_case("Bearer") {
        return None;
    }
    Some(secret)
}

fn required_key(key: Option<String>) -> KvResult<String> {
    key.filter(|value| !value.is_empty())
        .ok_or_else(|| KvError::InvalidRequest("Cache operation requires `key`".to_string()))
}

fn error_response(error: KvError) -> Response {
    let status = match error {
        KvError::Authentication(_) => StatusCode::UNAUTHORIZED,
        KvError::Authorization(_) => StatusCode::FORBIDDEN,
        KvError::InvalidName(_) | KvError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    status.into_response()
}

fn set_json(key: &str) -> Value {
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(true));
    output.insert("key".to_string(), Value::String(key.to_string()));
    Value::Object(output)
}

fn delete_json(deleted: bool) -> Value {
    let mut output = Map::new();
    output.insert("deleted".to_string(), Value::Bool(deleted));
    Value::Object(output)
}

fn clear_json(cleared: usize) -> Value {
    let mut output = Map::new();
    output.insert("cleared".to_string(), Value::from(cleared as u64));
    Value::Object(output)
}
