use super::*;

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
