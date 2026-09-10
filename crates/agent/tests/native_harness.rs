use dowe_agent::native_harness::*;
use serde_json::json;

#[test]
fn catalog_is_granular_versioned_and_preserves_public_bundles() {
    validate_catalog().unwrap();
    for id in [
        "core",
        "core/configuration",
        "core/validation",
        "theme",
        "views/layouts",
        "views/pages",
        "views/components",
        "server/entities",
        "server/handlers",
        "server/functions",
    ] {
        let unit = skill_unit(id).unwrap();
        assert!(!unit.content.is_empty(), "{id}");
        assert_eq!(unit.hash.len(), 64);
    }
    assert!(skill_unit("../../private").is_err());
    let selected = select_units("ajusta el tema", &["theme.dowe".into()]);
    assert!(selected.contains(&"theme".to_string()));
    assert!(!selected.iter().any(|id| id.starts_with("server/")));
    let views = dowe_agent::get_public_skill("views", true).unwrap();
    assert!(views.content.contains("references/layouts.md"));
    assert!(views.content.contains("Swap"));
    assert!(views.content.contains("Default-first output"));
    assert!(views.content.contains("`px`"));
    for forbidden_example in [
        "data:invoices\n        variant:\"solid\"",
        "data:members\n        variant:\"solid\"",
        "Avatar alt:\"Team member avatar\" variant:\"solid\"",
        "Footer boxed:true variant:\"solid\" scheme:\"surface\"",
        "Card variant:\"solid\" scheme:\"surface\" p:4",
        "Chip variant:\"solid\" scheme:\"warning\" size:\"sm\"",
    ] {
        assert!(
            !views.content.contains(forbidden_example),
            "default-emitting example remains: {forbidden_example}"
        );
    }
    assert!(dowe_agent::get_public_skill_resource("views", "references/layouts.md").is_ok());
    for bundle in dowe_agent::public_skills() {
        assert!(
            !dowe_agent::get_public_skill(&bundle.id, true)
                .unwrap()
                .content
                .is_empty()
        );
    }
}

#[test]
fn role_selection_has_explicit_precedence_and_no_implicit_fallback() {
    let active = ModelSelection::new("openai", "gpt-5.5");
    let planner = ModelSelection::new("anthropic", "claude-sonnet-4-6");
    let mut config = HarnessConfig::default();
    config.roles.insert(HarnessRole::Plan, planner.clone());
    assert_eq!(
        config.resolve(HarnessRole::Plan, None, &active).unwrap(),
        planner
    );
    assert_eq!(
        config
            .resolve(HarnessRole::Plan, Some(&active), &active)
            .unwrap(),
        active
    );
    assert_eq!(
        config.resolve(HarnessRole::Compact, None, &active).unwrap(),
        active
    );
    config
        .roles
        .insert(HarnessRole::Execute, ModelSelection::new("unknown", "bad"));
    assert!(config.resolve(HarnessRole::Execute, None, &active).is_err());
}

#[test]
fn file_scope_and_approval_reject_drift_and_preserve_env_values() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    std::fs::write(root.path().join(".env"), "SECRET=private-value\n").unwrap();
    let mut tools = HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    assert!(tools.read("../outside", 1, 20).is_err());
    assert!(tools.read(".dowe/agent/auth.json", 1, 20).is_err());
    let env = tools.read(".env", 1, 20).unwrap();
    assert!(!env.to_string().contains("private-value"));
    assert!(tools.prepare(&ToolCall::new("env", "write_file", json!({"path":".env","content":"SECRET=[REDACTED]\n","skill":"core/configuration","reason":"configure app"})), HarnessRole::Execute).is_err());
    let call = ToolCall::new(
        "write",
        "write_file",
        json!({"path":"main.dowe","content":"main { name:\"demo\" }\n","skill":"core","reason":"name app"}),
    );
    let approval = tools.prepare(&call, HarnessRole::Execute).unwrap().unwrap();
    assert!(tools.prepare(&call, HarnessRole::Plan).is_err());
    std::fs::write(root.path().join("main.dowe"), "user edit").unwrap();
    assert!(tools.apply_write(approval).is_err());
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "user edit"
    );
}

#[test]
fn memory_is_local_bounded_and_sessions_use_compare_and_swap() {
    let home = tempfile::tempdir().unwrap();
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), a.path()).unwrap();
    let other = HarnessStore::new(home.path(), b.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let legacy_memory = home
        .path()
        .join("harness")
        .join(&session.project)
        .join("memory.json");
    std::fs::write(&legacy_memory, b"legacy global memory").unwrap();
    let mut stale = store.load_session(&session.id).unwrap();
    store.save_session(&mut session).unwrap();
    assert!(store.save_session(&mut stale).is_err());
    assert!(other.load_session(&session.id).is_err());
    for n in 0..12 {
        store
            .remember(&format!("Theme {n}"), "Use semantic colors", "user", true)
            .unwrap();
    }
    assert_eq!(store.recall("Theme").unwrap().len(), 8);
    assert!(other.recall("Theme").unwrap().is_empty());
    assert!(a.path().join(".agents/memory.json").is_file());
    assert_eq!(std::fs::read(&legacy_memory).unwrap(), b"legacy global memory");
    store
        .remember("Unconfirmed", "Theme assumption", "model", false)
        .unwrap();
    assert!(store.recall("Unconfirmed").unwrap().is_empty());
    store.delete_session(&session.id).unwrap();
    assert!(!store.recall("Theme").unwrap().is_empty());
}

#[test]
fn changed_catalog_is_not_replayed_and_old_history_can_be_listed_and_deleted() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let path = home
        .path()
        .join("harness")
        .join(&session.project)
        .join("sessions")
        .join(format!("{}.json", session.id));
    let mut value = serde_json::to_value(&session).unwrap();
    value["catalog"] = json!("old-catalog");
    std::fs::write(path, value.to_string()).unwrap();
    assert!(store.load_session(&session.id).is_err());
    assert_eq!(store.sessions().unwrap(), vec![session.id.clone()]);
    store.delete_session(&session.id).unwrap();
}

#[test]
fn structural_paths_override_misleading_filename_words_and_connect_fullstack_units() {
    let units = select_units(
        "Corrige este archivo",
        &["server/entities/theme-settings.dowe".into()],
    );
    assert!(units.contains(&"server/entities".into()));
    assert!(!units.contains(&"theme".into()));
    let units = select_units(
        "Fix `views/pages/dashboard.dowe` and `server/handlers/dashboard.dowe`",
        &[],
    );
    for unit in [
        "views/pages",
        "views/requests",
        "server/handlers",
        "server/routes",
    ] {
        assert!(units.contains(&unit.into()), "{units:?}");
    }
    assert!(!units.contains(&"server/entities".into()));
}

#[test]
fn tools_only_responses_and_invalid_arguments_are_distinguished() {
    for payload in [
        json!({"output":[{"type":"function_call","call_id":"a","name":"read_file","arguments":"{\"path\":\"main.dowe\"}"}]}),
        json!({"choices":[{"message":{"tool_calls":[{"id":"a","type":"function","function":{"name":"read_file","arguments":"{\"path\":\"main.dowe\"}"}}]}}]}),
        json!({"content":[{"type":"tool_use","id":"a","name":"read_file","input":{"path":"main.dowe"}}]}),
        json!({"candidates":[{"content":{"parts":[{"functionCall":{"id":"a","name":"read_file","args":{"path":"main.dowe"}}}]}}]}),
        json!({"output":{"message":{"content":[{"toolUse":{"toolUseId":"a","name":"read_file","input":{"path":"main.dowe"}}}]}}}),
    ] {
        let turn = response_turn(&payload).unwrap();
        assert_eq!(turn.calls.len(), 1);
        assert_eq!(turn.calls[0].arguments["path"], "main.dowe");
    }
    assert!(response_turn(&json!({"choices":[{"message":{"tool_calls":[{"id":"a","function":{"name":"read_file","arguments":"invalid"}}]}}]})).is_err());
}
