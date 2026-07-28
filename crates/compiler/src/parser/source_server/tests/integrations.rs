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
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
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
    fn rejects_legacy_app_root() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"app
  server port:8080"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("renamed to `main`"));
    }

    #[test]
    fn rejects_legacy_backend_block() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  backend port:8080"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("renamed to `server`"));
    }

    #[test]
    fn rejects_legacy_endpoint_block() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    endpoint "/api/status"
      response text:"OK""#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("renamed to `route`"));
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
        query created db:db.insert table:"blogs" value:{ title:"First" }
        log created.title
        return json:created"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/blogs")
            .expect("route");

        assert!(endpoint.endpoint.action.statements.iter().any(|statement| matches!(
            statement,
            ServerStatement::Log(log)
                if log.values == vec![ServerLogValue::Reference("created.title".to_string())]
        )));
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
        query created db:db.insert table:"blogs" value:{ title:"First" }
        log created.missing
        return json:created"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("unknown field `created.missing`")
        );
    }

    #[test]
    fn rejects_unknown_store_insert_fields_in_json_responses() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/api/blogs"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created db:db.insert table:"blogs" value:{ title:"First" }
        return json:{ data:created.missing }"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("unknown field `created.missing`")
        );
    }

    #[test]
    fn validates_typed_request_body_references() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"type User
  name:string
  age:number

main
  server port:8080
    route "/api/users"
      method POST async req
        const body:User value:req.json
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created db:db.insert table:"users" value:{ name:body.name age:body.age }
        return json:{ ok:true user:created }"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/users")
            .expect("route");

        assert!(matches!(
            &endpoint.endpoint.action.statements[0],
            ServerStatement::RequestJson {
                binding,
                schema: Some(_)
            } if binding == "body"
        ));
    }

    #[test]
    fn rejects_legacy_request_json_assignment() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/api/users"
      method POST async req
        let body = await req.json()
        return json:body"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("const <binding[:Type]> value:req.json")
        );
    }

    #[test]
    fn validates_shared_type_imported_by_request_body() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("types")).expect("types");
        fs::write(
            root.join("types/users.dowe"),
            r#"type UserInput
  name:string
  age:number"#,
        )
        .expect("type source");
        let file = parse_source_file(
            root,
            &root.join("main.dowe"),
            r#"import UserInput from "@/types/users"

main
  server port:8080
    route "/api/users"
      method POST async req
        const body:UserInput value:req.json
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created db:db.insert table:"users" value:{ name:body.name age:body.age }
        return json:{ ok:true user:created }"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/users")
            .expect("route");

        assert!(matches!(
            &endpoint.endpoint.action.statements[0],
            ServerStatement::RequestJson {
                binding,
                schema: Some(_)
            } if binding == "body"
        ));
    }

    #[test]
    fn rejects_unknown_typed_request_body_fields_in_store_literals() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"type User
  name:string
  age:number

main
  server port:8080
    route "/api/users"
      method POST async req
        const body:User value:req.json
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created db:db.insert table:"users" value:{ name:body.email }
        return json:created"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("unknown field `body.email`"));
    }

    #[test]
    fn expands_grouped_endpoint_methods_and_websocket_middlewares() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("handlers")).expect("handlers");
        fs::create_dir_all(root.join("middlewares")).expect("middlewares");
        fs::write(
            root.join("handlers/blogs.dowe"),
            r#"handler listBlogs
  return text:"List"

handler createBlog
  return text:"Created""#,
        )
        .expect("handlers");
        fs::write(
            root.join("middlewares/auth.dowe"),
            r#"middleware requireBearer
  next"#,
        )
        .expect("middleware");
        fs::write(
            root.join("server.dowe"),
            r#"import { listBlogs, createBlog } from "@/handlers/blogs"
import requireBearer from "@/middlewares/auth"

endpoints apiRoutes
  group path:"/api/blogs" middleware:[requireBearer]
    get path:"" handler:listBlogs
    post path:"/create" handler:createBlog middleware:[requireBearer]
    websocket path:"/events" middleware:[requireBearer]
      open ws
        log "open""#,
        )
        .expect("endpoints");
        fs::write(
            root.join("main.dowe"),
            r#"import apiRoutes from "@/server"

main
  server port:8080
    endpoints:apiRoutes"#,
        )
        .expect("main");

        let source = fs::read_to_string(root.join("main.dowe")).expect("source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("main file");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");

        assert!(
            server
                .backend
                .find_endpoint(&HttpMethod::Get, "/api/blogs")
                .is_some()
        );
        let created = server
            .backend
            .find_endpoint(&HttpMethod::Post, "/api/blogs/create")
            .expect("created endpoint");
        assert_eq!(created.endpoint.middlewares.len(), 2);
        let websocket = server
            .backend
            .find_websocket("/api/blogs/events")
            .expect("websocket");
        assert_eq!(websocket.middlewares.len(), 2);
    }

