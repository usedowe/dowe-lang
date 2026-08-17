use crate::{
    QueueClient, QueueConfig, QueueError, QueueMessage, QueueProvider, QueueResponse,
    QueueServerConfig, create_account, init_namespace, list_namespaces, open_namespace,
    start_queue_server, topic_matches,
};
use serde_json::{Value, json};
use std::fs;
use tempfile::TempDir;
use tokio::sync::oneshot;
use tokio::time::{Duration, timeout};

#[test]
fn topic_patterns_follow_amqp_words() {
    assert!(topic_matches("orders.created", "orders.created"));
    assert!(topic_matches("orders.*", "orders.created"));
    assert!(!topic_matches("orders.*", "orders.created.eu"));
    assert!(topic_matches("orders.#", "orders"));
    assert!(topic_matches("orders.#", "orders.created.eu"));
    assert!(topic_matches("#.created", "orders.created"));
    assert!(!topic_matches("orders.#.created", "orders"));
}

#[test]
fn namespaces_persist_queues_bindings_and_publications() {
    let root = TempDir::new().expect("tempdir");
    let queue = open_namespace(root.path(), "orders").expect("queue");
    assert_eq!(
        queue.declare("workers").expect("declare").created,
        Some(true)
    );
    assert_eq!(
        queue.bind("workers", "orders.*").expect("bind").created,
        Some(true)
    );
    let report = queue
        .publish("orders.created", json!({"id": "one"}))
        .expect("publish");
    assert_eq!(report.destinations, Some(vec!["workers".to_string()]));
    drop(queue);

    let reopened = open_namespace(root.path(), "orders").expect("reopen");
    let inspection = reopened.inspect().expect("inspect");
    let queues = inspection.queues.expect("authoritative queues");
    assert_eq!(queues[0].bindings, vec!["orders.*"]);
    assert_eq!(queues[0].ready, 1);
    assert_eq!(
        reopened.inspect_messages("workers", 10).expect("messages")[0].topic,
        "orders.created"
    );
    assert_eq!(
        list_namespaces(root.path()).expect("namespaces"),
        vec!["orders"]
    );
}

#[tokio::test]
async fn deliveries_ack_nack_and_close_with_at_least_once_delivery() {
    let root = TempDir::new().expect("tempdir");
    let queue = open_namespace(root.path(), "orders").expect("queue");
    queue.declare("workers").expect("declare");
    queue.bind("workers", "orders.#").expect("bind");
    queue
        .publish("orders.created", json!({"id": "one"}))
        .expect("publish");
    let mut first = queue.subscribe("workers", "first").expect("subscribe");
    let mut delivery = first.next().await.expect("delivery").expect("some");
    assert!(!delivery.message.redelivered);
    delivery.nack(true).await.expect("nack");
    let mut redelivery = first.next().await.expect("redelivery").expect("some");
    assert!(redelivery.message.redelivered);
    redelivery.ack().await.expect("ack");
    assert_eq!(
        queue.inspect().expect("inspect").queues.expect("queues")[0].ready,
        0
    );

    queue
        .publish("orders.created", json!({"id": "two"}))
        .expect("publish");
    let held = first.next().await.expect("held").expect("some");
    drop(first);
    let mut replacement = queue.subscribe("workers", "second").expect("replacement");
    let mut redelivered = replacement
        .next()
        .await
        .expect("redelivered")
        .expect("some");
    assert!(redelivered.message.redelivered);
    assert!(matches!(held.message.value, serde_json::Value::Object(_)));
    redelivered.ack().await.expect("ack replacement");
}

#[tokio::test]
async fn local_pub_sub_copies_and_receipts_are_terminal_and_session_bound() {
    let root = TempDir::new().expect("tempdir");
    let queue = open_namespace(root.path(), "orders").expect("queue");
    for name in ["audit", "workers"] {
        queue.declare(name).expect("declare");
        queue.bind(name, "orders.created").expect("bind");
    }
    let report = queue
        .publish("orders.created", json!({"id": "copy"}))
        .expect("publish");
    assert_eq!(
        report.destinations,
        Some(vec!["audit".to_string(), "workers".to_string()])
    );

    let mut audit = queue.subscribe("audit", "audit-consumer").expect("audit");
    let mut workers = queue
        .subscribe("workers", "worker-consumer")
        .expect("workers");
    let mut audit_delivery = audit.next().await.expect("audit next").expect("audit");
    let mut worker_delivery = workers.next().await.expect("worker next").expect("worker");
    assert_eq!(audit_delivery.message.value, worker_delivery.message.value);
    audit_delivery.nack(false).await.expect("terminal nack");
    assert!(matches!(
        audit_delivery.nack(false).await,
        Err(QueueError::InvalidReceipt(_))
    ));
    worker_delivery.ack().await.expect("ack");
    assert!(matches!(
        worker_delivery.ack().await,
        Err(QueueError::InvalidReceipt(_))
    ));

    queue
        .publish("orders.created", json!({"id": "stale"}))
        .expect("publish stale");
    let mut stale = audit.next().await.expect("stale next").expect("stale");
    audit.close().await.expect("close audit");
    assert!(matches!(
        stale.ack().await,
        Err(QueueError::InvalidReceipt(_))
    ));
}

#[tokio::test]
async fn receipts_cannot_be_resolved_by_another_subscription() {
    let root = TempDir::new().expect("tempdir");
    let queue = open_namespace(root.path(), "orders").expect("queue");
    queue.declare("workers").expect("declare");
    queue.bind("workers", "orders.#").expect("bind");
    queue
        .publish("orders.created", json!({"id": "session"}))
        .expect("publish");
    let mut owner = queue.subscribe("workers", "owner").expect("owner");
    let other = queue.subscribe("workers", "other").expect("other");
    let frame = owner
        .next_frame()
        .await
        .expect("next frame")
        .expect("frame");

    assert!(matches!(
        other.ack_token(&frame.receipt),
        Err(QueueError::InvalidReceipt(_))
    ));
    owner.ack_token(&frame.receipt).expect("owner ack");
    assert_eq!(
        queue.inspect().expect("inspect").queues.expect("queues")[0].in_flight,
        0
    );
}

#[test]
fn restart_requeues_persisted_inflight_messages() {
    let root = TempDir::new().expect("tempdir");
    let namespace = root.path().join(".dowe/queue/orders");
    fs::create_dir_all(&namespace).expect("namespace");
    let state = json!({
        "version": 1,
        "name": "orders",
        "queues": {
            "workers": {
                "bindings": ["orders.#"],
                "ready": [],
                "in_flight": {
                    "expired": {
                        "session": "gone",
                        "message": {
                            "id": "01J00000000000000000000000",
                            "topic": "orders.created",
                            "value": {"id": "restart"},
                            "publishedAt": 1234,
                            "redelivered": false
                        }
                    }
                }
            }
        }
    });
    fs::write(
        namespace.join("state.json"),
        serde_json::to_vec(&state).expect("state"),
    )
    .expect("write state");

    let queue = open_namespace(root.path(), "orders").expect("reopen");
    let inspection = queue.inspect().expect("inspect");
    let queues = inspection.queues.expect("authoritative queues");
    assert_eq!(queues[0].ready, 1);
    assert_eq!(queues[0].in_flight, 0);
    let recovered: Value =
        serde_json::from_slice(&fs::read(namespace.join("state.json")).expect("recovered state"))
            .expect("decode state");
    assert_eq!(
        recovered["queues"]["workers"]["ready"][0]["redelivered"],
        true
    );
}

#[test]
fn auth_catalog_hashes_secrets_and_scopes_accounts() {
    let root = TempDir::new().expect("tempdir");
    let created =
        create_account(root.path(), "orders", "service", Some("private-secret")).expect("account");
    assert!(!format!("{created:?}").contains("private-secret"));
    let catalog =
        fs::read_to_string(root.path().join(".dowe/queue/_auth/accounts.json")).expect("catalog");
    assert!(!catalog.contains("private-secret"));
    crate::verify_account(root.path(), "orders", "service", "private-secret").expect("verify");
    assert!(matches!(
        crate::verify_account(root.path(), "other", "service", "private-secret"),
        Err(QueueError::Authorization(_))
    ));
}

#[test]
fn auth_catalog_scans_shared_account_records_before_authorizing() {
    let root = TempDir::new().expect("tempdir");
    create_account(root.path(), "orders", "service", Some("shared-secret"))
        .expect("orders account");
    create_account(root.path(), "billing", "service", Some("shared-secret"))
        .expect("billing account");
    crate::verify_account(root.path(), "orders", "service", "shared-secret")
        .expect("orders verify");
    crate::verify_account(root.path(), "billing", "service", "shared-secret")
        .expect("billing verify");
    assert!(matches!(
        crate::verify_account(root.path(), "other", "service", "shared-secret"),
        Err(QueueError::Authorization(_))
    ));
    assert!(matches!(
        crate::verify_account(root.path(), "orders", "service", "wrong-secret"),
        Err(QueueError::Authentication(_))
    ));
}

#[tokio::test]
async fn remote_ack_completes_while_next_waits_for_later_delivery() {
    let root = TempDir::new().expect("tempdir");
    create_account(root.path(), "orders", "service", Some("private-secret")).expect("account");
    let (server, client) = remote_server(&root).await;
    client.declare("workers").await.expect("declare");
    client.bind("workers", "orders.#").await.expect("bind");
    client
        .publish("orders.created", json!({"id": "first"}))
        .await
        .expect("publish");
    let mut subscription = client
        .subscribe("workers", "remote")
        .await
        .expect("subscribe");
    let delivery = subscription.next().await.expect("next").expect("delivery");
    assert_eq!(delivery.message.value["id"], "first");
    let (waiting, started) = oneshot::channel();
    let next = tokio::spawn(async move {
        let _ = waiting.send(());
        let result = subscription.next().await;
        (subscription, result)
    });
    started.await.expect("next started");
    let acknowledged = tokio::spawn(async move {
        let mut delivery = delivery;
        delivery.ack().await
    });
    timeout(Duration::from_secs(1), acknowledged)
        .await
        .expect("ack deadline")
        .expect("ack task")
        .expect("ack");
    client
        .publish("orders.created", json!({"id": "second"}))
        .await
        .expect("publish second");
    let (mut subscription, delivery) = timeout(Duration::from_secs(1), next)
        .await
        .expect("next deadline")
        .expect("next task");
    let mut delivery = delivery.expect("next result").expect("second delivery");
    assert_eq!(delivery.message.value["id"], "second");
    delivery.ack().await.expect("second ack");
    subscription.close().await.expect("close");
    server.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn remote_subscription_keeps_one_delivery_in_flight_until_settlement() {
    let root = TempDir::new().expect("tempdir");
    create_account(root.path(), "orders", "service", Some("private-secret")).expect("account");
    let (server, client) = remote_server(&root).await;
    client.declare("workers").await.expect("declare");
    client.bind("workers", "orders.#").await.expect("bind");
    client
        .publish("orders.created", json!({"id": "first"}))
        .await
        .expect("first publish");
    client
        .publish("orders.created", json!({"id": "second"}))
        .await
        .expect("second publish");
    let mut subscription = client
        .subscribe("workers", "remote")
        .await
        .expect("subscribe");
    let mut first = subscription
        .next()
        .await
        .expect("first next")
        .expect("first");
    tokio::task::yield_now().await;
    let inspection = open_namespace(root.path(), "orders")
        .expect("queue")
        .inspect()
        .expect("inspect");
    let queues = inspection.queues.expect("authoritative queues");
    assert_eq!(queues[0].ready, 1);
    assert_eq!(queues[0].in_flight, 1);
    first.ack().await.expect("first ack");
    let mut second = subscription
        .next()
        .await
        .expect("second next")
        .expect("second");
    assert_eq!(second.message.value["id"], "second");
    second.ack().await.expect("second ack");
    subscription.close().await.expect("close");
    server.shutdown().await.expect("shutdown");
}

#[test]
fn providers_validate_transport_without_contacting_rabbitmq() {
    let remote = QueueConfig {
        provider: QueueProvider::RabbitMq,
        host: "amqp://rabbit.example".to_string(),
        port: 5672,
        account: "service".to_string(),
        secret: "private-secret".to_string(),
        name: "orders".to_string(),
    };
    assert!(!format!("{remote:?}").contains("private-secret"));
    assert!(matches!(
        crate::rabbitmq::rabbitmq_endpoint(&remote),
        Err(QueueError::InvalidRequest(_))
    ));
    let local = QueueConfig {
        host: "127.0.0.1".to_string(),
        ..remote
    };
    assert!(
        crate::rabbitmq::rabbitmq_endpoint(&local)
            .expect("endpoint")
            .starts_with("amqp://")
    );
    assert!(
        QueueClient::new(QueueConfig {
            provider: QueueProvider::Dowe,
            host: "127.0.0.1".to_string(),
            port: 4150,
            account: "service".to_string(),
            secret: "private-secret".to_string(),
            name: "orders".to_string(),
        })
        .is_ok()
    );
}

#[test]
fn rabbitmq_messages_preserve_dowe_metadata_and_reject_invalid_external_json() {
    let message = QueueMessage {
        id: "01J00000000000000000000000".to_string(),
        topic: "orders.created".to_string(),
        value: json!({"id": "rabbit"}),
        published_at: 1_725_000_123_456,
        redelivered: false,
    };
    let (payload, properties) = crate::rabbitmq::encode_rabbit_message(&message).expect("encode");
    assert_eq!(
        properties
            .message_id()
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some(message.id.as_str())
    );
    assert_eq!(
        properties.timestamp().as_ref().copied(),
        Some(message.published_at)
    );
    let decoded =
        crate::rabbitmq::decode_rabbit_message(&payload, "orders.created", true, &properties)
            .expect("decode");
    assert_eq!(decoded.id, message.id);
    assert_eq!(decoded.published_at, message.published_at);
    assert_eq!(decoded.value, message.value);
    assert!(decoded.redelivered);
    assert_eq!(crate::rabbitmq::RABBIT_PREFETCH, 1);
    assert!(matches!(
        crate::rabbitmq::decode_rabbit_message(b"not-json", "orders.created", false, &properties),
        Err(QueueError::Remote(_))
    ));
}

#[test]
fn protocol_rejects_mismatched_correlations() {
    let response = QueueResponse::success("one".to_string(), json!({"ok": true}));
    assert!(matches!(
        response.into_result("two"),
        Err(QueueError::Remote(_))
    ));
}

#[test]
fn init_rejects_reserved_namespace() {
    let root = TempDir::new().expect("tempdir");
    assert!(init_namespace(root.path(), "_auth").is_err());
}

#[test]
fn report_wire_values_preserve_unknown_facts() {
    let report = crate::PublishReport {
        id: "01J00000000000000000000000".to_string(),
        destinations: None,
        confirmed: true,
    };
    let encoded = serde_json::to_value(&report).expect("encode report");
    assert!(encoded["destinations"].is_null());
    let decoded: crate::PublishReport = serde_json::from_value(encoded).expect("decode report");
    assert_eq!(decoded, report);
}

async fn remote_server(root: &TempDir) -> (crate::RunningQueueServer, QueueClient) {
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
        secret: "private-secret".to_string(),
        name: "orders".to_string(),
    })
    .expect("client");
    (server, client)
}
