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
