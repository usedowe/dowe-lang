#[test]
fn parses_native_ipc_function_registrations_without_desktop_server() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("server")).expect("server");
    fs::write(
        temp.path().join("server/settings.dowe"),
        "fn readSettings return:\"string\"\n  return value:\"ready\"\n",
    )
    .expect("function");
    let source = "import readSettings from \"@/server/settings\"\n\nmain\n  ipc functions:[readSettings]\n";
    let file = parse_source_file(temp.path(), &temp.path().join("main.dowe"), source.to_string()).expect("source");
    let server = parse_server_source(temp.path(), &file, &EnvironmentConfig::default()).expect("server");
    assert!(server.desktop_server.is_none());
    assert_eq!(server.native_ipc.functions[0].name, "readSettings");
}

#[test]
fn registers_imported_databases_for_native_ipc() {
    let root = TempDir::new().expect("root");
    fs::write(
        root.path().join("server-config.dowe"),
        r#"database LocalDb provider:"dowe" host:"local" port:1 account:"app" secret:"secret" name:"apps" entities:[] seeders:[]"#,
    )
    .expect("database config");
    fs::create_dir_all(root.path().join("server")).expect("server");
    fs::write(
        root.path().join("server/settings.dowe"),
        "fn readSettings return:\"string\"\n  return value:\"ready\"\n",
    )
    .expect("function");
    let main_path = root.path().join("main.dowe");
    let main = parse_source_file(
        root.path(),
        &main_path,
        r#"import LocalDb from "@/server-config"
import readSettings from "@/server/settings"

main
  ipc functions:[readSettings] databases:[LocalDb]"#
            .to_string(),
    )
    .expect("main");
    let server = parse_server_source(root.path(), &main, &EnvironmentConfig::default())
        .expect("server");
    assert_eq!(server.native_ipc.databases[0].binding, "LocalDb");
    assert_eq!(server.databases[0].connection.database, "apps");
}

#[test]
fn rejects_duplicate_native_ipc_function_registrations() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("server")).expect("server");
    fs::write(
        temp.path().join("server/settings.dowe"),
        "fn readSettings return:\"string\"\n  return value:\"ready\"\n",
    )
    .expect("function");
    let source = "import readSettings from \"@/server/settings\"\n\nmain\n  ipc functions:[readSettings readSettings]\n";
    let file = parse_source_file(temp.path(), &temp.path().join("main.dowe"), source.to_string()).expect("source");
    let error = parse_server_source(temp.path(), &file, &EnvironmentConfig::default()).expect_err("duplicate");
    assert!(error.message().contains("duplicate `functions` binding"));
}

#[test]
fn parses_main_server_route() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        r#"main
  server port:8080
    route "/api/status"
      response text:"OK""#
            .to_string(),
    )
    .expect("source");

    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/api/status")
        .expect("route");

    assert_eq!(
        endpoint.endpoint.behavior,
        EndpointBehavior::StaticText("OK".to_string())
    );
}

#[test]
fn parses_server_ai_chat_statement() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        r#"main
  server port:8080
    route "/api/generate"
      handler
        ai result source:"chat" prompt:"create a login" files:["main.dowe"] model:"gemma-4-e2b"
        return json:result"#
            .to_string(),
    )
    .expect("source");

    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let action = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/api/generate")
        .expect("route")
        .endpoint
        .action;
    assert!(matches!(action.statements.first(), Some(ServerStatement::AiChat(_))));
}

#[test]
fn registers_imported_databases_for_project_operations() {
    let root = TempDir::new().expect("root");
    fs::write(
        root.path().join("server-config.dowe"),
        r#"database IconDb provider:"dowe" host:"127.0.0.1" port:4147 account:"docs" secret:"secret" name:"icons" entities:[] seeders:[]"#,
    )
    .expect("database config");
    let main_path = root.path().join("main.dowe");
    let main = parse_source_file(
        root.path(),
        &main_path,
        r#"import IconDb from "@/server-config"

main
  server port:8080
    databases:[IconDb]"#
            .to_string(),
    )
    .expect("main");

    let server = parse_server_source(root.path(), &main, &EnvironmentConfig::default())
        .expect("server");

    assert_eq!(server.backend.databases.len(), 1);
    assert_eq!(server.backend.databases[0].binding, "IconDb");
    assert_eq!(server.databases.len(), 1);
    assert_eq!(server.databases[0].connection.database, "icons");
}

#[test]
fn development_server_skips_seeder_modules_entirely() {
    let root = TempDir::new().expect("root");
    fs::create_dir_all(root.path().join("server/config")).expect("config directory");
    fs::create_dir_all(root.path().join("server/seeders")).expect("seeders directory");
    fs::write(
        root.path().join("server/config/database.dowe"),
        r#"import Bootstrap from "@/server/seeders/bootstrap"

database AppDb provider:"dowe" host:"127.0.0.1" port:4147 account:"app" secret:"secret" name:"app" entities:[] seeders:[Bootstrap]"#,
    )
    .expect("database config");
    fs::write(
        root.path().join("server/seeders/bootstrap.dowe"),
        "seeder Bootstrap\n  insert entity:Missing value:{}",
    )
    .expect("seeder source");
    let main_path = root.path().join("main.dowe");
    let main = parse_source_file(
        root.path(),
        &main_path,
        r#"import AppDb from "@/server/config/database"

main
  server port:8080
    databases:[AppDb]"#
            .to_string(),
    )
    .expect("main");

    let server = parse_server_source_without_seeders(
        root.path(),
        &main,
        &EnvironmentConfig::default(),
    )
    .expect("development server");
    assert!(server.databases[0].connection.seeders.is_empty());
    assert!(
        !server
            .inspector
            .nodes
            .iter()
            .any(|node| node.kind == "seeder")
    );
}

#[test]
fn rejects_unimported_database_registration() {
    let root = TempDir::new().expect("root");
    let main_path = root.path().join("main.dowe");
    let main = parse_source_file(
        root.path(),
        &main_path,
        "main\n  server port:8080\n    databases:[MissingDb]".to_string(),
    )
    .expect("main");

    let error = parse_server_source(root.path(), &main, &EnvironmentConfig::default())
        .expect_err("missing database import");

    assert!(error.to_string().contains("unknown Database handle import `MissingDb`"));
}

#[test]
fn parses_acme_tls_with_managed_kv_domains() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        r#"main
  server port:443
    tls:
      mode:"acme"
      domains:["example.com", "www.example.com"]
      email:"admin@example.com"
      staging:false
      domainsFrom:{ kv:"domains" key:"tls" }
      refreshSeconds:90"#
            .to_string(),
    )
    .expect("source");

    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let tls = server.backend.tls.expect("tls");

    assert_eq!(tls.mode, TlsMode::Acme);
    assert_eq!(tls.domains, ["example.com", "www.example.com"]);
    assert_eq!(tls.email.as_deref(), Some("admin@example.com"));
    assert!(!tls.staging);
    assert_eq!(tls.refresh_seconds, 90);
    assert_eq!(
        tls.domains_from,
        Some(TlsDomainsSource::Kv {
            database: "domains".to_string(),
            key: "tls".to_string(),
        })
    );
}

#[test]
fn parses_local_tls_and_database_domain_source() {
    let local = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            "main\n  server port:8443\n    tls mode:\"local\" domains:[\"localhost\", \"app.localhost\"]\n"
                .to_string(),
        )
        .expect("source");
    let local =
        parse_server_file(Path::new("/project/main.dowe"), &local.nodes).expect("local server");
    assert_eq!(local.backend.tls.expect("tls").mode, TlsMode::Local);

    let database = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            "main\n  server port:443\n    tls mode:\"acme\" email:\"admin@example.com\" domainsFrom:{ db:\"admin\" table:\"domains\" field:\"hostname\" }\n"
                .to_string(),
        )
        .expect("source");
    let database = parse_server_file(Path::new("/project/main.dowe"), &database.nodes)
        .expect("database server");
    assert!(matches!(
        database.backend.tls.expect("tls").domains_from,
        Some(TlsDomainsSource::Database { .. })
    ));
}

#[test]
fn parses_tls_endpoint_domains_and_http_redirect_port() {
    let root = tempfile::tempdir().expect("root");
    let source = r#"main
  server port:443
    tls:
      mode:"acme"
      email:"admin@example.com"
      staging:false
      domainsFrom:{ endpoint:env.CLOUD_API_URL path:"/v1/domains" bearer:env.CLOUD_TOKEN timeoutMs:2500 }
      refreshSeconds:30
      httpPort:80"#;
    let path = root.path().join("main.dowe");
    let file = parse_source_file(root.path(), &path, source.to_string()).expect("source");
    let environment = EnvironmentConfig {
        variables: vec![
            EnvironmentVariable {
                name: "CLOUD_API_URL".to_string(),
                visibility: EnvironmentVisibility::Server,
                resolved_source: EnvironmentValueSource::Missing,
                resolved_value: None,
            },
            EnvironmentVariable {
                name: "CLOUD_TOKEN".to_string(),
                visibility: EnvironmentVisibility::Server,
                resolved_source: EnvironmentValueSource::Missing,
                resolved_value: None,
            },
        ],
    };
    let server = parse_server_source(root.path(), &file, &environment).expect("server");
    let tls = server.backend.tls.expect("tls");

    assert_eq!(tls.http_port, Some(80));
    assert!(matches!(
        tls.domains_from,
        Some(TlsDomainsSource::Endpoint {
            base: HttpConnectionValue::Environment(base),
            path,
            bearer: ServerSecret::Environment(bearer),
            timeout_ms: 2500,
        }) if base == "CLOUD_API_URL" && path == "/v1/domains" && bearer == "CLOUD_TOKEN"
    ));
}

