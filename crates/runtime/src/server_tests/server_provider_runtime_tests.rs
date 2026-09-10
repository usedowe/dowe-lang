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

