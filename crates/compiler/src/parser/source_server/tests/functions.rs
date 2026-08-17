#[test]
fn rejects_legacy_server_function_assignment() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/services")).expect("services");
    fs::write(
        root.join("main.dowe"),
        r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler
        let result = saveTicket args:{ title:"Open" }
        return json:result"#,
    )
    .expect("main");
    fs::write(
        root.join("server/services/tickets.dowe"),
        r#"fn saveTicket params:{ title:string }
  return value:{ title:args.title }"#,
    )
    .expect("function");

    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let error = parse_server_source(root, &file, &EnvironmentConfig::default())
        .expect_err("legacy function call");

    assert!(
        error
            .to_string()
            .contains("server function calls use `saveTicket result args:{ ... }`")
    );
}

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
    let inspector_entity = server
        .inspector
        .entities
        .iter()
        .find(|entity| entity.table == "directvAccounts")
        .expect("inspector entity");
    assert_eq!(inspector_entity.database, "iptv");
    assert_eq!(inspector_entity.field_details[0].field_type, "string");
    assert!(inspector_entity.field_details[0].primary);
    assert!(inspector_entity.field_details[1].required);
    assert!(inspector_entity.field_details[1].index);
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

#[test]
fn parses_task_and_cron_jobs_from_server_init() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/tasks")).expect("tasks");
    fs::write(
        root.join("main.dowe"),
        r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      runCleanup startupResult args:{ source:"direct" }
      task fn:runCleanup args:{ source:"startup" }
      cron fn:runCleanup schedule:"*/15 * * * *" args:{ source:"cron" }"#,
    )
    .expect("main");
    fs::write(
        root.join("server/tasks/cleanup.dowe"),
        r#"fn runCleanup params:{ source:string }
  log args.source
  return value:{ ok:true }"#,
    )
    .expect("function");
    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");

    assert!(matches!(
        &server.backend.init_action.statements[0],
        ServerStatement::Call(call)
            if call.binding == "startupResult" && call.target == "runCleanup"
    ));
    assert!(matches!(
        &server.backend.init_action.statements[1],
        ServerStatement::Task(job)
            if job.target.as_deref() == Some("runCleanup") && job.schedule.is_none()
    ));
    assert!(matches!(
        &server.backend.init_action.statements[2],
        ServerStatement::Cron(job)
            if job.target.as_deref() == Some("runCleanup")
                && job.schedule.as_deref() == Some("*/15 * * * *")
    ));
}

#[test]
fn rejects_positional_cron_target() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/tasks")).expect("tasks");
    fs::write(
        root.join("main.dowe"),
        r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      cron runCleanup schedule:"0 * * * *""#,
    )
    .expect("main");
    fs::write(
        root.join("server/tasks/cleanup.dowe"),
        r#"fn runCleanup
  return value:{ ok:true }"#,
    )
    .expect("function");

    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let error = parse_server_source(root, &file, &EnvironmentConfig::default())
        .expect_err("positional cron target");

    assert!(
        error
            .to_string()
            .contains("cron does not accept positional targets; use `fn:<imported-fn>`")
    );
}

#[test]
fn parses_named_task_with_handler_binding() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/tasks")).expect("tasks");
    fs::write(
        root.join("main.dowe"),
        r#"import emitTelemetry from "@/server/tasks/telemetry"

main
  server port:0
    route "/telemetry"
      handler req
        const event value:req.json
        task fn:emitTelemetry args:{ event:event }
        return status:202 json:{ queued:true }"#,
    )
    .expect("main");
    fs::write(
        root.join("server/tasks/telemetry.dowe"),
        r#"type TelemetryEvent
  projectId:string

fn emitTelemetry params:{ event:TelemetryEvent }
  return value:{ ok:true }"#,
    )
    .expect("function");
    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
    let statements = &server.backend.endpoints[0].action.statements;

    assert!(matches!(
        &statements[1],
        ServerStatement::Task(job)
            if job.timing == crate::model::ServerTaskTiming::Immediate && matches!(
                &job.args,
                StoreLiteral::Object(entries)
                    if entries == &vec![(
                        "event".to_string(),
                        StoreLiteral::Reference("event".to_string())
                    )]
            )
    ));
}

#[test]
fn parses_response_headers_tasks_for_reverse_proxy_handlers() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/tasks")).expect("tasks");
    fs::write(
        root.join("main.dowe"),
        r#"import emitTelemetry from "@/server/tasks/telemetry"

main
  server port:0
    route "/*path"
      method POST
        cache routes provider:"dowe" host:"local" port:4148 account:"proxy" secret:"secret" name:"routes"
        kv route conn:routes.get key:"route" required:true
        task fn:emitTelemetry args:{ event:{ projectId:route.projectId kind:"named" } } after:"headers"
        task args:{ event:{ projectId:route.projectId kind:"inline" } } after:"headers"
          log args.event.projectId
        return reverse:route.url"#,
    )
    .expect("main");
    fs::write(
        root.join("server/tasks/telemetry.dowe"),
        r#"type TelemetryEvent
  projectId:string
  kind:string

fn emitTelemetry params:{ event:TelemetryEvent }
  log args.event.projectId
  return value:null"#,
    )
    .expect("task");

    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
    let tasks = server.backend.endpoints[0]
        .action
        .statements
        .iter()
        .filter_map(|statement| match statement {
            ServerStatement::Task(job) => Some(job),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(tasks.len(), 2);
    assert_eq!(
        tasks[0].timing,
        crate::model::ServerTaskTiming::ResponseHeaders
    );
    assert_eq!(
        tasks[1].timing,
        crate::model::ServerTaskTiming::ResponseHeaders
    );
    assert_eq!(tasks[0].target.as_deref(), Some("emitTelemetry"));
    assert!(tasks[1].target.is_none());
}

#[test]
fn rejects_invalid_response_headers_task_contracts() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/tasks")).expect("tasks");
    fs::write(
        root.join("server/tasks/telemetry.dowe"),
        r#"type TelemetryEvent
  projectId:string

fn emitTelemetry params:{ event:TelemetryEvent }
  return value:null"#,
    )
    .expect("task");

    for (task, expected) in [
        (
            "task fn:emitTelemetry args:{ event:{ projectId:route.projectId } } after:\"body\"",
            "`after` must be \"headers\"",
        ),
        (
            "task fn:emitTelemetry args:{ event:{ projectId:route.projectId } } after:headers",
            "`after` must be the quoted string \"headers\"",
        ),
        (
            "task fn:emitTelemetry after:\"headers\"",
            "requires `args:{ event:{ ... } }`",
        ),
        (
            "task fn:emitTelemetry args:{ event:route.projectId } after:\"headers\"",
            "requires `args.event` to be an object",
        ),
    ] {
        fs::write(
            root.join("main.dowe"),
            format!(
                "import emitTelemetry from \"@/server/tasks/telemetry\"\n\nmain\n  server port:0\n    route \"/*path\"\n      method GET\n        cache routes provider:\"dowe\" host:\"local\" port:4148 account:\"proxy\" secret:\"secret\" name:\"routes\"\n        kv route conn:routes.get key:\"route\" required:true\n        {task}\n        return reverse:route.url"
            ),
        )
        .expect("main");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("invalid response headers task");
        let rendered = error.to_string();

        assert!(rendered.contains(expected), "{rendered}");
        assert!(rendered.contains("main.dowe"), "{rendered}");
    }
}

#[test]
fn rejects_response_headers_task_outside_a_direct_reverse_proxy_handler() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    let task = "task args:{ event:{ projectId:\"project\" } } after:\"headers\"\n        log args.event.projectId";

    for (source, expected) in [
        (
            format!("main\n  server port:0\n    init\n      {}", task),
            "only valid directly in an HTTP handler",
        ),
        (
            format!(
                "main\n  server port:0\n    route \"/plain\"\n      handler\n        {}\n        return text:\"OK\"",
                task.replace("\n        ", "\n          ")
            ),
            "only valid in an HTTP handler whose final response is `return reverse:...`",
        ),
        (
            format!(
                "main\n  server port:0\n    route \"/*path\"\n      handler\n        cache routes provider:\"dowe\" host:\"local\" port:4148 account:\"proxy\" secret:\"secret\" name:\"routes\"\n        kv route conn:routes.get key:\"route\" required:true\n        {}\n        return reverse:route.url\n        log \"after reverse\"",
                task.replace("\n        ", "\n          ")
            ),
            "only valid in an HTTP handler whose final response is `return reverse:...`",
        ),
        (
            format!(
                "main\n  server port:0\n    websocket \"/socket\"\n      message ws\n        {}",
                task.replace("\n        ", "\n          ")
            ),
            "only valid directly in an HTTP handler",
        ),
        (
            format!(
                "main\n  server port:0\n    udp name:\"udp\" bind:\"127.0.0.1\" port:5060\n      packet packet\n        {}",
                task.replace("\n        ", "\n          ")
            ),
            "only valid directly in an HTTP handler",
        ),
    ] {
        fs::write(root.join("main.dowe"), source).expect("main");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("invalid task scope");
        let rendered = error.to_string();

        assert!(rendered.contains(expected), "{rendered}");
        assert!(rendered.contains("main.dowe"), "{rendered}");
    }

    fs::write(
        root.join("main.dowe"),
        r#"main
  server port:0
    init
      cron fn:missing schedule:"0 * * * *" after:"headers""#,
    )
    .expect("main");
    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let error =
        parse_server_source(root, &file, &EnvironmentConfig::default()).expect_err("cron timing");

    assert!(
        error
            .to_string()
            .contains("`after` is only valid on a direct reverse-proxy task")
    );

    let function_path = root.join("server/functions/delayed.dowe");
    fs::create_dir_all(function_path.parent().expect("parent")).expect("functions");
    fs::write(
        &function_path,
        r#"fn delayed
  task args:{ event:{ projectId:"project" } } after:"headers"
    log args.event.projectId
  return value:null"#,
    )
    .expect("function");
    let source = fs::read_to_string(&function_path).expect("function source");
    let file = parse_source_file(root, &function_path, source).expect("function file");
    let error = super::validate_server_module_source(root, &file, &EnvironmentConfig::default())
        .expect_err("function task timing");
    assert!(
        error
            .to_string()
            .contains("only valid directly in an HTTP handler")
    );

    let middleware_path = root.join("server/middlewares/guard.dowe");
    fs::create_dir_all(middleware_path.parent().expect("parent")).expect("middlewares");
    fs::write(
        &middleware_path,
        r#"middleware guard
  if verification.valid
    task args:{ event:{ projectId:"project" } } after:"headers"
      log args.event.projectId
  next"#,
    )
    .expect("middleware");
    let source = fs::read_to_string(&middleware_path).expect("middleware source");
    let file = parse_source_file(root, &middleware_path, source).expect("middleware file");
    let error = super::validate_server_module_source(root, &file, &EnvironmentConfig::default())
        .expect_err("nested middleware task timing");
    assert!(
        error.to_string().contains("unsupported middleware action"),
        "{error}"
    );
}

#[test]
fn rejects_invalid_background_jobs() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/tasks")).expect("tasks");
    fs::write(
        root.join("main.dowe"),
        r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      cron fn:runCleanup schedule:"60 * * * *"
    route "/run"
      handler req
        task fn:runCleanup args:{ source:req.params.id }
        return text:"OK""#,
    )
    .expect("main");
    fs::write(
        root.join("server/tasks/cleanup.dowe"),
        r#"fn runCleanup
  return value:{ ok:true }"#,
    )
    .expect("function");
    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let error =
        parse_server_source(root, &file, &EnvironmentConfig::default()).expect_err("invalid cron");

    assert!(error.to_string().contains("cron value `60`"));
}

#[test]
fn parses_inline_task_with_dynamic_args_and_local_bindings() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::write(
        root.join("main.dowe"),
        r#"main
  server port:0
    route "/orders"
      handler
        const order value:req.json
        task args:{ orderId:order.id }
          str auditKey source:"join" values:["orders", args.orderId] delimiter:":"
          log auditKey
        return status:202 json:{ queued:true }"#,
    )
    .expect("main");

    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
    let statements = &server.backend.endpoints[0].action.statements;

    let ServerStatement::Task(job) = &statements[1] else {
        panic!("inline task");
    };
    assert!(job.target.is_none());
    assert!(job.id.ends_with(":task:inline"));
    assert!(matches!(
        &job.args,
        StoreLiteral::Object(entries)
            if entries == &vec![(
                "orderId".to_string(),
                StoreLiteral::Reference("order.id".to_string())
            )]
    ));
    assert!(matches!(
        job.action.statements.as_slice(),
        [ServerStatement::Stdlib(_), ServerStatement::Log(_)]
    ));
}

#[test]
fn validates_named_task_shape_and_static_background_arguments() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/tasks")).expect("tasks");
    fs::write(
        root.join("server/tasks/cleanup.dowe"),
        r#"fn runCleanup params:{ source:string }
  return value:{ ok:true }"#,
    )
    .expect("function");

    for (source, expected) in [
        (
            r#"main
  server port:0
    init
      task"#,
            "task must declare one imported target or a non-empty inline body",
        ),
        (
            r#"main
  server port:0
    init
      cron schedule:"0 * * * *""#,
            "cron must declare `fn:<imported-fn>`",
        ),
        (
            r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      task runCleanup extra"#,
            "task does not accept positional targets; use `fn:<imported-fn>`",
        ),
        (
            r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      task fn:runCleanup
        log "invalid""#,
            "named task does not accept child blocks",
        ),
        (
            r#"main
  server port:0
    init
      task fn:missing"#,
            "missing server function import `missing`",
        ),
        (
            r#"main
  server port:0
    init
      task args:{ source:req.params.id }
        log args.source"#,
            "background args must be static JSON",
        ),
        (
            r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      cron fn:runCleanup schedule:"0 * * * *" args:{ source:req.params.id }"#,
            "background args must be static JSON",
        ),
        (
            r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      task fn:runCleanup args:{ ...payload }"#,
            "store literals do not support spread",
        ),
        (
            r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      task fn:runCleanup"#,
            "function call is missing required argument `source`",
        ),
    ] {
        fs::write(root.join("main.dowe"), source).expect("main");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("invalid task");

        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn rejects_inline_task_captures_and_control_statements() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();

    for (body, expected) in [
        (
            "log order.id",
            "inline task body cannot capture outer binding `order`",
        ),
        (
            "log req.params.id",
            "inline task body cannot capture outer binding `req`",
        ),
        (
            "log env.SECRET",
            "inline task body cannot capture outer binding `env`",
        ),
        (
            "return value:{ ok:true }",
            "inline task body cannot use `return`",
        ),
        (
            "task\n  log \"nested\"",
            "inline task body cannot use `task`",
        ),
        (
            "cron fn:runCleanup schedule:\"0 * * * *\"",
            "inline task body cannot use `cron`",
        ),
        (
            "response text:\"invalid\"",
            "inline task body cannot use `response`",
        ),
        (
            "send ws json:{ ok:true }",
            "inline task body cannot use `send`",
        ),
    ] {
        fs::write(
            root.join("main.dowe"),
            format!(
                "main\n  server port:0\n    route \"/orders\"\n      handler\n        const order value:req.json\n        task\n          {}\n        return status:202 json:{{ queued:true }}",
                body.replace('\n', "\n          ")
            ),
        )
        .expect("main");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("invalid inline task");

        let rendered = error.to_string();
        assert!(rendered.contains(expected), "{rendered}");
        assert!(rendered.contains("main.dowe"), "{rendered}");
    }
}

#[test]
fn rejects_legacy_go_with_a_task_repair() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/tasks")).expect("tasks");
    fs::write(
        root.join("server/tasks/cleanup.dowe"),
        r#"fn runCleanup
  return value:{ ok:true }"#,
    )
    .expect("function");
    fs::write(
        root.join("main.dowe"),
        r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      go runCleanup"#,
    )
    .expect("main");

    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let error =
        parse_server_source(root, &file, &EnvironmentConfig::default()).expect_err("legacy go");

    assert!(
        error
            .to_string()
            .contains("`go` was renamed to `task`; use `task fn:runCleanup`")
    );
}

#[test]
fn functions_import_store_handles_from_config_modules() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/handlers")).expect("handlers");
    fs::create_dir_all(root.join("server/services")).expect("services");
    fs::create_dir_all(root.join("server/repositories")).expect("repositories");
    fs::create_dir_all(root.join("server/config")).expect("config");
    fs::write(
        root.join("main.dowe"),
        r#"import listAccounts from "@/server/handlers/accounts"

main
  server port:0
    route "/api/accounts"
      method GET handler:listAccounts"#,
    )
    .expect("main");
    fs::write(
        root.join("server/handlers/accounts.dowe"),
        r#"import listAccountsService from "../services/accounts"

handler listAccounts req
  listAccountsService result
  return json:result"#,
    )
    .expect("handler");
    fs::write(
        root.join("server/services/accounts.dowe"),
        r#"import listAccountsRepository from "../repositories/accounts"

fn listAccountsService
  listAccountsRepository result
  return value:{ rows:result.rows }"#,
    )
    .expect("function");
    fs::write(
            root.join("server/config/db.dowe"),
            r#"entity Accounts
  id:string primary:true
  name:string required:true index:true

seeder Bootstrap
  insert entity:Accounts value:{ id:"01ARZ3NDEKTSV4RRFFQ69G5FAV" name:"Primary" }

database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"iptv" entities:[Accounts] seeders:[Bootstrap]"#,
        )
        .expect("config");
    fs::write(
        root.join("server/repositories/accounts.dowe"),
        r#"import db from "../config/db"

fn listAccountsRepository
  query rows conn:db.list table:"directvAccounts"
  return value:{ rows:rows }"#,
    )
    .expect("function");
    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/api/accounts")
        .expect("endpoint");

    let ServerStatement::Call(service_call) = &endpoint.endpoint.action.statements[0] else {
        panic!("function call");
    };
    let ServerStatement::Call(repository_call) = &service_call.action.statements[0] else {
        panic!("nested function call");
    };
    assert!(matches!(
        &repository_call.action.statements[0],
        ServerStatement::Store(crate::model::ServerStoreStatement::Handle {
            connection
        }) if connection.binding == "db"
            && connection.database == "iptv"
            && connection.entities.len() == 1
            && connection.entities[0].binding == "Accounts"
            && connection.seeders.len() == 1
            && connection.seeders[0].binding == "Bootstrap"
    ));
    assert!(matches!(
        &repository_call.action.statements[1],
        ServerStatement::Store(crate::model::ServerStoreStatement::List {
            handle,
            table,
            ..
        }) if handle == "db" && table == "directvAccounts"
    ));
}

#[test]
fn accepts_server_functions_from_arbitrary_module_paths() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();

    for relative in [
        "server/services/example.dowe",
        "domains/accounts/application/example.dowe",
        "shared/example.dowe",
        "example.dowe",
    ] {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("directory");
        fs::write(&path, "fn example\n  return value:{ ok:true }\n").expect("function");
        let source = fs::read_to_string(&path).expect("source");
        let file = parse_source_file(root, &path, source).expect("file");

        super::validate_server_module_source(root, &file, &EnvironmentConfig::default())
            .expect("declaration-based function module");
    }
}

#[test]
fn accepts_server_config_from_arbitrary_module_path() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    let path = root.join("domains/accounts/storage.dowe");
    fs::create_dir_all(path.parent().expect("parent")).expect("directory");
    fs::write(
            &path,
            "database accountsDb provider:\"dowe\" host:\"127.0.0.1\" port:4147 account:\"api\" secret:\"secret\" name:\"accounts\"\n",
        )
        .expect("config");
    let source = fs::read_to_string(&path).expect("source");
    let file = parse_source_file(root, &path, source).expect("file");

    super::validate_server_module_source(root, &file, &EnvironmentConfig::default())
        .expect("declaration-based config module");
}
