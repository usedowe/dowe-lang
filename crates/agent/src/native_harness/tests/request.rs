use super::*;

#[test]
fn built_harness_request_contains_shared_screenshot_policy() {
    let root = tempfile::tempdir().expect("root");
    std::fs::write(root.path().join("main.dowe"), "main").expect("Dowe marker");
    std::fs::write(root.path().join("AGENTS.md"), "Prefer focused validation.")
        .expect("instructions");
    let state = tempfile::tempdir().expect("state");
    let store = HarnessStore::new(state.path(), root.path()).expect("store");
    let session = store.create_session().expect("session");
    let request = build_request(
        &store,
        &session,
        &HarnessConfig::default(),
        HarnessRole::Execute,
        &ModelSelection::new("openai", "gpt-5.5"),
        "author a screenshot-driven dashboard",
        None,
    )
    .expect("request");
    let AgentMessageContent::Text(system) = &request.messages[0].content else {
        panic!("system text")
    };

    assert!(system.contains(crate::prompts::SCREENSHOT_UI_POLICY));
    assert!(system.contains("get_skill"));
    assert!(system.contains("visual QA as not run"));
    assert!(system.contains("Prefer focused validation."));
    assert_eq!(request.extra["task_packet"]["role"], "execute");
    assert_eq!(request.extra["task_packet"]["objective"], "author a screenshot-driven dashboard");

    let codegraph = build_request(
        &store,
        &session,
        &HarnessConfig::default(),
        HarnessRole::Codegraph,
        &ModelSelection::new("openai", "gpt-5.5"),
        "enrich the repository index",
        None,
    )
    .expect("codegraph request");
    assert_eq!(
        codegraph.metadata.as_ref().unwrap().get("harness_role"),
        Some(&"codegraph".to_string())
    );
    let AgentMessageContent::Text(codegraph_system) = &codegraph.messages[0].content else {
        panic!("system text")
    };
    assert!(codegraph_system.contains("optional semantic index enrichment"));
    assert!(codegraph_system.contains("Never author changes"));
}

#[test]
fn review_request_contains_only_confirmed_change_hunks() {
    let root = tempfile::tempdir().expect("root");
    std::fs::write(root.path().join("src.rs"), "after").expect("source");
    let state = tempfile::tempdir().expect("state");
    let store = HarnessStore::new(state.path(), root.path()).expect("store");
    let mut session = store.create_session().expect("session");
    session.events = vec![
        serde_json::json!({"event":"task_started","role":"execute"}),
        serde_json::json!({
            "event":"approval_required",
            "approval":{"call":{"id":"old-call"},"details":{"before":"old","after":"old-after"}}
        }),
        serde_json::json!({
            "event":"operation_finished",
            "call_id":"old-call",
            "failed":false,
            "receipt":{"path":"old.rs"}
        }),
        serde_json::json!({"event":"task_started","role":"execute"}),
        serde_json::json!({
            "event":"approval_required",
            "approval":{"call":{"id":"call-1"},"details":{"before":"before","after":"after"}}
        }),
        serde_json::json!({
            "event":"operation_finished",
            "call_id":"call-1",
            "failed":false,
            "receipt":{"path":"src.rs"}
        }),
        serde_json::json!({
            "event":"operation_finished",
            "call_id":"call-2",
            "failed":false,
            "receipt":{"path":"other.rs"}
        }),
        serde_json::json!({"event":"task_started","role":"review"}),
    ];
    let request = build_request(
        &store,
        &session,
        &HarnessConfig::default(),
        HarnessRole::Review,
        &ModelSelection::new("openai", "gpt-5.5"),
        "review the task",
        None,
    )
    .expect("request");
    let turns = request.extra["dowe_harness_turns"].to_string();
    assert!(turns.contains("src.rs"));
    assert!(turns.contains("before"));
    assert!(!turns.contains("other.rs"));
    assert!(!turns.contains("old.rs"));
}

#[test]
fn review_request_labels_added_deleted_and_renamed_confirmed_paths() {
    let root = tempfile::tempdir().expect("root");
    let state = tempfile::tempdir().expect("state");
    let store = HarnessStore::new(state.path(), root.path()).expect("store");
    let mut session = store.create_session().expect("session");
    session.events = vec![
        serde_json::json!({"event":"task_started","role":"execute"}),
        serde_json::json!({"event":"approval_required","approval":{"call":{"id":"add"},"details":{"before":"<missing>","after":"renamed body"}}}),
        serde_json::json!({"event":"operation_finished","call_id":"add","failed":false,"receipt":{"path":"new.rs"}}),
        serde_json::json!({"event":"approval_required","approval":{"call":{"id":"delete"},"details":{"before":"renamed body","after":"<missing>"}}}),
        serde_json::json!({"event":"operation_finished","call_id":"delete","failed":false,"receipt":{"path":"old.rs"}}),
        serde_json::json!({"event":"task_started","role":"review"}),
    ];
    let request = build_request(
        &store,
        &session,
        &HarnessConfig::default(),
        HarnessRole::Review,
        &ModelSelection::new("openai", "gpt-5.5"),
        "review the task",
        None,
    )
    .expect("request");
    let turns = request.extra["dowe_harness_turns"].to_string();
    assert!(turns.contains("rename old.rs -> new.rs"));
    assert!(!turns.contains("(added)"));
    assert!(!turns.contains("(deleted)"));
}

#[test]
fn review_request_uses_task_baseline_for_external_changes_and_renames() {
    let root = tempfile::tempdir().expect("root");
    std::fs::write(root.path().join("old.rs"), "same body").expect("old source");
    std::fs::write(root.path().join("untouched.rs"), "before").expect("untracked source");
    let state = tempfile::tempdir().expect("state");
    let store = HarnessStore::new(state.path(), root.path()).expect("store");
    let baseline = task_baseline(root.path());
    std::fs::rename(root.path().join("old.rs"), root.path().join("new.rs")).expect("rename");
    std::fs::write(root.path().join("untouched.rs"), "changed externally").expect("external edit");
    let mut session = store.create_session().expect("session");
    session.events = vec![
        serde_json::json!({"event":"task_started","taskId":"task","role":"execute","baseline":baseline}),
        serde_json::json!({"event":"task_started","taskId":"review","role":"review","baseline":task_baseline(root.path())}),
    ];
    // The review marker's baseline is the current state, so replace it with a
    // marker without a baseline to model the legacy review boundary. The
    // immediately preceding task still owns the captured baseline.
    session.events[1] = serde_json::json!({"event":"task_started","taskId":"review","role":"review"});
    let request = build_request(
        &store,
        &session,
        &HarnessConfig::default(),
        HarnessRole::Review,
        &ModelSelection::new("openai", "gpt-5.5"),
        "review the task",
        None,
    )
    .expect("request");
    let turns = request.extra["dowe_harness_turns"].to_string();
    assert!(turns.contains("rename old.rs -> new.rs"));
    assert!(turns.contains("same body"));
    assert!(turns.contains("untouched.rs (modified external)"));
    assert!(turns.contains("changed externally"));
}

#[test]
fn task_baseline_omits_generated_capability_map_files() {
    let root = tempfile::tempdir().expect("root");
    std::fs::create_dir_all(root.path().join(".agents/capabilities")).expect("map");
    std::fs::write(root.path().join(".agents/capabilities/index.md"), "# generated\n")
        .expect("index");
    std::fs::write(root.path().join("README.md"), "read me\n").expect("readme");
    let baseline = task_baseline(root.path());
    assert!(baseline
        .as_array()
        .unwrap()
        .iter()
        .all(|entry| !entry["path"].as_str().unwrap().starts_with(".agents/capabilities/")));
    assert!(baseline
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == "README.md"));
}
