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
async fn backend_server_uses_first_available_development_port_and_public_urls() {
    let preferred = match TcpListener::bind("127.0.0.1:7754").await {
        Ok(listener) => Some(listener),
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => None,
        Err(error) => panic!("preferred port: {error}"),
    };
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 8081);
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: true,
            views: true,
            desktop: false,
        },
    )
    .await
    .expect("servers");

    let backend_addr = servers.backend_addr.expect("backend addr");
    if preferred.is_some() {
        assert!(backend_addr.port() > 7754);
    } else {
        assert!(backend_addr.port() >= 7754);
    }
    assert_ne!(backend_addr.port(), 8081);
    let views = format!("http://{}", servers.views_addr.expect("views addr"));
    let public_env = reqwest::get(format!("{views}/env.json"))
        .await
        .expect("env")
        .text()
        .await
        .expect("env text");
    assert!(public_env.contains(&format!(r#""BACKEND_URL":"http://{backend_addr}""#)));
    assert!(public_env.contains(&format!(r#""SERVER_URL":"http://{backend_addr}""#)));
    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn desktop_server_uses_first_available_development_port_and_public_url() {
    let preferred = match TcpListener::bind("127.0.0.1:7854").await {
        Ok(listener) => Some(listener),
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => None,
        Err(error) => panic!("preferred port: {error}"),
    };
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 8081);
    let mut main = fs::read_to_string(temp.path().join("main.dowe")).expect("main");
    main.push_str(
        r#"
  desktop
    server port:8181
      route "/desktop/status"
        response text:"Desktop OK""#,
    );
    fs::write(temp.path().join("main.dowe"), main).expect("main");
    let project = compile_dev(temp.path()).expect("project");
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

    let desktop_addr = servers.desktop_addr.expect("desktop addr");
    if preferred.is_some() {
        assert!(desktop_addr.port() > 7854);
    } else {
        assert!(desktop_addr.port() >= 7854);
    }
    assert_ne!(desktop_addr.port(), 8181);
    let views = format!("http://{}", servers.views_addr.expect("views addr"));
    let public_env = reqwest::get(format!("{views}/env.json"))
        .await
        .expect("env")
        .text()
        .await
        .expect("env text");
    assert!(public_env.contains(&format!(r#""BACKEND_DESKTOP_URL":"http://{desktop_addr}""#)));
    assert!(public_env.contains(&format!(r#""SERVER_DESKTOP_URL":"http://{desktop_addr}""#)));
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
