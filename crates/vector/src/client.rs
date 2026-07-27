use crate::error::{VectorError, VectorResult};
use crate::protocol::{VectorRequest, VectorResponse};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::{Error as WebSocketError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type SharedSocket = Arc<Mutex<Option<Socket>>>;

static CONNECTIONS: OnceLock<StdMutex<HashMap<DoweVectorConfig, SharedSocket>>> = OnceLock::new();
static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DoweVectorConfig {
    pub host: String,
    pub port: u16,
    pub account: String,
    pub secret: String,
    pub name: String,
}

#[derive(Clone)]
pub struct DoweVectorClient {
    config: DoweVectorConfig,
    socket: SharedSocket,
}

impl DoweVectorClient {
    pub fn new(config: DoweVectorConfig) -> VectorResult<Self> {
        validate_config(&config)?;
        endpoint(&config)?;
        let socket = shared_socket(&config)?;
        Ok(Self { config, socket })
    }

    pub async fn upsert(&self, id: &str, vector: Vec<f32>, metadata: Value) -> VectorResult<Value> {
        self.send(VectorRequest {
            id: next_id(),
            operation: "upsert".to_string(),
            embedding_id: Some(id.to_string()),
            vector: Some(vector),
            metadata: Some(metadata),
            filter: None,
            limit: None,
            min_score: None,
            required: false,
        })
        .await
    }

    pub async fn search(
        &self,
        vector: Vec<f32>,
        limit: usize,
        min_score: f32,
        filter: Option<Value>,
    ) -> VectorResult<Value> {
        self.send(VectorRequest {
            id: next_id(),
            operation: "search".to_string(),
            embedding_id: None,
            vector: Some(vector),
            metadata: None,
            filter,
            limit: Some(limit),
            min_score: Some(min_score),
            required: false,
        })
        .await
    }

    pub async fn read(&self, id: &str, required: bool) -> VectorResult<Value> {
        self.send(VectorRequest {
            id: next_id(),
            operation: "read".to_string(),
            embedding_id: Some(id.to_string()),
            vector: None,
            metadata: None,
            filter: None,
            limit: None,
            min_score: None,
            required,
        })
        .await
    }

    pub async fn delete(&self, id: &str) -> VectorResult<Value> {
        self.send(VectorRequest {
            id: next_id(),
            operation: "delete".to_string(),
            embedding_id: Some(id.to_string()),
            vector: None,
            metadata: None,
            filter: None,
            limit: None,
            min_score: None,
            required: false,
        })
        .await
    }

    pub async fn list(&self, limit: usize, filter: Option<Value>) -> VectorResult<Value> {
        self.send(VectorRequest {
            id: next_id(),
            operation: "list".to_string(),
            embedding_id: None,
            vector: None,
            metadata: None,
            filter,
            limit: Some(limit),
            min_score: None,
            required: false,
        })
        .await
    }

    async fn send(&self, request: VectorRequest) -> VectorResult<Value> {
        let expected_id = request.id.clone();
        for attempt in 0..2 {
            let mut socket = self.socket.lock().await;
            if socket.is_none() {
                *socket = Some(self.connect().await?);
            }
            let result = exchange(socket.as_mut().expect("socket"), &request).await;
            if let Ok(response) = result {
                return response.into_result(&expected_id);
            }
            *socket = None;
            if attempt == 1 {
                return Err(VectorError::Remote(
                    "Dowe Vector WebSocket operation failed".to_string(),
                ));
            }
        }
        unreachable!()
    }

    async fn connect(&self) -> VectorResult<Socket> {
        let mut request = endpoint(&self.config)?.into_client_request().map_err(|_| {
            VectorError::InvalidRequest("Dowe Vector endpoint is invalid".to_string())
        })?;
        request.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", self.config.secret)).map_err(|_| {
                VectorError::Authentication("Dowe Vector secret is invalid".to_string())
            })?,
        );
        request.headers_mut().insert(
            "X-Dowe-Vector-Account",
            HeaderValue::from_str(&self.config.account).map_err(|_| {
                VectorError::Authentication("Dowe Vector account is invalid".to_string())
            })?,
        );
        connect_async(request)
            .await
            .map(|(socket, _)| socket)
            .map_err(websocket_error)
    }
}

async fn exchange(socket: &mut Socket, request: &VectorRequest) -> VectorResult<VectorResponse> {
    socket
        .send(Message::Text(serde_json::to_string(request)?.into()))
        .await
        .map_err(websocket_error)?;
    loop {
        let message = socket
            .next()
            .await
            .ok_or_else(|| VectorError::Remote("Dowe Vector WebSocket closed".to_string()))?
            .map_err(websocket_error)?;
        match message {
            Message::Text(text) => {
                return serde_json::from_str::<VectorResponse>(&text).map_err(|_| {
                    VectorError::Remote("Dowe Vector returned invalid JSON".to_string())
                });
            }
            Message::Binary(bytes) => {
                return serde_json::from_slice::<VectorResponse>(&bytes).map_err(|_| {
                    VectorError::Remote("Dowe Vector returned invalid JSON".to_string())
                });
            }
            Message::Ping(payload) => {
                socket
                    .send(Message::Pong(payload))
                    .await
                    .map_err(websocket_error)?;
            }
            Message::Close(_) => {
                return Err(VectorError::Remote(
                    "Dowe Vector WebSocket closed".to_string(),
                ));
            }
            Message::Pong(_) | Message::Frame(_) => {}
        }
    }
}

fn validate_config(config: &DoweVectorConfig) -> VectorResult<()> {
    for (name, value) in [
        ("host", config.host.as_str()),
        ("account", config.account.as_str()),
        ("secret", config.secret.as_str()),
        ("name", config.name.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(VectorError::InvalidRequest(format!(
                "Vector connection `{name}` is empty"
            )));
        }
    }
    if config.port == 0 {
        return Err(VectorError::InvalidRequest(
            "Vector connection port must be greater than zero".to_string(),
        ));
    }
    Ok(())
}

fn shared_socket(config: &DoweVectorConfig) -> VectorResult<SharedSocket> {
    let mut connections = CONNECTIONS
        .get_or_init(|| StdMutex::new(HashMap::new()))
        .lock()
        .map_err(|_| VectorError::Remote("Dowe Vector connection pool failed".to_string()))?;
    if let Some(socket) = connections.get(config) {
        return Ok(socket.clone());
    }
    let socket = Arc::new(Mutex::new(None));
    connections.insert(config.clone(), socket.clone());
    Ok(socket)
}

pub fn close_remote_connections() {
    let sockets = CONNECTIONS
        .get()
        .and_then(|connections| connections.lock().ok())
        .map(|mut connections| {
            connections
                .drain()
                .map(|(_, socket)| socket)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for socket in sockets {
        if let Ok(mut socket) = socket.try_lock() {
            socket.take();
        }
    }
}

fn next_id() -> String {
    REQUEST_ID.fetch_add(1, Ordering::Relaxed).to_string()
}

fn endpoint(config: &DoweVectorConfig) -> VectorResult<String> {
    let host = config
        .host
        .strip_prefix("ws://")
        .or_else(|| config.host.strip_prefix("wss://"))
        .unwrap_or(&config.host);
    let loopback = is_loopback(host);
    let scheme = if config.host.starts_with("ws://") {
        if !loopback {
            return Err(VectorError::InvalidRequest(
                "Dowe Vector requires WSS outside loopback".to_string(),
            ));
        }
        "ws"
    } else if config.host.starts_with("wss://") || !loopback {
        "wss"
    } else {
        "ws"
    };
    let mut url = reqwest::Url::parse(&format!("{scheme}://localhost"))
        .map_err(|_| VectorError::InvalidRequest("Dowe Vector endpoint is invalid".to_string()))?;
    url.set_host(Some(host))
        .map_err(|_| VectorError::InvalidRequest("Dowe Vector host is invalid".to_string()))?;
    url.set_port(Some(config.port))
        .map_err(|_| VectorError::InvalidRequest("Dowe Vector port is invalid".to_string()))?;
    {
        let mut segments = url.path_segments_mut().map_err(|_| {
            VectorError::InvalidRequest("Dowe Vector host cannot be a base URL".to_string())
        })?;
        segments.extend(["v1", "vectors", config.name.as_str()]);
    }
    Ok(url.to_string())
}

fn is_loopback(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

fn websocket_error(error: WebSocketError) -> VectorError {
    match error {
        WebSocketError::Http(response) if response.status().as_u16() == 401 => {
            VectorError::Authentication("Dowe Vector authentication failed".to_string())
        }
        WebSocketError::Http(response) if response.status().as_u16() == 403 => {
            VectorError::Authorization("Dowe Vector authorization failed".to_string())
        }
        _ => VectorError::Remote("Dowe Vector WebSocket transport failed".to_string()),
    }
}
