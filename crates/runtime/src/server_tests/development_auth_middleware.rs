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

