use crate::auth::verify_account;
use crate::engine::{object, open_database};
use crate::error::{VectorError, VectorResult};
use crate::protocol::{VectorRequest, VectorResponse};
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
pub struct VectorServerConfig {
    pub root: PathBuf,
    pub host: String,
    pub port: u16,
}

pub struct RunningVectorServer {
    pub addr: std::net::SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<VectorResult<()>>,
}

#[derive(Clone)]
struct VectorServerState {
    root: PathBuf,
}

impl Default for VectorServerConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            host: "127.0.0.1".to_string(),
            port: 4149,
        }
    }
}

pub async fn start_vector_server(config: VectorServerConfig) -> VectorResult<RunningVectorServer> {
    let listener = TcpListener::bind((config.host.as_str(), config.port))
        .await
        .map_err(|_| VectorError::Remote("Dowe Vector could not bind its listener".to_string()))?;
    let addr = listener
        .local_addr()
        .map_err(|_| VectorError::Remote("Dowe Vector listener is unavailable".to_string()))?;
    let router = Router::new()
        .route("/v1/vectors/{name}", get(service_handler))
        .with_state(VectorServerState { root: config.root });
    let (shutdown, signal) = oneshot::channel();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = signal.await;
            })
            .await
            .map_err(|_| VectorError::Remote("Dowe Vector server failed".to_string()))
    });
    Ok(RunningVectorServer {
        addr,
        shutdown: Some(shutdown),
        handle,
    })
}

pub async fn serve_vector_server(config: VectorServerConfig) -> VectorResult<()> {
    start_vector_server(config).await?.wait().await
}

pub async fn vector_service_upgrade(
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

impl RunningVectorServer {
    pub async fn shutdown(mut self) -> VectorResult<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.handle
            .await
            .map_err(|_| VectorError::Remote("Dowe Vector server task failed".to_string()))?
    }

    pub async fn wait(mut self) -> VectorResult<()> {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(|_| VectorError::Remote(
                    "Dowe Vector signal failed".to_string()
                ))?;
                self.shutdown().await
            }
            result = &mut self.handle => {
                result.map_err(|_| VectorError::Remote(
                    "Dowe Vector server task failed".to_string()
                ))?
            },
        }
    }
}

async fn service_handler(
    State(state): State<VectorServerState>,
    AxumPath(name): AxumPath<String>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    vector_service_upgrade(state.root, name, headers, upgrade).await
}

async fn serve_socket(mut socket: WebSocket, root: PathBuf, name: String) {
    while let Some(message) = socket.recv().await {
        let response = match message {
            Ok(Message::Text(text)) => handle_text(&root, &name, text.as_str()),
            Ok(Message::Binary(bytes)) => handle_bytes(&root, &name, &bytes),
            Ok(Message::Ping(payload)) => {
                if socket.send(Message::Pong(payload)).await.is_err() {
                    break;
                }
                continue;
            }
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(Message::Pong(_)) => continue,
        };
        if socket
            .send(Message::Text(response_text(&response).into()))
            .await
            .is_err()
        {
            break;
        }
    }
}

fn handle_text(root: &Path, name: &str, text: &str) -> VectorResponse {
    match serde_json::from_str::<VectorRequest>(text) {
        Ok(request) => execute(root, name, request),
        Err(_) => VectorResponse::failure(
            String::new(),
            &VectorError::InvalidRequest("Dowe Vector request is invalid JSON".to_string()),
        ),
    }
}

fn handle_bytes(root: &Path, name: &str, bytes: &[u8]) -> VectorResponse {
    match serde_json::from_slice::<VectorRequest>(bytes) {
        Ok(request) => execute(root, name, request),
        Err(_) => VectorResponse::failure(
            String::new(),
            &VectorError::InvalidRequest("Dowe Vector request is invalid JSON".to_string()),
        ),
    }
}

fn response_text(response: &VectorResponse) -> String {
    serde_json::to_string(response).unwrap_or_else(|_| {
        "{\"id\":\"\",\"ok\":false,\"error\":{\"category\":\"InvalidRequest\",\"message\":\"Dowe Vector response serialization failed\"}}".to_string()
    })
}

fn execute(root: &Path, name: &str, request: VectorRequest) -> VectorResponse {
    let id = request.id.clone();
    match execute_operation(root, name, request) {
        Ok(value) => VectorResponse::success(id, value),
        Err(error) => VectorResponse::failure(id, &error),
    }
}

fn execute_operation(root: &Path, name: &str, request: VectorRequest) -> VectorResult<Value> {
    if request.id.is_empty() {
        return Err(VectorError::InvalidRequest(
            "Dowe Vector request ID is required".to_string(),
        ));
    }
    let database = open_database(root, name, true)?;
    match request.operation.as_str() {
        "upsert" => {
            let id = required_id(request.embedding_id)?;
            let vector = request.vector.ok_or_else(|| {
                VectorError::InvalidRequest("Vector upsert requires `vector`".to_string())
            })?;
            let metadata = request.metadata.unwrap_or_else(object);
            Ok(serde_json::to_value(
                database.upsert(&id, vector, metadata)?,
            )?)
        }
        "search" => {
            let vector = request.vector.ok_or_else(|| {
                VectorError::InvalidRequest("Vector search requires `vector`".to_string())
            })?;
            Ok(serde_json::to_value(database.search(
                &vector,
                request.limit.unwrap_or(10),
                request.min_score.unwrap_or(-1.0),
                request.filter.as_ref(),
            )?)?)
        }
        "read" => {
            let id = required_id(request.embedding_id)?;
            let value = database.read(&id)?;
            if value.is_none() && request.required {
                return Err(VectorError::NotFound(
                    "Vector embedding was not found".to_string(),
                ));
            }
            value
                .map(serde_json::to_value)
                .transpose()
                .map(|value| value.unwrap_or(Value::Null))
                .map_err(Into::into)
        }
        "delete" => {
            let id = required_id(request.embedding_id)?;
            let mut result = Map::new();
            result.insert("deleted".to_string(), Value::Bool(database.delete(&id)?));
            Ok(Value::Object(result))
        }
        "list" => Ok(serde_json::to_value(
            database.list(request.limit.unwrap_or(100), request.filter.as_ref())?,
        )?),
        operation => Err(VectorError::InvalidRequest(format!(
            "unsupported Vector operation `{operation}`"
        ))),
    }
}

fn authorize(root: &Path, name: &str, headers: &HeaderMap) -> VectorResult<()> {
    let account = headers
        .get("X-Dowe-Vector-Account")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| VectorError::Authentication("Vector account is required".to_string()))?;
    let secret = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(parse_bearer)
        .ok_or_else(|| {
            VectorError::Authentication("Vector bearer secret is required".to_string())
        })?;
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

fn required_id(id: Option<String>) -> VectorResult<String> {
    id.filter(|value| !value.is_empty())
        .ok_or_else(|| VectorError::InvalidRequest("Vector operation requires `id`".to_string()))
}

fn error_response(error: VectorError) -> Response {
    let status = match error {
        VectorError::Authentication(_) => StatusCode::UNAUTHORIZED,
        VectorError::Authorization(_) => StatusCode::FORBIDDEN,
        VectorError::InvalidName(_) | VectorError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    status.into_response()
}
