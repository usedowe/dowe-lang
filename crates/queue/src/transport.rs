use crate::error::{QueueError, QueueResult};
use crate::model::QueueConfig;
use crate::protocol::{QueueRequest, QueueWireFrame};
use futures_util::{SinkExt, StreamExt};
use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::{Error as WebSocketError, Message};
#[cfg(test)]
use tokio_tungstenite::{Connector, connect_async_tls_with_config};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub(crate) async fn connect(config: &QueueConfig) -> QueueResult<Socket> {
    connect_async(authenticated_request(config)?)
        .await
        .map(|(socket, _)| socket)
        .map_err(websocket_error)
}

#[cfg(test)]
pub(crate) async fn connect_with_connector(
    config: &QueueConfig,
    connector: Connector,
) -> QueueResult<Socket> {
    connect_async_tls_with_config(authenticated_request(config)?, None, false, Some(connector))
        .await
        .map(|(socket, _)| socket)
        .map_err(websocket_error)
}

fn authenticated_request(
    config: &QueueConfig,
) -> QueueResult<tokio_tungstenite::tungstenite::http::Request<()>> {
    let mut request = dowe_endpoint(config)?
        .into_client_request()
        .map_err(|_| QueueError::InvalidRequest("Dowe Queue endpoint is invalid".to_string()))?;
    request.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", config.secret))
            .map_err(|_| QueueError::Authentication("Dowe Queue secret is invalid".to_string()))?,
    );
    request.headers_mut().insert(
        "X-Dowe-Queue-Account",
        HeaderValue::from_str(&config.account)
            .map_err(|_| QueueError::Authentication("Dowe Queue account is invalid".to_string()))?,
    );
    Ok(request)
}

pub(crate) async fn send_request(socket: &mut Socket, request: &QueueRequest) -> QueueResult<()> {
    let text = serde_json::to_string(request)
        .map_err(|_| QueueError::InvalidRequest("Queue request cannot be encoded".to_string()))?;
    socket
        .send(Message::Text(text.into()))
        .await
        .map_err(websocket_error)
}

pub(crate) async fn receive_frame(socket: &mut Socket) -> QueueResult<QueueWireFrame> {
    loop {
        let message = socket
            .next()
            .await
            .ok_or_else(|| QueueError::Remote("Dowe Queue WebSocket closed".to_string()))?
            .map_err(websocket_error)?;
        match message {
            Message::Text(text) => return serde_json::from_str(&text).map_err(invalid_wire),
            Message::Binary(bytes) => return serde_json::from_slice(&bytes).map_err(invalid_wire),
            Message::Ping(payload) => socket
                .send(Message::Pong(payload))
                .await
                .map_err(websocket_error)?,
            Message::Close(_) => {
                return Err(QueueError::Remote(
                    "Dowe Queue WebSocket closed".to_string(),
                ));
            }
            Message::Pong(_) | Message::Frame(_) => {}
        }
    }
}

pub(crate) fn dowe_endpoint(config: &QueueConfig) -> QueueResult<String> {
    let raw = config.host.trim().trim_end_matches('/');
    let (scheme, authority) = if let Some(authority) = raw.strip_prefix("wss://") {
        ("wss", authority)
    } else if let Some(authority) = raw.strip_prefix("ws://") {
        ("ws", authority)
    } else if let Some(authority) = raw.strip_prefix("https://") {
        ("wss", authority)
    } else if let Some(authority) = raw.strip_prefix("http://") {
        ("ws", authority)
    } else if is_loopback(raw.split('/').next().unwrap_or_default()) {
        ("ws", raw)
    } else {
        ("wss", raw)
    };
    let mut url = reqwest::Url::parse(&format!("{scheme}://{authority}"))
        .map_err(|_| QueueError::InvalidRequest("Dowe Queue host is invalid".to_string()))?;
    let host = url
        .host_str()
        .ok_or_else(|| QueueError::InvalidRequest("Dowe Queue host is invalid".to_string()))?;
    if scheme == "ws" && !is_loopback(host) {
        return Err(QueueError::InvalidRequest(
            "Dowe Queue requires WSS outside loopback".to_string(),
        ));
    }
    url.set_port(Some(config.port))
        .map_err(|_| QueueError::InvalidRequest("Dowe Queue port is invalid".to_string()))?;
    url.set_path(&format!("/v1/queues/{}", config.name));
    Ok(url.to_string())
}

pub(crate) fn websocket_error(error: WebSocketError) -> QueueError {
    match error {
        WebSocketError::Http(response) if response.status().as_u16() == 401 => {
            QueueError::Authentication("Dowe Queue authentication failed".to_string())
        }
        WebSocketError::Http(response) if response.status().as_u16() == 403 => {
            QueueError::Authorization("Dowe Queue authorization failed".to_string())
        }
        _ => QueueError::Remote("Dowe Queue WebSocket transport failed".to_string()),
    }
}

pub(crate) fn next_id() -> String {
    REQUEST_ID.fetch_add(1, Ordering::Relaxed).to_string()
}

fn invalid_wire(_: serde_json::Error) -> QueueError {
    QueueError::Remote("Dowe Queue returned invalid JSON".to_string())
}

fn is_loopback(host: &str) -> bool {
    let host = host.trim();
    let host = if let Some(value) = host.strip_prefix('[') {
        value.split(']').next().unwrap_or(value)
    } else if host.parse::<IpAddr>().is_ok() {
        host
    } else {
        host.split(':').next().unwrap_or(host)
    };
    host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}
