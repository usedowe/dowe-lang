    #[test]
    fn rejects_nested_endpoint_groups() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server")).expect("server");
        fs::write(
            root.join("main.dowe"),
            r#"import apiRoutes from "@/server/endpoints"

main
  server port:8080
    endpoints:apiRoutes"#,
        )
        .expect("main");
        fs::write(
            root.join("server/endpoints.dowe"),
            r#"endpoints apiRoutes
  group path:"/api"
    group middleware:[requireBearer]
      get path:"/blogs" handler:listBlogs"#,
        )
        .expect("endpoints");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");

        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("nested endpoint group");

        assert!(error.to_string().contains(
            "`endpoints` groups cannot contain another `group`; put middleware on the group or its HTTP method"
        ), "{error}");
    }

    #[test]
    fn parses_database_service_and_reserves_its_websocket_path() {
        let source = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            "main\n  server port:4147\n    database service\n".to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &source.nodes).expect("server");
        assert!(server.backend.database_service);

        let source = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:4147
    database service
    websocket path:"/v1/databases/:name"
      open ws
        log "open""#
                .to_string(),
        )
        .expect("source");
        let error = parse_server_file(Path::new("/project/main.dowe"), &source.nodes)
            .expect_err("reserved route");
        assert!(
            error
                .to_string()
                .contains("reserves WebSocket path `/v1/databases/:name`"),
            "{error}"
        );
    }
