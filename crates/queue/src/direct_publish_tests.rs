use crate::{
    QueueClient, QueueConfig, QueueError, QueueProvider, QueueServerConfig, create_account,
    open_namespace, start_queue_server,
};
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn direct_publish_persists_only_to_the_declared_target_queue() {
    let root = TempDir::new().expect("tempdir");
    let queue = open_namespace(root.path(), "jobs").expect("queue");
    queue.declare("notifications").expect("target declaration");
    queue
        .declare("topic-listener")
        .expect("listener declaration");
    queue
        .bind("topic-listener", "notifications")
        .expect("topic binding");

    let report = queue
        .publish_direct("notifications", json!({"event": "created"}))
        .expect("direct publish");
    assert!(report.confirmed);
    let inspection = queue.inspect().expect("inspection");
    let queues = inspection.queues.expect("queues");
    assert_eq!(queues[0].queue, "notifications");
    assert_eq!(queues[0].ready, 1);
    assert_eq!(queues[1].queue, "topic-listener");
    assert_eq!(queues[1].ready, 0);
    drop(queue);

    let reopened = open_namespace(root.path(), "jobs").expect("reopened queue");
    let mut subscription = reopened
        .subscribe("notifications", "worker")
        .expect("subscription");
    let mut delivery = subscription.next().await.expect("next").expect("delivery");
    assert_eq!(delivery.message.value, json!({"event": "created"}));
    delivery.ack().await.expect("ack");
}

#[test]
fn direct_publish_missing_queue_fails_without_creating_topology() {
    let root = TempDir::new().expect("tempdir");
    let queue = open_namespace(root.path(), "jobs").expect("queue");

    assert!(matches!(
        queue.publish_direct("missing", json!({"event": "created"})),
        Err(QueueError::QueueNotFound(_))
    ));
    assert_eq!(
        queue.inspect().expect("inspection").queues,
        Some(Vec::new())
    );
}

#[tokio::test]
async fn direct_publish_uses_the_authenticated_protocol_and_persists_payload() {
    let root = TempDir::new().expect("tempdir");
    let account = create_account(root.path(), "jobs", "service", None).expect("account");
    let server = start_queue_server(QueueServerConfig {
        root: root.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("server");
    let client = QueueClient::new(QueueConfig {
        provider: QueueProvider::Dowe,
        host: server.addr.ip().to_string(),
        port: server.addr.port(),
        account: "service".to_string(),
        secret: account.secret,
        name: "jobs".to_string(),
    })
    .expect("client");
    client.declare("notifications").await.expect("declare");
    let report = client
        .publish_direct("notifications", json!({"event": "created"}))
        .await
        .expect("direct publish");
    assert!(report.confirmed);
    let queues = client
        .inspect()
        .await
        .expect("inspection")
        .queues
        .expect("queues");
    assert_eq!(queues[0].ready, 1);
    server.shutdown().await.expect("shutdown");
}
