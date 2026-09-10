#[tokio::test]
async fn server_standard_library_reusable_fn_chains_parse_sort_and_math() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("server/handlers")).expect("handlers directory");
    fs::create_dir_all(temp.path().join("server/functions")).expect("functions directory");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import summarize from "@/server/handlers/summarize"

main
  server port:0
    route "/api/stdlib"
      method POST handler:summarize"#,
    )
    .expect("main");
    fs::write(
        temp.path().join("server/handlers/summarize.dowe"),
        r#"import summarizeScores from "@/server/functions/scores"

handler summarize
  const body value:req.json
  summarizeScores result args:{ payload:body.payload }
  return json:result"#,
    )
    .expect("handler");
    fs::write(
        temp.path().join("server/functions/scores.dowe"),
        r#"fn summarizeScores params:{ payload:string }
  parse parsed source:"json" value:args.payload fallback:[]
  sort sorted source:"asc" values:parsed
  math total source:"sum" values:sorted
  return value:{ total:total sorted:sorted }"#,
    )
    .expect("function");

    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: true,
            views: false,
            desktop: false,
        },
    )
    .await
    .expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let result = client
        .post(format!("{backend}/api/stdlib"))
        .json(&json!({ "payload": "[3,1,2]" }))
        .send()
        .await
        .expect("stdlib request");
    assert_eq!(result.status(), reqwest::StatusCode::OK);
    let result = result
        .json::<serde_json::Value>()
        .await
        .expect("result json");
    assert_eq!(result["total"], json!(6.0));
    assert_eq!(result["sorted"], json!([1, 2, 3]));

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn simplified_http_handler_and_function_resolve_dynamic_task_args() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("server/tasks")).expect("tasks directory");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import dispatch from "@/server/tasks/dispatch"
import recordAudit from "@/server/tasks/record-audit"

main
  server port:0
    route "/api/posts"
      method POST
        const body value:req.json
        str auditKey source:"join" values:["post", body.id] delimiter:":"
        task fn:recordAudit args:{ requestId:body.id auditKey:auditKey }
        task args:{ requestId:body.id auditKey:auditKey }
          log args.auditKey
        dispatch dispatched args:{ requestId:body.id auditKey:auditKey }
        return json:{ created:true ...body }"#,
    )
    .expect("main");
    fs::write(
        temp.path().join("server/tasks/record-audit.dowe"),
        r#"fn recordAudit params:{ requestId:string auditKey:string }
  log args.auditKey
  return value:null"#,
    )
    .expect("task");
    fs::write(
        temp.path().join("server/tasks/dispatch.dowe"),
        r#"import recordAudit from "./record-audit"

fn dispatch params:{ requestId:string auditKey:string }
  task fn:recordAudit args:{ requestId:args.requestId auditKey:args.auditKey }
  task args:{ requestId:args.requestId auditKey:args.auditKey }
    log args.auditKey
  return value:null"#,
    )
    .expect("dispatch");

    let project = compile_dev(temp.path()).expect("project");
    let capture_root = project.root.clone();
    crate::background_jobs::start_task_launch_capture(&capture_root);
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: true,
            views: false,
            desktop: false,
        },
    )
    .await
    .expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let response = client
        .post(format!("{backend}/api/posts"))
        .json(&json!({ "id": "order-7", "title": "Task order" }))
        .send()
        .await
        .expect("post request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response
            .json::<serde_json::Value>()
            .await
            .expect("response json"),
        json!({ "created": true, "id": "order-7", "title": "Task order" })
    );

    let launches = crate::background_jobs::take_task_launches(&capture_root);
    assert_eq!(launches.len(), 4);
    assert_eq!(launches[0].target.as_deref(), Some("recordAudit"));
    assert_eq!(
        launches[0].args,
        json!({ "requestId": "order-7", "auditKey": "post:order-7" })
    );
    assert_eq!(launches[1].target, None);
    assert_eq!(
        launches[1].args,
        json!({ "requestId": "order-7", "auditKey": "post:order-7" })
    );
    assert_eq!(launches[2].target.as_deref(), Some("recordAudit"));
    assert_eq!(
        launches[2].args,
        json!({ "requestId": "order-7", "auditKey": "post:order-7" })
    );
    assert_eq!(launches[3].target, None);
    assert_eq!(
        launches[3].args,
        json!({ "requestId": "order-7", "auditKey": "post:order-7" })
    );

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn direct_store_task_handlers_resolve_dynamic_task_args_once() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("server/tasks")).expect("tasks directory");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import dispatch from "@/server/tasks/dispatch"
import recordAudit from "@/server/tasks/record-audit"

main
  server port:0
    route "/api/events"
      method POST
        const body value:req.json
        str auditKey source:"join" values:["event", body.id] delimiter:":"
        database db provider:"postgres" host:"unreachable.invalid" port:5432 account:"unused" secret:"unused" name:"events"
        query created conn:db.insert table:"events" value:{ kind:"task" }
        task fn:recordAudit args:{ requestId:body.id auditKey:auditKey }
        task args:{ requestId:body.id auditKey:auditKey }
          log args.auditKey
        dispatch dispatched args:{ requestId:body.id auditKey:auditKey }
        return json:created
      method GET
        database db provider:"postgres" host:"unreachable.invalid" port:5432 account:"unused" secret:"unused" name:"events"
        query events conn:db.list table:"events"
        return json:events"#,
    )
    .expect("main");
    fs::write(
        temp.path().join("server/tasks/record-audit.dowe"),
        r#"fn recordAudit params:{ requestId:string auditKey:string }
  log args.auditKey
  return value:null"#,
    )
    .expect("task");
    fs::write(
        temp.path().join("server/tasks/dispatch.dowe"),
        r#"import recordAudit from "./record-audit"

fn dispatch params:{ requestId:string auditKey:string }
  task fn:recordAudit args:{ requestId:args.requestId auditKey:args.auditKey }
  task args:{ requestId:args.requestId auditKey:args.auditKey }
    log args.auditKey
  return value:null"#,
    )
    .expect("dispatch");

    let project = compile_dev(temp.path()).expect("project");
    let capture_root = project.root.clone();
    crate::background_jobs::start_task_launch_capture(&capture_root);
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: true,
            views: false,
            desktop: false,
        },
    )
    .await
    .expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
    let created = client
        .post(format!("{backend}/api/events"))
        .json(&json!({ "id": "event-7" }))
        .send()
        .await
        .expect("create request");
    assert_eq!(created.status(), reqwest::StatusCode::OK);
    let created = created
        .json::<serde_json::Value>()
        .await
        .expect("created json");
    assert_eq!(created["kind"], "task");
    assert!(created["id"].as_str().is_some());

    let launches = crate::background_jobs::take_task_launches(&capture_root);
    assert_eq!(launches.len(), 4);
    assert_eq!(launches[0].target.as_deref(), Some("recordAudit"));
    assert_eq!(launches[1].target, None);
    assert_eq!(launches[2].target.as_deref(), Some("recordAudit"));
    assert_eq!(launches[3].target, None);
    for launch in launches {
        assert_eq!(
            launch.args,
            json!({ "requestId": "event-7", "auditKey": "event:event-7" })
        );
    }

    let events = client
        .get(format!("{backend}/api/events"))
        .send()
        .await
        .expect("list request");
    assert_eq!(events.status(), reqwest::StatusCode::OK);
    let events = events
        .json::<serde_json::Value>()
        .await
        .expect("events json");
    assert_eq!(events.as_array().expect("events").len(), 1);
    assert_eq!(events[0]["kind"], "task");

    servers.shutdown().await.expect("shutdown");
}

