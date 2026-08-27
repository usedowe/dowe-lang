use super::*;

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
    let hash = Sha256::digest(b"stage-password-123")
        .iter()
        .map(|value| format!("{value:02x}"))
        .collect::<String>();
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
