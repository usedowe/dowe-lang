    #[test]
    fn rejects_store_operations_inside_config_modules() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("config")).expect("config");
        fs::write(
            root.join("config/db.dowe"),
            r#"query rows db:db.list table:"directvAccounts""#,
        )
        .expect("config");
        let source = fs::read_to_string(root.join("config/db.dowe")).expect("config source");
        let file = parse_source_file(root, &root.join("config/db.dowe"), source).expect("source");
        let error =
            super::validate_server_module_source(root, &file, &EnvironmentConfig::default())
                .expect_err("config operation error");

        assert!(
            error
                .to_string()
                .contains("config modules only support database handle bindings")
        );
    }

    #[test]
    fn rejects_legacy_service_declarations() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("handlers")).expect("handlers");
        fs::write(
            root.join("main.dowe"),
            r#"import listTickets from "@/handlers/tickets"

main
  server port:0
    route "/api/tickets"
      method GET handler:listTickets"#,
        )
        .expect("main");
        fs::write(
            root.join("handlers/tickets.dowe"),
            r#"service listTickets
  return value:{ ok:true }"#,
        )
        .expect("legacy service");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("legacy service error");

        assert!(error.to_string().contains("replaced by `fn`"));
    }

    #[test]
    fn rejects_response_return_inside_server_function() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/handlers")).expect("handlers");
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import listTickets from "@/server/handlers/tickets"

main
  server port:0
    route "/api/tickets"
      method GET handler:listTickets"#,
        )
        .expect("main");
        fs::write(
            root.join("server/handlers/tickets.dowe"),
            r#"import listTicketsService from "../services/tickets"

handler listTickets req
  listTicketsService result
  return json:result"#,
        )
        .expect("handler");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"fn listTicketsService
  return json:{ ok:true }"#,
        )
        .expect("function");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("function return error");

        assert!(error.to_string().contains("return value"));
    }

    #[test]
    fn rejects_client_environment_for_remote_store_credentials() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api-user" secret:env.DB_TOKEN name:"db1"
        query users db:db.list table:"users"
        return json:{ data:users }"#
                .to_string(),
        )
        .expect("source");
        let environment = EnvironmentConfig {
            variables: vec![EnvironmentVariable {
                name: "DB_TOKEN".to_string(),
                visibility: EnvironmentVisibility::Client,
                resolved_source: EnvironmentValueSource::Missing,
                resolved_value: None,
            }],
        };
        let error = parse_server_source(root, &file, &environment).expect_err("error");

        assert!(error.to_string().contains("must be server-only"));
    }

    #[test]
    fn parses_outbound_http_proxy_response() {
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
        let environment = openrouter_environment(EnvironmentVisibility::Server);
        let server = parse_server_source(root, &file, &environment).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/v1/chat/completions")
            .expect("endpoint");

        assert!(matches!(
            endpoint.endpoint.behavior,
            EndpointBehavior::HttpProxy(_)
        ));
        assert!(matches!(
            &endpoint.endpoint.action.statements[1],
            ServerStatement::Http(request)
                if request.mode == HttpResponseMode::Proxy
                    && request.base == HttpConnectionValue::Environment("OPENROUTER_BASE_URL".to_string())
        ));
    }

    #[test]
    fn parses_general_outbound_http_request_options() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/products"
      method PATCH async req
        const body value:req.json
        http upstream method:"patch" base:env.OPENROUTER_BASE_URL path:"/v1/products" headers:[{ name:"Accept" value:"application/json" }, { name:"X-Api-Key" value:env.OPENROUTER_API_KEY }] json:body redirect:"manual" timeoutMs:5000 mode:"json"
        return json:{ ok:upstream.ok status:upstream.status location:upstream.location data:upstream.json }"#
                .to_string(),
        )
        .expect("source");
        let environment = openrouter_environment(EnvironmentVisibility::Server);
        let server = parse_server_source(root, &file, &environment).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Patch, "/api/products")
            .expect("endpoint");

        assert!(matches!(
            &endpoint.endpoint.action.statements[1],
            ServerStatement::Http(request)
                if request.method == HttpMethod::Patch
                    && request.redirect == HttpRedirectPolicy::Manual
                    && request.timeout_ms == Some(5000)
                    && request.headers.len() == 2
                    && request.headers[0].name == "Accept"
                    && request.headers[0].value == HttpHeaderValue::Static("application/json".to_string())
                    && request.headers[1].name == "X-Api-Key"
                    && request.headers[1].value == HttpHeaderValue::Environment("OPENROUTER_API_KEY".to_string())
        ));
    }

    #[test]
    fn parses_stdlib_capability_statement() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"type User
  name:string

main
  server port:8080
    route "/api/normalize"
      method POST async req
        const body:User value:req.json
        str normalized source:"trim" value:body.name
        return json:{ name:normalized }"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/normalize")
            .expect("route");

        assert!(matches!(
            &endpoint.endpoint.action.statements[1],
            ServerStatement::Stdlib(statement)
                if statement.binding == "normalized"
                    && statement.call.namespace == "str"
                    && statement.call.function == "trim"
                    && statement.call.args[0].name == "value"
        ));
    }

    #[test]
    fn rejects_legacy_stdlib_assignment() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/api/session"
      handler
        let sessionKey = str.join values:["session", req.params.id] delimiter:":"
        return json:{ key:sessionKey }"#
                .to_string(),
        )
        .expect("source");
        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("legacy stdlib assignment");

        assert!(error.to_string().contains("str sessionKey source:\"join\""));
    }

    #[test]
    fn parses_request_websocket_and_agent_capabilities() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/inspect"
      handler
        request query source:"query"
        request range source:"header" name:"Range"
        return json:{ query:query range:range }
    websocket "/agent"
      message ws
        ws request source:"json"
        agent chat source:"chat" request:request
        send ws json:chat"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/inspect")
            .expect("endpoint");

        assert!(matches!(
            &endpoint.endpoint.action.statements[0],
            ServerStatement::RequestQuery { binding } if binding == "query"
        ));
        assert!(matches!(
            &endpoint.endpoint.action.statements[1],
            ServerStatement::RequestHeader { binding, name }
                if binding == "range" && name == "Range"
        ));
        assert!(matches!(
            &server.backend.websockets[0].handlers.message.statements[0],
            ServerStatement::WebSocketJson(statement) if statement.binding == "request"
        ));
        assert!(matches!(
            &server.backend.websockets[0].handlers.message.statements[1],
            ServerStatement::AgentChat(statement)
                if statement.binding == "chat" && statement.source == "request"
        ));
    }

    #[test]
    fn rejects_legacy_response_selector() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/health"
      handler
        return response json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("legacy response selector");

        assert!(
            error
                .to_string()
                .contains("HTTP returns use `return <props>`; remove `response`")
        );
    }

    #[test]
    fn rejects_authorization_header_on_outbound_http_request() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/products"
      method GET async req
        http upstream method:"get" base:env.OPENROUTER_BASE_URL path:"/v1/products" headers:[{ name:"Authorization" value:"Bearer token" }]
        return json:upstream"#
                .to_string(),
        )
        .expect("source");
        let environment = openrouter_environment(EnvironmentVisibility::Server);
        let error = parse_server_source(root, &file, &environment).expect_err("error");

        assert!(error.to_string().contains("not allowed"));
    }

    #[test]
    fn rejects_client_environment_for_outbound_http_header() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/products"
      method GET async req
        http upstream method:"get" base:env.OPENROUTER_BASE_URL path:"/v1/products" headers:[{ name:"X-Api-Key" value:env.OPENROUTER_API_KEY }]
        return json:upstream"#
                .to_string(),
        )
        .expect("source");
        let environment = openrouter_environment(EnvironmentVisibility::Client);
        let error = parse_server_source(root, &file, &environment).expect_err("error");

        assert!(error.to_string().contains("must be server-only"));
    }

    #[test]
    fn rejects_outbound_http_request_without_method() {
        let root = Path::new("/project");
        let file = parse_source_file(
            root,
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/products"
      method GET async req
        http upstream base:env.OPENROUTER_BASE_URL path:"/v1/products"
        return json:upstream"#
                .to_string(),
        )
        .expect("source");
        let environment = openrouter_environment(EnvironmentVisibility::Server);
        let error = parse_server_source(root, &file, &environment).expect_err("error");

        assert!(error.to_string().contains("missing `method`"));
    }

