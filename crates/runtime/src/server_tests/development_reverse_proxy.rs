#[tokio::test]
async fn reverse_proxy_response_headers_tasks_wait_for_real_upstream_headers() {
    let upstream_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("upstream listener");
    let upstream_addr = upstream_listener.local_addr().expect("upstream address");
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("server/tasks")).expect("tasks directory");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import delayedFirst from "@/server/tasks/delayed-first"
import delayedSecond from "@/server/tasks/delayed-second"
import immediateFirst from "@/server/tasks/immediate-first"
import immediateSecond from "@/server/tasks/immediate-second"

main
  server port:0
    route "/setup"
      handler
        cache routes provider:"dowe" host:"local" port:4148 account:"proxy" secret:"secret" name:"routes"
        kv saved conn:routes.set key:"route" value:{ url:"UPSTREAM_URL" projectId:"project_1" state:"ready" }
        return json:saved
    route "/proxy/*path"
      method POST
        cache routes provider:"dowe" host:"local" port:4148 account:"proxy" secret:"secret" name:"routes"
        kv route conn:routes.get key:"route" required:true
        task fn:immediateFirst args:{ event:{ phase:"first" } }
        task fn:delayedFirst args:{ event:{ projectId:route.projectId label:"first" custom:"custom-first" status:0 method:"placeholder" path:"placeholder" latencyMs:0 bytesIn:0 bytesOut:0 } } after:"headers"
        task fn:immediateSecond args:{ event:{ phase:"second" } }
        task fn:delayedSecond args:{ event:{ projectId:route.projectId label:"second" custom:"custom-second" status:0 method:"placeholder" path:"placeholder" latencyMs:0 bytesIn:0 bytesOut:0 } } after:"headers"
        return reverse:route.url"#
            .replace("UPSTREAM_URL", &format!("http://{upstream_addr}")),
    )
    .expect("main");
    fs::write(
        temp.path().join("server/tasks/immediate-first.dowe"),
        r#"type ImmediateFirstEvent
  phase:string

fn immediateFirst params:{ event:ImmediateFirstEvent }
  return value:null"#,
    )
    .expect("immediate first");
    fs::write(
        temp.path().join("server/tasks/immediate-second.dowe"),
        r#"type ImmediateSecondEvent
  phase:string

fn immediateSecond params:{ event:ImmediateSecondEvent }
  return value:null"#,
    )
    .expect("immediate second");
    fs::write(
        temp.path().join("server/tasks/delayed-first.dowe"),
        r#"type DelayedFirstEvent
  projectId:string
  label:string
  custom:string
  status:number
  method:string
  path:string
  latencyMs:number
  bytesIn:number
  bytesOut:number

fn delayedFirst params:{ event:DelayedFirstEvent }
  return value:null"#,
    )
    .expect("delayed first");
    fs::write(
        temp.path().join("server/tasks/delayed-second.dowe"),
        r#"type DelayedSecondEvent
  projectId:string
  label:string
  custom:string
  status:number
  method:string
  path:string
  latencyMs:number
  bytesIn:number
  bytesOut:number

fn delayedSecond params:{ event:DelayedSecondEvent }
  return value:null"#,
    )
    .expect("delayed second");

    let project = compile_dev(temp.path()).expect("project");
    let capture_root = project.root.clone();
    crate::background_jobs::start_task_launch_capture(&capture_root);
    let observed_at_headers = Arc::new(Mutex::new(Vec::new()));
    let upstream_capture_root = capture_root.clone();
    let upstream_observed = observed_at_headers.clone();
    let upstream_server = tokio::spawn(async move {
        let upstream = Router::new().fallback(move || {
            let capture_root = upstream_capture_root.clone();
            let observed = upstream_observed.clone();
            async move {
                *observed.lock().await = crate::background_jobs::task_launches(&capture_root)
                    .into_iter()
                    .filter_map(|launch| launch.target)
                    .collect();
                let mut response = "1234567890123456789".into_response();
                *response.status_mut() = StatusCode::CREATED;
                response
                    .headers_mut()
                    .insert("content-length", axum::http::HeaderValue::from_static("19"));
                response
            }
        });
        axum::serve(upstream_listener, upstream)
            .await
            .expect("upstream server");
    });
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
    let backend = format!("http://{}", servers.backend_addr.expect("backend address"));
    let client = reqwest::Client::new();
    let setup = client
        .get(format!("{backend}/setup"))
        .send()
        .await
        .expect("setup request");
    assert_eq!(setup.status(), reqwest::StatusCode::OK);

    let forwarded = client
        .post(format!("{backend}/proxy/items"))
        .body("payload")
        .send()
        .await
        .expect("forwarded request");
    assert_eq!(forwarded.status(), reqwest::StatusCode::CREATED);
    assert_eq!(forwarded.text().await.expect("forwarded body").len(), 19);
    assert_eq!(
        *observed_at_headers.lock().await,
        vec!["immediateFirst".to_string(), "immediateSecond".to_string()]
    );

    let launches = crate::background_jobs::take_task_launches(&capture_root);
    assert_eq!(launches.len(), 4);
    assert_eq!(launches[0].target.as_deref(), Some("immediateFirst"));
    assert_eq!(launches[1].target.as_deref(), Some("immediateSecond"));
    assert_eq!(launches[2].target.as_deref(), Some("delayedFirst"));
    assert_eq!(launches[3].target.as_deref(), Some("delayedSecond"));
    assert_eq!(launches[0].args, json!({ "event": { "phase": "first" } }));
    assert_eq!(launches[1].args, json!({ "event": { "phase": "second" } }));
    for launch in &launches[2..] {
        assert_eq!(launch.args["event"]["projectId"], "project_1");
        assert_eq!(launch.args["event"]["status"], 201);
        assert_eq!(launch.args["event"]["method"], "POST");
        assert_eq!(launch.args["event"]["path"], "/proxy/items");
        assert!(launch.args["event"]["latencyMs"].as_f64().is_some());
        assert_eq!(launch.args["event"]["bytesIn"], 7);
        assert_eq!(launch.args["event"]["bytesOut"], 19);
    }
    assert_eq!(launches[2].args["event"]["label"], "first");
    assert_eq!(launches[3].args["event"]["label"], "second");
    assert_eq!(launches[2].args["event"]["custom"], "custom-first");
    assert_eq!(launches[3].args["event"]["custom"], "custom-second");

    servers.shutdown().await.expect("shutdown");
    upstream_server.abort();
}
