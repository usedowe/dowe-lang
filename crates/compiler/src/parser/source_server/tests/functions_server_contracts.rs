#[test]
fn builds_inspector_request_metadata_from_compiled_actions() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        r#"type CreateUser
  name:string
  age:number

main
  server port:8080
    route "/users/:id"
      method POST async req
        const body:CreateUser value:req.json
        request query source:"query"
        request auth source:"header" name:"Authorization"
        return json:{ ok:true }
    websocket "/events"
      message ws
        ws incoming source:"json"
        send ws json:incoming"#
            .to_string(),
    )
    .expect("source");
    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let route = &server.inspector.routes[0];
    assert_eq!(route.parameters[0].name, "id");
    assert_eq!(route.parameters[0].location, "path");
    assert!(
        route
            .parameters
            .iter()
            .any(|parameter| parameter.name == "query")
    );
    assert_eq!(route.headers[0].name, "Authorization");
    assert_eq!(
        route.body.as_ref().expect("body").fields[0].field_type,
        "string"
    );
    assert_eq!(server.inspector.websockets[0].message_format, "json");
}

#[test]
fn allows_imported_server_function_calls_inside_websockets() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/services")).expect("services");
    fs::write(
        root.join("main.dowe"),
        r#"import resolveSkill from "@/server/services/skills"

main
  server port:0
    websocket "/events"
      message ws
        ws event source:"json"
        resolveSkill plan args:{ profile:"views" }
        send ws json:plan"#,
    )
    .expect("main");
    fs::write(
        root.join("server/services/skills.dowe"),
        r#"type SkillPlan
  valid:bool

fn resolveSkill params:{ profile:string } return:"SkillPlan"
  return value:{ valid:true }"#,
    )
    .expect("service");

    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
    let websocket = &server.backend.websockets[0];
    assert!(matches!(
        &websocket.handlers.message.statements[1],
        ServerStatement::Call(call) if call.target == "resolveSkill"
    ));
}

#[test]
fn rejects_invalid_server_function_call_shape() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/services")).expect("services");
    fs::write(
        root.join("server/services/tickets.dowe"),
        r#"fn saveTicket
  return value:{ ok:true }"#,
    )
    .expect("function");

    for (call, expected) in [
        (
            "saveTicket",
            "server function call requires one result binding",
        ),
        (
            "saveTicket first second",
            "server function call requires one result binding",
        ),
        (
            "saveTicket result unsupported:true",
            "unknown prop `unsupported`",
        ),
    ] {
        fs::write(
                root.join("main.dowe"),
                format!(
                    "import saveTicket from \"@/server/services/tickets\"\n\nmain\n  server port:0\n    route \"/api/tickets\"\n      handler\n        {call}\n        return json:{{ ok:true }}"
                ),
            )
            .expect("main");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("invalid function call");

        assert!(error.to_string().contains(expected), "{call}: {error}");
    }
}

#[test]
fn validates_server_function_params_and_return_contracts() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/services")).expect("services");
    fs::write(
        root.join("main.dowe"),
        r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler req
        saveTicket result args:{ ticket:{ title:"Open" } }
        return json:result"#,
    )
    .expect("main");
    fs::write(
        root.join("server/services/tickets.dowe"),
        r#"type TicketInput
  title:string

type TicketOutput
  ok:boolean

fn saveTicket params:{ ticket:TicketInput } return:"TicketOutput"
  return value:{ ok:true }"#,
    )
    .expect("function");

    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/api/tickets")
        .expect("endpoint");
    let ServerStatement::Call(call) = &endpoint.endpoint.action.statements[0] else {
        panic!("server function call");
    };
    assert_eq!(call.action.params[0].name, "ticket");
    assert_eq!(call.action.params[0].type_name, "TicketInput");
    assert_eq!(
        call.action
            .return_type
            .as_ref()
            .map(|value| value.type_name.as_str()),
        Some("TicketOutput")
    );
}

#[test]
fn rejects_incompatible_server_function_return() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/services")).expect("services");
    fs::write(
        root.join("main.dowe"),
        r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler req
        saveTicket result args:{ ticket:"invalid" }
        return json:result"#,
    )
    .expect("main");
    fs::write(
        root.join("server/services/tickets.dowe"),
        r#"type TicketInput
  title:string

type TicketOutput
  ok:boolean

fn saveTicket params:{ ticket:TicketInput } return:"TicketOutput"
  return value:{ ok:"invalid" }"#,
    )
    .expect("function");

    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let error = parse_server_source(root, &file, &EnvironmentConfig::default())
        .expect_err("function return error");

    assert!(
        error
            .to_string()
            .contains("function return value is incompatible")
    );
}

#[test]
fn rejects_incompatible_server_function_args() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/services")).expect("services");
    fs::write(
        root.join("main.dowe"),
        r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler req
        saveTicket result args:{ ticket:"invalid" }
        return json:result"#,
    )
    .expect("main");
    fs::write(
        root.join("server/services/tickets.dowe"),
        r#"type TicketInput
  title:string

fn saveTicket params:{ ticket:TicketInput }
  return value:{ ok:true }"#,
    )
    .expect("function");

    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let error = parse_server_source(root, &file, &EnvironmentConfig::default())
        .expect_err("function argument error");

    assert!(
        error
            .to_string()
            .contains("argument `ticket` is incompatible")
    );
}


