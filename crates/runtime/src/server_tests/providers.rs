use super::*;

#[tokio::test]
async fn serves_vector_handlers_with_local_persistence() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        r#"main
  server port:0
    route "/api/vector"
      handler
        vector appVector provider:"dowe" host:"unresolved.example" port:4149 account:"unused" secret:"unused" name:"articles"
        emb alpha conn:appVector.upsert id:"alpha" vector:[1, 0] metadata:{ kind:"guide" }
        emb beta conn:appVector.upsert id:"beta" vector:[0.8, 0.2] metadata:{ kind:"guide" }
        emb matches conn:appVector.search vector:req.body.vector limit:2 minScore:0.5 where:{ kind:"guide" }
        return json:{ alpha:alpha matches:matches }"#,
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
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let response = reqwest::Client::new()
        .get(format!("{backend}/api/vector"))
        .timeout(Duration::from_secs(5))
        .json(&json!({ "vector": [1, 0] }))
        .send()
        .await
        .expect("vector")
        .json::<serde_json::Value>()
        .await
        .expect("json");
    assert_eq!(
        response["alpha"]["dimensions"], 2,
        "unexpected response: {response}"
    );
    assert_eq!(response["matches"][0]["id"], "alpha");
    assert!(temp.path().join(".dowe/vector/articles").exists());
    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_queue_handlers_with_local_durable_direct_publish() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    let queue = open_namespace(temp.path(), "jobs").expect("queue");
    queue.declare("notifications").expect("declare");
    drop(queue);
    fs::write(
        temp.path().join("main.dowe"),
        r#"main
  server port:0
    route "/api/messages"
      handler
        queue appQueue provider:"dowe" host:"unresolved.example" port:4150 account:"unused" secret:"unused" vhost:"jobs"
        msg sent conn:appQueue.publish queue:"notifications" payload:{ userId:"123" event:"user_created" }
        return json:{ ok:sent.ok messageId:sent.id }
    route "/api/cloudflare"
      handler
        queue appQueue provider:"cloudflare" host:"unresolved.example" port:4150 account:"unused" secret:"unused" vhost:"jobs"
        msg sent conn:appQueue.publish queue:"notifications" payload:{ userId:"123" event:"cloudflare" }
        return json:{ ok:sent.ok messageId:sent.id }
    route "/api/vercel"
      handler
        queue appQueue provider:"vercel" host:"unresolved.example" port:443 account:"unused" secret:"unused" vhost:"jobs"
        msg sent conn:appQueue.publish queue:"notifications" payload:{ userId:"123" event:"vercel" }
        return json:{ ok:sent.ok messageId:sent.id }
    route "/api/missing"
      handler
        queue appQueue provider:"dowe" host:"unresolved.example" port:4150 account:"unused" secret:"unused" vhost:"jobs"
        msg sent conn:appQueue.publish queue:"missing" payload:{ event:"ignored" }
        return json:sent"#,
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
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let client = reqwest::Client::new();
    let sent = client
        .get(format!("{backend}/api/messages"))
        .send()
        .await
        .expect("message")
        .json::<serde_json::Value>()
        .await
        .expect("message json");

    assert_eq!(sent["ok"], true, "unexpected response: {sent}");
    assert!(
        sent["messageId"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    for path in ["cloudflare", "vercel"] {
        let response = client
            .get(format!("{backend}/api/{path}"))
            .send()
            .await
            .expect("managed provider message")
            .json::<serde_json::Value>()
            .await
            .expect("managed provider json");
        assert_eq!(
            response["ok"], true,
            "unexpected {path} response: {response}"
        );
        assert!(
            response["messageId"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
        );
    }
    let missing = client
        .get(format!("{backend}/api/missing"))
        .send()
        .await
        .expect("missing");
    assert_eq!(missing.status(), reqwest::StatusCode::NOT_FOUND);
    let missing = missing
        .json::<serde_json::Value>()
        .await
        .expect("missing json");
    assert_eq!(missing["error"]["code"], "not_found");

    let queue = open_namespace(temp.path(), "jobs").expect("reopen queue");
    let mut subscription = queue
        .subscribe("notifications", "runtime")
        .expect("subscription");
    for event in ["user_created", "cloudflare", "vercel"] {
        let mut delivery = subscription.next().await.expect("next").expect("delivery");
        assert_eq!(delivery.message.value["userId"], "123");
        assert_eq!(delivery.message.value["event"], event);
        delivery.ack().await.expect("ack");
    }
    assert!(temp.path().join(".dowe/queue/jobs").exists());
    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn production_vector_host_local_uses_embedded_storage() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        r#"main
  server port:0
    route "/api/vector"
      handler
        vector appVector provider:"dowe" host:"local" port:4149 account:"unused" secret:"unused" name:"articles"
        emb saved conn:appVector.upsert id:"alpha" vector:[1, 0]
        return json:saved"#,
    )
    .expect("server");
    let project = compile_dev(temp.path()).expect("project");
    let server = start_production(project, SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("server");
    let response = reqwest::get(format!("http://{}/api/vector", server.addr))
        .await
        .expect("vector")
        .json::<serde_json::Value>()
        .await
        .expect("json");
    assert_eq!(response["id"], "alpha");
    assert!(temp.path().join(".dowe/vector/articles").exists());
    server.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn production_dowe_vector_uses_websocket_without_local_storage() {
    let app = TempDir::new().expect("app tempdir");
    let remote = TempDir::new().expect("remote tempdir");
    create_vector_account(
        remote.path(),
        "articles",
        "search-api",
        Some("secret-token"),
    )
    .expect("account");
    let vector_server = start_vector_server(VectorServerConfig {
        root: remote.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("vector server");
    write_fixture(app.path(), 0);
    fs::write(
        app.path().join(".env"),
        format!(
            "VECTOR_HOST={}\nVECTOR_PORT={}\nVECTOR_USER=search-api\nVECTOR_PASSWORD=secret-token\nVECTOR_DATABASE=articles\n",
            vector_server.addr.ip(),
            vector_server.addr.port()
        ),
    )
    .expect("env");
    fs::write(
        app.path().join(".env.example"),
        "VECTOR_HOST=\nVECTOR_PORT=\nVECTOR_USER=\nVECTOR_PASSWORD=\nVECTOR_DATABASE=\n",
    )
    .expect("env contract");
    fs::write(
        app.path().join("main.dowe"),
        r#"main
  server port:0
    route "/api/vector"
      handler
        vector appVector provider:"dowe" host:env.VECTOR_HOST port:env.VECTOR_PORT account:env.VECTOR_USER secret:env.VECTOR_PASSWORD name:env.VECTOR_DATABASE
        emb saved conn:appVector.upsert id:"alpha" vector:[1, 0] metadata:{ kind:"guide" }
        emb matches conn:appVector.search vector:[1, 0] limit:5 minScore:0.5 where:{ kind:"guide" }
        return json:{ saved:saved matches:matches }"#,
    )
    .expect("server");
    let project = compile_dev(app.path()).expect("project");
    let server = start_production(project, SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("server");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/api/vector", server.addr))
        .send()
        .await
        .expect("vector")
        .json::<serde_json::Value>()
        .await
        .expect("json");

    assert_eq!(response["saved"]["id"], "alpha");
    assert_eq!(response["matches"][0]["id"], "alpha");
    assert!(!app.path().join(".dowe/vector/articles").exists());
    assert!(remote.path().join(".dowe/vector/articles").exists());

    drop(client);
    server.shutdown().await.expect("shutdown");
    close_vector_connections();
    vector_server.shutdown().await.expect("vector shutdown");
}

#[tokio::test]
async fn file_storage_round_trips_request_bytes() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::create_dir_all(temp.path().join("server/handlers")).expect("server directory");
    let storage = temp.path().join("registry");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import artifactEndpoints from "@/server/endpoints"

main
  server port:0
    endpoints:artifactEndpoints
"#,
    )
    .expect("main");
    fs::write(
        temp.path().join("server/endpoints.dowe"),
        r#"import { uploadArtifact, readArtifact } from "@/server/handlers/artifacts"

endpoints artifactEndpoints
  group path:"/artifacts"
    post path:"/:name" handler:uploadArtifact
    get path:"/:name" handler:readArtifact
"#,
    )
    .expect("endpoints");
    fs::write(
        temp.path().join("server/handlers/artifacts.dowe"),
        format!(
            r#"handler uploadArtifact
  request payload source:"bytes"
  file stored source:"write" root:"{}" path:req.params.name data:payload
  return status:201 json:stored

handler readArtifact
  file artifact source:"read" root:"{}" path:req.params.name
  return bytes:artifact contentType:"application/octet-stream"
"#,
            storage.display(),
            storage.display()
        ),
    )
    .expect("handlers");
    let project = compile_dev(temp.path()).expect("project");
    let server = start_production(project, SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("server");
    let client = reqwest::Client::new();
    let payload = vec![0, 1, 2, 127, 128, 255];
    let written = client
        .post(format!("http://{}/artifacts/build.dowebin", server.addr))
        .body(payload.clone())
        .send()
        .await
        .expect("write");
    let written_status = written.status();
    let written_body = written.text().await.expect("write response");
    assert_eq!(
        written_status,
        reqwest::StatusCode::CREATED,
        "{written_body}"
    );
    let downloaded = client
        .get(format!("http://{}/artifacts/build.dowebin", server.addr))
        .send()
        .await
        .expect("read")
        .bytes()
        .await
        .expect("bytes");
    assert_eq!(downloaded.as_ref(), payload.as_slice());
    assert_eq!(
        fs::read(storage.join("build.dowebin")).expect("stored"),
        payload
    );
    server.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn vector_service_declaration_hosts_authenticated_websocket() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    create_vector_account(temp.path(), "articles", "search-api", Some("secret-token"))
        .expect("account");
    fs::write(
        temp.path().join("main.dowe"),
        "main\n  server port:0\n    vector service\n",
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
    let client = DoweVectorClient::new(DoweVectorConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
        account: "search-api".to_string(),
        secret: "secret-token".to_string(),
        name: "articles".to_string(),
    })
    .expect("client");
    client
        .upsert("alpha", vec![1.0, 0.0], json!({ "kind": "guide" }))
        .await
        .expect("upsert");
    let matches = client
        .search(vec![1.0, 0.0], 10, 0.5, None)
        .await
        .expect("search");
    assert_eq!(matches[0]["id"], "alpha");
    drop(client);
    close_vector_connections();
    servers.shutdown().await.expect("shutdown");
}

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
