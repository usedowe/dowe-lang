use dowe_agent::native_harness::{
    HarnessConfig, HarnessPermissionMode, HarnessRole, HarnessTools, ToolCall,
};
use serde_json::json;

fn write_call(path: &str) -> ToolCall {
    ToolCall::new(
        "write",
        "write_file",
        json!({
            "path": path,
            "content": "application content\n",
            "skill": "full-access",
            "reason": "update application"
        }),
    )
}

#[test]
fn full_access_allows_unclassified_application_files() {
    let root = tempfile::tempdir().unwrap();
    let mut tools = HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    tools.set_permission_mode(HarnessPermissionMode::FullAccess);

    let approval = tools
        .prepare(&write_call("Dockerfile"), HarnessRole::Execute)
        .unwrap()
        .unwrap();
    tools.apply_write(approval).unwrap();

    assert_eq!(
        std::fs::read_to_string(root.path().join("Dockerfile")).unwrap(),
        "application content\n"
    );
}

#[test]
fn full_access_keeps_private_and_secret_paths_closed() {
    let root = tempfile::tempdir().unwrap();
    let mut tools = HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    tools.set_permission_mode(HarnessPermissionMode::FullAccess);

    for path in [".env", ".git/config", ".agents/notes.txt", "AGENTS.md"] {
        assert!(
            tools
                .prepare(&write_call(path), HarnessRole::Execute)
                .is_err(),
            "{path} must remain protected"
        );
    }
}

#[test]
fn permission_mode_is_not_serialized_into_persistent_configuration() {
    let value = serde_json::to_value(HarnessConfig::default()).unwrap();
    assert!(value.get("permission_mode").is_none());
}
