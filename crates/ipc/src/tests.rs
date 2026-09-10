use super::{
    AgentPrepareOptions, AgentRequestType, BuildOptions, BuildTarget, CodeGraphBuildOptions,
    DeployOptions, DeploySurface, DeployTarget, DevTarget, DevTargetSelection, GenerateIconOptions,
    HostOs, IconRounded, IconTarget, InitOptions, SpawnConfig, SpawnEvent, ThinkingLevel,
    agent_model_details, agent_response_usage, build_codegraph, build_project, deploy_project,
    generate_project_icons, get_agent_public_skill, get_agent_public_skill_resource,
    handle_agent_mcp_message, init_agent_harness, list_agent_public_skills,
    prepare_agent_project_context, prepare_agent_request, run_spawn, search_agent_public_examples,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn agent_conversation_uses_the_shared_contract_without_changing_inference() {
    let root = TempDir::new().unwrap();
    let mut conversation = super::AgentConversation::default();
    let first = conversation
        .prepare(root.path(), "hola", AgentPrepareOptions::default())
        .unwrap()
        .request;
    assert_eq!(first.request_type, AgentRequestType::Conversation);
    let payload = serde_json::json!({"output_text":"¡Hola!"});
    assert_eq!(super::agent_response_text(&payload).unwrap(), "¡Hola!");
    let response = super::AgentServerResponse {
        request_id: first.request_id.clone(),
        request_type: first.request_type,
        model: first.model.clone(),
        payload,
    };
    conversation.record_response(&first, &response).unwrap();
    let next = conversation
        .prepare(root.path(), "seguimos", AgentPrepareOptions::default())
        .unwrap()
        .request;
    assert_eq!(next.messages.len(), 4);
    let structured =
        prepare_agent_request(root.path(), "hola", AgentPrepareOptions::default()).unwrap();
    assert_eq!(structured.request.request_type, AgentRequestType::Clarify);
}

#[test]
fn serializes_spawn_events_for_ipc() {
    let event = SpawnEvent::Started {
        spawn_id: 1,
        system_pid: Some(2),
        command: "echo".to_string(),
        pty: false,
    };

    let encoded = serde_json::to_string(&event).expect("event");

    assert!(encoded.contains("Started"));
    assert!(encoded.contains("echo"));
}

#[test]
fn serializes_dev_target_selection_for_ipc() {
    let selection = DevTargetSelection::new([DevTarget::Server, DevTarget::Web], HostOs::Linux)
        .expect("selection");

    let encoded = serde_json::to_string(&selection).expect("selection");

    assert!(encoded.contains("Server"));
    assert!(encoded.contains("Web"));
}

#[test]
fn initializes_agent_harness_through_ipc_wrapper() {
    let temp = TempDir::new().expect("tempdir");

    let report = init_agent_harness(temp.path(), InitOptions::default()).expect("harness");
    let encoded = serde_json::to_string(&report).expect("report");

    assert!(encoded.contains(".agents/AGENTS.md"));
    assert!(temp.path().join(".agents/manifest.json").exists());
    assert!(!temp.path().join("agents").exists());
}

#[test]
fn serializes_agent_harness_manifest_for_ipc() {
    let temp = TempDir::new().expect("tempdir");
    init_agent_harness(temp.path(), InitOptions::default()).expect("harness");
    let content = fs::read_to_string(temp.path().join(".agents/manifest.json")).expect("manifest");

    assert!(content.contains(r#""mode": "project""#));
    assert!(content.contains(r#""tddRequired": true"#));
}

#[test]
fn serializes_codegraph_for_ipc() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("src/routes")).expect("src");
    fs::write(
        temp.path().join("src/routes/view.dowe"),
        "views viewRoutes\n",
    )
    .expect("src");

    let graph = build_codegraph(temp.path(), CodeGraphBuildOptions::default()).expect("graph");
    let encoded = serde_json::to_string(&graph).expect("graph");

    assert!(encoded.contains("routes/view.dowe"));
}

#[test]
fn prepares_agent_request_through_ipc_wrapper() {
    let temp = TempDir::new().expect("tempdir");
    let prepared = prepare_agent_request(
        temp.path(),
        "create a fullstack dashboard with routes",
        AgentPrepareOptions {
            request_type: Some(AgentRequestType::SpecPlan),
            ..AgentPrepareOptions::default()
        },
    )
    .expect("agent");
    let encoded = serde_json::to_string(&prepared.request).expect("request");

    assert_eq!(prepared.request.request_type, AgentRequestType::SpecPlan);
    assert!(encoded.contains("requestType"));
    assert!(encoded.contains("openai/gpt-5.5"));
}

#[test]
fn agent_codex_default_and_retirement_are_shared_with_ipc() {
    let temp = TempDir::new().unwrap();
    let options = AgentPrepareOptions {
        provider: Some("openai-codex".into()),
        ..Default::default()
    };
    assert_eq!(
        prepare_agent_request(temp.path(), "hola", options.clone())
            .unwrap()
            .request
            .model,
        "gpt-5.5"
    );
    let error = prepare_agent_request(
        temp.path(),
        "hola",
        AgentPrepareOptions {
            model: Some("gpt-5.3-codex".into()),
            ..options
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("no longer supported"));
    assert!(error.to_string().contains("gpt-5.5"));
}

#[test]
fn agent_thinking_and_usage_use_shared_contracts() {
    let temp = TempDir::new().unwrap();
    let options = AgentPrepareOptions {
        provider: Some("openai-codex".into()),
        model: Some("gpt-5.5".into()),
        thinking_level: Some(ThinkingLevel::High),
        ..Default::default()
    };
    let prepared = prepare_agent_request(temp.path(), "hello", options.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(&prepared.request).unwrap()["thinkingLevel"],
        "high"
    );
    let invalid = AgentPrepareOptions {
        model: Some("custom".into()),
        ..options
    };
    assert!(prepare_agent_request(temp.path(), "hello", invalid).is_err());
    let payload = serde_json::json!({"usage":{"input_tokens":100,"output_tokens":20}});
    assert_eq!(
        agent_response_usage("openai-codex", "gpt-5.5", &payload)
            .unwrap()
            .context_tokens(),
        120
    );
    assert_eq!(
        agent_model_details("openai-codex", "gpt-5.5")
            .unwrap()
            .context_window,
        Some(272000)
    );
}

#[test]
fn serializes_read_only_session_observer_through_ipc() {
    let home = TempDir::new().unwrap();
    let root = TempDir::new().unwrap();
    let store = dowe_agent::native_harness::HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    session.events.push(serde_json::json!({"event":"response", "prompt":"omit", "secret":"omit"}));
    store.save_session(&mut session).unwrap();
    let facade = super::SessionObserver::new(store, session.id).unwrap();
    let handle = facade.subscribe();
    let page = facade.poll(&handle, 0, super::SessionEventLimits::default()).unwrap();
    let encoded = serde_json::to_string(&page).unwrap();
    assert!(encoded.contains("response"));
    assert!(!encoded.contains("secret"));
    assert!(!encoded.contains("prompt"));
}

#[test]
fn exposes_public_agent_bridge_through_ipc() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");

    let skills = list_agent_public_skills();
    let views = get_agent_public_skill("views", false).expect("views");
    let styles =
        get_agent_public_skill_resource("views", "references/styles.md").expect("styles resource");
    let examples = search_agent_public_examples("dashboard sidebar form", 3).expect("examples");
    let context = prepare_agent_project_context(temp.path()).expect("context");
    let mcp = handle_agent_mcp_message(temp.path(), r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#)
        .expect("mcp")
        .expect("response");

    assert_eq!(skills.len(), 6);
    assert_eq!(views.id, "views");
    assert_eq!(styles.path, "references/styles.md");
    assert_eq!(examples.results[0].id, "dashboard-layout");
    assert_eq!(context.mode, "project");
    assert!(mcp.contains(r#""result":{}"#));
}

#[test]
fn deploys_static_package_through_ipc_wrapper() {
    let temp = TempDir::new().expect("tempdir");
    write_deploy_fixture(temp.path());

    let report =
        deploy_project(DeployOptions::new(temp.path(), DeployTarget::Static)).expect("deploy");

    assert_eq!(report.target, DeployTarget::Static);
    assert!(report.output_dir.join("index.html").is_file());
}

#[test]
fn generates_project_icons_through_ipc_wrapper() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("assets")).expect("assets");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    fs::write(
        temp.path().join("assets/icon.svg"),
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path fill="#fff" d="M0 0h10v10H0z"/></svg>"##,
    )
    .expect("svg");

    let report = generate_project_icons(
        GenerateIconOptions::new(temp.path(), "assets/icon.svg", "#123456", IconRounded::Sm)
            .with_targets([IconTarget::Web]),
    )
    .expect("icons");

    assert_eq!(report.targets, [IconTarget::Web]);
    let serialized = serde_json::to_value(&report).expect("serialized report");
    assert_eq!(serialized["targets"], serde_json::json!(["web"]));
    assert!(temp.path().join("icons/web/favicon.ico").is_file());
}

#[test]
fn deploys_cloudflare_pages_package_through_ipc_wrapper() {
    let temp = TempDir::new().expect("tempdir");
    write_deploy_fixture(temp.path());
    let mut options = DeployOptions::new(temp.path(), DeployTarget::CloudflarePages);
    options.name = Some("ipc-pages".to_string());

    let report = deploy_project(options).expect("deploy");

    assert_eq!(report.target, DeployTarget::CloudflarePages);
    assert!(report.output_dir.join("assets/index.html").is_file());
}

#[test]
fn deploys_vercel_package_through_ipc_wrapper() {
    let temp = TempDir::new().expect("tempdir");
    write_deploy_fixture(temp.path());
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Vercel);
    options.surface = Some(DeploySurface::Web);
    options.name = Some("ipc-vercel".to_string());

    let report = deploy_project(options).expect("deploy");

    assert_eq!(report.target, DeployTarget::Vercel);
    assert!(
        report
            .output_dir
            .join(".vercel/output/static/index.html")
            .is_file()
    );
}

#[test]
fn exposes_ssh_deploy_contract_through_ipc() {
    let mut options = DeployOptions::new("/project", DeployTarget::Ssh);
    options.ssh_host = Some("server.example.com".into());
    options.ssh_user = Some("deploy".into());
    options.ssh_key_file = Some(std::path::PathBuf::from("/keys/deploy"));

    assert_eq!(options.target, DeployTarget::Ssh);
    assert_eq!(options.ssh_host.as_deref(), Some("server.example.com"));
    assert_eq!(options.ssh_user.as_deref(), Some("deploy"));
    assert_eq!(
        options.ssh_key_file,
        Some(std::path::PathBuf::from("/keys/deploy"))
    );
}

#[test]
fn plans_native_build_through_ipc_wrapper() {
    let temp = TempDir::new().expect("tempdir");
    write_deploy_fixture(temp.path());
    let mut options = BuildOptions::new(temp.path(), BuildTarget::Android);
    options.dry_run = true;

    let report = build_project(options).expect("build plan");

    assert_eq!(report.target, BuildTarget::Android);
    assert!(!report.built);
    assert!(report.artifact.ends_with("DoweDev.apk"));
}

#[tokio::test]
async fn runs_spawn_through_ipc_wrapper() {
    let output = run_spawn(shell_config("printf ipc")).await.expect("output");

    assert_eq!(output.stdout_bytes, b"ipc");
}

fn shell_config(script: impl Into<String>) -> SpawnConfig {
    let script = script.into();
    if cfg!(windows) {
        SpawnConfig::new("cmd", ["/C".to_string(), script])
    } else {
        SpawnConfig::new("sh", ["-c".to_string(), script])
    }
}

fn write_deploy_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("layouts")).expect("layouts");
    fs::create_dir_all(root.join("pages")).expect("pages");
    fs::create_dir_all(root.join("routes")).expect("routes");
    fs::write(
        root.join("main.dowe"),
        "import viewRoutes from \"@/routes/view\"\n\nmain\n  views:viewRoutes\n  server port:8080\n    route \"/api/status\"\n      response text:\"OK\"\n",
    )
    .expect("main");
    fs::write(
        root.join("routes/view.dowe"),
        "import RootLayout from \"../layouts/root\"\nimport homePage from \"../pages/home\"\n\nviews viewRoutes\n  group path:\"/\" layout:RootLayout\n    route path:\"\" page:homePage\n",
    )
    .expect("views");
    fs::write(
        root.join("layouts/root.dowe"),
        "layout RootLayout\n  Box\n    children\n",
    )
    .expect("layout");
    fs::write(
        root.join("pages/home.dowe"),
        "page homePage\n  Text\n    \"Home\"\n",
    )
    .expect("page");
}
