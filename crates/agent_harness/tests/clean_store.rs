use dowe_agent_harness::clean::*;

#[test]
fn durable_workflow_is_saved_under_agent_and_runtime_is_separate() {
    let root = tempfile::tempdir().unwrap();
    let store = WorkflowStore::new(root.path()).unwrap();
    let workflow = Workflow::new("feature-1", Intent::Plan, Budget::default());
    let path = store.save(&workflow).unwrap();
    assert!(path.ends_with(".agent/workflows/feature-1.json"));
    assert!(store.load("feature-1").is_ok());
    let runtime = store.runtime_path("session-1").unwrap();
    assert!(runtime.ends_with(".dowe/agent-runtime/session-1"));
}

#[test]
fn durable_workflow_ids_cannot_escape_project_root() {
    let root = tempfile::tempdir().unwrap();
    let store = WorkflowStore::new(root.path()).unwrap();
    assert!(store.runtime_path("../outside").is_err());
}
