use super::{
    AgentPrepareOptions, AgentRequestType, BuildOptions, BuildTarget, CodeGraphBuildOptions,
    DeployOptions, DeployTarget, DevTarget, DevTargetSelection, GenerateIconOptions, HostOs,
    IconRounded, IconTarget, InitOptions, InitProjectOptions, ProjectTemplate, SpawnConfig,
    SpawnEvent, build_codegraph, build_project, deploy_project, generate_project_icons,
    get_agent_public_skill, handle_agent_mcp_message, init_agent_harness, init_dowe_project,
    init_external_agent_project, list_agent_public_skills, prepare_agent_project_context,
    prepare_agent_request, run_spawn, search_agent_public_examples, update_external_agent_project,
};
use std::fs;
use tempfile::TempDir;

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
fn initializes_external_agent_project_through_ipc_wrapper() {
    let temp = TempDir::new().expect("tempdir");

    let report = init_external_agent_project(temp.path()).expect("init");

    assert!(report.created.iter().any(|file| file.path == "AGENTS.md"));
    assert!(report.created.iter().any(|file| file.path == "CLAUDE.md"));
    assert!(temp.path().join("AGENTS.md").is_file());
    assert!(temp.path().join("CLAUDE.md").is_file());
    assert!(temp.path().join(".agents/manifest.json").is_file());
    assert!(
        temp.path()
            .join(".agents/skills/dowe-core/SKILL.md")
            .is_file()
    );
}

#[test]
fn initializes_complete_dowe_project_through_ipc_wrapper() {
    let temp = TempDir::new().expect("tempdir");

    let report = init_dowe_project(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::Crud).with_i18n(true),
    )
    .expect("init");

    assert_eq!(report.project.template(), ProjectTemplate::Crud);
    assert!(report.project.i18n_enabled());
    assert!(temp.path().join("main.dowe").is_file());
    assert!(temp.path().join("i18n/es.dowe").is_file());
    assert!(temp.path().join(".agents/manifest.json").is_file());
}

#[test]
fn reinstalls_complete_dowe_project_through_ipc_wrapper() {
    let temp = TempDir::new().expect("tempdir");
    init_dowe_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Blank)).expect("init");
    fs::write(temp.path().join("main.dowe"), "stale main").expect("main");

    let report = init_dowe_project(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::Blank).with_reinstall(true),
    )
    .expect("reinstall");

    assert!(report.project.reinstalled());
    assert_ne!(
        fs::read_to_string(temp.path().join("main.dowe")).expect("main"),
        "stale main"
    );
}

#[test]
fn updates_external_agent_project_through_ipc_wrapper() {
    let temp = TempDir::new().expect("tempdir");
    init_external_agent_project(temp.path()).expect("init");
    fs::write(
        temp.path().join(".agents/skills/dowe-core/SKILL.md"),
        "stale",
    )
    .expect("stale");

    update_external_agent_project(temp.path()).expect("update");

    assert!(
        fs::read_to_string(temp.path().join(".agents/skills/dowe-core/SKILL.md"))
            .expect("updated")
            .starts_with("---\nname: dowe-core\n")
    );
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
fn exposes_public_agent_bridge_through_ipc() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");

    let skills = list_agent_public_skills();
    let views = get_agent_public_skill("views", false).expect("views");
    let examples = search_agent_public_examples("dashboard sidebar form", 3).expect("examples");
    let context = prepare_agent_project_context(temp.path()).expect("context");
    let mcp = handle_agent_mcp_message(temp.path(), r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#)
        .expect("mcp")
        .expect("response");

    assert_eq!(skills.len(), 4);
    assert_eq!(views.id, "views");
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
    assert!(temp.path().join("assets/icons/web/favicon.ico").is_file());
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
