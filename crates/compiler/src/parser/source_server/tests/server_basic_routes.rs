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
