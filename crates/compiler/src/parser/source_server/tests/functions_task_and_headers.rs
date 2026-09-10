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

