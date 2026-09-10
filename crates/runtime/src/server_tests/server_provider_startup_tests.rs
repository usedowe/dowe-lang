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
async fn serves_notification_actions_with_a_durable_intent() {
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
        r#"import requireBearer from "@/middlewares/auth"

main
  server port:0
    route "/api/notify" middleware:[requireBearer]
      handler
        notify sent category:"process" user:req.context.auth.subject title:"Ready" body:"The export finished" route:"/reports/latest" data:{ exportId:"export-1" }
        return json:sent"#,
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
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let token = sign_jws_hs256(
        &json!({"sub":"user-1","exp":4102444800u64}),
        "01234567890123456789012345678901",
    )
    .expect("token");
    unsafe {
        std::env::set_var(
            "DOWE_NOTIFICATION_JWT_SECRET",
            "01234567890123456789012345678901",
        );
    }
    let registration = reqwest::Client::new()
        .post(format!("{backend}/_dowe/notifications/installations"))
        .bearer_auth(&token)
        .json(&json!({
            "id":"web-1",
            "platform":"web",
            "provider":"web-push",
            "token":"{\"endpoint\":\"https://push.example/1\",\"keys\":{\"p256dh\":\"key\",\"auth\":\"auth\"}}"
        }))
        .send()
        .await
        .expect("registration");
    assert_eq!(registration.status(), reqwest::StatusCode::CREATED);
    let registration = registration
        .json::<serde_json::Value>()
        .await
        .expect("registration json");
    assert_eq!(registration["user"], "user-1");
    let revoked = reqwest::Client::new()
        .delete(format!("{backend}/_dowe/notifications/installations/web-1"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("revoke");
    assert_eq!(revoked.status(), reqwest::StatusCode::NO_CONTENT);
    let response = reqwest::Client::new()
        .get(format!("{backend}/api/notify"))
        .bearer_auth(token)
        .send()
        .await
        .expect("notify")
        .json::<serde_json::Value>()
        .await
        .expect("notify json");
    assert_eq!(response["ok"], true);
    assert_eq!(response["deliveries"], 0);
    assert!(temp.path().join(".dowe/notifications").exists());
    unsafe {
        std::env::remove_var("DOWE_NOTIFICATION_JWT_SECRET");
    }
    servers.shutdown().await.expect("shutdown");
}

