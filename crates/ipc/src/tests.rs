use super::{
    CodeGraphBuildOptions, DeployOptions, DeployTarget, DevTarget, DevTargetSelection, HostOs,
    InitOptions, SpawnConfig, SpawnEvent, build_codegraph, deploy_project, init_agent_harness,
    run_spawn,
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
    fs::create_dir_all(temp.path().join("src")).expect("src");
    fs::write(temp.path().join("src/views.dowe"), "views\n").expect("src");

    let graph = build_codegraph(temp.path(), CodeGraphBuildOptions::default()).expect("graph");
    let encoded = serde_json::to_string(&graph).expect("graph");

    assert!(encoded.contains("views.dowe"));
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
    fs::create_dir_all(root.join("src/layouts")).expect("layouts");
    fs::create_dir_all(root.join("src/pages")).expect("pages");
    fs::write(
        root.join("src/main.dowe"),
        "main\n  server port:8080\n    route \"/api/status\"\n      response text:\"OK\"\n",
    )
    .expect("main");
    fs::write(
        root.join("src/views.dowe"),
        "import RootLayout from \"./layouts/root\"\nimport homePage from \"./pages/home\"\n\nviews\n  route path:\"/\" layout:RootLayout\n    page path:\"\" component:homePage\n",
    )
    .expect("views");
    fs::write(
        root.join("src/layouts/root.dowe"),
        "layout RootLayout\n  Box\n    children\n",
    )
    .expect("layout");
    fs::write(
        root.join("src/pages/home.dowe"),
        "page homePage\n  Text\n    Home\n",
    )
    .expect("page");
}
