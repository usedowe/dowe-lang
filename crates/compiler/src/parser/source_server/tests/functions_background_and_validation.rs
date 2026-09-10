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
fn parses_inline_named_task_mode() {
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
    route "/run"
      handler
        task fn:runCleanup mode:"inline"
        return text:"OK""#,
    )
    .expect("main");
    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
    let ServerStatement::Task(job) = &server.backend.endpoints[0].action.statements[0] else {
        panic!("named task");
    };
    assert!(job.inline);
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

