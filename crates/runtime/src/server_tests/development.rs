use super::*;

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
    assert!(
        dashboard_text.contains(
            "border-color: transparent; background: transparent; color: var(--dowe-muted)"
        )
    );
    assert!(
        dashboard_text
            .contains(".nav button.active .nav-icon { background: transparent; color: inherit; }")
    );
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
    let design_chunk_paths = project
        .web
        .pages
        .iter()
        .flat_map(|page| page.css_chunks.iter())
        .filter(|path| path.starts_with("chunks/design/") && path.ends_with(".css"))
        .map(|path| format!("/{path}"))
        .collect::<std::collections::BTreeSet<_>>();
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
    assert!(html.contains(">Layout</p>"));
    assert!(html.contains(">Login</p>"));
    assert!(html.contains(&format!(
        r#"<link data-dowe-design rel="stylesheet" href="{design_path}">"#
    )));
    assert!(html.contains(r#"/chunks/pages/"#));
    assert!(html.contains(r#"data-dowe-router type="module" src="/router.js"#));
    assert!(html.contains(r#"/_dowe/dev/client.js"#));

    let css = client
        .get(format!("{views}{design_path}"))
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

    for path in design_chunk_paths {
        let chunk = client
            .get(format!("{views}{path}"))
            .send()
            .await
            .expect("design css chunk");
        assert_eq!(chunk.status(), reqwest::StatusCode::OK, "{path}");
        assert!(
            chunk
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.contains("text/css")),
            "{path}"
        );
    }

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
async fn serves_each_design_name_referenced_by_an_active_page() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    let mut project = compile_dev(temp.path()).expect("project");
    let design_alias = "design-hot-reload.css".to_string();
    let mut active_page = project.web.pages[0].as_ref().clone();
    active_page.design_file_name.clone_from(&design_alias);
    project.web.pages.push(Arc::new(active_page));
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
    let response = reqwest::Client::new()
        .get(format!("{views}/{design_alias}"))
        .send()
        .await
        .expect("design css");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert!(response.text().await.expect("css").contains(".card"));

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
