use super::*;

#[test]
fn generates_cloudflare_queue_binding_for_static_publication() {
    let temp = TempDir::new().expect("project");
    fs::write(
        temp.path().join("main.dowe"),
        r#"main
  server port:8080
    route "/enqueue"
      handler
        queue appQueue provider:"cloudflare" host:env.QUEUE_HOST port:env.QUEUE_PORT account:env.QUEUE_USER secret:env.QUEUE_PASSWORD vhost:env.QUEUE_VHOST
        msg sent conn:appQueue.publish queue:"notifications" payload:{ userId:"123" event:"user_created" }
        return json:{ ok:sent.ok messageId:sent.id }
"#,
    )
    .expect("source");
    fs::write(
        temp.path().join(".env.example"),
        "QUEUE_HOST=\nQUEUE_PORT=\nQUEUE_USER=\nQUEUE_PASSWORD=\nQUEUE_VHOST=\n",
    )
    .expect("environment example");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Cloudflare);
    options.name = Some("queue-worker".to_string());

    let report = deploy(options).expect("cloudflare queue deploy");
    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(report.output_dir.join("worker/wrangler.jsonc"))
            .expect("wrangler config"),
    )
    .expect("wrangler json");
    let adapter =
        fs::read_to_string(report.output_dir.join("worker/index.js")).expect("worker adapter");

    assert_eq!(config["queues"]["producers"][0]["queue"], "notifications");
    assert_eq!(
        config["queues"]["producers"][0]["binding"],
        "DOWE_QUEUE_NOTIFICATIONS"
    );
    assert!(adapter.contains("queue.send"));
    assert!(!adapter.contains("QUEUE_PASSWORD"));
}

#[test]
fn generates_vercel_queue_adapter_without_embedding_credentials() {
    let temp = TempDir::new().expect("project");
    fs::write(
        temp.path().join("main.dowe"),
        r#"main
  server port:8080
    route "/enqueue"
      handler
        queue appQueue provider:"vercel" host:env.QUEUE_HOST port:env.QUEUE_PORT account:env.QUEUE_USER secret:env.QUEUE_PASSWORD vhost:env.QUEUE_VHOST
        msg sent conn:appQueue.publish queue:"notifications" payload:{ userId:"123" event:"user_created" }
        return json:{ ok:sent.ok messageId:sent.id }
"#,
    )
    .expect("source");
    fs::write(
        temp.path().join(".env.example"),
        "QUEUE_HOST=\nQUEUE_PORT=\nQUEUE_USER=\nQUEUE_PASSWORD=\nQUEUE_VHOST=\n",
    )
    .expect("environment example");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Vercel);
    options.name = Some("queue-vercel".to_string());

    let report = deploy(options).expect("vercel queue deploy");
    let function = report
        .output_dir
        .join(".vercel/output/functions/index.func");
    let adapter = fs::read_to_string(function.join("index.js")).expect("vercel adapter");
    let wasm = fs::read(function.join("dowe-server.wasm")).expect("wasm");

    wasmparser::Validator::new()
        .validate_all(&wasm)
        .expect("valid wasm");
    assert!(adapter.contains("enqueueVercel"));
    assert!(adapter.contains("process.env"));
    assert!(!adapter.contains("QUEUE_PASSWORD"));
    assert!(!adapter.contains("user_created"));
}

#[test]
fn rejects_queue_edge_provider_mismatch_and_dynamic_payload() {
    let mismatch = TempDir::new().expect("mismatch project");
    fs::write(
        mismatch.path().join("main.dowe"),
        r#"main
  server port:8080
    route "/enqueue"
      handler
        queue appQueue provider:"vercel" host:env.QUEUE_HOST port:env.QUEUE_PORT account:env.QUEUE_USER secret:env.QUEUE_PASSWORD vhost:env.QUEUE_VHOST
        msg sent conn:appQueue.publish queue:"notifications" payload:{ ok:true }
        return json:{ ok:sent.ok messageId:sent.id }
"#,
    )
    .expect("source");
    fs::write(
        mismatch.path().join(".env.example"),
        "QUEUE_HOST=\nQUEUE_PORT=\nQUEUE_USER=\nQUEUE_PASSWORD=\nQUEUE_VHOST=\n",
    )
    .expect("environment example");
    let error = deploy(DeployOptions::new(
        mismatch.path(),
        DeployTarget::Cloudflare,
    ))
    .expect_err("mismatched provider");
    assert!(
        error
            .to_string()
            .contains("Queue connection provider does not match the deploy target")
    );

    let dynamic = TempDir::new().expect("dynamic project");
    fs::write(
        dynamic.path().join("main.dowe"),
        r#"main
  server port:8080
    route "/enqueue"
      handler
        queue appQueue provider:"cloudflare" host:env.QUEUE_HOST port:env.QUEUE_PORT account:env.QUEUE_USER secret:env.QUEUE_PASSWORD vhost:env.QUEUE_VHOST
        msg sent conn:appQueue.publish queue:"notifications" payload:{ event:req.body.event }
        return json:{ ok:sent.ok messageId:sent.id }
"#,
    )
    .expect("source");
    fs::write(
        dynamic.path().join(".env.example"),
        "QUEUE_HOST=\nQUEUE_PORT=\nQUEUE_USER=\nQUEUE_PASSWORD=\nQUEUE_VHOST=\n",
    )
    .expect("environment example");
    let error = deploy(DeployOptions::new(dynamic.path(), DeployTarget::Cloudflare))
        .expect_err("dynamic payload");
    assert!(
        error
            .to_string()
            .contains("Queue Edge payload cannot use dynamic references")
    );
}
