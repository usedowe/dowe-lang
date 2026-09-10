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

