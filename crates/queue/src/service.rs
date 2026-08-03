use crate::auth::verify_account;
use crate::engine::{DoweQueue, open_namespace};
use crate::error::{QueueError, QueueResult};
use crate::protocol::{QueueRequest, QueueResponse, QueueWireFrame};
use axum::Router;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path as AxumPath, State, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueServerConfig {
    pub root: PathBuf,
    pub host: String,
    pub port: u16,
}

pub struct RunningQueueServer {
    pub addr: std::net::SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<QueueResult<()>>,
}

#[derive(Clone)]
struct QueueServiceState {
    root: PathBuf,
}

impl Default for QueueServerConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            host: "127.0.0.1".to_string(),
            port: 4150,
        }
    }
}

pub async fn start_queue_server(config: QueueServerConfig) -> QueueResult<RunningQueueServer> {
    let listener = TcpListener::bind((config.host.as_str(), config.port))
        .await
        .map_err(|_| QueueError::Remote("Dowe Queue could not bind its listener".to_string()))?;
    let addr = listener
        .local_addr()
        .map_err(|_| QueueError::Remote("Dowe Queue listener is unavailable".to_string()))?;
    let router = queue_service_router(config.root);
    let (shutdown, signal) = oneshot::channel();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = signal.await;
            })
            .await
            .map_err(|_| QueueError::Remote("Dowe Queue server failed".to_string()))
    });
    Ok(RunningQueueServer {
        addr,
        shutdown: Some(shutdown),
        handle,
    })
}

pub async fn serve_queue_server(config: QueueServerConfig) -> QueueResult<()> {
    start_queue_server(config).await?.wait().await
}

pub fn queue_service_router(root: PathBuf) -> Router {
    Router::new()
        .route("/v1/queues/{name}", get(service_handler))
        .with_state(QueueServiceState { root })
}

pub async fn queue_service_upgrade(
    root: PathBuf,
    name: String,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    match authorize(&root, &name, &headers) {
        Ok(()) => upgrade
            .on_upgrade(move |socket| serve_socket(socket, root, name))
            .into_response(),
        Err(QueueError::Authorization(_)) => StatusCode::FORBIDDEN.into_response(),
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}

impl RunningQueueServer {
    pub async fn shutdown(mut self) -> QueueResult<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.handle
            .await
            .map_err(|_| QueueError::Remote("Dowe Queue server task failed".to_string()))?
    }

    pub async fn wait(mut self) -> QueueResult<()> {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(|_| QueueError::Remote("Dowe Queue signal failed".to_string()))?;
                self.shutdown().await
            }
            result = &mut self.handle => {
                result.map_err(|_| QueueError::Remote("Dowe Queue server task failed".to_string()))?
            }
        }
    }
}

async fn service_handler(
    State(state): State<QueueServiceState>,
    AxumPath(name): AxumPath<String>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    queue_service_upgrade(state.root, name, headers, upgrade).await
}

async fn serve_socket(socket: WebSocket, root: PathBuf, name: String) {
    let Ok(engine) = open_namespace(&root, &name) else {
        return;
    };
    let (mut sender, mut receiver) = socket.split();
    while let Some(message) = receiver.next().await {
        let request = match request_from_message(message, &mut sender).await {
            Some(request) => request,
            None => continue,
        };
        if request.operation == "subscribe" {
            let id = request.id.clone();
            let response = match subscription_request(&engine, request) {
                Ok(subscription) => {
                    if !send_response(
                        &mut sender,
                        QueueResponse::success(id, json!({"subscribed": true})),
                    )
                    .await
                    {
                        return;
                    }
                    serve_subscription(&mut sender, &mut receiver, subscription).await;
                    return;
                }
                Err(error) => QueueResponse::failure(id, &error),
            };
            if !send_response(&mut sender, response).await {
                return;
            }
            continue;
        }
        if !send_response(&mut sender, execute_management(&engine, request)).await {
            return;
        }
    }
}

async fn serve_subscription(
    sender: &mut SplitSink<WebSocket, Message>,
    receiver: &mut futures_util::stream::SplitStream<WebSocket>,
    mut subscription: crate::engine::DoweSubscription,
) {
    let mut outstanding = false;
    loop {
        if outstanding {
            let Some(message) = receiver.next().await else {
                return;
            };
            let request = match request_from_message(message, sender).await {
                Some(request) => request,
                None => continue,
            };
            let Some(resolved) = handle_subscription_request(sender, &subscription, request).await
            else {
                return;
            };
            if resolved {
                outstanding = false;
            }
            continue;
        }
        tokio::select! {
            delivery = subscription.next_frame() => {
                let Ok(Some(delivery)) = delivery else {
                    return;
                };
                if !send_frame(sender, QueueWireFrame::Delivery(delivery)).await {
                    return;
                }
                outstanding = true;
            }
            inbound = receiver.next() => {
                let Some(message) = inbound else {
                    return;
                };
                let request = match request_from_message(message, sender).await {
                    Some(request) => request,
                    None => continue,
                };
                if handle_subscription_request(sender, &subscription, request).await.is_none() {
                    return;
                }
            }
        }
    }
}

async fn handle_subscription_request(
    sender: &mut SplitSink<WebSocket, Message>,
    subscription: &crate::engine::DoweSubscription,
    request: QueueRequest,
) -> Option<bool> {
    let id = request.id.clone();
    let result = match request.operation.as_str() {
        "ack" => required_receipt(&request).and_then(|receipt| subscription.ack_token(receipt)),
        "nack" => required_receipt(&request)
            .and_then(|receipt| subscription.nack_token(receipt, request.requeue)),
        _ => Err(QueueError::InvalidRequest(
            "Queue subscription accepts only ACK or NACK".to_string(),
        )),
    };
    let resolved = result.is_ok();
    let response = match result {
        Ok(()) => QueueResponse::success(id, json!({"resolved": true})),
        Err(error) => QueueResponse::failure(id, &error),
    };
    send_response(sender, response).await.then_some(resolved)
}

fn execute_management(engine: &DoweQueue, request: QueueRequest) -> QueueResponse {
    let id = request.id.clone();
    let result = if request.id.is_empty() {
        Err(QueueError::InvalidRequest(
            "Queue request ID is required".to_string(),
        ))
    } else {
        match request.operation.as_str() {
            "declare" => {
                required_queue(&request).and_then(|queue| engine.declare(queue).and_then(to_value))
            }
            "bind" => required_queue(&request).and_then(|queue| {
                required_pattern(&request)
                    .and_then(|pattern| engine.bind(queue, pattern))
                    .and_then(to_value)
            }),
            "publish" => required_topic(&request).and_then(|topic| {
                request
                    .value
                    .clone()
                    .ok_or_else(|| {
                        QueueError::InvalidRequest("Queue publish requires JSON value".to_string())
                    })
                    .and_then(|value| engine.publish(topic, value))
                    .and_then(to_value)
            }),
            "publish_direct" => required_queue(&request).and_then(|queue| {
                request
                    .value
                    .clone()
                    .ok_or_else(|| {
                        QueueError::InvalidRequest(
                            "Queue direct publish requires JSON value".to_string(),
                        )
                    })
                    .and_then(|value| engine.publish_direct(queue, value))
                    .and_then(to_value)
            }),
            "inspect" => engine.inspect().and_then(to_value),
            "purge" => required_queue(&request)
                .and_then(|queue| engine.purge(queue))
                .and_then(to_value),
            _ => Err(QueueError::InvalidRequest(
                "Queue request operation is unsupported".to_string(),
            )),
        }
    };
    match result {
        Ok(value) => QueueResponse::success(id, value),
        Err(error) => QueueResponse::failure(id, &error),
    }
}

fn subscription_request(
    engine: &DoweQueue,
    request: QueueRequest,
) -> QueueResult<crate::engine::DoweSubscription> {
    if request.id.is_empty() {
        return Err(QueueError::InvalidRequest(
            "Queue request ID is required".to_string(),
        ));
    }
    let queue = required_queue(&request)?;
    let consumer = request.consumer.as_deref().ok_or_else(|| {
        QueueError::InvalidRequest("Queue subscribe requires consumer".to_string())
    })?;
    engine.subscribe(queue, consumer)
}

fn required_queue(request: &QueueRequest) -> QueueResult<&str> {
    request
        .queue
        .as_deref()
        .ok_or_else(|| QueueError::InvalidRequest("Queue request requires queue".to_string()))
}

fn required_pattern(request: &QueueRequest) -> QueueResult<&str> {
    request
        .pattern
        .as_deref()
        .ok_or_else(|| QueueError::InvalidRequest("Queue bind requires pattern".to_string()))
}

fn required_topic(request: &QueueRequest) -> QueueResult<&str> {
    request
        .topic
        .as_deref()
        .ok_or_else(|| QueueError::InvalidRequest("Queue publish requires topic".to_string()))
}

fn required_receipt(request: &QueueRequest) -> QueueResult<&str> {
    request
        .receipt
        .as_deref()
        .ok_or_else(|| QueueError::InvalidRequest("Queue delivery receipt is required".to_string()))
}

fn to_value<T: serde::Serialize>(value: T) -> QueueResult<Value> {
    serde_json::to_value(value)
        .map_err(|_| QueueError::Remote("Queue response serialization failed".to_string()))
}

async fn request_from_message(
    message: Result<Message, axum::Error>,
    sender: &mut SplitSink<WebSocket, Message>,
) -> Option<QueueRequest> {
    let request = match message {
        Ok(Message::Text(text)) => serde_json::from_str(&text),
        Ok(Message::Binary(bytes)) => serde_json::from_slice(&bytes),
        Ok(Message::Ping(payload)) => {
            let _ = sender.send(Message::Pong(payload)).await;
            return None;
        }
        Ok(Message::Pong(_)) => return None,
        Ok(Message::Close(_)) | Err(_) => return None,
    };
    match request {
        Ok(request) => Some(request),
        Err(_) => {
            let error = QueueError::InvalidRequest("Queue request is invalid JSON".to_string());
            let _ = send_response(sender, QueueResponse::failure(String::new(), &error)).await;
            None
        }
    }
}

async fn send_response(
    sender: &mut SplitSink<WebSocket, Message>,
    response: QueueResponse,
) -> bool {
    send_frame(sender, QueueWireFrame::Response(response)).await
}

async fn send_frame(sender: &mut SplitSink<WebSocket, Message>, frame: QueueWireFrame) -> bool {
    let Ok(text) = serde_json::to_string(&frame) else {
        return false;
    };
    sender.send(Message::Text(text.into())).await.is_ok()
}

fn authorize(root: &Path, name: &str, headers: &HeaderMap) -> QueueResult<()> {
    let account = headers
        .get("X-Dowe-Queue-Account")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| QueueError::Authentication("Queue account is required".to_string()))?;
    let secret = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(parse_bearer)
        .ok_or_else(|| QueueError::Authentication("Queue bearer secret is required".to_string()))?;
    verify_account(root, name, account, secret)
}

fn parse_bearer(value: &str) -> Option<&str> {
    let mut parts = value.split_whitespace();
    let scheme = parts.next()?;
    let secret = parts.next()?;
    (parts.next().is_none() && !secret.is_empty() && scheme.eq_ignore_ascii_case("Bearer"))
        .then_some(secret)
}
