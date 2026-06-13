use crate::DevEventType;
use crate::server::{DevServerTargets, start_dev, start_dev_servers, start_production};
use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use dowe_compiler::compile_dev;
use dowe_crypto::sign_jws_hs256;
use dowe_kv::{
    KvServerConfig, clear_memory as clear_kv_memory, create_user as create_kv_user, start_kv_server,
};
use dowe_store::{StoreServerConfig, create_user, start_store_server};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn serves_backend_views_and_websocket() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("src/config.dowe"),
        r#"config
  fonts default:"inter" install:["inter"]
  env
    variable name:"BACKEND_URL" visibility:"client" required:false default:""
    variable name:"INTERNAL_TOKEN" visibility:"server" required:false"#,
    )
    .expect("config");
    fs::write(
        temp.path().join(".env"),
        "BACKEND_URL=https://runtime.example.com\nINTERNAL_TOKEN=secret\n",
    )
    .expect("dotenv");
    fs::create_dir_all(temp.path().join("src/i18n")).expect("i18n");
    fs::write(
        temp.path().join("src/i18n/en.dowe"),
        r#"translations default:true
  translation key:"home.hero.title" value:"Dowe builds systems.""#,
    )
    .expect("english");
    fs::write(
        temp.path().join("src/i18n/es.dowe"),
        r#"translations
  translation key:"home.hero.title" value:"Dowe construye sistemas.""#,
    )
    .expect("spanish");
    let project = compile_dev(temp.path()).expect("project");
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
    assert!(html.contains(r#"<link rel="stylesheet" href="/design.css">"#));
    assert!(html.contains(r#"/chunks/pages/"#));
    assert!(html.contains(r#"/router.js"#));
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
    servers.events().emit(
        DevEventType::Reload,
        Some("web"),
        None::<String>,
        vec!["src/pages/login.dowe".to_string()],
    );
    let event = dev_websocket
        .next()
        .await
        .expect("dev event")
        .expect("dev event message")
        .to_text()
        .expect("dev event text")
        .to_string();
    assert!(event.contains(r#""type":"reload""#));
    assert!(event.contains(r#""target":"web""#));
    dev_websocket.close(None).await.expect("dev close");

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_llm_http_proxy_agent_response_and_websocket_bridge() {
    let upstream = MockOpenRouter::start().await;
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("src/config.dowe"),
        r#"config
  fonts default:"inter" install:["inter"]
  env
    variable name:"OPENROUTER_API_KEY" visibility:"server" required:true
    variable name:"OPENROUTER_BASE_URL" visibility:"server" required:true"#,
    )
    .expect("config");
    fs::write(
        temp.path().join(".env"),
        format!(
            "OPENROUTER_API_KEY=test-token\nOPENROUTER_BASE_URL=http://{}\n",
            upstream.addr
        ),
    )
    .expect("env");
    fs::write(
        temp.path().join("src/main.dowe"),
        r#"main
  server port:0
    route "/api/v1/chat/completions"
      method POST async req
        let body = await req.json()
        let upstream = http.post base:env.OPENROUTER_BASE_URL path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:body mode:"proxy"
        return response proxy:upstream
    route "/api/v1/agent"
      method POST async req
        let request = await req.json()
        let chat = agent.chat request
        let upstream = http.post base:env.OPENROUTER_BASE_URL path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:chat mode:"json"
        return response agent:upstream request:request
    websocket "/api/v1/agent/ws"
      message ws
        let request = ws.json
        send ws json:{ event:"started" requestId:request.requestId requestType:request.requestType model:request.model payload:{ stream:request.stream } }
        let chat = agent.chat request
        let upstream = http.post base:env.OPENROUTER_BASE_URL path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:chat mode:"proxy"
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
async fn protects_route_with_bearer_jwt_middleware() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::create_dir_all(temp.path().join("src/middlewares")).expect("middlewares");
    fs::write(
        temp.path().join("src/config.dowe"),
        r#"config
  fonts default:"inter" install:["inter"]
  env
    variable name:"JWT_SECRET" visibility:"server" required:true"#,
    )
    .expect("config");
    fs::write(
        temp.path().join(".env"),
        "JWT_SECRET=01234567890123456789012345678901\n",
    )
    .expect("env");
    fs::write(
        temp.path().join("src/main.dowe"),
        r#"import requireBearer from "./middlewares/auth"

main
  server port:0
    route "/users/:id" middleware:[requireBearer]
      handler req
        return response text:"Hello {req.context.auth.subject}!"
    route "/api/status"
      response text:"OK""#,
    )
    .expect("server");
    fs::write(
        temp.path().join("src/middlewares/auth.dowe"),
        r#"middleware requireBearer async req
  let authorization = req.header name:"Authorization"
  let token = bearer authorization
  let verified = jwt.verify token secret:env.JWT_SECRET algorithm:"HS256"
  if verified.valid
    return continue context:{ auth:{ subject:verified.claims.sub claims:verified.claims } }
  return response status:401 json:{ ok:false error:"Unauthorized" }"#,
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
    let project = compile_dev(temp.path()).expect("project");
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

    let html = client
        .get(format!("{origin}/"))
        .send()
        .await
        .expect("html")
        .text()
        .await
        .expect("html text");
    assert!(html.contains("Layout"));
    assert!(html.contains("Login"));
    assert!(!html.contains("/_dowe/dev/client.js"));

    let dev_client = client
        .get(format!("{origin}/_dowe/dev/client.js"))
        .send()
        .await
        .expect("dev client");
    assert_eq!(dev_client.status(), reqwest::StatusCode::NOT_FOUND);

    server.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_static_desktop_entry_and_local_routes_from_one_origin() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("src/main.dowe"),
        r#"main
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
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: false,
            views: false,
            desktop: true,
        },
    )
    .await
    .expect("servers");
    assert!(servers.backend_addr.is_none());
    assert!(servers.views_addr.is_none());
    let desktop = format!("http://{}", servers.desktop_addr.expect("desktop addr"));
    let client = reqwest::Client::new();

    let html = client
        .get(format!("{desktop}/"))
        .send()
        .await
        .expect("desktop entry")
        .text()
        .await
        .expect("desktop html");
    assert!(html.contains("Layout"));
    assert!(html.contains(r#"src="router.js""#));
    assert!(html.contains(r#"src="/_dowe/dev/client.js""#));

    let status = client
        .get(format!("{desktop}/api/status"))
        .send()
        .await
        .expect("desktop status")
        .text()
        .await
        .expect("desktop status text");
    assert_eq!(status, "Desktop OK");

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn permits_managed_desktop_origin_for_backend_requests() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("src/main.dowe"),
        r#"main
  server port:0
    route "/api/status"
      response text:"OK"
  desktop
    server port:0
      route "/api/status"
        response text:"Desktop OK""#,
    )
    .expect("server");
    fs::write(
        temp.path().join("src/config.dowe"),
        r#"config
  fonts default:"inter" install:["inter"]
  server
    cors target:"server" devOrigins:true headers:["Content-Type"]"#,
    )
    .expect("config");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: true,
            views: false,
            desktop: true,
        },
    )
    .await
    .expect("servers");
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let desktop_origin = format!("http://{}", servers.desktop_addr.expect("desktop addr"));
    let allowed = reqwest::Client::new()
        .request(reqwest::Method::OPTIONS, format!("{backend}/api/status"))
        .header("Origin", desktop_origin.as_str())
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
        Some(desktop_origin.as_str())
    );

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_store_insert_and_query_endpoints() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("src/main.dowe"),
        r#"main
  server port:0
    route "/api/users/create"
      handler
        let db = store database:"db1"
        let created = db.insert table:"users" value:{ name:"Ana" roleId:"admin" }
        return response json:created
    route "/api/users"
      handler
        let db = store database:"db1"
        let rows = db.query sql:"select * from users where roleId = \"admin\""
        return response json:rows"#,
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
    assert!(temp.path().join(".dowe/store/db1/users").exists());

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_kv_handlers_with_persistent_fallback() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("src/main.dowe"),
        r#"main
  server port:0
    route "/api/cache/save"
      handler
        let db = kv database:"clinic" persist:true
        let saved = db.set key:"appointment:1" value:{ patientName:"Ana" }
        return response json:saved
    route "/api/cache/read"
      handler
        let db = kv database:"clinic" persist:true
        let value = db.get key:"appointment:1" required:true
        let keys = db.keys prefix:"appointment:"
        return response json:{ patientName:value.patientName keys:keys }"#,
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
async fn serves_remote_kv_handlers_without_local_kv_database() {
    let app = TempDir::new().expect("app tempdir");
    let remote = TempDir::new().expect("remote tempdir");
    create_kv_user(remote.path(), "clinic", "clinic-api", Some("secret-token")).expect("user");
    let kv_server = start_kv_server(KvServerConfig {
        root: remote.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("kv server");
    write_fixture(app.path(), 0);
    fs::write(
        app.path().join("src/config.dowe"),
        r#"config
  env
    variable name:"KV_HOST" visibility:"server" required:true
    variable name:"KV_TOKEN" visibility:"server" required:true"#,
    )
    .expect("config");
    fs::write(
        app.path().join(".env"),
        format!("KV_HOST=http://{}\nKV_TOKEN=secret-token\n", kv_server.addr),
    )
    .expect("env");
    fs::write(
        app.path().join("src/main.dowe"),
        r#"main
  server port:0
    route "/api/cache"
      handler
        let db = kv database:"clinic" persist:true host:env.KV_HOST user:"clinic-api" token:env.KV_TOKEN
        let saved = db.set key:"appointment:1" value:{ patientName:"Ana" }
        let value = db.get key:"appointment:1" required:true
        let keys = db.keys prefix:"appointment:"
        return response json:{ ok:saved.ok patientName:value.patientName keys:keys }"#,
    )
    .expect("server");
    let project = compile_dev(app.path()).expect("project");
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

    servers.shutdown().await.expect("shutdown");
    kv_server.shutdown().await.expect("kv shutdown");
}

#[tokio::test]
async fn serves_remote_store_handlers_without_local_store_database() {
    let app = TempDir::new().expect("app tempdir");
    let remote = TempDir::new().expect("remote tempdir");
    create_user(remote.path(), "clinic", "clinic-api", Some("secret-token")).expect("user");
    let store_server = start_store_server(StoreServerConfig {
        root: remote.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("store server");
    write_fixture(app.path(), 0);
    fs::write(
        app.path().join("src/config.dowe"),
        r#"config
  env
    variable name:"STORE_HOST" visibility:"server" required:true
    variable name:"STORE_TOKEN" visibility:"server" required:true"#,
    )
    .expect("config");
    fs::write(
        app.path().join(".env"),
        format!(
            "STORE_HOST=http://{}\nSTORE_TOKEN=secret-token\n",
            store_server.addr
        ),
    )
    .expect("env");
    fs::write(
        app.path().join("src/main.dowe"),
        r#"main
  server port:0
    route "/api/appointments"
      handler
        let db = store database:"clinic" host:env.STORE_HOST user:"clinic-api" token:env.STORE_TOKEN
        let created = db.insert table:"appointments" value:{ patientName:"Ana" }
        let appointments = db.list table:"appointments"
        return response json:{ ok:true data:appointments created:created.patientName }"#,
    )
    .expect("server");
    let project = compile_dev(app.path()).expect("project");
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
    assert!(!app.path().join(".dowe/store/clinic").exists());
    assert!(remote.path().join(".dowe/store/clinic").exists());

    servers.shutdown().await.expect("shutdown");
    store_server.shutdown().await.expect("store shutdown");
}

#[tokio::test]
async fn maps_remote_store_authentication_failures_for_direct_endpoints() {
    let app = TempDir::new().expect("app tempdir");
    let remote = TempDir::new().expect("remote tempdir");
    create_user(remote.path(), "clinic", "clinic-api", Some("correct-token")).expect("user");
    let store_server = start_store_server(StoreServerConfig {
        root: remote.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("store server");
    write_fixture(app.path(), 0);
    fs::write(
        app.path().join("src/config.dowe"),
        r#"config
  env
    variable name:"STORE_HOST" visibility:"server" required:true
    variable name:"STORE_TOKEN" visibility:"server" required:true"#,
    )
    .expect("config");
    fs::write(
        app.path().join(".env"),
        format!(
            "STORE_HOST=http://{}\nSTORE_TOKEN=wrong-token\n",
            store_server.addr
        ),
    )
    .expect("env");
    fs::write(
        app.path().join("src/main.dowe"),
        r#"main
  server port:0
    route "/api/appointments"
      handler
        let db = store database:"clinic" host:env.STORE_HOST user:"clinic-api" token:env.STORE_TOKEN
        let created = db.insert table:"appointments" value:{ patientName:"Ana" }
        return response json:created"#,
    )
    .expect("server");
    let project = compile_dev(app.path()).expect("project");
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
        .get(format!("{backend}/api/appointments"))
        .send()
        .await
        .expect("appointments");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert!(!app.path().join(".dowe/store/clinic").exists());

    servers.shutdown().await.expect("shutdown");
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
    assert!(temp.path().join(".dowe/store/app/blogs").exists());

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
    write_blog_server_fixture(temp.path());
    fs::write(
        temp.path().join("src/config.dowe"),
        r#"config
  fonts default:"inter" install:["inter"]
  server
    cors target:"server" origins:["http://127.0.0.1:56035"] methods:["GET","POST","PATCH","DELETE"] headers:["Content-Type"] exposeHeaders:["X-Request-Id"] credentials:false maxAge:600"#,
    )
    .expect("config");
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
    write_blog_server_fixture(temp.path());
    fs::write(
        temp.path().join("src/config.dowe"),
        r#"config
  fonts default:"inter" install:["inter"]
  server
    cors target:"server" origins:["http://127.0.0.1:56035"] methods:["GET","POST"] headers:["Content-Type"]"#,
    )
    .expect("config");
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
        temp.path().join("src/config.dowe"),
        r#"config
  fonts default:"inter" install:["inter"]
  server
    cors target:"server" devOrigins:true headers:["Content-Type"]"#,
    )
    .expect("config");
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
        temp.path().join("src/main.dowe"),
        r#"main
  server port:0
    route "/api/users/create"
      handler
        let db = store database:"db1"
        let created = db.insert table:"users" value:{ name:"Ana" roleId:"admin" }
        return response json:created"#,
    )
    .expect("server");
    fs::write(
        temp.path().join("src/config.dowe"),
        r#"config
  fonts default:"inter" install:["inter"]
  server
    cors target:"server" origins:["http://127.0.0.1:56035"] methods:["GET"] headers:["Content-Type"]"#,
    )
    .expect("config");
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
    assert!(!temp.path().join(".dowe/store/db1/users").exists());

    servers.shutdown().await.expect("shutdown");
}

#[derive(Clone, Debug)]
struct MockOpenRouterRequest {
    authorization: Option<String>,
    body: serde_json::Value,
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

fn write_fixture(root: &Path, port: u16) {
    fs::create_dir_all(root.join("src/layouts")).expect("layouts");
    fs::create_dir_all(root.join("src/pages")).expect("pages");
    fs::write(
        root.join("src/main.dowe"),
        format!(
            r#"main
  server port:{port}
    route "/api/status"
      response text:"OK"
    route "/users/:id"
      handler req
        return response text:"Hello User {{req.params.id}}!"
    route "/api/posts"
      method GET
        return response text:"List posts"
      method POST async req
        let body = await req.json()
        return response json:{{ created:true ...body }}
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
        root.join("src/views.dowe"),
        r#"import AuthLayout from "./layouts/auth"
import loginPage from "./pages/login"

views
  route path:"/" layout:AuthLayout
    page path:"" component:loginPage"#,
    )
    .expect("views");
    fs::write(
        root.join("src/layouts/auth.dowe"),
        r#"layout AuthLayout
  Box
    Text
      Layout
    children"#,
    )
    .expect("layout");
    fs::write(
        root.join("src/pages/login.dowe"),
        r#"page loginPage
  Box
    Text
      Login"#,
    )
    .expect("page");
}

fn write_blog_server_fixture(root: &Path) {
    fs::create_dir_all(root.join("src/handlers")).expect("handlers");
    fs::write(
        root.join("src/main.dowe"),
        r#"import listBlogs from "./handlers/blogs"
import createBlog from "./handlers/blogs"
import readBlog from "./handlers/blogs"
import updateBlog from "./handlers/blogs"
import deleteBlog from "./handlers/blogs"

main
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
        root.join("src/handlers/blogs.dowe"),
r#"type BlogInput
  title:string
  content:string

type BlogPatch
  id?:string
  title?:string
  content?:string

handler listBlogs req
  let db = store database:"app"
  let blogs = db.list table:"blogs"
  return response json:{ ok:true data:blogs }

handler createBlog async req
  let body:BlogInput = await req.json()
  let db = store database:"app"
  let created = db.insert table:"blogs" value:{ title:body.title content:body.content createdAt:now updatedAt:now } required:["title","content"]
  log created.title
  let blogs = db.list table:"blogs"
  return response status:201 json:{ ok:true data:blogs }

handler readBlog req
  let db = store database:"app"
  let blog = db.read table:"blogs" where:{ id:req.params.id } required:true
  return response json:{ ok:true data:blog }

handler updateBlog async req
  let body:BlogPatch = await req.json()
  let db = store database:"app"
  let updated = db.update table:"blogs" where:{ id:req.params.id } value:{ title:body.title content:body.content updatedAt:now } required:true match:{ id:req.params.id }
  let blogs = db.list table:"blogs"
  return response json:{ ok:true data:blogs }

handler deleteBlog req
  let db = store database:"app"
  let deleted = db.delete table:"blogs" where:{ id:req.params.id } required:true
  let blogs = db.list table:"blogs"
  return response json:{ ok:true data:blogs }"#,
    )
    .expect("blogs handler");
}
