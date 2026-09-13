use dowe_agent::native_harness::HarnessConfig;
use serde_json::json;

#[test]
fn project_skill_loading_is_bounded_untrusted_and_hash_paged() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join(".agents/skills/local");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("SKILL.md"), "first\nsecond\nthird\n").unwrap();
    let tools = dowe_agent::native_harness::HarnessTools::new(
        root.path(),
        "session",
        HarnessConfig::default(),
    )
    .unwrap();
    let first = tools
        .execute_read(&dowe_agent::native_harness::ToolCall::new(
            "skill",
            "get_skill",
            json!({"id":"local","source":"project"}),
        ))
        .unwrap();
    assert_eq!(first["source"], "project");
    assert_eq!(first["relative_path"], ".agents/skills/local/SKILL.md");
    assert_eq!(first["untrusted"], true);
    let hash = first["hash"].as_str().unwrap();
    let next = tools
        .execute_read(&dowe_agent::native_harness::ToolCall::new(
            "skill",
            "get_skill",
            json!({"id":"local","source":"project","offset":2,"hash":hash}),
        ))
        .unwrap();
    assert_eq!(next["content"], "second\nthird\n");
    assert!(
        tools
            .execute_read(&dowe_agent::native_harness::ToolCall::new(
                "skill",
                "get_skill",
                json!({"id":"local","source":"project","offset":2})
            ))
            .is_err()
    );
    assert!(
        tools
            .execute_read(&dowe_agent::native_harness::ToolCall::new(
                "skill",
                "get_skill",
                json!({"id":"local","source":"project","offset":2,"hash":"stale"})
            ))
            .is_err()
    );
}

#[test]
fn project_skill_selection_rejects_collisions_resources_and_invalid_ids() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join(".agents/skills/core");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("SKILL.md"), "shadow\n").unwrap();
    let tools = dowe_agent::native_harness::HarnessTools::new(
        root.path(),
        "session",
        HarnessConfig::default(),
    )
    .unwrap();
    let collision = tools
        .execute_read(&dowe_agent::native_harness::ToolCall::new(
            "skill",
            "get_skill",
            json!({"id":"core","source":"project"}),
        ))
        .unwrap_err();
    assert!(collision.to_string().contains("collides"));
    for args in [
        json!({"id":"core","source":"project","resource":"SKILL.md"}),
        json!({"id":"../escape","source":"project"}),
        json!({"id":"missing","source":"project"}),
    ] {
        assert!(
            tools
                .execute_read(&dowe_agent::native_harness::ToolCall::new(
                    "skill",
                    "get_skill",
                    args
                ))
                .is_err()
        );
    }
}
