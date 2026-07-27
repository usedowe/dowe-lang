use crate::error::{KvError, KvResult};
use crate::protocol::{CacheRequest, CacheResponse};
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

static CONNECTIONS: OnceLock<StdMutex<HashMap<DoweCacheConfig, SharedSocket>>> = OnceLock::new();
static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DoweCacheConfig {
    pub host: String,
    pub port: u16,
    pub account: String,
    pub secret: String,
    pub name: String,
}

#[derive(Clone)]
pub struct DoweCacheClient {
    config: DoweCacheConfig,
    socket: SharedSocket,
}

impl DoweCacheClient {
    pub fn new(config: DoweCacheConfig) -> KvResult<Self> {
        let endpoint = endpoint(&config)?;
        let socket = shared_socket(&config)?;
        drop(endpoint);
        Ok(Self { config, socket })
    }

    pub async fn get(&self, key: &str, required: bool) -> KvResult<Value> {
        self.send("get", Some(key), None, None, required).await
    }

    pub async fn set(&self, key: &str, value: Value) -> KvResult<Value> {
        self.send("set", Some(key), Some(value), None, false).await
    }

    pub async fn delete(&self, key: &str) -> KvResult<Value> {
        self.send("delete", Some(key), None, None, false).await
    }

    pub async fn keys(&self, prefix: Option<&str>) -> KvResult<Value> {
        self.send("keys", None, None, prefix, false).await
    }

    pub async fn clear(&self) -> KvResult<Value> {
        self.send("clear", None, None, None, false).await
    }

    async fn send(
        &self,
        operation: &str,
        key: Option<&str>,
        value: Option<Value>,
        prefix: Option<&str>,
        required: bool,
    ) -> KvResult<Value> {
        let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed).to_string();
        let request = CacheRequest {
            id: id.clone(),
            operation: operation.to_string(),
            key: key.map(str::to_string),
            value,
            prefix: prefix.map(str::to_string),
            required,
        };
        for attempt in 0..2 {
            let mut socket = self.socket.lock().await;
            if socket.is_none() {
                *socket = Some(self.connect().await?);
            }
            let result = exchange(socket.as_mut().expect("socket"), &request).await;
            if let Ok(response) = result {
                return response.into_result(&id);
            }
            *socket = None;
            if attempt == 1 {
                return Err(KvError::Remote(
                    "Dowe Cache WebSocket operation failed".to_string(),
                ));
            }
        }
        unreachable!()
    }

    async fn connect(&self) -> KvResult<Socket> {
        let mut request = endpoint(&self.config)?
            .into_client_request()
            .map_err(|_| KvError::InvalidRequest("Dowe Cache endpoint is invalid".to_string()))?;
        request.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", self.config.secret))
                .map_err(|_| KvError::Authentication("Dowe Cache secret is invalid".to_string()))?,
        );
        request.headers_mut().insert(
            "X-Dowe-Cache-Account",
            HeaderValue::from_str(&self.config.account).map_err(|_| {
                KvError::Authentication("Dowe Cache account is invalid".to_string())
            })?,
        );
        connect_async(request)
            .await
            .map(|(socket, _)| socket)
            .map_err(websocket_error)
    }
}

async fn exchange(socket: &mut Socket, request: &CacheRequest) -> KvResult<CacheResponse> {
    let text = serde_json::to_string(request)?;
    socket
        .send(Message::Text(text.into()))
        .await
        .map_err(websocket_error)?;
    loop {
        let message = socket
            .next()
            .await
            .ok_or_else(|| KvError::Remote("Dowe Cache WebSocket closed".to_string()))?
            .map_err(websocket_error)?;
        match message {
            Message::Text(text) => {
                return serde_json::from_str::<CacheResponse>(&text)
                    .map_err(|_| KvError::Remote("Dowe Cache returned invalid JSON".to_string()));
            }
            Message::Binary(bytes) => {
                return serde_json::from_slice::<CacheResponse>(&bytes)
                    .map_err(|_| KvError::Remote("Dowe Cache returned invalid JSON".to_string()));
            }
            Message::Ping(payload) => {
                socket
                    .send(Message::Pong(payload))
                    .await
                    .map_err(websocket_error)?;
            }
            Message::Close(_) => {
                return Err(KvError::Remote("Dowe Cache WebSocket closed".to_string()));
            }
            Message::Pong(_) | Message::Frame(_) => {}
        }
    }
}

fn shared_socket(config: &DoweCacheConfig) -> KvResult<SharedSocket> {
    let mut connections = CONNECTIONS
        .get_or_init(|| StdMutex::new(HashMap::new()))
        .lock()
        .map_err(|_| KvError::Remote("Dowe Cache connection pool failed".to_string()))?;
    if let Some(socket) = connections.get(config) {
        return Ok(socket.clone());
    }
    let socket = Arc::new(Mutex::new(None));
    connections.insert(config.clone(), socket.clone());
    Ok(socket)
}

pub(crate) fn clear_connections() {
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

fn endpoint(config: &DoweCacheConfig) -> KvResult<String> {
    let host = config
        .host
        .strip_prefix("ws://")
        .or_else(|| config.host.strip_prefix("wss://"))
        .unwrap_or(&config.host);
    let loopback = is_loopback(host);
    let scheme = if config.host.starts_with("ws://") {
        if !loopback {
            return Err(KvError::InvalidRequest(
                "Dowe Cache requires WSS outside loopback".to_string(),
            ));
        }
        "ws"
    } else if config.host.starts_with("wss://") || !loopback {
        "wss"
    } else {
        "ws"
    };
    let mut url = reqwest::Url::parse(&format!("{scheme}://localhost"))
        .map_err(|_| KvError::InvalidRequest("Dowe Cache endpoint is invalid".to_string()))?;
    url.set_host(Some(host))
        .map_err(|_| KvError::InvalidRequest("Dowe Cache host is invalid".to_string()))?;
    url.set_port(Some(config.port))
        .map_err(|_| KvError::InvalidRequest("Dowe Cache port is invalid".to_string()))?;
    {
        let mut segments = url.path_segments_mut().map_err(|_| {
            KvError::InvalidRequest("Dowe Cache host cannot be a base URL".to_string())
        })?;
        segments.extend(["v1", "caches", config.name.as_str()]);
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

fn websocket_error(error: WebSocketError) -> KvError {
    match error {
        WebSocketError::Http(response) if response.status().as_u16() == 401 => {
            KvError::Authentication("Dowe Cache authentication failed".to_string())
        }
        WebSocketError::Http(response) if response.status().as_u16() == 403 => {
            KvError::Authorization("Dowe Cache authorization failed".to_string())
        }
        _ => KvError::Remote("Dowe Cache WebSocket transport failed".to_string()),
    }
}
