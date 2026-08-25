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

#[test]
fn rejects_invalid_tls_contracts() {
    for (tls, message) in [
        (
            "tls mode:\"acme\" domains:[\"localhost\"] email:\"admin@example.com\"",
            "invalid public ACME domain",
        ),
        (
            "tls mode:\"acme\" domains:[\"192.0.2.1\"] email:\"admin@example.com\"",
            "invalid public ACME domain",
        ),
        (
            "tls mode:\"local\" domains:[\"example.com\"]",
            "local TLS does not support public domain",
        ),
        (
            "tls mode:\"acme\" domains:[\"example.com\"]",
            "requires a valid `email`",
        ),
        (
            "tls mode:\"acme\" domains:[\"example.com\"] email:\"admin@example.com\" cache:\"../tls\"",
            "must stay inside `.dowe`",
        ),
        (
            "tls mode:\"acme\" email:\"admin@example.com\" domainsFrom:{ kv:\"domains\" table:\"domains\" }",
            "must be a KV, Database, or authenticated endpoint source",
        ),
        (
            "tls mode:\"local\" domains:[\"localhost\"] httpPort:443",
            "different from `server.port`",
        ),
    ] {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            format!("main\n  server port:443\n    {tls}\n"),
        )
        .expect("source");
        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("invalid tls");
        assert!(error.to_string().contains(message), "{error}");
    }
}

#[test]
fn parses_protocol_transports_and_rtp_pool() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        r#"main
  server port:8080
    udp name:"sip-udp" bind:"0.0.0.0" port:5060
      packet pkt
        log "udp" pkt.addr pkt.text pkt.bytes
    tcp name:"sip-tcp" bind:"0.0.0.0" port:5060
      connection conn
        log "tcp" conn.addr conn.text conn.bytes
    rtp bind:"0.0.0.0" min:40000 max:40100
    route "/api/status"
      response text:"OK""#
            .to_string(),
    )
    .expect("source");

    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");

    assert_eq!(server.backend.transports.len(), 2);
    assert_eq!(server.backend.transports[0].name, "sip-udp");
    assert_eq!(
        server.backend.transports[0].protocol,
        ServerTransportProtocol::Udp
    );
    assert_eq!(server.backend.transports[0].binding, "pkt");
    assert_eq!(
        server.backend.transports[1].protocol,
        ServerTransportProtocol::Tcp
    );
    assert_eq!(server.backend.transports[1].binding, "conn");
    let rtp = server.backend.rtp.expect("rtp");
    assert_eq!(rtp.bind, "0.0.0.0");
    assert!(rtp.contains(40000));
    assert!(rtp.contains(40100));
    assert!(!rtp.contains(40101));
}

#[test]
fn parses_server_model_declarations() {
    let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    model name:"voice-vad" kind:"vad.silero" engine:"candle" format:"onnx" source:"assets/silero_vad.onnx" sampleRates:[8000,16000]
    route "/api/status"
      response text:"OK""#
                .to_string(),
        )
        .expect("source");

    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let model = server.backend.models.first().expect("model");

    assert_eq!(model.name, "voice-vad");
    assert_eq!(model.kind, ServerModelKind::VadSilero);
    assert_eq!(model.engine, ServerModelEngine::Candle);
    assert_eq!(model.format, ServerModelFormat::Onnx);
    assert_eq!(model.sample_rates, vec![8_000, 16_000]);
}

#[test]
fn parses_media_proxy_primitives() {
    let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/dash/:name/*segment"
      method GET async req
        request query source:"query"
        request raw source:"rawQuery"
        request range source:"header" name:"Range"
        request session source:"cookie" name:"session"
        http upstream method:"get" base:"https://media.example" path:"/segment.m4s" mode:"bytes" headers:[{ name:"Accept" value:"*/*" }]
        crypto decrypted encryption:"aesCtr" data:upstream key:"00000000000000000000000000000000" iv:"00000000000000000000000000000000"
        crypto cenc encryption:"cencAesCtr" data:decrypted key:"00000000000000000000000000000000" iv:"0000000000000000" subsamples:[{ clear:5 encrypted:10 }]
        spawn ffmpeg command:"ffmpeg" args:["-version"] timeoutMs:1000 maxOutputBytes:4096
        return bytes:cenc contentType:"video/mp4" headers:[{ name:"Cache-Control" value:"no-store" }] cookies:[{ name:"session" value:session path:"/" httpOnly:true sameSite:"Lax" maxAge:60 }]"#
                .to_string(),
        )
        .expect("source");

    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/dash/news/video/1.m4s")
        .expect("route");

    assert!(matches!(
        endpoint.endpoint.behavior,
        EndpointBehavior::HttpBytes(_)
    ));
    assert!(matches!(
        endpoint.endpoint.action.statements[0],
        ServerStatement::RequestQuery { .. }
    ));
    assert!(matches!(
        endpoint.endpoint.action.statements[4],
        ServerStatement::Http(_)
    ));
    assert!(matches!(
        endpoint.endpoint.action.statements[5],
        ServerStatement::CryptoAesCtr(_)
    ));
    assert!(matches!(
        endpoint.endpoint.action.statements[6],
        ServerStatement::CryptoCencAesCtr(_)
    ));
    assert!(matches!(
        endpoint.endpoint.action.statements[7],
        ServerStatement::Spawn(_)
    ));
}
#[test]
fn parses_cenc_crypto_declaration_without_subsamples() {
    let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/segment"
      method GET async req
        http upstream method:"get" base:"https://media.example" path:"/segment.m4s" mode:"bytes"
        crypto encrypted encryption:"cencAesCtr" data:upstream key:"00000000000000000000000000000000" iv:"0000000000000000"
        return bytes:encrypted contentType:"video/mp4""#
                .to_string(),
        )
        .expect("source");

    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/segment")
        .expect("route");

    assert!(matches!(
        endpoint.endpoint.action.statements[1],
        ServerStatement::CryptoCencAesCtr(_)
    ));
}

#[test]
fn parses_route_middlewares_from_imports() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("features/blogs")).expect("api");
    fs::create_dir_all(root.join("shared/authentication")).expect("middleware");
    fs::write(
        root.join("main.dowe"),
        r#"import apiRoutes from "@/features/blogs/api"

main
  server port:8080
    endpoints:apiRoutes"#,
    )
    .expect("main");
    fs::write(
        root.join("features/blogs/api.dowe"),
        r#"import requireBearer from "../../shared/authentication/bearer"

endpoints apiRoutes
  get path:"/users/:id" middleware:[requireBearer]
    return text:"Hello""#,
    )
    .expect("api");
    fs::write(
        root.join("shared/authentication/bearer.dowe"),
        r#"middleware requireBearer params:{}
  bearer token value:req.header.Authorization
  jwt verified secret:env.JWT_SECRET algorithm:"HS256" token:token
  if verified.valid
    next context:{ auth:{ subject:verified.claims.sub claims:verified.claims } }
  return status:401 json:{ ok:false error:"Unauthorized" }"#,
    )
    .expect("middleware");
    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let environment = EnvironmentConfig {
        variables: vec![EnvironmentVariable {
            name: "JWT_SECRET".to_string(),
            visibility: EnvironmentVisibility::Server,
            resolved_source: EnvironmentValueSource::Missing,
            resolved_value: None,
        }],
    };
    let server = parse_server_source(root, &file, &environment).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/users/123")
        .expect("endpoint");

    assert_eq!(endpoint.endpoint.middlewares.len(), 1);
    assert_eq!(endpoint.endpoint.middlewares[0].name, "requireBearer");
    assert!(matches!(
        &endpoint.endpoint.middlewares[0].action.statements[1],
        ServerMiddlewareStatement::Jwt(ServerJwtStatement::Verify {
            secret: ServerSecret::Environment(name),
            algorithm,
            ..
        }) if name == "JWT_SECRET" && algorithm == "HS256"
    ));
}
