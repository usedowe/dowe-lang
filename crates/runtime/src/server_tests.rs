use crate::DevEventType;
use crate::ProductionAccess;
use crate::server::{
    DevServerTargets, start_dev, start_dev_servers, start_production, start_production_with_access,
};
use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE, LOCATION};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use dowe_cache::{
    CacheServerConfig, clear_memory as clear_kv_memory, create_account as create_cache_account,
    start_cache_server,
};
use dowe_compiler::{compile_dev, compile_dev_web, compile_dev_with_seeders};
use dowe_crypto::sign_jws_hs256;
use dowe_database::{
    DatabaseServiceConfig, DoweDatabaseClient, DoweDatabaseConfig, create_account,
    start_database_service,
};
use dowe_queue::{
    QueueClient, QueueConfig, QueueProvider, create_account as create_queue_account, open_namespace,
};
use dowe_vector::{
    DoweVectorClient, DoweVectorConfig, VectorServerConfig,
    close_remote_connections as close_vector_connections, create_account as create_vector_account,
    start_vector_server,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn views_server_advances_when_the_preferred_port_is_occupied() {
    let preferred = match TcpListener::bind("127.0.0.1:7654").await {
        Ok(listener) => Some(listener),
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => None,
        Err(error) => panic!("preferred port: {error}"),
    };
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: false,
            views: true,
            desktop: false,
        },
    )
    .await
    .expect("servers");

    if let Some(listener) = preferred {
        assert_eq!(listener.local_addr().expect("preferred addr").port(), 7654);
    }
    assert!(servers.views_addr.expect("views addr").port() > 7654);
    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn server_inspector_is_available_only_on_the_development_backend() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
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
    .expect("development backend");
    let origin = format!("http://{}", servers.backend_addr.expect("backend"));
    let client = reqwest::Client::new();
    let dashboard = client
        .get(format!("{origin}/_dowe/dev/server/"))
        .send()
        .await
        .expect("dashboard");
    assert_eq!(dashboard.status(), reqwest::StatusCode::OK);
    let dashboard_text = dashboard.text().await.expect("dashboard text");
    assert!(dashboard_text.contains("Dowe Server Inspector"));
    assert!(dashboard_text.contains("Data studio"));
    assert!(dashboard_text.contains("data-data-name-select"));
    assert!(dashboard_text.contains("Endpoints"));
    assert!(dashboard_text.contains("WebSockets"));
    assert!(dashboard_text.contains("--dowe-nav-active: #56687a"));
    assert!(dashboard_text.contains(
        "border-color: transparent; background: transparent; color: var(--dowe-muted)"
    ));
    assert!(dashboard_text.contains(
        ".nav button.active .nav-icon { background: transparent; color: inherit; }"
    ));
    assert!(dashboard_text.contains("data-endpoint-execute"));
    assert!(dashboard_text.contains("data-endpoint-try"));
    assert!(dashboard_text.contains("data-endpoint-modal-close"));
    assert!(dashboard_text.contains("role=\"dialog\" aria-modal=\"true\""));
    assert!(!dashboard_text.contains("Source selection</h2>"));
    assert!(!dashboard_text.contains("Server map"));
    assert!(!dashboard_text.contains("Copy manifest"));
    let manifest = client
        .get(format!("{origin}/_dowe/dev/server/manifest.json"))
        .send()
        .await
        .expect("manifest");
    assert_eq!(manifest.status(), reqwest::StatusCode::OK);
    assert_eq!(
        manifest.headers()[reqwest::header::CACHE_CONTROL],
        "no-store"
    );
    let value: serde_json::Value = manifest.json().await.expect("manifest json");
    let data = client
        .get(format!("{origin}/_dowe/dev/server/data/database"))
        .send()
        .await
        .expect("database data");
    assert_eq!(data.status(), reqwest::StatusCode::OK);
    assert_eq!(
        data.json::<serde_json::Value>()
            .await
            .expect("database data json")["kind"],
        "database"
    );
    let route_id = value["routes"][0]["id"].as_str().expect("route id");
    let source = client
        .get(format!("{origin}/_dowe/dev/server/source/{route_id}"))
        .send()
        .await
        .expect("source");
    assert_eq!(source.status(), reqwest::StatusCode::OK);
    let selection = client
        .post(format!("{origin}/_dowe/dev/server/selection"))
        .json(&json!({ "id": route_id }))
        .send()
        .await
        .expect("selection");
    assert_eq!(selection.status(), reqwest::StatusCode::NO_CONTENT);
    assert!(
        temp.path()
            .join(".dowe/dev/server-inspector-selection.json")
            .is_file()
    );
    let status_route = value["routes"]
        .as_array()
        .expect("routes")
        .iter()
        .find(|route| route["path"] == "/api/status" && route["method"] == "GET")
        .expect("status route");
    let execute = client
        .post(format!("{origin}/_dowe/dev/server/execute"))
        .json(&json!({
            "id": status_route["id"],
            "method": "GET",
            "path": "/api/status"
        }))
        .send()
        .await
        .expect("execute");
    assert_eq!(execute.status(), reqwest::StatusCode::OK);
    let executed: serde_json::Value = execute.json().await.expect("execute json");
    assert_eq!(executed["status"], 200);
    assert_eq!(executed["body"], "OK");
    let create_route = value["routes"]
        .as_array()
        .expect("routes")
        .iter()
        .find(|route| route["path"] == "/api/posts" && route["method"] == "POST")
        .expect("create route");
    let create = client
        .post(format!("{origin}/_dowe/dev/server/execute"))
        .json(&json!({
            "id": create_route["id"],
            "method": "POST",
            "path": "/api/posts",
            "body": { "title": "Inspector" }
        }))
        .send()
        .await
        .expect("create execute");
    assert_eq!(create.status(), reqwest::StatusCode::OK);
    let created: serde_json::Value = create.json().await.expect("created json");
    assert_eq!(created["status"], 200);
    let created_body: serde_json::Value =
        serde_json::from_str(created["body"].as_str().expect("created body")).expect("body json");
    assert_eq!(created_body["created"], true);
    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_project_icons_without_cache_and_rejects_traversal() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    let icon = temp.path().join("icons/web/favicon-32x32.png");
    fs::create_dir_all(icon.parent().expect("icon parent")).expect("icon directory");
    fs::write(&icon, "png").expect("icon");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: false,
            views: true,
            desktop: false,
        },
    )
    .await
    .expect("servers");
    let views = format!("http://{}", servers.views_addr.expect("views addr"));
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{views}/icons/web/favicon-32x32.png"))
        .send()
        .await
        .expect("icon response");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("image/png")
    );
    let traversal = client
        .get(format!("{views}/assets/%2e%2e/main.dowe"))
        .send()
        .await
        .expect("traversal");
    assert_eq!(traversal.status(), reqwest::StatusCode::NOT_FOUND);

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_backend_views_and_websocket() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join(".env"),
        "BACKEND_URL=https://runtime.example.com\nINTERNAL_TOKEN=secret\n",
    )
    .expect("env");
    fs::write(
        temp.path().join("pages/login.dowe"),
        r#"page loginPage
  fn load
    request status method:"GET" route:"/api/status"
  Box
    Text
      "Login""#,
    )
    .expect("page");
    fs::create_dir_all(temp.path().join("i18n")).expect("i18n");
    fs::write(
        temp.path().join("i18n/en.dowe"),
        r#"translations default:true
  translation key:"home.hero.title" value:"Dowe builds systems.""#,
    )
    .expect("english");
    fs::write(
        temp.path().join("i18n/es.dowe"),
        r#"translations
  translation key:"home.hero.title" value:"Dowe construye sistemas.""#,
    )
    .expect("spanish");
    let project = compile_dev(temp.path()).expect("project");
    let design_path = format!("/{}", project.web.design_file_name());
    let translation_path = project
        .web
        .translation_chunks
        .iter()
        .find(|chunk| chunk.locale == "es")
        .and_then(|chunk| chunk.relative_path.strip_prefix("web").ok())
        .map(|path| format!("/{}", path.display()))
        .expect("translation chunk");
    let servers = start_dev(project).await.expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let views = format!("http://{}", servers.views_addr.expect("views addr"));

    let status = client
        .get(format!("{backend}/api/status"))
        .send()
        .await
        .expect("status")
        .text()
        .await
        .expect("status text");
    assert_eq!(status, "OK");

    let user = client
        .get(format!("{backend}/users/123"))
        .send()
        .await
        .expect("user")
        .text()
        .await
        .expect("user text");
    assert_eq!(user, "Hello User 123!");

    let post = client
        .post(format!("{backend}/api/posts"))
        .json(&json!({"title":"A"}))
        .send()
        .await
        .expect("post")
        .json::<serde_json::Value>()
        .await
        .expect("post json");
    assert_eq!(post["created"], true);
    assert_eq!(post["title"], "A");

    let missing_method = client
        .put(format!("{backend}/api/posts"))
        .send()
        .await
        .expect("missing method")
        .status();
    assert_eq!(missing_method, reqwest::StatusCode::METHOD_NOT_ALLOWED);

    let html = client
        .get(format!("{views}/"))
        .send()
        .await
        .expect("view")
        .text()
        .await
        .expect("view text");
    assert!(html.contains("Layout"));
    assert!(html.contains("Login"));
    assert!(html.contains(r#"<p class="text-md">Layout</p>"#));
    assert!(html.contains(r#"<p class="text-md">Login</p>"#));
    assert!(html.contains(&format!(
        r#"<link data-dowe-design rel="stylesheet" href="{design_path}">"#
    )));
    assert!(html.contains(r#"/chunks/pages/"#));
    assert!(html.contains(r#"data-dowe-router type="module" src="/router.js"#));
    assert!(html.contains(r#"/_dowe/dev/client.js"#));

    let css = client
        .get(format!("{views}/design.css"))
        .send()
        .await
        .expect("design css");
    assert_eq!(css.status(), reqwest::StatusCode::OK);
    let content_type = css
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let css = css.text().await.expect("design css text");
    assert!(content_type.contains("text/css"));
    assert!(css.contains(".card"));
    assert!(!css.contains(".p-96"));

    let client_script = client
        .get(format!("{views}/_dowe/dev/client.js"))
        .send()
        .await
        .expect("dev client")
        .text()
        .await
        .expect("dev client text");
    assert!(client_script.contains("/_dowe/dev/ws"));
    assert!(client_script.contains("location.reload"));
    assert!(client_script.contains("window.__doweHotUpdate"));
    assert!(client_script.contains("module_update"));

    let manifest = client
        .get(format!("{views}/manifest.json"))
        .send()
        .await
        .expect("web manifest");
    assert_eq!(manifest.status(), reqwest::StatusCode::OK);
    assert_eq!(
        manifest
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert!(
        manifest
            .text()
            .await
            .expect("web manifest text")
            .contains(r#""routes""#)
    );

    let module_source = temp.path().join("test-module.dex");
    fs::write(&module_source, b"test dex").expect("module source");
    crate::dev_modules::publish_dev_module(
        temp.path(),
        "android",
        "test123",
        "dex",
        &module_source,
    )
    .expect("module publish");
    let module_manifest = client
        .get(format!("{views}/_dowe/dev/modules/manifest.json"))
        .send()
        .await
        .expect("module manifest");
    assert_eq!(module_manifest.status(), reqwest::StatusCode::OK);
    assert!(
        module_manifest
            .text()
            .await
            .expect("module manifest text")
            .contains("test123")
    );
    let module = client
        .get(format!("{views}/_dowe/dev/modules/android/test123.dex"))
        .send()
        .await
        .expect("module");
    assert_eq!(
        module.bytes().await.expect("module bytes").as_ref(),
        b"test dex"
    );
    let traversal = client
        .get(format!(
            "{views}/_dowe/dev/modules/android/%2e%2e%2fmanifest.json"
        ))
        .send()
        .await
        .expect("module traversal");
    assert_eq!(traversal.status(), reqwest::StatusCode::NOT_FOUND);

    let public_env = client
        .get(format!("{views}/env.json"))
        .send()
        .await
        .expect("env")
        .text()
        .await
        .expect("env text");
    assert!(public_env.contains(r#""BACKEND_URL":"https://runtime.example.com""#));
    assert!(!public_env.contains("INTERNAL_TOKEN"));

    let translation = client
        .get(format!("{views}{translation_path}"))
        .send()
        .await
        .expect("translation");
    assert_eq!(translation.status(), reqwest::StatusCode::OK);
    let content_type = translation
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let translation = translation.text().await.expect("translation text");
    assert!(content_type.contains("application/javascript"));
    assert!(translation.contains("Dowe construye sistemas."));

    let (mut websocket, _) = connect_async(format!(
        "ws://{}/ws",
        servers.backend_addr.expect("backend addr")
    ))
    .await
    .expect("websocket");
    websocket
        .send(Message::Text("hello".into()))
        .await
        .expect("send");
    websocket.close(None).await.expect("close");
    let _ = websocket.next().await;

    let (mut dev_websocket, _) = connect_async(format!(
        "ws://{}/_dowe/dev/ws",
        servers.views_addr.expect("views addr")
    ))
    .await
    .expect("dev websocket");
    servers
        .events()
        .emit_module_update("web", "abc123", vec!["pages/login.dowe".to_string()]);
    let event = dev_websocket
        .next()
        .await
        .expect("dev event")
        .expect("dev event message")
        .to_text()
        .expect("dev event text")
        .to_string();
    assert!(event.contains(r#""type":"module_update""#));
    assert!(event.contains(r#""target":"web""#));
    assert!(event.contains(r#""version":"abc123""#));
    dev_websocket.close(None).await.expect("dev close");

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn starts_declared_udp_tcp_transports() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    udp name:"sip-udp" port:0
      packet pkt
        log "udp" pkt.addr pkt.text pkt.bytes
    tcp name:"sip-tcp" port:0
      connection conn
        log "tcp" conn.addr conn.text conn.bytes
    rtp min:40000 max:40002
    route "/api/status"
      response text:"OK""#,
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

    let udp_addr = servers
        .backend_transport_addrs
        .iter()
        .find(|(name, _)| name == "sip-udp")
        .map(|(_, addr)| *addr)
        .expect("udp addr");
    let tcp_addr = servers
        .backend_transport_addrs
        .iter()
        .find(|(name, _)| name == "sip-tcp")
        .map(|(_, addr)| *addr)
        .expect("tcp addr");

    let udp = UdpSocket::bind("127.0.0.1:0").await.expect("udp bind");
    udp.send_to(b"OPTIONS sip:test SIP/2.0", udp_addr)
        .await
        .expect("udp send");

    let mut tcp = TcpStream::connect(tcp_addr).await.expect("tcp connect");
    tcp.write_all(b"REGISTER sip:test SIP/2.0")
        .await
        .expect("tcp write");
    tcp.shutdown().await.expect("tcp shutdown");

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_llm_http_proxy_agent_response_and_websocket_bridge() {
    let upstream = MockOpenRouter::start().await;
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join(".env"),
        format!(
            "OPENROUTER_API_KEY=test-token\nOPENROUTER_BASE_URL=http://{}\n",
            upstream.addr
        ),
    )
    .expect("env");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/v1/chat/completions"
      method POST async req
        const body value:req.json
        http upstream method:"post" base:env.OPENROUTER_BASE_URL path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:body mode:"proxy"
        return proxy:upstream
    route "/api/v1/agent"
      method POST async req
        const request value:req.json
        agent chat source:"chat" request:request
        http upstream method:"post" base:env.OPENROUTER_BASE_URL path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:chat mode:"json"
        return agent:upstream request:request
    websocket "/api/v1/agent/ws"
      message ws
        ws request source:"json"
        send ws json:{ event:"started" requestId:request.requestId requestType:request.requestType model:request.model payload:{ stream:request.stream } }
        agent chat source:"chat" request:request
        http upstream method:"post" base:env.OPENROUTER_BASE_URL path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:chat mode:"proxy"
        bridge sse:upstream to:ws requestId:request.requestId requestType:request.requestType model:request.model"#,
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
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let chat = client
        .post(format!("{backend}/api/v1/chat/completions"))
        .json(&json!({"model":"openai/test","messages":[{"role":"user","content":"hello"}],"stream":false}))
        .send()
        .await
        .expect("chat");
    assert_eq!(chat.status(), reqwest::StatusCode::OK);
    let chat = chat.json::<serde_json::Value>().await.expect("chat json");
    assert_eq!(chat["choices"][0]["message"]["content"], "mock message");

    let agent = client
        .post(format!("{backend}/api/v1/agent"))
        .json(&json!({
            "requestId":"req-1",
            "requestType":"clarify",
            "model":"openai/test",
            "messages":[{"role":"user","content":"hello"}],
            "stream":false
        }))
        .send()
        .await
        .expect("agent")
        .json::<serde_json::Value>()
        .await
        .expect("agent json");
    assert_eq!(agent["requestId"], "req-1");
    assert_eq!(agent["requestType"], "clarify");
    assert_eq!(
        agent["payload"]["choices"][0]["message"]["content"],
        "mock message"
    );

    let seen = upstream.requests().await;
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[0].authorization, Some("Bearer test-token".to_string()));
    assert_eq!(seen[1].body["metadata"]["dowe_request_type"], "clarify");
    assert!(seen[1].body.get("requestId").is_none());
    assert!(seen[1].body.get("requestType").is_none());

    let before_stream_reject = upstream.requests().await.len();
    let rejected = client
        .post(format!("{backend}/api/v1/agent"))
        .json(&json!({
            "requestId":"req-http-stream",
            "requestType":"clarify",
            "model":"openai/test",
            "messages":[{"role":"user","content":"stream"}],
            "stream":true
        }))
        .send()
        .await
        .expect("stream reject");
    assert_eq!(rejected.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(upstream.requests().await.len(), before_stream_reject);

    let (mut websocket, _) = connect_async(format!(
        "ws://{}/api/v1/agent/ws",
        servers.backend_addr.expect("backend addr")
    ))
    .await
    .expect("websocket");
    websocket
        .send(Message::Text(
            json!({
                "requestId":"req-ws",
                "requestType":"clarify",
                "model":"openai/test",
                "messages":[{"role":"user","content":"stream"}],
                "stream":true
            })
            .to_string()
            .into(),
        ))
        .await
        .expect("send");
    let started = websocket_json(&mut websocket).await;
    let delta = websocket_json(&mut websocket).await;
    let done = websocket_json(&mut websocket).await;
    assert_eq!(started["event"], "started");
    assert_eq!(started["requestId"], "req-ws");
    assert_eq!(started["payload"]["stream"], true);
    assert_eq!(delta["event"], "delta");
    assert_eq!(delta["content"], "mock delta");
    assert_eq!(done["event"], "done");
    assert_eq!(done["payload"]["ok"], true);
    websocket.close(None).await.expect("close");

    servers.shutdown().await.expect("shutdown");
    upstream.shutdown().await;
}

#[tokio::test]
async fn serves_general_outbound_http_request_options() {
    let upstream = MockExternalApi::start().await;
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join(".env"),
        format!(
            "CATALOG_BASE_URL=http://{}\nCATALOG_TOKEN=test-catalog-token\n",
            upstream.addr
        ),
    )
    .expect("env");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/products"
      method GET async req
        http upstream method:"get" base:env.CATALOG_BASE_URL path:"/v1/products" headers:[{ name:"Accept" value:"application/json" }, { name:"X-Api-Key" value:env.CATALOG_TOKEN }] redirect:"manual" timeoutMs:5000 mode:"json"
        return json:upstream
    route "/api/redirect"
      method GET async req
        http upstream method:"get" base:env.CATALOG_BASE_URL path:"/redirect" redirect:"manual" mode:"json"
        return json:upstream
    route "/api/redirect-error"
      method GET async req
        http upstream method:"get" base:env.CATALOG_BASE_URL path:"/redirect" redirect:"error" mode:"json"
        return json:upstream
    route "/api/timeout"
      method GET async req
        http upstream method:"get" base:env.CATALOG_BASE_URL path:"/slow" timeoutMs:1 mode:"json"
        return json:upstream"#,
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
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let products = client
        .get(format!("{backend}/api/products"))
        .send()
        .await
        .expect("products")
        .json::<serde_json::Value>()
        .await
        .expect("products json");
    assert_eq!(products["ok"], true);
    assert_eq!(products["status"], 200);
    assert_eq!(products["redirected"], false);
    assert_eq!(products["json"]["items"][0]["name"], "Dowe Kit");
    assert_eq!(
        products["headers"]["content-type"],
        "application/json; charset=utf-8"
    );
    let seen = upstream.requests().await;
    assert_eq!(seen[0].accept, Some("application/json".to_string()));
    assert_eq!(seen[0].api_key, Some("test-catalog-token".to_string()));

    let redirect = client
        .get(format!("{backend}/api/redirect"))
        .send()
        .await
        .expect("redirect")
        .json::<serde_json::Value>()
        .await
        .expect("redirect json");
    assert_eq!(redirect["status"], 302);
    assert_eq!(redirect["ok"], false);
    assert_eq!(redirect["redirected"], false);
    assert_eq!(redirect["location"], "/v1/products");

    let blocked = client
        .get(format!("{backend}/api/redirect-error"))
        .send()
        .await
        .expect("redirect error");
    assert_eq!(blocked.status(), reqwest::StatusCode::BAD_GATEWAY);
    let blocked = blocked
        .json::<serde_json::Value>()
        .await
        .expect("blocked json");
    assert_eq!(blocked["error"]["code"], "http_redirect");

    let timeout = client
        .get(format!("{backend}/api/timeout"))
        .send()
        .await
        .expect("timeout");
    assert_eq!(timeout.status(), reqwest::StatusCode::GATEWAY_TIMEOUT);
    let timeout = timeout
        .json::<serde_json::Value>()
        .await
        .expect("timeout json");
    assert_eq!(timeout["error"]["code"], "http_timeout");

    servers.shutdown().await.expect("shutdown");
    upstream.shutdown().await;
}

#[tokio::test]
async fn protects_route_with_bearer_jwt_middleware() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::create_dir_all(temp.path().join("middlewares")).expect("middlewares");
    fs::write(
        temp.path().join(".env"),
        "JWT_SECRET=01234567890123456789012345678901\n",
    )
    .expect("env");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"
import requireBearer from "@/middlewares/auth"

main
  views:viewRoutes
  server port:0
    route "/users/:id" middleware:[requireBearer]
      handler req
        return text:"Hello {req.context.auth.subject}!"
    route "/api/status"
      response text:"OK""#,
    )
    .expect("server");
    fs::write(
        temp.path().join("middlewares/auth.dowe"),
        r#"middleware requireBearer params:{}
  bearer token value:req.header.Authorization
  jwt verified secret:env.JWT_SECRET algorithm:"HS256" token:token
  if verified.valid
    next context:{ auth:{ subject:verified.claims.sub claims:verified.claims } }
  return status:401 json:{ ok:false error:"Unauthorized" }"#,
    )
    .expect("middleware");
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
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let missing = client
        .get(format!("{backend}/users/123"))
        .send()
        .await
        .expect("missing");
    assert_eq!(missing.status(), reqwest::StatusCode::UNAUTHORIZED);

    let bad_scheme = client
        .get(format!("{backend}/users/123"))
        .header(reqwest::header::AUTHORIZATION, "Basic nope")
        .send()
        .await
        .expect("bad scheme");
    assert_eq!(bad_scheme.status(), reqwest::StatusCode::UNAUTHORIZED);

    let invalid = client
        .get(format!("{backend}/users/123"))
        .bearer_auth("not-a-jwt")
        .send()
        .await
        .expect("invalid");
    assert_eq!(invalid.status(), reqwest::StatusCode::UNAUTHORIZED);

    let token = sign_jws_hs256(
        &json!({"sub":"user-123","exp":4102444800u64}),
        "01234567890123456789012345678901",
    )
    .expect("token");
    let authorized = client
        .get(format!("{backend}/users/123"))
        .bearer_auth(token)
        .send()
        .await
        .expect("authorized")
        .text()
        .await
        .expect("body");
    assert_eq!(authorized, "Hello user-123!");

    let status = client
        .get(format!("{backend}/api/status"))
        .send()
        .await
        .expect("status")
        .text()
        .await
        .expect("status text");
    assert_eq!(status, "OK");

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn protects_grouped_websocket_with_query_jwt_middleware() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("middlewares")).expect("middlewares");
    fs::create_dir_all(temp.path().join("routes")).expect("routes");
    fs::write(
        temp.path().join(".env"),
        "JWT_SECRET=01234567890123456789012345678901\n",
    )
    .expect("env");
    fs::write(
        temp.path().join("middlewares/socket.dowe"),
        r#"middleware requireSocketToken
  jwt verified secret:env.JWT_SECRET algorithm:"HS256" token:req.query.token
  if verified.valid
    next
  return status:401 json:{ ok:false error:"Unauthorized" }"#,
    )
    .expect("middleware");
    fs::write(
        temp.path().join("routes/control.dowe"),
        r#"import requireSocketToken from "@/middlewares/socket"

endpoints controlRoutes
  group path:"/api/v1/sip"
    websocket path:"/control" middleware:[requireSocketToken]
      message ws
        send ws json:{ ok:true channel:"sip-control" }"#,
    )
    .expect("routes");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import controlRoutes from "@/routes/control"

main
  server port:0
    endpoints:controlRoutes"#,
    )
    .expect("main");

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
    let backend = servers.backend_addr.expect("backend");

    let missing = connect_async(format!("ws://{backend}/api/v1/sip/control"))
        .await
        .expect_err("missing token");
    assert!(missing.to_string().contains("401"));

    let token = sign_jws_hs256(
        &json!({"sub":"socket-user","exp":4102444800u64}),
        "01234567890123456789012345678901",
    )
    .expect("token");
    let (mut websocket, _) =
        connect_async(format!("ws://{backend}/api/v1/sip/control?token={token}"))
            .await
            .expect("authorized websocket");
    websocket
        .send(Message::Text("{}".into()))
        .await
        .expect("message");
    let response = websocket_json(&mut websocket).await;
    assert_eq!(response["channel"], "sip-control");
    websocket.close(None).await.expect("close");
    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn resolves_middleware_context_in_store_actions() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::create_dir_all(temp.path().join("middlewares")).expect("middlewares");
    fs::write(
        temp.path().join(".env"),
        "JWT_SECRET=01234567890123456789012345678901\n",
    )
    .expect("env");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"
import requireBearer from "@/middlewares/auth"

main
  views:viewRoutes
  server port:0
    route "/api/blogs" middleware:[requireBearer]
      method POST async req
        const body value:req.json
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created conn:db.insert table:"blogs" value:{ title:body.title ownerId:req.context.auth.subject }
        return status:201 json:created
    route "/api/blogs/:id/edit" middleware:[requireBearer]
      method PATCH async req
        const body value:req.json
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query updated conn:db.update table:"blogs" where:{ id:req.params.id ownerId:req.context.auth.subject } value:{ title:body.title } required:true
        return json:updated"#,
    )
    .expect("server");
    fs::write(
        temp.path().join("middlewares/auth.dowe"),
        r#"middleware requireBearer params:{}
  bearer token value:req.header.Authorization
  jwt verified secret:env.JWT_SECRET algorithm:"HS256" token:token
  if verified.valid
    next context:{ auth:{ subject:verified.claims.sub } }
  return status:401 json:{ ok:false error:"Unauthorized" }"#,
    )
    .expect("middleware");
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
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let unauthorized = client
        .post(format!("{backend}/api/blogs"))
        .send()
        .await
        .expect("unauthorized");
    assert_eq!(unauthorized.status(), reqwest::StatusCode::UNAUTHORIZED);

    let token = sign_jws_hs256(
        &json!({"sub":"blog-owner","exp":4102444800u64}),
        "01234567890123456789012345678901",
    )
    .expect("token");
    let created = client
        .post(format!("{backend}/api/blogs"))
        .bearer_auth(token)
        .json(&json!({"title":"Owned post"}))
        .send()
        .await
        .expect("create");
    assert!(created.status().is_success());
    let created = created
        .json::<serde_json::Value>()
        .await
        .expect("created json");
    assert_eq!(created["ownerId"], "blog-owner");

    let other_token = sign_jws_hs256(
        &json!({"sub":"other-owner","exp":4102444800u64}),
        "01234567890123456789012345678901",
    )
    .expect("other token");
    let denied = client
        .patch(format!("{backend}/api/blogs/{}/edit", created["id"]))
        .bearer_auth(other_token)
        .json(&json!({"title":"Attempted edit"}))
        .send()
        .await
        .expect("denied edit");
    assert_eq!(denied.status(), reqwest::StatusCode::NOT_FOUND);

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_layered_backend_with_middleware_functions_store_and_kv() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("layouts/auth.dowe"),
        r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
    )
    .expect("layout");
    fs::write(
        temp.path().join("pages/login.dowe"),
        r#"page loginPage
  Box
    Text
      "Login""#,
    )
    .expect("page");
    fs::create_dir_all(temp.path().join("handlers")).expect("handlers");
    fs::create_dir_all(temp.path().join("middlewares")).expect("middlewares");
    fs::create_dir_all(temp.path().join("server/services")).expect("services");
    fs::create_dir_all(temp.path().join("server/repositories")).expect("repositories");
    fs::create_dir_all(temp.path().join("types")).expect("types");
    fs::write(
        temp.path().join(".env"),
        "JWT_SECRET=01234567890123456789012345678901\n",
    )
    .expect("env");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"
import requireBearer from "@/middlewares/auth"
import listTickets from "@/handlers/tickets"
import createTicket from "@/handlers/tickets"

main
  views:viewRoutes
  server port:0
    route "/api/tickets" middleware:[requireBearer]
      method GET handler:listTickets
      method POST handler:createTicket"#,
    )
    .expect("main");
    fs::write(
        temp.path().join("middlewares/auth.dowe"),
        r#"middleware requireBearer params:{}
  bearer token value:req.header.Authorization
  jwt verified secret:env.JWT_SECRET algorithm:"HS256" token:token
  if verified.valid
    next context:{ auth:{ subject:verified.claims.sub } }
  return status:401 json:{ ok:false error:"Unauthorized" }"#,
    )
    .expect("middleware");
    fs::write(
        temp.path().join("types/tickets.dowe"),
        r#"type TicketInput
  title:string
  priority:string"#,
    )
    .expect("types");
    fs::write(
        temp.path().join("handlers/tickets.dowe"),
        r#"import TicketInput from "../types/tickets"
import listTicketsService from "@/server/services/tickets"
import createTicketService from "@/server/services/tickets"

handler listTickets req
  listTicketsService result args:{ status:"open" }
  return json:result

handler createTicket
  const body:TicketInput value:req.json
  createTicketService result args:{ title:body.title priority:body.priority status:"open" }
  return status:201 json:result"#,
    )
    .expect("handler");
    fs::write(
        temp.path().join("server/services/tickets.dowe"),
        r#"import listTicketsRepository from "@/server/repositories/tickets"
import createTicketRepository from "@/server/repositories/tickets"

fn listTicketsService params:{ status:string }
  listTicketsRepository result args:{ status:args.status }
  return value:{ ok:true data:result.rows cache:result.cache }

fn createTicketService params:{ title:string priority:string status:string }
  createTicketRepository result args:{ title:args.title priority:args.priority status:args.status }
  return value:{ ok:true data:result.rows created:result.created cache:result.cache }"#,
    )
    .expect("function");
    fs::write(
        temp.path().join("server/repositories/tickets.dowe"),
        r#"fn listTicketsRepository params:{ status:string }
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"support"
  query rows conn:db.list table:"tickets"
  cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"app" secret:"secret" name:"support-cache"
  kv saved conn:appCache.set key:"tickets:last-list" value:{ status:args.status }
  return value:{ rows:rows cache:saved }

fn createTicketRepository params:{ title:string priority:string status:string }
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"support"
  query created conn:db.insert table:"tickets" value:{ title:args.title priority:args.priority status:args.status createdAt:now updatedAt:now } required:["title","priority","status"]
  query rows conn:db.list table:"tickets"
  cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"app" secret:"secret" name:"support-cache"
  kv saved conn:appCache.set key:"tickets:last-created" value:{ id:created.id title:created.title }
  return value:{ rows:rows created:created cache:saved }"#,
    )
    .expect("function");
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
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let unauthorized = client
        .get(format!("{backend}/api/tickets"))
        .send()
        .await
        .expect("unauthorized");
    assert_eq!(unauthorized.status(), reqwest::StatusCode::UNAUTHORIZED);

    let token = sign_jws_hs256(
        &json!({"sub":"agent-1","exp":4102444800u64}),
        "01234567890123456789012345678901",
    )
    .expect("token");
    let created = client
        .post(format!("{backend}/api/tickets"))
        .bearer_auth(&token)
        .json(&json!({"title":"Printer offline","priority":"high"}))
        .send()
        .await
        .expect("create");
    assert_eq!(created.status(), reqwest::StatusCode::CREATED);
    let created = created
        .json::<serde_json::Value>()
        .await
        .expect("created json");
    assert_eq!(created["ok"], true);
    assert_eq!(created["created"]["title"], "Printer offline");
    assert_eq!(created["cache"]["key"], "tickets:last-created");

    let listed = client
        .get(format!("{backend}/api/tickets"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("list")
        .json::<serde_json::Value>()
        .await
        .expect("list json");
    assert_eq!(listed["ok"], true);
    assert_eq!(listed["data"].as_array().expect("tickets").len(), 1);
    assert_eq!(listed["cache"]["key"], "tickets:last-list");
    assert!(temp.path().join(".dowe/db/support/tickets").exists());
    assert!(temp.path().join(".dowe/kv/support-cache").exists());

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn server_standard_library_reusable_fn_chains_parse_sort_and_math() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("server/handlers")).expect("handlers directory");
    fs::create_dir_all(temp.path().join("server/functions")).expect("functions directory");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import summarize from "@/server/handlers/summarize"

main
  server port:0
    route "/api/stdlib"
      method POST handler:summarize"#,
    )
    .expect("main");
    fs::write(
        temp.path().join("server/handlers/summarize.dowe"),
        r#"import summarizeScores from "@/server/functions/scores"

handler summarize
  const body value:req.json
  summarizeScores result args:{ payload:body.payload }
  return json:result"#,
    )
    .expect("handler");
    fs::write(
        temp.path().join("server/functions/scores.dowe"),
        r#"fn summarizeScores params:{ payload:string }
  parse parsed source:"json" value:args.payload fallback:[]
  sort sorted source:"asc" values:parsed
  math total source:"sum" values:sorted
  return value:{ total:total sorted:sorted }"#,
    )
    .expect("function");

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
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let result = client
        .post(format!("{backend}/api/stdlib"))
        .json(&json!({ "payload": "[3,1,2]" }))
        .send()
        .await
        .expect("stdlib request");
    assert_eq!(result.status(), reqwest::StatusCode::OK);
    let result = result
        .json::<serde_json::Value>()
        .await
        .expect("result json");
    assert_eq!(result["total"], json!(6.0));
    assert_eq!(result["sorted"], json!([1, 2, 3]));

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn simplified_http_handler_and_function_resolve_dynamic_task_args() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("server/tasks")).expect("tasks directory");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import dispatch from "@/server/tasks/dispatch"
import recordAudit from "@/server/tasks/record-audit"

main
  server port:0
    route "/api/posts"
      method POST
        const body value:req.json
        str auditKey source:"join" values:["post", body.id] delimiter:":"
        task fn:recordAudit args:{ requestId:body.id auditKey:auditKey }
        task args:{ requestId:body.id auditKey:auditKey }
          log args.auditKey
        dispatch dispatched args:{ requestId:body.id auditKey:auditKey }
        return json:{ created:true ...body }"#,
    )
    .expect("main");
    fs::write(
        temp.path().join("server/tasks/record-audit.dowe"),
        r#"fn recordAudit params:{ requestId:string auditKey:string }
  log args.auditKey
  return value:null"#,
    )
    .expect("task");
    fs::write(
        temp.path().join("server/tasks/dispatch.dowe"),
        r#"import recordAudit from "./record-audit"

fn dispatch params:{ requestId:string auditKey:string }
  task fn:recordAudit args:{ requestId:args.requestId auditKey:args.auditKey }
  task args:{ requestId:args.requestId auditKey:args.auditKey }
    log args.auditKey
  return value:null"#,
    )
    .expect("dispatch");

    let project = compile_dev(temp.path()).expect("project");
    let capture_root = project.root.clone();
    crate::background_jobs::start_task_launch_capture(&capture_root);
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
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let response = client
        .post(format!("{backend}/api/posts"))
        .json(&json!({ "id": "order-7", "title": "Task order" }))
        .send()
        .await
        .expect("post request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response
            .json::<serde_json::Value>()
            .await
            .expect("response json"),
        json!({ "created": true, "id": "order-7", "title": "Task order" })
    );

    let launches = crate::background_jobs::take_task_launches(&capture_root);
    assert_eq!(launches.len(), 4);
    assert_eq!(launches[0].target.as_deref(), Some("recordAudit"));
    assert_eq!(
        launches[0].args,
        json!({ "requestId": "order-7", "auditKey": "post:order-7" })
    );
    assert_eq!(launches[1].target, None);
    assert_eq!(
        launches[1].args,
        json!({ "requestId": "order-7", "auditKey": "post:order-7" })
    );
    assert_eq!(launches[2].target.as_deref(), Some("recordAudit"));
    assert_eq!(
        launches[2].args,
        json!({ "requestId": "order-7", "auditKey": "post:order-7" })
    );
    assert_eq!(launches[3].target, None);
    assert_eq!(
        launches[3].args,
        json!({ "requestId": "order-7", "auditKey": "post:order-7" })
    );

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn direct_store_task_handlers_resolve_dynamic_task_args_once() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("server/tasks")).expect("tasks directory");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import dispatch from "@/server/tasks/dispatch"
import recordAudit from "@/server/tasks/record-audit"

main
  server port:0
    route "/api/events"
      method POST
        const body value:req.json
        str auditKey source:"join" values:["event", body.id] delimiter:":"
        database db provider:"postgres" host:"unreachable.invalid" port:5432 account:"unused" secret:"unused" name:"events"
        query created conn:db.insert table:"events" value:{ kind:"task" }
        task fn:recordAudit args:{ requestId:body.id auditKey:auditKey }
        task args:{ requestId:body.id auditKey:auditKey }
          log args.auditKey
        dispatch dispatched args:{ requestId:body.id auditKey:auditKey }
        return json:created
      method GET
        database db provider:"postgres" host:"unreachable.invalid" port:5432 account:"unused" secret:"unused" name:"events"
        query events conn:db.list table:"events"
        return json:events"#,
    )
    .expect("main");
    fs::write(
        temp.path().join("server/tasks/record-audit.dowe"),
        r#"fn recordAudit params:{ requestId:string auditKey:string }
  log args.auditKey
  return value:null"#,
    )
    .expect("task");
    fs::write(
        temp.path().join("server/tasks/dispatch.dowe"),
        r#"import recordAudit from "./record-audit"

fn dispatch params:{ requestId:string auditKey:string }
  task fn:recordAudit args:{ requestId:args.requestId auditKey:args.auditKey }
  task args:{ requestId:args.requestId auditKey:args.auditKey }
    log args.auditKey
  return value:null"#,
    )
    .expect("dispatch");

    let project = compile_dev(temp.path()).expect("project");
    let capture_root = project.root.clone();
    crate::background_jobs::start_task_launch_capture(&capture_root);
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
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let created = client
        .post(format!("{backend}/api/events"))
        .json(&json!({ "id": "event-7" }))
        .send()
        .await
        .expect("create request");
    assert_eq!(created.status(), reqwest::StatusCode::OK);
    let created = created
        .json::<serde_json::Value>()
        .await
        .expect("created json");
    assert_eq!(created["kind"], "task");
    assert!(created["id"].as_str().is_some());

    let launches = crate::background_jobs::take_task_launches(&capture_root);
    assert_eq!(launches.len(), 4);
    assert_eq!(launches[0].target.as_deref(), Some("recordAudit"));
    assert_eq!(launches[1].target, None);
    assert_eq!(launches[2].target.as_deref(), Some("recordAudit"));
    assert_eq!(launches[3].target, None);
    for launch in launches {
        assert_eq!(
            launch.args,
            json!({ "requestId": "event-7", "auditKey": "event:event-7" })
        );
    }

    let events = client
        .get(format!("{backend}/api/events"))
        .send()
        .await
        .expect("list request");
    assert_eq!(events.status(), reqwest::StatusCode::OK);
    let events = events
        .json::<serde_json::Value>()
        .await
        .expect("events json");
    assert_eq!(events.as_array().expect("events").len(), 1);
    assert_eq!(events[0]["kind"], "task");

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn reverse_proxy_response_headers_tasks_wait_for_real_upstream_headers() {
    let upstream_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("upstream listener");
    let upstream_addr = upstream_listener.local_addr().expect("upstream address");
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("server/tasks")).expect("tasks directory");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import delayedFirst from "@/server/tasks/delayed-first"
import delayedSecond from "@/server/tasks/delayed-second"
import immediateFirst from "@/server/tasks/immediate-first"
import immediateSecond from "@/server/tasks/immediate-second"

main
  server port:0
    route "/setup"
      handler
        cache routes provider:"dowe" host:"local" port:4148 account:"proxy" secret:"secret" name:"routes"
        kv saved conn:routes.set key:"route" value:{ url:"UPSTREAM_URL" projectId:"project_1" state:"ready" }
        return json:saved
    route "/proxy/*path"
      method POST
        cache routes provider:"dowe" host:"local" port:4148 account:"proxy" secret:"secret" name:"routes"
        kv route conn:routes.get key:"route" required:true
        task fn:immediateFirst args:{ event:{ phase:"first" } }
        task fn:delayedFirst args:{ event:{ projectId:route.projectId label:"first" custom:"custom-first" status:0 method:"placeholder" path:"placeholder" latencyMs:0 bytesIn:0 bytesOut:0 } } after:"headers"
        task fn:immediateSecond args:{ event:{ phase:"second" } }
        task fn:delayedSecond args:{ event:{ projectId:route.projectId label:"second" custom:"custom-second" status:0 method:"placeholder" path:"placeholder" latencyMs:0 bytesIn:0 bytesOut:0 } } after:"headers"
        return reverse:route.url"#
            .replace("UPSTREAM_URL", &format!("http://{upstream_addr}")),
    )
    .expect("main");
    fs::write(
        temp.path().join("server/tasks/immediate-first.dowe"),
        r#"type ImmediateFirstEvent
  phase:string

fn immediateFirst params:{ event:ImmediateFirstEvent }
  return value:null"#,
    )
    .expect("immediate first");
    fs::write(
        temp.path().join("server/tasks/immediate-second.dowe"),
        r#"type ImmediateSecondEvent
  phase:string

fn immediateSecond params:{ event:ImmediateSecondEvent }
  return value:null"#,
    )
    .expect("immediate second");
    fs::write(
        temp.path().join("server/tasks/delayed-first.dowe"),
        r#"type DelayedFirstEvent
  projectId:string
  label:string
  custom:string
  status:number
  method:string
  path:string
  latencyMs:number
  bytesIn:number
  bytesOut:number

fn delayedFirst params:{ event:DelayedFirstEvent }
  return value:null"#,
    )
    .expect("delayed first");
    fs::write(
        temp.path().join("server/tasks/delayed-second.dowe"),
        r#"type DelayedSecondEvent
  projectId:string
  label:string
  custom:string
  status:number
  method:string
  path:string
  latencyMs:number
  bytesIn:number
  bytesOut:number

fn delayedSecond params:{ event:DelayedSecondEvent }
  return value:null"#,
    )
    .expect("delayed second");

    let project = compile_dev(temp.path()).expect("project");
    let capture_root = project.root.clone();
    crate::background_jobs::start_task_launch_capture(&capture_root);
    let observed_at_headers = Arc::new(Mutex::new(Vec::new()));
    let upstream_capture_root = capture_root.clone();
    let upstream_observed = observed_at_headers.clone();
    let upstream_server = tokio::spawn(async move {
        let upstream = Router::new().fallback(move || {
            let capture_root = upstream_capture_root.clone();
            let observed = upstream_observed.clone();
            async move {
                *observed.lock().await = crate::background_jobs::task_launches(&capture_root)
                    .into_iter()
                    .filter_map(|launch| launch.target)
                    .collect();
                let mut response = "1234567890123456789".into_response();
                *response.status_mut() = StatusCode::CREATED;
                response
                    .headers_mut()
                    .insert("content-length", axum::http::HeaderValue::from_static("19"));
                response
            }
        });
        axum::serve(upstream_listener, upstream)
            .await
            .expect("upstream server");
    });
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
    let backend = format!("http://{}", servers.backend_addr.expect("backend address"));
    let client = reqwest::Client::new();
    let setup = client
        .get(format!("{backend}/setup"))
        .send()
        .await
        .expect("setup request");
    assert_eq!(setup.status(), reqwest::StatusCode::OK);

    let forwarded = client
        .post(format!("{backend}/proxy/items"))
        .body("payload")
        .send()
        .await
        .expect("forwarded request");
    assert_eq!(forwarded.status(), reqwest::StatusCode::CREATED);
    assert_eq!(forwarded.text().await.expect("forwarded body").len(), 19);
    assert_eq!(
        *observed_at_headers.lock().await,
        vec!["immediateFirst".to_string(), "immediateSecond".to_string()]
    );

    let launches = crate::background_jobs::take_task_launches(&capture_root);
    assert_eq!(launches.len(), 4);
    assert_eq!(launches[0].target.as_deref(), Some("immediateFirst"));
    assert_eq!(launches[1].target.as_deref(), Some("immediateSecond"));
    assert_eq!(launches[2].target.as_deref(), Some("delayedFirst"));
    assert_eq!(launches[3].target.as_deref(), Some("delayedSecond"));
    assert_eq!(launches[0].args, json!({ "event": { "phase": "first" } }));
    assert_eq!(launches[1].args, json!({ "event": { "phase": "second" } }));
    for launch in &launches[2..] {
        assert_eq!(launch.args["event"]["projectId"], "project_1");
        assert_eq!(launch.args["event"]["status"], 201);
        assert_eq!(launch.args["event"]["method"], "POST");
        assert_eq!(launch.args["event"]["path"], "/proxy/items");
        assert!(launch.args["event"]["latencyMs"].as_f64().is_some());
        assert_eq!(launch.args["event"]["bytesIn"], 7);
        assert_eq!(launch.args["event"]["bytesOut"], 19);
    }
    assert_eq!(launches[2].args["event"]["label"], "first");
    assert_eq!(launches[3].args["event"]["label"], "second");
    assert_eq!(launches[2].args["event"]["custom"], "custom-first");
    assert_eq!(launches[3].args["event"]["custom"], "custom-second");

    servers.shutdown().await.expect("shutdown");
    upstream_server.abort();
}

#[tokio::test]
async fn starts_only_selected_backend_server() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
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

    assert!(servers.backend_addr.is_some());
    assert!(servers.views_addr.is_none());

    let (mut dev_websocket, _) = connect_async(format!(
        "ws://{}/_dowe/dev/ws",
        servers.backend_addr.expect("backend addr")
    ))
    .await
    .expect("dev websocket");
    servers.events().emit(
        DevEventType::WatchReady,
        None::<String>,
        Some("ready"),
        Vec::new(),
    );
    let event = dev_websocket
        .next()
        .await
        .expect("dev event")
        .expect("dev event message")
        .to_text()
        .expect("dev event text")
        .to_string();
    assert!(event.contains(r#""type":"watch_ready""#));
    dev_websocket.close(None).await.expect("dev close");

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn starts_only_selected_views_server() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: false,
            views: true,
            desktop: false,
        },
    )
    .await
    .expect("servers");

    assert!(servers.backend_addr.is_none());
    assert!(servers.views_addr.is_some());

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn production_server_serves_backend_and_web_without_dev_endpoints() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("pages/login.dowe"),
        r#"page loginPage
  Box
    Text
      "Login"
    Input label:"Email""#,
    )
    .expect("page");
    let project = compile_dev(temp.path()).expect("project");
    let router_path = format!("/{}", project.web.router_file_name());
    let design_path = format!("/{}", project.web.design_file_name());
    let style_path = format!(
        "/{}",
        project.web.pages[0]
            .css_chunks
            .iter()
            .find(|path| path.starts_with("chunks/design/"))
            .expect("style capability")
    );
    let server = start_production(project, "127.0.0.1:0".parse().expect("addr"))
        .await
        .expect("server");
    let origin = format!("http://{}", server.addr);
    let client = reqwest::Client::new();

    let status = client
        .get(format!("{origin}/api/status"))
        .send()
        .await
        .expect("status")
        .text()
        .await
        .expect("status text");
    assert_eq!(status, "OK");

    let html_response = client.get(format!("{origin}/")).send().await.expect("html");
    assert_eq!(
        html_response.headers()[reqwest::header::CACHE_CONTROL],
        "no-cache"
    );
    let html = html_response.text().await.expect("html text");
    assert!(html.contains("Layout"));
    assert!(html.contains("Login"));
    assert!(!html.contains("/_dowe/dev/client.js"));
    assert!(html.contains(&format!(r#"href="{design_path}""#)));

    let identity = client
        .get(format!("{origin}{router_path}"))
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .send()
        .await
        .expect("identity router");
    assert_eq!(identity.status(), reqwest::StatusCode::OK);
    assert_eq!(
        identity.headers()[reqwest::header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    assert!(
        !identity
            .headers()
            .contains_key(reqwest::header::CONTENT_ENCODING)
    );
    let etag = identity.headers()[reqwest::header::ETAG]
        .to_str()
        .expect("etag")
        .to_string();

    let not_modified = client
        .get(format!("{origin}{router_path}"))
        .header(reqwest::header::IF_NONE_MATCH, &etag)
        .send()
        .await
        .expect("conditional router");
    assert_eq!(not_modified.status(), reqwest::StatusCode::NOT_MODIFIED);

    let brotli = client
        .get(format!("{origin}{router_path}"))
        .header(reqwest::header::ACCEPT_ENCODING, "gzip, br")
        .send()
        .await
        .expect("brotli router");
    assert_eq!(brotli.headers()[reqwest::header::CONTENT_ENCODING], "br");
    assert!(
        brotli.headers()[reqwest::header::VARY]
            .to_str()
            .expect("vary")
            .to_ascii_lowercase()
            .contains("accept-encoding")
    );

    let gzip = client
        .get(format!("{origin}{router_path}"))
        .header(reqwest::header::ACCEPT_ENCODING, "gzip")
        .send()
        .await
        .expect("gzip router");
    assert_eq!(gzip.headers()[reqwest::header::CONTENT_ENCODING], "gzip");

    let design = client
        .get(format!("{origin}{design_path}"))
        .header(reqwest::header::ACCEPT_ENCODING, "gzip")
        .send()
        .await
        .expect("design css");
    assert_eq!(design.status(), reqwest::StatusCode::OK);
    assert_eq!(
        design.headers()[reqwest::header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    assert_eq!(design.headers()[reqwest::header::CONTENT_ENCODING], "gzip");
    assert!(design.headers().contains_key(reqwest::header::ETAG));

    let style = client
        .get(format!("{origin}{style_path}"))
        .header(reqwest::header::ACCEPT_ENCODING, "gzip")
        .send()
        .await
        .expect("style capability");
    assert_eq!(style.status(), reqwest::StatusCode::OK);
    assert_eq!(
        style.headers()[reqwest::header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    assert_eq!(style.headers()[reqwest::header::CONTENT_ENCODING], "gzip");
    assert!(style.headers().contains_key(reqwest::header::ETAG));

    let environment = client
        .get(format!("{origin}/env.json"))
        .send()
        .await
        .expect("environment");
    assert_eq!(
        environment.headers()[reqwest::header::CACHE_CONTROL],
        "no-store"
    );

    let dev_client = client
        .get(format!("{origin}/_dowe/dev/client.js"))
        .send()
        .await
        .expect("dev client");
    assert_eq!(dev_client.status(), reqwest::StatusCode::NOT_FOUND);

    for path in [
        "/_dowe/dev/inspector.json",
        "/_dowe/dev/inspector-selection",
        "/_dowe/dev/server/",
        "/_dowe/dev/server/manifest.json",
        "/_dowe/dev/server/execute",
    ] {
        let inspector = client
            .get(format!("{origin}{path}"))
            .send()
            .await
            .expect("production inspector");
        assert_eq!(inspector.status(), reqwest::StatusCode::NOT_FOUND);
    }

    server.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn production_server_serves_view_only_projects() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        "import viewRoutes from \"@/routes/view\"\n\nmain\n  views:viewRoutes\n",
    )
    .expect("view-only main");
    let project = compile_dev_web(temp.path()).expect("project");
    assert!(!project.capabilities.server);
    assert!(project.capabilities.views);

    let server = start_production(project, "127.0.0.1:0".parse().expect("addr"))
        .await
        .expect("server");
    let response = reqwest::get(format!("http://{}/", server.addr))
        .await
        .expect("response");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let html = response.text().await.expect("html");
    assert!(html.contains("Layout"));
    assert!(html.contains("Login"));

    server.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn production_access_protects_routes_and_assets_before_the_application() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    let project = compile_dev(temp.path()).expect("project");
    let hash = format!("{:x}", Sha256::digest(b"stage-password-123"));
    let access = ProductionAccess::new("stage", &hash).expect("access");
    let server =
        start_production_with_access(project, "127.0.0.1:0".parse().expect("addr"), Some(access))
            .await
            .expect("server");
    let origin = format!("http://{}", server.addr);
    let client = reqwest::Client::new();

    for path in ["/api/status", "/design.css"] {
        let blocked = client
            .get(format!("{origin}{path}"))
            .send()
            .await
            .expect("blocked request");
        assert_eq!(blocked.status(), reqwest::StatusCode::UNAUTHORIZED);
        assert_eq!(blocked.headers()["cache-control"], "no-store");
        assert!(blocked.headers().contains_key("www-authenticate"));
    }

    let allowed = client
        .get(format!("{origin}/api/status"))
        .basic_auth("tester@example.com", Some("stage-password-123"))
        .send()
        .await
        .expect("allowed request");
    assert_eq!(allowed.status(), reqwest::StatusCode::OK);
    assert_eq!(allowed.headers()["x-robots-tag"], "noindex");
    assert_eq!(allowed.text().await.expect("body"), "OK");

    server.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn separates_views_from_the_local_desktop_server() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"

views viewRoutes
  group path:"/" layout:AuthLayout
    route path:"" page:loginPage
    route path:"docs/functions" page:loginPage"#,
    )
    .expect("views");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/status"
      response text:"Backend OK"
  desktop
    server port:0
      route "/api/status"
        response text:"Desktop OK""#,
    )
    .expect("server");
    let project = compile_dev(temp.path()).expect("project");
    let design_path = format!("/{}", project.web.design_file_name());
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: false,
            views: true,
            desktop: true,
        },
    )
    .await
    .expect("servers");
    assert!(servers.backend_addr.is_none());
    let views = format!("http://{}", servers.views_addr.expect("views addr"));
    let desktop = format!("http://{}", servers.desktop_addr.expect("desktop addr"));
    let client = reqwest::Client::new();

    let html = client
        .get(format!("{views}/"))
        .send()
        .await
        .expect("desktop entry")
        .text()
        .await
        .expect("desktop html");
    assert!(html.contains("Layout"));
    assert!(html.contains(r#"data-dowe-router type="module" src="/router.js"#));
    assert!(html.contains(r#"src="/_dowe/dev/client.js""#));

    let nested_html = client
        .get(format!("{views}/docs/functions"))
        .send()
        .await
        .expect("desktop nested route")
        .text()
        .await
        .expect("desktop nested html");
    assert!(nested_html.contains(&format!(r#"href="{design_path}""#)));
    assert!(nested_html.contains(r#"data-dowe-router type="module" src="/router.js"#));
    assert!(nested_html.contains("/chunks/layouts/"));
    assert!(nested_html.contains("/chunks/pages/"));
    assert!(!nested_html.contains(r#"src="../router.js"#));

    let status = client
        .get(format!("{desktop}/api/status"))
        .send()
        .await
        .expect("desktop status")
        .text()
        .await
        .expect("desktop status text");
    assert_eq!(status, "Desktop OK");

    let desktop_entry = client
        .get(format!("{desktop}/"))
        .send()
        .await
        .expect("desktop entry");
    assert_eq!(desktop_entry.status(), reqwest::StatusCode::NOT_FOUND);

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn desktop_without_a_local_server_reuses_only_the_views_listener() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    let project = compile_dev(temp.path()).expect("project");
    assert!(project.desktop_server.is_none());
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: false,
            views: true,
            desktop: true,
        },
    )
    .await
    .expect("servers");
    assert!(servers.backend_addr.is_none());
    assert!(servers.desktop_addr.is_none());
    let views = format!("http://{}", servers.views_addr.expect("views addr"));
    let html = reqwest::get(format!("{views}/"))
        .await
        .expect("desktop entry")
        .text()
        .await
        .expect("desktop html");

    assert!(html.contains("Layout"));
    assert!(html.contains("Login"));
    assert!(html.contains(r#"data-dowe-router type="module" src="/router.js"#));
    assert!(html.contains(r#"src="/_dowe/dev/client.js""#));

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn permits_managed_desktop_origin_for_backend_requests() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    cors target:"server" devOrigins:true headers:["Content-Type"]
    route "/api/status"
      response text:"OK"
  desktop
    server port:0
      route "/api/status"
        response text:"Desktop OK""#,
    )
    .expect("server");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: true,
            views: true,
            desktop: true,
        },
    )
    .await
    .expect("servers");
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let views_origin = format!("http://{}", servers.views_addr.expect("views addr"));
    let allowed = reqwest::Client::new()
        .request(reqwest::Method::OPTIONS, format!("{backend}/api/status"))
        .header("Origin", views_origin.as_str())
        .header("Access-Control-Request-Method", "GET")
        .send()
        .await
        .expect("allowed");

    assert_eq!(allowed.status(), reqwest::StatusCode::NO_CONTENT);
    assert_eq!(
        allowed
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some(views_origin.as_str())
    );

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn development_uses_embedded_database_for_authored_providers() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/users/create"
      handler
        database db provider:"postgres" host:"unreachable.invalid" port:5432 account:"unused" secret:"unused" name:"db1"
        query created conn:db.insert table:"users" value:{ name:"Ana" roleId:"admin" }
        return json:created
    route "/api/users"
      handler
        database db provider:"d1" account:"unused" secret:"unused" name:"db1"
        query rows conn:db.query sql:"select * from users where roleId = \"admin\""
        return json:rows"#,
    )
    .expect("server");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev(project).await.expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let created = client
        .get(format!("{backend}/api/users/create"))
        .send()
        .await
        .expect("create")
        .json::<serde_json::Value>()
        .await
        .expect("created json");
    assert_eq!(created["name"], "Ana");
    assert!(created["id"].as_str().is_some());

    let rows = client
        .get(format!("{backend}/api/users"))
        .send()
        .await
        .expect("query")
        .json::<serde_json::Value>()
        .await
        .expect("query json");
    assert_eq!(rows.as_array().expect("rows").len(), 1);
    assert!(temp.path().join(".dowe/db/db1/users").exists());

    drop(client);
    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn development_applies_database_seeders_once_by_fingerprint() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::create_dir_all(temp.path().join("server/config")).expect("config");
    fs::write(
        temp.path().join("server/config/database.dowe"),
        r#"entity Users
  id:string primary:true
  name:string required:true

seeder Bootstrap
  insert entity:Users value:{ id:"01ARZ3NDEKTSV4RRFFQ69G5FAV" name:"Admin" }

database appDb provider:"postgres" host:"unreachable.invalid" port:5432 account:"unused" secret:"unused" name:"seeded" entities:[Users] seeders:[Bootstrap]
"#,
    )
    .expect("database");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"
import appDb from "@/server/config/database"

main
  views:viewRoutes
  server port:0
    route "/api/users"
      handler
        query users conn:appDb.list table:"users"
        return json:users"#,
    )
    .expect("server");

    let development_project = compile_dev(temp.path()).expect("development project");
    assert!(
        development_project.databases[0]
            .connection
            .seeders
            .is_empty()
    );

    let seed_project = compile_dev_with_seeders(temp.path()).expect("seeder project");
    crate::seed_local_databases(seed_project)
        .await
        .expect("seed local database");
    let second_seed_project = compile_dev_with_seeders(temp.path()).expect("second seeder project");
    crate::seed_local_databases(second_seed_project)
        .await
        .expect("seed local database twice");

    for _ in 0..2 {
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
        let users = reqwest::Client::new()
            .get(format!("{backend}/api/users"))
            .header(reqwest::header::CONNECTION, "close")
            .send()
            .await
            .expect("users")
            .json::<serde_json::Value>()
            .await
            .expect("users json");
        assert_eq!(users.as_array().expect("users array").len(), 1);
        assert_eq!(users[0]["name"], "Admin");
        servers.shutdown().await.expect("shutdown");
    }
}

#[tokio::test]
async fn database_service_declaration_hosts_authenticated_websocket() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    create_account(temp.path(), "clinic", "clinic-api", Some("secret-token")).expect("account");
    fs::write(
        temp.path().join("main.dowe"),
        "main\n  server port:0\n    database service\n",
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
    let client = DoweDatabaseClient::new(DoweDatabaseConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
        database: "clinic".to_string(),
        account: "clinic-api".to_string(),
        secret: "secret-token".to_string(),
    })
    .expect("client");

    let created = client
        .insert("appointments", json!({ "patientName": "Ana" }))
        .await
        .expect("insert");
    assert_eq!(created["patientName"], "Ana");

    drop(client);
    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_kv_handlers_with_persistent_fallback() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/cache/save"
      handler
        cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"unused" secret:"unused" name:"clinic"
        kv saved conn:appCache.set key:"appointment:1" value:{ patientName:"Ana" }
        return json:saved
    route "/api/cache/read"
      handler
        cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"unused" secret:"unused" name:"clinic"
        kv value conn:appCache.get key:"appointment:1" required:true
        kv keys conn:appCache.keys prefix:"appointment:"
        return json:{ patientName:value.patientName keys:keys }"#,
    )
    .expect("server");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev(project).await.expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let saved = client
        .get(format!("{backend}/api/cache/save"))
        .send()
        .await
        .expect("save")
        .json::<serde_json::Value>()
        .await
        .expect("save json");
    assert_eq!(saved["ok"], true);
    assert_eq!(saved["key"], "appointment:1");
    clear_kv_memory(temp.path(), "clinic").expect("clear kv memory");

    let read = client
        .get(format!("{backend}/api/cache/read"))
        .send()
        .await
        .expect("read")
        .json::<serde_json::Value>()
        .await
        .expect("read json");
    assert_eq!(read["patientName"], "Ana");
    assert_eq!(read["keys"], json!(["appointment:1"]));
    assert!(temp.path().join(".dowe/kv/clinic").exists());

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn production_dowe_cache_uses_websocket_without_local_cache() {
    let app = TempDir::new().expect("app tempdir");
    let remote = TempDir::new().expect("remote tempdir");
    create_cache_account(remote.path(), "clinic", "clinic-api", Some("secret-token"))
        .expect("account");
    let cache_server = start_cache_server(CacheServerConfig {
        root: remote.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("cache server");
    write_fixture(app.path(), 0);
    fs::write(
        app.path().join(".env"),
        format!(
            "CACHE_HOST={}\nCACHE_PORT={}\nCACHE_USER=clinic-api\nCACHE_PASSWORD=secret-token\nCACHE_DATABASE=clinic\n",
            cache_server.addr.ip(),
            cache_server.addr.port()
        ),
    )
    .expect("env");
    fs::write(
        app.path().join(".env.example"),
        "CACHE_HOST=\nCACHE_PORT=\nCACHE_USER=\nCACHE_PASSWORD=\nCACHE_DATABASE=\n",
    )
    .expect("env contract");
    fs::write(
        app.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/cache"
      handler
        cache appCache provider:"dowe" host:env.CACHE_HOST port:env.CACHE_PORT account:env.CACHE_USER secret:env.CACHE_PASSWORD name:env.CACHE_DATABASE
        kv saved conn:appCache.set key:"appointment:1" value:{ patientName:"Ana" }
        kv value conn:appCache.get key:"appointment:1" required:true
        kv keys conn:appCache.keys prefix:"appointment:"
        return json:{ ok:saved.ok patientName:value.patientName keys:keys }"#,
    )
    .expect("server");
    let project = compile_dev(app.path()).expect("project");
    let server = start_production(project, "127.0.0.1:0".parse().expect("addr"))
        .await
        .expect("server");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", server.addr);

    let response = client
        .get(format!("{backend}/api/cache"))
        .send()
        .await
        .expect("cache")
        .json::<serde_json::Value>()
        .await
        .expect("json");

    assert_eq!(response["ok"], true);
    assert_eq!(response["patientName"], "Ana");
    assert_eq!(response["keys"], json!(["appointment:1"]));
    assert!(!app.path().join(".dowe/kv/clinic").exists());
    assert!(remote.path().join(".dowe/kv/clinic").exists());

    drop(client);
    server.shutdown().await.expect("shutdown");
    cache_server.shutdown().await.expect("cache shutdown");
}

#[tokio::test]
async fn production_serves_dowe_database_handlers_over_websocket() {
    let app = TempDir::new().expect("app tempdir");
    let remote = TempDir::new().expect("remote tempdir");
    create_account(remote.path(), "clinic", "clinic-api", Some("secret-token")).expect("account");
    let store_server = start_database_service(DatabaseServiceConfig {
        root: remote.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("store server");
    write_fixture(app.path(), 0);
    fs::write(
        app.path().join(".env"),
        format!(
            "DB_HOST=127.0.0.1\nDB_PORT={}\nDB_TOKEN=secret-token\n",
            store_server.addr.port()
        ),
    )
    .expect("env");
    fs::write(
        app.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/appointments"
      handler
        database db provider:"dowe" host:env.DB_HOST port:env.DB_PORT account:"clinic-api" secret:env.DB_TOKEN name:"clinic"
        query created conn:db.tx
          query appointment conn:db.insert table:"appointments" value:{ patientName:"Ana" }
          query outboxEntry conn:db.insert table:"outbox" value:{ event:"appointment.created" }
          commit value:appointment
        query appointments conn:db.list table:"appointments"
        query outbox conn:db.list table:"outbox"
        return json:{ ok:true data:appointments outbox:outbox created:created.patientName }"#,
    )
    .expect("server");
    let project = compile_dev(app.path()).expect("project");
    let server = start_production(project, SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("server");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", server.addr);

    let response = client
        .get(format!("{backend}/api/appointments"))
        .send()
        .await
        .expect("appointments")
        .json::<serde_json::Value>()
        .await
        .expect("json");

    assert_eq!(response["ok"], true);
    assert_eq!(response["created"], "Ana");
    assert_eq!(response["data"][0]["patientName"], "Ana");
    assert_eq!(response["outbox"][0]["event"], "appointment.created");
    assert!(!app.path().join(".dowe/db/clinic").exists());
    assert!(remote.path().join(".dowe/db/clinic").exists());

    server.shutdown().await.expect("shutdown");
    store_server.shutdown().await.expect("store shutdown");
}

#[tokio::test]
async fn production_fails_before_listening_on_database_authentication_error() {
    let app = TempDir::new().expect("app tempdir");
    let remote = TempDir::new().expect("remote tempdir");
    create_account(remote.path(), "clinic", "clinic-api", Some("correct-token")).expect("account");
    let store_server = start_database_service(DatabaseServiceConfig {
        root: remote.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("store server");
    write_fixture(app.path(), 0);
    fs::write(
        app.path().join(".env"),
        format!(
            "DB_HOST=127.0.0.1\nDB_PORT={}\nDB_TOKEN=wrong-token\n",
            store_server.addr.port()
        ),
    )
    .expect("env");
    fs::write(
        app.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/appointments"
      handler
        database db provider:"dowe" host:env.DB_HOST port:env.DB_PORT account:"clinic-api" secret:env.DB_TOKEN name:"clinic"
        query created conn:db.insert table:"appointments" value:{ patientName:"Ana" }
        return json:created"#,
    )
    .expect("server");
    let project = compile_dev(app.path()).expect("project");
    let error = match start_production(project, SocketAddr::from(([127, 0, 0, 1], 0))).await {
        Ok(server) => {
            server.shutdown().await.expect("shutdown");
            panic!("authentication must fail")
        }
        Err(error) => error,
    };

    assert!(error.to_string().contains("Authentication"));
    assert!(!app.path().join(".dowe/db/clinic").exists());

    store_server.shutdown().await.expect("store shutdown");
}

#[tokio::test]
async fn serves_store_backed_blog_crud_endpoints() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    write_blog_server_fixture(temp.path());
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev(project).await.expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let created = client
        .post(format!("{backend}/api/blogs"))
        .json(&json!({"title":"First","content":"Body"}))
        .send()
        .await
        .expect("create");
    assert_eq!(created.status(), reqwest::StatusCode::CREATED);
    let created = created
        .json::<serde_json::Value>()
        .await
        .expect("created json");
    assert_eq!(created["ok"], true);
    let blogs = created["data"].as_array().expect("created data");
    assert_eq!(blogs.len(), 1);
    let id = blogs[0]["id"].as_str().expect("blog id").to_string();
    assert_eq!(blogs[0]["title"], "First");

    let missing_required = client
        .post(format!("{backend}/api/blogs"))
        .json(&json!({"title":"Missing content"}))
        .send()
        .await
        .expect("missing required");
    assert_eq!(missing_required.status(), reqwest::StatusCode::BAD_REQUEST);

    let wrong_type = client
        .post(format!("{backend}/api/blogs"))
        .json(&json!({"title":"Wrong","content":42}))
        .send()
        .await
        .expect("wrong type");
    assert_eq!(wrong_type.status(), reqwest::StatusCode::BAD_REQUEST);

    let read = client
        .get(format!("{backend}/api/blogs/{id}"))
        .send()
        .await
        .expect("read")
        .json::<serde_json::Value>()
        .await
        .expect("read json");
    assert_eq!(read["data"]["content"], "Body");

    let updated = client
        .patch(format!("{backend}/api/blogs/{id}"))
        .json(&json!({"title":"Updated"}))
        .send()
        .await
        .expect("update")
        .json::<serde_json::Value>()
        .await
        .expect("updated json");
    assert_eq!(updated["data"][0]["title"], "Updated");

    let deleted = client
        .delete(format!("{backend}/api/blogs/{id}"))
        .send()
        .await
        .expect("delete")
        .json::<serde_json::Value>()
        .await
        .expect("delete json");
    assert_eq!(deleted["data"].as_array().expect("deleted data").len(), 0);
    assert!(temp.path().join(".dowe/db/app/blogs").exists());

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn accepts_blog_form_shape_from_generated_view() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    write_blog_server_fixture(temp.path());
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev(project).await.expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let created = client
        .post(format!("{backend}/api/blogs"))
        .json(&json!({"id":null,"title":"Frontend","content":"Body","admin":true}))
        .send()
        .await
        .expect("create");
    assert_eq!(created.status(), reqwest::StatusCode::CREATED);
    let created = created
        .json::<serde_json::Value>()
        .await
        .expect("created json");
    let blogs = created["data"].as_array().expect("created data");
    assert_eq!(blogs.len(), 1);
    let id = blogs[0]["id"].as_str().expect("blog id").to_string();
    assert_ne!(id, "");
    assert_eq!(blogs[0]["title"], "Frontend");
    assert!(blogs[0].get("admin").is_none());

    let mut form_body = blogs[0].clone();
    form_body["title"] = json!("Frontend edited");
    let updated = client
        .patch(format!("{backend}/api/blogs/{id}"))
        .json(&form_body)
        .send()
        .await
        .expect("update");
    assert_eq!(updated.status(), reqwest::StatusCode::OK);
    let updated = updated
        .json::<serde_json::Value>()
        .await
        .expect("updated json");
    assert_eq!(updated["data"][0]["id"], id);
    assert_eq!(updated["data"][0]["title"], "Frontend edited");

    let rejected = client
        .patch(format!("{backend}/api/blogs/{id}"))
        .json(&json!({"id":"different","title":"Rejected"}))
        .send()
        .await
        .expect("rejected");
    assert_eq!(rejected.status(), reqwest::StatusCode::BAD_REQUEST);

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_cors_preflight_and_actual_blog_responses() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    write_blog_server_fixture_with_cors(
        temp.path(),
        r#"cors target:"server" origins:["http://127.0.0.1:56035"] methods:["GET","POST","PATCH","DELETE"] headers:["Content-Type"] exposeHeaders:["X-Request-Id"] credentials:false maxAge:600"#,
    );
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev(project).await.expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let preflight = client
        .request(reqwest::Method::OPTIONS, format!("{backend}/api/blogs"))
        .header("Origin", "http://127.0.0.1:56035")
        .header("Access-Control-Request-Method", "POST")
        .header("Access-Control-Request-Headers", "Content-Type")
        .send()
        .await
        .expect("preflight");
    assert_eq!(preflight.status(), reqwest::StatusCode::NO_CONTENT);
    assert_eq!(
        preflight
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("http://127.0.0.1:56035")
    );
    assert!(
        preflight
            .headers()
            .get("access-control-allow-methods")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .contains("POST")
    );
    assert_eq!(
        preflight
            .headers()
            .get("access-control-allow-headers")
            .and_then(|value| value.to_str().ok()),
        Some("Content-Type")
    );
    assert_eq!(
        preflight
            .headers()
            .get("access-control-max-age")
            .and_then(|value| value.to_str().ok()),
        Some("600")
    );
    assert_eq!(
        preflight
            .headers()
            .get("vary")
            .and_then(|value| value.to_str().ok()),
        Some("Origin")
    );

    let created = client
        .post(format!("{backend}/api/blogs"))
        .header("Origin", "http://127.0.0.1:56035")
        .json(&json!({"title":"Cors","content":"Body"}))
        .send()
        .await
        .expect("create");
    assert_eq!(created.status(), reqwest::StatusCode::CREATED);
    assert_eq!(
        created
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("http://127.0.0.1:56035")
    );
    assert_eq!(
        created
            .headers()
            .get("access-control-expose-headers")
            .and_then(|value| value.to_str().ok()),
        Some("X-Request-Id")
    );

    let no_origin = client
        .get(format!("{backend}/api/blogs"))
        .send()
        .await
        .expect("no origin");
    assert!(
        no_origin
            .headers()
            .get("access-control-allow-origin")
            .is_none()
    );

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn rejects_cors_preflight_for_disallowed_inputs() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    write_blog_server_fixture_with_cors(
        temp.path(),
        r#"cors target:"server" origins:["http://127.0.0.1:56035"] methods:["GET","POST"] headers:["Content-Type"]"#,
    );
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev(project).await.expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let bad_origin = client
        .request(reqwest::Method::OPTIONS, format!("{backend}/api/blogs"))
        .header("Origin", "http://127.0.0.1:56036")
        .header("Access-Control-Request-Method", "GET")
        .send()
        .await
        .expect("bad origin");
    assert_eq!(bad_origin.status(), reqwest::StatusCode::FORBIDDEN);
    assert!(
        bad_origin
            .headers()
            .get("access-control-allow-origin")
            .is_none()
    );

    let bad_method = client
        .request(reqwest::Method::OPTIONS, format!("{backend}/api/blogs"))
        .header("Origin", "http://127.0.0.1:56035")
        .header("Access-Control-Request-Method", "DELETE")
        .send()
        .await
        .expect("bad method");
    assert_eq!(bad_method.status(), reqwest::StatusCode::METHOD_NOT_ALLOWED);

    let bad_header = client
        .request(reqwest::Method::OPTIONS, format!("{backend}/api/blogs"))
        .header("Origin", "http://127.0.0.1:56035")
        .header("Access-Control-Request-Method", "POST")
        .header("Access-Control-Request-Headers", "Authorization")
        .send()
        .await
        .expect("bad header");
    assert_eq!(bad_header.status(), reqwest::StatusCode::FORBIDDEN);

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn permits_managed_dev_origin_when_configured() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    cors target:"server" devOrigins:true headers:["Content-Type"]
    route "/api/status"
      response text:"OK""#,
    )
    .expect("server");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev(project).await.expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let views_origin = format!("http://{}", servers.views_addr.expect("views addr"));

    let allowed = client
        .request(reqwest::Method::OPTIONS, format!("{backend}/api/status"))
        .header("Origin", views_origin.as_str())
        .header("Access-Control-Request-Method", "GET")
        .send()
        .await
        .expect("allowed");
    assert_eq!(allowed.status(), reqwest::StatusCode::NO_CONTENT);
    assert_eq!(
        allowed
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some(views_origin.as_str())
    );

    let external = client
        .request(reqwest::Method::OPTIONS, format!("{backend}/api/status"))
        .header("Origin", "http://127.0.0.1:1")
        .header("Access-Control-Request-Method", "GET")
        .send()
        .await
        .expect("external");
    assert_eq!(external.status(), reqwest::StatusCode::FORBIDDEN);

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn cors_preflight_does_not_execute_handlers() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/users/create"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"db1"
        query created conn:db.insert table:"users" value:{ name:"Ana" roleId:"admin" }
        return json:created"#,
    )
    .expect("server");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    cors target:"server" origins:["http://127.0.0.1:56035"] methods:["GET"] headers:["Content-Type"]
    route "/api/users/create"
      method GET
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"db1"
        query created conn:db.insert table:"users" value:{ name:"Ana" roleId:"admin" }
        return json:created"#,
    )
    .expect("server");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev(project).await.expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let preflight = client
        .request(
            reqwest::Method::OPTIONS,
            format!("{backend}/api/users/create"),
        )
        .header("Origin", "http://127.0.0.1:56035")
        .header("Access-Control-Request-Method", "GET")
        .send()
        .await
        .expect("preflight");
    assert_eq!(preflight.status(), reqwest::StatusCode::NO_CONTENT);
    assert!(!temp.path().join(".dowe/db/db1/users").exists());

    servers.shutdown().await.expect("shutdown");
}

#[derive(Clone, Debug)]
struct MockOpenRouterRequest {
    authorization: Option<String>,
    body: serde_json::Value,
}

#[derive(Clone, Debug)]
struct MockExternalRequest {
    accept: Option<String>,
    api_key: Option<String>,
}

struct MockExternalApi {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<MockExternalRequest>>>,
    shutdown: oneshot::Sender<()>,
    handle: JoinHandle<()>,
}

impl MockExternalApi {
    async fn start() -> Self {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let router = Router::new()
            .route("/v1/products", get(mock_external_products))
            .route("/redirect", get(mock_external_redirect))
            .route("/slow", get(mock_external_slow))
            .with_state(requests.clone());
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("mock listener");
        let addr = listener.local_addr().expect("mock addr");
        let (shutdown, receiver) = oneshot::channel();
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = receiver.await;
                })
                .await;
        });
        Self {
            addr,
            requests,
            shutdown,
            handle,
        }
    }

    async fn requests(&self) -> Vec<MockExternalRequest> {
        self.requests.lock().await.clone()
    }

    async fn shutdown(self) {
        let _ = self.shutdown.send(());
        let _ = self.handle.await;
    }
}

async fn mock_external_products(
    State(requests): State<Arc<Mutex<Vec<MockExternalRequest>>>>,
    headers: HeaderMap,
) -> Response {
    let accept = headers
        .get("accept")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let api_key = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    requests
        .lock()
        .await
        .push(MockExternalRequest { accept, api_key });
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json; charset=utf-8")],
        json!({"items":[{"name":"Dowe Kit"}]}).to_string(),
    )
        .into_response()
}

async fn mock_external_redirect() -> Response {
    (StatusCode::FOUND, [(LOCATION, "/v1/products")], "").into_response()
}

async fn mock_external_slow() -> Response {
    sleep(Duration::from_millis(50)).await;
    (StatusCode::OK, "slow").into_response()
}

struct MockOpenRouter {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<MockOpenRouterRequest>>>,
    shutdown: oneshot::Sender<()>,
    handle: JoinHandle<()>,
}

impl MockOpenRouter {
    async fn start() -> Self {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let router = Router::new()
            .route("/api/v1/chat/completions", post(mock_openrouter_chat))
            .with_state(requests.clone());
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("mock listener");
        let addr = listener.local_addr().expect("mock addr");
        let (shutdown, receiver) = oneshot::channel();
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = receiver.await;
                })
                .await;
        });
        Self {
            addr,
            requests,
            shutdown,
            handle,
        }
    }

    async fn requests(&self) -> Vec<MockOpenRouterRequest> {
        self.requests.lock().await.clone()
    }

    async fn shutdown(self) {
        let _ = self.shutdown.send(());
        let _ = self.handle.await;
    }
}

async fn mock_openrouter_chat(
    State(requests): State<Arc<Mutex<Vec<MockOpenRouterRequest>>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let value = serde_json::from_slice::<serde_json::Value>(&body).expect("mock request json");
    let authorization = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    requests.lock().await.push(MockOpenRouterRequest {
        authorization,
        body: value.clone(),
    });
    if value
        .get("stream")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/event-stream")],
            "data: {\"choices\":[{\"delta\":{\"content\":\"mock delta\"}}]}\ndata: [DONE]\n",
        )
            .into_response();
    }
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json; charset=utf-8")],
        json!({"choices":[{"message":{"content":"mock message"}}]}).to_string(),
    )
        .into_response()
}

async fn websocket_json(
    websocket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> serde_json::Value {
    let message = websocket
        .next()
        .await
        .expect("websocket message")
        .expect("websocket result");
    serde_json::from_str(message.to_text().expect("websocket text")).expect("websocket json")
}

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

fn write_fixture(root: &Path, port: u16) {
    fs::create_dir_all(root.join("layouts")).expect("layouts");
    fs::create_dir_all(root.join("pages")).expect("pages");
    fs::create_dir_all(root.join("routes")).expect("routes");
    fs::write(
        root.join("main.dowe"),
        format!(
            r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:{port}
    route "/api/status"
      response text:"OK"
    route "/users/:id"
      handler req
        return text:"Hello User {{req.params.id}}!"
    route "/api/posts"
      method GET
        return text:"List posts"
      method POST async req
        const body value:req.json
        return json:{{ created:true ...body }}
    websocket "/ws"
      open ws
      message ws data
      close ws code reason
      drain ws
    init
      log "Server inicializado""#
        ),
    )
    .expect("server");
    fs::write(
        root.join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"

views viewRoutes
  group path:"/" layout:AuthLayout
    route path:"" page:loginPage"#,
    )
    .expect("views");
    fs::write(
        root.join("layouts/auth.dowe"),
        r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
    )
    .expect("layout");
    fs::write(
        root.join("pages/login.dowe"),
        r#"page loginPage
  Box
    Text
      "Login""#,
    )
    .expect("page");
}

fn write_blog_server_fixture(root: &Path) {
    fs::create_dir_all(root.join("handlers")).expect("handlers");
    fs::write(
        root.join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"
import listBlogs from "@/handlers/blogs"
import createBlog from "@/handlers/blogs"
import readBlog from "@/handlers/blogs"
import updateBlog from "@/handlers/blogs"
import deleteBlog from "@/handlers/blogs"

main
  views:viewRoutes
  server port:0
    route "/api/blogs"
      method GET handler:listBlogs
      method POST handler:createBlog
    route "/api/blogs/:id"
      method GET handler:readBlog
      method PATCH handler:updateBlog
      method DELETE handler:deleteBlog"#,
    )
    .expect("server");
    fs::write(
        root.join("handlers/blogs.dowe"),
r#"type BlogInput
  title:string
  content:string

type BlogPatch
  id?:string
  title?:string
  content?:string

handler listBlogs req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }

handler createBlog
  const body:BlogInput value:req.json
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query created conn:db.insert table:"blogs" value:{ title:body.title content:body.content createdAt:now updatedAt:now } required:["title","content"]
  log created.title
  query blogs conn:db.list table:"blogs"
  return status:201 json:{ ok:true data:blogs }

handler readBlog req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query blog conn:db.read table:"blogs" where:{ id:req.params.id } required:true
  return json:{ ok:true data:blog }

handler updateBlog
  const body:BlogPatch value:req.json
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query updated conn:db.update table:"blogs" where:{ id:req.params.id } value:{ title:body.title content:body.content updatedAt:now } required:true match:{ id:req.params.id }
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }

handler deleteBlog req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query deleted conn:db.delete table:"blogs" where:{ id:req.params.id } required:true
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }"#,
    )
    .expect("blogs handler");
}

fn write_blog_server_fixture_with_cors(root: &Path, cors: &str) {
    write_blog_server_fixture(root);
    fs::write(
        root.join("main.dowe"),
        format!(
            r#"import viewRoutes from "@/routes/view"
import listBlogs from "@/handlers/blogs"
import createBlog from "@/handlers/blogs"
import readBlog from "@/handlers/blogs"
import updateBlog from "@/handlers/blogs"
import deleteBlog from "@/handlers/blogs"

main
  views:viewRoutes
  server port:0
    {cors}
    route "/api/blogs"
      method GET handler:listBlogs
      method POST handler:createBlog
    route "/api/blogs/:id"
      method GET handler:readBlog
      method PATCH handler:updateBlog
      method DELETE handler:deleteBlog"#
        ),
    )
    .expect("server");
}
