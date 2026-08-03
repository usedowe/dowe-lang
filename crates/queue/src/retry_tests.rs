use crate::{QueueClient, QueueConfig, QueueError, QueueProvider, QueueRequest, open_namespace};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::time::{Duration, timeout};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn purge_response_loss_does_not_retry_after_commit() {
    let root = TempDir::new().expect("tempdir");
    let queue = open_namespace(root.path(), "orders").expect("queue");
    queue.declare("workers").expect("declare");
    queue.bind("workers", "orders.#").expect("bind");
    queue
        .publish("orders.created", json!({"id": "one"}))
        .expect("publish");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("listener address");
    let server_queue = queue.clone();
    let server = tokio::spawn(async move {
        let (stream, _) = timeout(Duration::from_secs(1), listener.accept())
            .await
            .expect("initial connection deadline")
            .expect("initial connection");
        let mut socket = accept_async(stream).await.expect("websocket handshake");
        let message = timeout(Duration::from_secs(1), socket.next())
            .await
            .expect("purge request deadline")
            .expect("purge request")
            .expect("valid purge request");
        let request: QueueRequest = match message {
            Message::Text(text) => serde_json::from_str(&text).expect("purge request JSON"),
            _ => panic!("purge request must be text"),
        };
        assert_eq!(request.operation, "purge");
        let report = server_queue
            .purge(request.queue.as_deref().expect("purge queue"))
            .expect("commit purge");
        drop(socket);
        let connections = match timeout(Duration::from_secs(1), listener.accept()).await {
            Ok(Ok((_stream, _))) => 2,
            Ok(Err(error)) => panic!("retry listener failed: {error}"),
            Err(_) => 1,
        };
        (report, connections)
    });
    let client = QueueClient::new(QueueConfig {
        provider: QueueProvider::Dowe,
        host: address.ip().to_string(),
        port: address.port(),
        account: "service".to_string(),
        secret: "private-secret".to_string(),
        name: "orders".to_string(),
    })
    .expect("client");

    let result = timeout(Duration::from_secs(1), client.purge("workers"))
        .await
        .expect("purge deadline");
    assert!(matches!(result, Err(QueueError::Remote(_))));
    let (report, connections) = timeout(Duration::from_secs(2), server)
        .await
        .expect("server deadline")
        .expect("server task");
    assert_eq!(report.removed, 1);
    assert_eq!(connections, 1);
    let queues = queue
        .inspect()
        .expect("inspect")
        .queues
        .expect("authoritative queues");
    assert_eq!(queues[0].ready, 0);
}

#[tokio::test]
async fn direct_publish_response_loss_does_not_retry_after_commit() {
    let root = TempDir::new().expect("tempdir");
    let queue = open_namespace(root.path(), "orders").expect("queue");
    queue.declare("notifications").expect("declare");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("listener address");
    let server_queue = queue.clone();
    let server = tokio::spawn(async move {
        let (stream, _) = timeout(Duration::from_secs(1), listener.accept())
            .await
            .expect("initial connection deadline")
            .expect("initial connection");
        let mut socket = accept_async(stream).await.expect("websocket handshake");
        let message = timeout(Duration::from_secs(1), socket.next())
            .await
            .expect("publish request deadline")
            .expect("publish request")
            .expect("valid publish request");
        let request: QueueRequest = match message {
            Message::Text(text) => serde_json::from_str(&text).expect("publish request JSON"),
            _ => panic!("publish request must be text"),
        };
        assert_eq!(request.operation, "publish_direct");
        let report = server_queue
            .publish_direct(
                request.queue.as_deref().expect("publish queue"),
                request.value.expect("publish payload"),
            )
            .expect("commit publish");
        drop(socket);
        let connections = match timeout(Duration::from_secs(1), listener.accept()).await {
            Ok(Ok((_stream, _))) => 2,
            Ok(Err(error)) => panic!("retry listener failed: {error}"),
            Err(_) => 1,
        };
        (report, connections)
    });
    let client = QueueClient::new(QueueConfig {
        provider: QueueProvider::Dowe,
        host: address.ip().to_string(),
        port: address.port(),
        account: "service".to_string(),
        secret: "private-secret".to_string(),
        name: "orders".to_string(),
    })
    .expect("client");

    let result = timeout(
        Duration::from_secs(1),
        client.publish_direct("notifications", json!({"id": "one"})),
    )
    .await
    .expect("publish deadline");
    assert!(matches!(result, Err(QueueError::Remote(_))));
    let (report, connections) = timeout(Duration::from_secs(2), server)
        .await
        .expect("server deadline")
        .expect("server task");
    assert!(report.confirmed);
    assert_eq!(connections, 1);
    assert_eq!(
        queue.inspect().expect("inspection").queues.expect("queues")[0].ready,
        1
    );
}

#[tokio::test]
async fn inspect_retries_once_after_remote_response_loss() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("listener address");
    let server = tokio::spawn(async move {
        let (first_stream, _) = timeout(Duration::from_secs(1), listener.accept())
            .await
            .expect("first connection deadline")
            .expect("first connection");
        let mut first_socket = accept_async(first_stream).await.expect("first handshake");
        let first_message = timeout(Duration::from_secs(1), first_socket.next())
            .await
            .expect("first inspect deadline")
            .expect("first inspect")
            .expect("valid first inspect");
        let first: QueueRequest = match first_message {
            Message::Text(text) => serde_json::from_str(&text).expect("first inspect JSON"),
            _ => panic!("first inspect must be text"),
        };
        drop(first_socket);
        let (second_stream, _) = timeout(Duration::from_secs(1), listener.accept())
            .await
            .expect("second connection deadline")
            .expect("second connection");
        let mut second_socket = accept_async(second_stream).await.expect("second handshake");
        let second_message = timeout(Duration::from_secs(1), second_socket.next())
            .await
            .expect("second inspect deadline")
            .expect("second inspect")
            .expect("valid second inspect");
        let second: QueueRequest = match second_message {
            Message::Text(text) => serde_json::from_str(&text).expect("second inspect JSON"),
            _ => panic!("second inspect must be text"),
        };
        let response = serde_json::json!({
            "kind": "response",
            "payload": {
                "id": second.id,
                "ok": true,
                "data": {"name": "orders", "queues": []},
                "error": null
            }
        });
        second_socket
            .send(Message::Text(response.to_string().into()))
            .await
            .expect("inspect response");
        (first.operation, second.operation)
    });
    let client = QueueClient::new(QueueConfig {
        provider: QueueProvider::Dowe,
        host: address.ip().to_string(),
        port: address.port(),
        account: "service".to_string(),
        secret: "private-secret".to_string(),
        name: "orders".to_string(),
    })
    .expect("client");

    let inspection = timeout(Duration::from_secs(2), client.inspect())
        .await
        .expect("inspect deadline")
        .expect("inspect retry");
    assert_eq!(inspection.queues, Some(Vec::new()));
    let operations = timeout(Duration::from_secs(1), server)
        .await
        .expect("server deadline")
        .expect("server task");
    assert_eq!(operations, ("inspect".to_string(), "inspect".to_string()));
}
