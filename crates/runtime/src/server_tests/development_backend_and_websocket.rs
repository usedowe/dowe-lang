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
    assert!(client_script.contains(
        "message.type===\"reload\"&&(message.target===\"web\"||message.target===\"desktop\")){queueHotUpdate"
    ));
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
    let backend_addr = servers.backend_addr.expect("backend addr");
    assert!(public_env.contains(&format!(r#""BACKEND_URL":"http://{backend_addr}""#)));
    assert!(public_env.contains(&format!(r#""SERVER_URL":"http://{backend_addr}""#)));
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

    let (mut websocket, _) = connect_async(format!("ws://{backend_addr}/ws"))
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

