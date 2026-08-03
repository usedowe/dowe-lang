use crate::error::{QueueError, QueueResult};
use crate::model::{DeliveryReceipt, QueueDelivery, delivery};
use crate::protocol::{QueueDeliveryFrame, QueueRequest, QueueWireFrame};
use crate::transport::{Socket, next_id, websocket_error};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use tokio::sync::{mpsc, oneshot};
use tokio::task::AbortHandle;
use tokio_tungstenite::tungstenite::{Error as WebSocketError, Message};

const COMMAND_CAPACITY: usize = 4;
const DELIVERY_CAPACITY: usize = 1;

pub struct DoweRemoteSubscription {
    inner: Arc<RemoteSubscriptionInner>,
    deliveries: mpsc::Receiver<QueueDeliveryFrame>,
}

struct RemoteSubscriptionInner {
    commands: mpsc::Sender<SubscriptionCommand>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    abort: Mutex<Option<AbortHandle>>,
    closed: AtomicBool,
    terminal: Mutex<Option<QueueError>>,
}

struct SubscriptionCommand {
    request: QueueRequest,
    response: oneshot::Sender<QueueResult<()>>,
}

struct RemoteReceipt {
    inner: Weak<RemoteSubscriptionInner>,
    receipt: String,
    resolved: bool,
}

type SocketSender = SplitSink<Socket, Message>;

impl DoweRemoteSubscription {
    pub(crate) fn new(socket: Socket) -> Self {
        let (commands, command_receiver) = mpsc::channel(COMMAND_CAPACITY);
        let (delivery_sender, deliveries) = mpsc::channel(DELIVERY_CAPACITY);
        let (shutdown, shutdown_receiver) = oneshot::channel();
        let inner = Arc::new(RemoteSubscriptionInner {
            commands,
            shutdown: Mutex::new(Some(shutdown)),
            abort: Mutex::new(None),
            closed: AtomicBool::new(false),
            terminal: Mutex::new(None),
        });
        let task_inner = Arc::downgrade(&inner);
        let handle = tokio::spawn(dispatch_subscription(
            socket,
            command_receiver,
            delivery_sender,
            shutdown_receiver,
            task_inner,
        ));
        if let Ok(mut abort) = inner.abort.lock() {
            *abort = Some(handle.abort_handle());
        }
        Self { inner, deliveries }
    }

    pub(crate) async fn next(&mut self) -> QueueResult<Option<QueueDelivery>> {
        match self.deliveries.recv().await {
            Some(frame) => Ok(Some(delivery(
                frame.message,
                RemoteReceipt {
                    inner: Arc::downgrade(&self.inner),
                    receipt: frame.receipt,
                    resolved: false,
                },
            ))),
            None => match self.inner.terminal() {
                Some(error) => Err(error),
                None => Ok(None),
            },
        }
    }

    pub(crate) async fn close(&mut self) -> QueueResult<()> {
        self.inner.stop();
        Ok(())
    }
}

impl Drop for DoweRemoteSubscription {
    fn drop(&mut self) {
        self.inner.stop();
    }
}

impl RemoteSubscriptionInner {
    async fn settle(&self, request: QueueRequest) -> QueueResult<()> {
        if self.closed.load(Ordering::Acquire) {
            return Err(expired_receipt());
        }
        let (response, received) = oneshot::channel();
        self.commands
            .send(SubscriptionCommand { request, response })
            .await
            .map_err(|_| expired_receipt())?;
        received.await.map_err(|_| expired_receipt())?
    }

    fn stop(&self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        if let Ok(mut shutdown) = self.shutdown.lock() {
            if let Some(shutdown) = shutdown.take() {
                let _ = shutdown.send(());
            }
        }
        if let Ok(mut abort) = self.abort.lock() {
            if let Some(abort) = abort.take() {
                abort.abort();
            }
        }
    }

    fn finish(&self, error: QueueError) {
        self.closed.store(true, Ordering::Release);
        if let Ok(mut terminal) = self.terminal.lock() {
            if terminal.is_none() {
                *terminal = Some(error);
            }
        }
    }

    fn terminal(&self) -> Option<QueueError> {
        self.terminal.lock().ok().and_then(|error| error.clone())
    }
}

impl RemoteReceipt {
    async fn settle(&mut self, operation: &str, requeue: bool) -> QueueResult<()> {
        if self.resolved {
            return Err(QueueError::InvalidReceipt(
                "Queue delivery receipt is already resolved".to_string(),
            ));
        }
        let inner = self.inner.upgrade().ok_or_else(expired_receipt)?;
        self.resolved = true;
        inner
            .settle(QueueRequest {
                id: next_id(),
                operation: operation.to_string(),
                queue: None,
                pattern: None,
                topic: None,
                value: None,
                consumer: None,
                receipt: Some(self.receipt.clone()),
                requeue,
            })
            .await
    }
}

impl DeliveryReceipt for RemoteReceipt {
    fn ack<'a>(
        &'a mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = QueueResult<()>> + Send + 'a>> {
        Box::pin(async move { self.settle("ack", false).await })
    }

    fn nack<'a>(
        &'a mut self,
        requeue: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = QueueResult<()>> + Send + 'a>> {
        Box::pin(async move { self.settle("nack", requeue).await })
    }
}

async fn dispatch_subscription(
    socket: Socket,
    mut commands: mpsc::Receiver<SubscriptionCommand>,
    deliveries: mpsc::Sender<QueueDeliveryFrame>,
    mut shutdown: oneshot::Receiver<()>,
    inner: Weak<RemoteSubscriptionInner>,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut pending = HashMap::new();
    let mut terminal = None;
    loop {
        tokio::select! {
            _ = &mut shutdown => break,
            command = commands.recv() => {
                let Some(command) = command else { break; };
                if let Err(error) = send_command(&mut sender, &command.request).await {
                    let _ = command.response.send(Err(error.clone()));
                    terminal = Some(error);
                    break;
                }
                pending.insert(command.request.id.clone(), command.response);
            }
            inbound = receiver.next() => {
                let frame = match receive_subscription_frame(&mut sender, inbound).await {
                    Ok(Some(frame)) => frame,
                    Ok(None) => continue,
                    Err(error) => {
                        terminal = Some(error);
                        break;
                    }
                };
                match frame {
                    QueueWireFrame::Response(response) => {
                        let id = response.id.clone();
                        let Some(reply) = pending.remove(&id) else {
                            terminal = Some(QueueError::Remote(
                                "Queue response correlation ID is unknown".to_string(),
                            ));
                            break;
                        };
                        let _ = reply.send(response.into_result(&id).map(|_| ()));
                    }
                    QueueWireFrame::Delivery(frame) => {
                        if deliveries.try_send(frame).is_err() {
                            terminal = Some(QueueError::Remote(
                                "Queue subscription exceeded its delivery flow limit".to_string(),
                            ));
                            break;
                        }
                    }
                }
            }
        }
    }
    if let Some(error) = terminal {
        if let Some(inner) = inner.upgrade() {
            inner.finish(error.clone());
        }
        for reply in pending.into_values() {
            let _ = reply.send(Err(error.clone()));
        }
    }
}

async fn send_command(sender: &mut SocketSender, request: &QueueRequest) -> QueueResult<()> {
    let text = serde_json::to_string(request)
        .map_err(|_| QueueError::InvalidRequest("Queue request cannot be encoded".to_string()))?;
    sender
        .send(Message::Text(text.into()))
        .await
        .map_err(websocket_error)
}

async fn receive_subscription_frame(
    sender: &mut SocketSender,
    inbound: Option<Result<Message, WebSocketError>>,
) -> QueueResult<Option<QueueWireFrame>> {
    let message = inbound
        .ok_or_else(|| QueueError::Remote("Dowe Queue WebSocket closed".to_string()))?
        .map_err(websocket_error)?;
    match message {
        Message::Text(text) => serde_json::from_str(&text).map(Some).map_err(invalid_wire),
        Message::Binary(bytes) => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(invalid_wire),
        Message::Ping(payload) => {
            sender
                .send(Message::Pong(payload))
                .await
                .map_err(websocket_error)?;
            Ok(None)
        }
        Message::Close(_) => Err(QueueError::Remote(
            "Dowe Queue WebSocket closed".to_string(),
        )),
        Message::Pong(_) | Message::Frame(_) => Ok(None),
    }
}

fn expired_receipt() -> QueueError {
    QueueError::InvalidReceipt("Queue delivery receipt is expired".to_string())
}

fn invalid_wire(_: serde_json::Error) -> QueueError {
    QueueError::Remote("Dowe Queue returned invalid JSON".to_string())
}
