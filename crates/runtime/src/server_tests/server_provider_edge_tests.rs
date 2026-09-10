#[tokio::test]
async fn queue_service_declaration_hosts_authenticated_websocket() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    create_queue_account(temp.path(), "orders", "worker-api", Some("secret-token"))
        .expect("account");
    fs::write(
        temp.path().join("main.dowe"),
        "main\n  server port:0\n    queue service\n",
    )
    .expect("server");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: true,
            views: false,
            desktop: false,
        },
    )
    .await
    .expect("servers");
    let addr = servers.backend_addr.expect("backend addr");
    let client = QueueClient::new(QueueConfig {
        provider: QueueProvider::Dowe,
        host: addr.ip().to_string(),
        port: addr.port(),
        account: "worker-api".to_string(),
        secret: "secret-token".to_string(),
        name: "orders".to_string(),
    })
    .expect("client");
    client.declare("workers").await.expect("declare");
    client.bind("workers", "orders.#").await.expect("bind");
    client
        .publish("orders.created", json!({ "id": "one" }))
        .await
        .expect("publish");
    let mut subscription = client
        .subscribe("workers", "runtime")
        .await
        .expect("subscribe");
    let mut delivery = subscription.next().await.expect("next").expect("delivery");
    assert_eq!(delivery.message.value["id"], "one");
    delivery.ack().await.expect("ack");
    subscription.close().await.expect("close");
    servers.shutdown().await.expect("shutdown");
}
