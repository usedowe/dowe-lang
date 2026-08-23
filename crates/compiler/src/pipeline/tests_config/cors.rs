#[test]
fn compiles_backend_cors_config() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
            temp.path().join("main.dowe"),
            r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:"server" devOrigins:true origins:["http://127.0.0.1:56035"] methods:["GET","POST","PATCH","DELETE"] headers:["Content-Type"] exposeHeaders:["X-Request-Id"] credentials:false maxAge:600
    route "/api/status"
      response text:"OK""#,
        )
        .expect("config");

    let project = compile_dev(temp.path()).expect("project");

    assert!(project.backend.cors.enabled);
    assert!(project.backend.cors.allow_dev_origins);
    assert_eq!(
        project.backend.cors.origins,
        vec!["http://127.0.0.1:56035".to_string()]
    );
    assert!(project.backend.cors.methods.contains(&"GET".to_string()));
    assert!(project.backend.cors.methods.contains(&"POST".to_string()));
    assert_eq!(
        project.backend.cors.headers,
        vec!["Content-Type".to_string()]
    );
    assert_eq!(
        project.backend.cors.expose_headers,
        vec!["X-Request-Id".to_string()]
    );
    assert_eq!(project.backend.cors.max_age, Some(600));
}

#[test]
fn expands_cors_target_all_to_backend_and_desktop() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:"all" origins:["https://app.example.com"] headers:["Content-Type"]
    route "/api/status"
      response text:"OK"
  desktop
    server port:4500
      cors target:"all" origins:["https://app.example.com"] headers:["Content-Type"]
      route "/api/status"
        response text:"OK""#,
    )
    .expect("config");

    let project = compile_dev(temp.path()).expect("project");

    assert!(project.backend.cors.enabled);
    assert!(
        project
            .desktop_server
            .as_ref()
            .expect("desktop")
            .cors
            .enabled
    );
}

#[test]
fn rejects_invalid_cors_origin() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:"server" origins:["https://app.example.com/path"]
    route "/api/status"
      response text:"OK""#,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("invalid CORS origin"));
}

#[test]
fn rejects_wildcard_cors_with_credentials() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:"server" origins:["*"] credentials:true
    route "/api/status"
      response text:"OK""#,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("credentials:true"));
}

#[test]
fn rejects_duplicate_cors_policy_for_target() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:"server" origins:["https://one.example.com"]
    cors target:"server" origins:["https://two.example.com"]
    route "/api/status"
      response text:"OK""#,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("duplicate `cors` block"));
}

