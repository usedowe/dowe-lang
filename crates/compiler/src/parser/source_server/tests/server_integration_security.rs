#[test]
fn rejects_client_environment_for_outbound_http_bearer() {
    let root = Path::new("/project");
    let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/v1/chat/completions"
      method POST async req
        const body value:req.json
        http upstream method:"post" base:env.OPENROUTER_BASE_URL path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:body mode:"proxy"
        return proxy:upstream"#
                .to_string(),
        )
        .expect("source");
    let environment = openrouter_environment(EnvironmentVisibility::Client);
    let error = parse_server_source(root, &file, &environment).expect_err("error");

    assert!(error.to_string().contains("must be server-only"));
}

#[test]
fn parses_static_json_response() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        r#"main
  server port:8080
    route "/health"
      handler
        return json:{ ok:true service:"dowe-llm-server" }"#
            .to_string(),
    )
    .expect("source");
    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/health")
        .expect("endpoint");

    assert!(matches!(
        endpoint.endpoint.behavior,
        EndpointBehavior::HttpActionJson(_)
    ));
}

#[test]
fn parses_request_bytes_and_file_storage() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        r#"main
  server port:8080
    route "/artifacts/:hash"
      method POST
        request payload source:"bytes"
        file stored source:"write" root:"storage" path:req.params.hash data:payload
        return status:201 json:stored"#
            .to_string(),
    )
    .expect("source");
    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Post, "/artifacts/hash")
        .expect("endpoint");
    assert!(matches!(
        &endpoint.endpoint.action.statements[0],
        ServerStatement::RequestBytes { binding } if binding == "payload"
    ));
    assert!(matches!(
        &endpoint.endpoint.action.statements[1],
        ServerStatement::File(ServerFileStatement::Write { binding, data, .. })
            if binding == "stored" && data == "payload"
    ));
}

#[test]
fn parses_password_hash_and_verify() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        r#"main
  server port:8080
    route "/password"
      method POST
        const body value:req.json
        password passwordHash source:"hash" value:body.password
        password verified source:"verify" value:body.password hash:passwordHash
        return json:{ hash:passwordHash valid:verified.valid }"#
            .to_string(),
    )
    .expect("source");
    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Post, "/password")
        .expect("endpoint");
    assert!(matches!(
        &endpoint.endpoint.action.statements[1],
        ServerStatement::Password(ServerPasswordStatement::Hash { binding, .. })
            if binding == "passwordHash"
    ));
    assert!(matches!(
        &endpoint.endpoint.action.statements[2],
        ServerStatement::Password(ServerPasswordStatement::Verify { binding, .. })
            if binding == "verified"
    ));
}

#[test]
fn parses_reverse_proxy_from_required_cache_route() {
    let root = Path::new("/project");
    let file = parse_source_file(
        root,
        Path::new("/project/main.dowe"),
        r#"main
  server port:8080
    route "/*path"
      method GET
        cache routes provider:"dowe" host:"local" port:"4148" account:"proxy" secret:"secret" name:"routes"
        request host source:"header" name:"Host"
        kv route conn:routes.get key:host required:true
        return reverse:route.url"#
            .to_string(),
    )
    .expect("source");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/health")
        .expect("endpoint");

    assert!(matches!(
        endpoint.endpoint.behavior,
        EndpointBehavior::HttpReverseProxy(ref response) if response.upstream == "route.url"
    ));
}

#[test]
fn parses_round_robin_reverse_proxy_from_required_cache_route() {
    let root = Path::new("/project");
    let file = parse_source_file(
        root,
        Path::new("/project/main.dowe"),
        r#"main
  server port:8080
    route "/*path"
      method GET
        cache routes provider:"dowe" host:"local" port:"4148" account:"proxy" secret:"secret" name:"routes"
        request host source:"header" name:"Host"
        kv route conn:routes.get key:host required:true
        return reverse:route.upstreams strategy:"roundRobin" state:route.state loadingUrl:route.loadingUrl errorUrl:route.errorUrl"#
            .to_string(),
    )
    .expect("source");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/health")
        .expect("endpoint");

    assert!(matches!(
        endpoint.endpoint.behavior,
        EndpointBehavior::HttpReverseProxy(ref response)
            if response.upstream == "route.upstreams"
                && response.strategy == ReverseProxyStrategy::RoundRobin
                && response.state.as_deref() == Some("route.state")
                && response.loading_url.as_deref() == Some("route.loadingUrl")
                && response.error_url.as_deref() == Some("route.errorUrl")
    ));
}

#[test]
fn rejects_reverse_proxy_from_request_controlled_value() {
    let root = Path::new("/project");
    let file = parse_source_file(
        root,
        Path::new("/project/main.dowe"),
        r#"main
  server port:8080
    route "/*path"
      method GET
        const body value:req.json
        return reverse:body.url"#
            .to_string(),
    )
    .expect("source");
    let error = parse_server_source(root, &file, &EnvironmentConfig::default())
        .expect_err("request-controlled reverse proxy");

    assert!(error.to_string().contains("required Cache.get binding"));
}

#[test]
fn rejects_reverse_proxy_fallback_from_another_binding() {
    let root = Path::new("/project");
    let file = parse_source_file(
        root,
        Path::new("/project/main.dowe"),
        r#"main
  server port:8080
    route "/*path"
      method GET
        cache routes provider:"dowe" host:"local" port:"4148" account:"proxy" secret:"secret" name:"routes"
        kv route conn:routes.get key:"route" required:true
        kv fallback conn:routes.get key:"fallback" required:true
        return reverse:route.upstreams strategy:"roundRobin" errorUrl:fallback.errorUrl"#
            .to_string(),
    )
    .expect("source");
    let error = parse_server_source(root, &file, &EnvironmentConfig::default())
        .expect_err("fallback from another binding");

    assert!(error.to_string().contains("same required Cache.get binding"));
}

#[test]
fn parses_declared_websocket_http_bridge() {
    let root = Path::new("/project");
    let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    websocket "/api/v1/agent/ws"
      message ws
        ws request source:"json"
        send ws json:{ event:"started" requestId:request.requestId requestType:request.requestType model:request.model payload:{ stream:request.stream } }
        agent chat source:"chat" request:request
        http upstream method:"post" base:"https://openrouter.ai" path:"/api/v1/chat/completions" bearer:env.OPENROUTER_API_KEY json:chat mode:"proxy"
        bridge sse:upstream to:ws requestId:request.requestId requestType:request.requestType model:request.model"#
                .to_string(),
        )
        .expect("source");
    let environment = openrouter_environment(EnvironmentVisibility::Server);
    let server = parse_server_source(root, &file, &environment).expect("server");
    let route = server
        .backend
        .find_websocket("/api/v1/agent/ws")
        .expect("websocket");
    let statements = &route.handlers.message.statements;

    assert!(matches!(&statements[0], ServerStatement::WebSocketJson(_)));
    assert!(matches!(
        &statements[1],
        ServerStatement::WebSocketSendJson(_)
    ));
    assert!(matches!(&statements[2], ServerStatement::AgentChat(_)));
    assert!(matches!(&statements[3], ServerStatement::Http(_)));
    assert!(matches!(
        &statements[4],
        ServerStatement::WebSocketSseBridge(_)
    ));
}
#[test]
fn infers_store_insert_fields_for_log_references() {
    let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/api/blogs"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created conn:db.insert table:"blogs" value:{ title:"First" }
        log created.title
        return json:created"#
                .to_string(),
        )
        .expect("source");

    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/api/blogs")
        .expect("route");

    assert!(
        endpoint
            .endpoint
            .action
            .statements
            .iter()
            .any(|statement| matches!(
                statement,
                ServerStatement::Log(log)
                    if log.values == vec![ServerLogValue::Reference("created.title".to_string())]
            ))
    );
    assert!(matches!(
        endpoint.endpoint.behavior,
        EndpointBehavior::StoreActionJson(_)
    ));
}

#[test]
fn rejects_unknown_store_insert_fields_in_logs() {
    let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/api/blogs"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created conn:db.insert table:"blogs" value:{ title:"First" }
        log created.missing
        return json:created"#
                .to_string(),
        )
        .expect("source");

    let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

    assert!(
        error
            .to_string()
            .contains("unknown field `created.missing`")
    );
}

