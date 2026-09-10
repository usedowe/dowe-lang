use dowe_agent::native_harness::{HarnessConfig, HarnessTools, ToolCall};
use serde_json::json;

#[test]
fn independent_read_tools_run_from_an_immutable_tools_view_and_keep_provider_order() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("one.txt"), "needle in the first file\n").unwrap();
    std::fs::write(root.path().join("two.txt"), "second file\n").unwrap();
    let tools = HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    let calls = vec![
        ToolCall::new("read", "read_file", json!({"path":"one.txt"})),
        ToolCall::new("list", "list_files", json!({"path":"."})),
        ToolCall::new(
            "search",
            "search",
            json!({"path":"one.txt","query":"needle"}),
        ),
    ];

    let results = std::thread::scope(|scope| {
        let workers = calls
            .iter()
            .map(|call| scope.spawn(|| tools.execute_read(call)))
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap().unwrap())
            .collect::<Vec<_>>()
    });

    assert_eq!(results[0]["path"], "one.txt");
    assert!(
        results[1]["paths"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "one.txt")
    );
    assert_eq!(results[2]["matches"][0]["line"], 1);
}

#[test]
fn read_execution_does_not_mutate_loaded_skill_state() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("one.txt"), "content\n").unwrap();
    let tools = HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    let read = ToolCall::new("read", "read_file", json!({"path":"one.txt"}));
    assert!(tools.execute_read(&read).is_ok());
    let skill = ToolCall::new("skill", "get_skill", json!({"id":"core","offset":1}));
    let first = tools.execute_skill(&skill).unwrap();
    assert_ne!(first["status"], "already_loaded");
    assert_eq!(
        tools.execute_skill(&skill).unwrap()["status"],
        "already_loaded"
    );
}
