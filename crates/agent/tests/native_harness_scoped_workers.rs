use dowe_agent::native_harness::{HarnessConfig, HarnessRole, HarnessTools, ToolCall};
use dowe_agent_harness::AllowedEditSurface;
use serde_json::json;

fn scoped_tools(root: &std::path::Path) -> HarnessTools {
    HarnessTools::with_scope(
        root,
        "scoped-worker",
        HarnessConfig::default(),
        Some(vec![AllowedEditSurface::new("src").unwrap()]),
    )
    .unwrap()
}

#[test]
fn in_scope_write_is_prepared_for_approval() {
    let root = tempfile::tempdir().unwrap();
    let mut tools = scoped_tools(root.path());
    let call = ToolCall::new(
        "write-in-scope",
        "write_file",
        json!({
            "path": "src/main.dowe",
            "content": "main {}\n",
            "skill": "core",
            "reason": "create scoped source"
        }),
    );

    assert!(tools.prepare(&call, HarnessRole::Execute).unwrap().is_some());
}

#[test]
fn out_of_scope_write_is_rejected_before_approval() {
    let root = tempfile::tempdir().unwrap();
    let mut tools = scoped_tools(root.path());
    let call = ToolCall::new(
        "write-out-of-scope",
        "write_file",
        json!({
            "path": "main.dowe",
            "content": "main {}\n",
            "skill": "core",
            "reason": "attempt root write"
        }),
    );

    let error = tools.prepare(&call, HarnessRole::Execute).unwrap_err();
    assert!(error.to_string().contains("outside the worker edit scope"));
}

#[cfg(unix)]
#[test]
fn shell_cwd_must_stay_inside_the_worker_scope() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("src")).unwrap();
    let mut tools = HarnessTools::with_scope(
        root.path(),
        "scoped-worker",
        HarnessConfig { shell: Some("/bin/sh".into()), ..Default::default() },
        Some(vec![AllowedEditSurface::new("src").unwrap()]),
    )
    .unwrap();
    let call = ToolCall::new(
        "shell-out-of-scope",
        "shell",
        json!({
            "command": "pwd",
            "cwd": ".",
            "reason": "inspect project root"
        }),
    );

    let error = tools.prepare(&call, HarnessRole::Execute).unwrap_err();
    assert!(error.to_string().contains("outside the worker edit scope"));
}
