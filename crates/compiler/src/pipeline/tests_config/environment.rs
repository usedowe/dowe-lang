#[test]
fn isolates_development_and_deploy_environment_files() {
    let temp = TempDir::new().expect("tempdir");
    write_blog_fixture(temp.path());
    fs::write(temp.path().join(".env.example"), "BACKEND_URL=\n").expect("env example");
    fs::write(
        temp.path().join(".env"),
        "BACKEND_URL=https://dev.example.com\n",
    )
    .expect("development env");
    fs::write(
        temp.path().join(".env.live"),
        "BACKEND_URL=https://live.example.com\n",
    )
    .expect("live env");
    fs::write(
        temp.path().join(".env.stage"),
        "BACKEND_URL=https://stage.example.com\n",
    )
    .expect("stage env");
    fs::write(
        temp.path().join(".env.uat"),
        "BACKEND_URL=https://uat.example.com\n",
    )
    .expect("uat env");

    let development = compile_dev(temp.path()).expect("development project");
    let live = compile_for_environment(temp.path(), CompileEnvironment::Live).expect("live");
    let stage = compile_for_environment(temp.path(), CompileEnvironment::Stage).expect("stage");
    let uat = compile_for_environment(temp.path(), CompileEnvironment::Uat).expect("uat");

    assert_eq!(
        development
            .environment_config
            .variable("BACKEND_URL")
            .and_then(|variable| variable.resolved_value.as_deref()),
        Some("https://dev.example.com")
    );
    assert_eq!(
        live
            .environment_config
            .variable("BACKEND_URL")
            .and_then(|variable| variable.resolved_value.as_deref()),
        Some("https://live.example.com")
    );
    assert_eq!(
        stage
            .environment_config
            .variable("BACKEND_URL")
            .and_then(|variable| variable.resolved_value.as_deref()),
        Some("https://stage.example.com")
    );
    assert_eq!(
        uat.environment_config
            .variable("BACKEND_URL")
            .and_then(|variable| variable.resolved_value.as_deref()),
        Some("https://uat.example.com")
    );
}

#[test]
fn rejects_view_environment_references_in_server_only_http_configuration() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join(".env.example"),
        "SHARED_URL=\n",
    )
    .expect("env example");
    fs::write(
        temp.path().join("pages/login.dowe"),
        r#"page loginPage
  fn load
    request status method:"GET" route:"/status" base:env.SHARED_URL
  Text
    "Login""#,
    )
    .expect("page");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    route "/api/proxy"
      handler req
        http upstream method:"get" base:env.SHARED_URL path:"/status" mode:"json"
        return json:upstream.json"#,
    )
    .expect("main");

    let error = compile_dev(temp.path()).expect_err("public server HTTP base");

    assert!(error.to_string().contains("SHARED_URL"));
    assert!(error.to_string().contains("must be server-only"));
}

#[test]
fn resolves_environment_from_operating_system_before_dotenv() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join(".env.example"),
        "DOWE_TEST_BACKEND_URL=\n",
    )
    .expect("config");
    fs::write(
        temp.path().join(".env"),
        "DOWE_TEST_BACKEND_URL=https://local.example.com\n",
    )
    .expect("local env");
    unsafe {
        std::env::set_var("DOWE_TEST_BACKEND_URL", "https://os.example.com");
    }

    let project = compile_dev(temp.path()).expect("project");

    unsafe {
        std::env::remove_var("DOWE_TEST_BACKEND_URL");
    }
    let variable = project
        .environment_config
        .variable("DOWE_TEST_BACKEND_URL")
        .expect("variable");
    assert_eq!(variable.resolved_source, EnvironmentValueSource::Os);
    assert_eq!(
        variable.resolved_value.as_deref(),
        Some("https://os.example.com")
    );
}

#[test]
fn rejects_process_environment_names_without_dotenv_declaration() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("pages/login.dowe"),
        r#"page loginPage
  fn load
    request status method:"GET" route:"/status" base:env.DOWE_OS_ONLY_URL
  Text
    "Login""#,
    )
    .expect("page");
    unsafe {
        std::env::set_var("DOWE_OS_ONLY_URL", "https://os.example.com");
    }

    let error = compile_dev(temp.path()).expect_err("undeclared process variable");

    unsafe {
        std::env::remove_var("DOWE_OS_ONLY_URL");
    }
    assert!(error.to_string().contains("DOWE_OS_ONLY_URL"));
    assert!(error.to_string().contains("unknown environment variable"));
}

#[test]
fn ignores_env_example_values_when_local_and_process_values_are_absent() {
    let temp = TempDir::new().expect("tempdir");
    write_blog_fixture(temp.path());
    fs::write(
        temp.path().join(".env.example"),
        "BACKEND_URL=https://placeholder.example\nINTERNAL_TOKEN=replace-me\n",
    )
    .expect("env example");
    fs::remove_file(temp.path().join(".env")).expect("remove local env");

    let project = compile_dev(temp.path()).expect("project");
    let backend = project
        .environment_config
        .variable("BACKEND_URL")
        .expect("backend url");

    assert_eq!(backend.visibility, EnvironmentVisibility::Client);
    assert_eq!(backend.resolved_source, EnvironmentValueSource::Missing);
    assert_eq!(backend.resolved_value, None);
    assert_eq!(project.environment_config.client_json(), r#"{"BACKEND_URL":""}"#);
}

#[test]
fn rejects_invalid_request_base_url() {
    let temp = TempDir::new().expect("tempdir");
    write_blog_fixture(temp.path());
    fs::write(
        temp.path().join(".env"),
        "BACKEND_URL=file:///tmp/api\nINTERNAL_TOKEN=\n",
    )
    .expect("env");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("http or https URL"));
    assert!(error.to_string().contains("BACKEND_URL"));
}

#[test]
fn rejects_unknown_environment_variable() {
    let temp = TempDir::new().expect("tempdir");
    write_blog_fixture(temp.path());
    fs::write(temp.path().join(".env.example"), "INTERNAL_TOKEN=\n").expect("env example");
    fs::write(temp.path().join(".env"), "INTERNAL_TOKEN=secret\n").expect("env");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("BACKEND_URL"));
    assert!(error.to_string().contains("unknown environment variable"));
}

#[test]
fn rejects_duplicate_dotenv_keys() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join(".env"),
        "BACKEND_URL=\nBACKEND_URL=https://api.example.com\n",
    )
    .expect("env");

    let error = compile_dev(temp.path()).expect_err("duplicate dotenv key");

    assert!(error.to_string().contains(".env"));
    assert!(error.to_string().contains("duplicate environment variable `BACKEND_URL`"));
}

