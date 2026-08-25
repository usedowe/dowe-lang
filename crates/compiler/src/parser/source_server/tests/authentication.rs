    #[test]
    fn parses_capability_first_session_verification() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        write_session_middleware_project(
            root,
            "session verified cache:appCache database:appDb token:token maxAge:2592000",
        );

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/private")
            .expect("endpoint");

        assert!(matches!(
            &endpoint.endpoint.middlewares[0].action.statements[1],
            ServerMiddlewareStatement::SessionVerify {
                binding,
                max_age_seconds: 2_592_000,
                ..
            } if binding == "verified"
        ));
    }
    #[test]
    fn parses_server_function_call_from_middleware() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/middlewares")).expect("middleware");
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import requireAccess from "@/server/middlewares/access"

main
  server port:8080
    route "/api/private" middleware:[requireAccess]
      response text:"Private""#,
        )
        .expect("main");
        fs::write(
            root.join("server/middlewares/access.dowe"),
            r#"import authorizeRequest from "../services/access"

middleware requireAccess
  bearer token value:req.header.Authorization
  authorizeRequest verification args:{ authorization:token }
  if verification.valid
    next
  return status:401 json:{ ok:false error:"Unauthorized" }"#,
        )
        .expect("middleware");
        fs::write(
            root.join("server/services/access.dowe"),
            r#"fn authorizeRequest params:{ authorization:string }
  return value:{ valid:true }"#,
        )
        .expect("function");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/private")
            .expect("endpoint");

        assert!(matches!(
            &endpoint.endpoint.middlewares[0].action.statements[1],
            ServerMiddlewareStatement::Call(call)
                if call.binding == "verification" && call.target == "authorizeRequest"
        ));
    }

    #[test]
    fn parses_multiple_handlers_imported_from_one_module() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/handlers")).expect("handlers");
        fs::write(
            root.join("main.dowe"),
            r#"import apiRoutes from "@/server/api"

main
  server port:8080
    endpoints:apiRoutes"#,
        )
        .expect("main");
        fs::write(
            root.join("server/api.dowe"),
            r#"import { listBlogs, createBlog } from "./handlers/blogs"

endpoints apiRoutes
  group path:"/api/blogs"
    get path:"" handler:listBlogs
    post path:"" handler:createBlog"#,
        )
        .expect("api");
        fs::write(
            root.join("server/handlers/blogs.dowe"),
            r#"handler listBlogs
  return text:"List"

handler createBlog
  return text:"Created""#,
        )
        .expect("handlers");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");

        assert!(
            server
                .backend
                .find_endpoint(&HttpMethod::Get, "/api/blogs")
                .is_some()
        );
        assert!(
            server
                .backend
                .find_endpoint(&HttpMethod::Post, "/api/blogs")
                .is_some()
        );
    }

    #[test]
    fn parses_jwt_result_binding_in_handler() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("src")).expect("src");
        fs::write(
            root.join("main.dowe"),
            r#"main
  server port:8080
    route "/login"
      handler
        jwt token secret:env.JWT_SECRET algorithm:"HS256" claims:{ sub:"user-1" }
        return json:{ ok:true data:{ token:token } }"#,
        )
        .expect("main");
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
            .find_endpoint(&HttpMethod::Get, "/login")
            .expect("endpoint");

        assert!(matches!(
            endpoint.endpoint.action.statements[0],
            ServerStatement::Jwt(ServerJwtStatement::Sign { .. })
        ));
    }

    #[test]
    fn rejects_middleware_without_next_or_response() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("middlewares")).expect("middlewares");
        fs::write(
            root.join("main.dowe"),
            r#"import requireBearer from "@/middlewares/auth"

main
  server port:8080
    route "/users" middleware:[requireBearer]
      handler
        return json:{ ok:true }"#,
        )
        .expect("main");
        fs::write(
            root.join("middlewares/auth.dowe"),
            r#"middleware requireBearer
  bearer token value:req.header.Authorization"#,
        )
        .expect("middleware");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("middleware must not fall through");

        assert!(
            error
                .to_string()
                .contains("must call `next` or return a response")
        );
    }

    #[test]
    fn parses_implicit_handler_request_binding() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("handlers")).expect("handlers");
        fs::write(
            root.join("main.dowe"),
            r#"import blogDetail from "@/handlers/blogs"

main
  server port:0
    route "/blogs/:id"
      method GET handler:blogDetail"#,
        )
        .expect("main");
        fs::write(
            root.join("handlers/blogs.dowe"),
            r#"handler blogDetail
  return json:{ id:req.params.id }"#,
        )
        .expect("handler");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/blogs/:id")
            .expect("endpoint");

        assert!(matches!(
            endpoint.endpoint.behavior,
            EndpointBehavior::HttpActionJson(_)
        ));
    }

    #[test]
    fn parses_implicit_async_handler_operations() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/products"
      handler fetchProducts
        http upstream method:"get" base:"https://example.com" path:"/products"
        return json:{ ok:upstream.ok data:upstream.json }"#
                .to_string(),
        )
        .expect("source");

        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/products")
            .expect("endpoint");

        assert!(matches!(
            endpoint.endpoint.action.statements[0],
            ServerStatement::Http(_)
        ));
    }

    #[test]
    fn rejects_explicit_async_handler_marker() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/products"
      handler fetchProducts async
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("explicit async handler");

        assert!(
            error
                .to_string()
                .contains("handlers are asynchronous by default; remove `async`")
        );
    }

    #[test]
    fn preserves_explicit_handler_request_compatibility() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/blogs/:id"
      handler blogDetail req
        return json:{ id:req.params.id }"#
                .to_string(),
        )
        .expect("source");

        parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect("explicit request alias");
    }

    #[test]
    fn parses_encrypted_jwt_handler_without_request_binding() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/session"
      handler createEncryptedSession
        jwt token key:env.JWT_KEY algorithm:"dir" encryption:"A256GCM" claims:{ sub:"user-1" }
        return json:{ token:token }"#
                .to_string(),
        )
        .expect("source");
        let environment = EnvironmentConfig {
            variables: vec![EnvironmentVariable {
                name: "JWT_KEY".to_string(),
                visibility: EnvironmentVisibility::Server,
                resolved_source: EnvironmentValueSource::Missing,
                resolved_value: None,
            }],
        };
        let server =
            parse_server_source(Path::new("/project"), &file, &environment).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/session")
            .expect("endpoint");

        assert!(matches!(
            endpoint.endpoint.action.statements[0],
            ServerStatement::Jwt(ServerJwtStatement::Encrypt { .. })
        ));
    }
    #[test]
    fn parses_typed_server_function_call_chain() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/handlers")).expect("handlers");
        fs::create_dir_all(root.join("server/handlers")).expect("handlers");
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::create_dir_all(root.join("server/repositories")).expect("repositories");
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
  listTicketsService result args:{ status:"open" }
  return json:result"#,
        )
        .expect("handler");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"import listTicketsRepository from "../repositories/tickets"

fn listTicketsService params:{ status:string }
  listTicketsRepository result args:{ status:args.status }
  return value:{ ok:true data:result.rows cache:result.cache }"#,
        )
        .expect("function");
        fs::write(
            root.join("server/repositories/tickets.dowe"),
            r#"fn listTicketsRepository params:{ status:string }
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"support"
  query rows conn:db.list table:"tickets"
  cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"app" secret:"secret" name:"support-cache"
  kv saved conn:appCache.set key:"tickets:last-list" value:{ status:args.status }
  return value:{ rows:rows cache:saved }"#,
        )
        .expect("function");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/tickets")
            .expect("endpoint");

        assert!(matches!(
            &endpoint.endpoint.action.statements[0],
            ServerStatement::Call(call) if call.binding == "result" && call.target == "listTicketsService"
        ));
        assert!(matches!(
            endpoint.endpoint.behavior,
            EndpointBehavior::HttpActionJson(_)
        ));
    }

