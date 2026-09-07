use dowe_agent::native_harness::*;
use serde_json::json;

#[test]
fn recovery_crash_fixture() {
    let Some(root) = std::env::var_os("DOWE_RECOVERY_FIXTURE_ROOT") else {
        return;
    };
    let home = std::env::var_os("DOWE_RECOVERY_FIXTURE_HOME").unwrap();
    let phase = std::env::var("DOWE_RECOVERY_FIXTURE_PHASE").unwrap();
    let store = HarnessStore::new(home, &root).unwrap();
    let mut session = store.create_session().unwrap();
    let mut tools = HarnessTools::new(&root, &session.id, HarnessConfig::default()).unwrap();
    tools
        .execute_read(&ToolCall::new(
            "skill",
            "get_skill",
            json!({"id":"core/configuration"}),
        ))
        .unwrap();
    let call = ToolCall::new(
        "write",
        "write_file",
        json!({"path":"main.dowe","content":"main\n","skill":"core/configuration","reason":"recovery fixture"}),
    );
    let approval = tools.prepare(&call, HarnessRole::Execute).unwrap().unwrap();
    session.interrupted = true;
    session
        .events
        .push(json!({"event":"approval_required","approval":approval}));
    session
        .events
        .push(json!({"event":"operation_started","call_id":"write"}));
    store.save_session(&mut session).unwrap();
    std::fs::write(
        std::path::Path::new(&root).join("fixture-session"),
        &session.id,
    )
    .unwrap();
    if phase != "before_effect" {
        let result = tools.apply_write(approval).unwrap();
        if phase == "after_receipt" {
            session.events.push(json!({"event":"operation_finished","call_id":"write","result":result,"failed":false}));
            store.save_session(&mut session).unwrap();
        }
    }
    std::process::exit(23);
}

#[test]
fn abrupt_exit_before_effect_after_effect_and_after_receipt_never_replays() {
    for phase in ["before_effect", "after_effect", "after_receipt"] {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "recovery_crash_fixture"])
            .env_clear()
            .env("DOWE_RECOVERY_FIXTURE_ROOT", root.path())
            .env("DOWE_RECOVERY_FIXTURE_HOME", home.path())
            .env("DOWE_RECOVERY_FIXTURE_PHASE", phase)
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if std::time::Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("crash fixture timed out");
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        assert_eq!(status.code(), Some(23));
        let store = HarnessStore::new(home.path(), root.path()).unwrap();
        let id = std::fs::read_to_string(root.path().join("fixture-session")).unwrap();
        let before = std::fs::read(root.path().join("main.dowe")).ok();
        assert_eq!(before.is_some(), phase != "before_effect");
        let report = store.inspect_session(&id, 0).unwrap();
        assert_eq!(
            report["unresolved_operations"],
            if phase == "after_receipt" { 0 } else { 1 }
        );
        let fresh = store
            .recover_session(
                &id,
                report["ticket"].as_str().unwrap(),
                "Inspect and continue only after review",
            )
            .unwrap();
        assert!(fresh.turns.is_empty());
        assert_eq!(std::fs::read(root.path().join("main.dowe")).ok(), before);
        assert!(store.load_session(&id).unwrap().interrupted);
    }
}

#[test]
fn inspection_binds_evidence_and_recovery_never_replays_or_changes_the_original() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    std::fs::write(root.path().join("main.dowe"), "main\n").unwrap();
    let mut source = store.create_session().unwrap();
    source.interrupted = true;
    source.events = vec![
        json!({"event":"approval_required","approval":{"id":"approval","call":{"name":"write_file"},"details":{"path":"main.dowe"}}}),
        json!({"event":"operation_started","call_id":"write","approval_id":"approval"}),
        json!({"event":"operation_finished","call_id":"write","failed":false,"result":null}),
    ];
    store.save_session(&mut source).unwrap();
    let original = serde_json::to_value(store.load_session(&source.id).unwrap()).unwrap();
    let inspection = store.inspect_session(&source.id, 0).unwrap();
    assert_eq!(inspection["unresolved_operations"], 1);
    let ticket = inspection["ticket"].as_str().unwrap();
    std::fs::write(root.path().join("main.dowe"), "main views:[]\n").unwrap();
    assert!(
        store
            .recover_session(&source.id, ticket, "Inspect current application")
            .is_err()
    );
    let inspection = store.inspect_session(&source.id, 0).unwrap();
    let recovered = store
        .recover_session(
            &source.id,
            inspection["ticket"].as_str().unwrap(),
            "Inspect current application",
        )
        .unwrap();
    assert_ne!(source.id, recovered.id);
    assert!(recovered.turns.is_empty());
    assert!(!recovered.interrupted);
    assert!(recovered.summary.as_ref().unwrap().contains("never replay"));
    assert_eq!(
        serde_json::to_value(store.load_session(&source.id).unwrap()).unwrap(),
        original
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "main views:[]\n"
    );
}

#[test]
fn damaged_and_old_catalog_histories_are_inspectable_without_poisoning_inventory() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let source = store.create_session().unwrap();
    let directory = home
        .path()
        .join("harness")
        .join(&source.project)
        .join("sessions");
    let path = directory.join(format!("{}.json", source.id));
    let mut old = serde_json::to_value(&source).unwrap();
    old["catalog"] = json!("previous-catalog");
    old["events"] = json!(
        (0..50)
            .map(|index| json!({"event":"test","index":index,"text":"x".repeat(20000)}))
            .collect::<Vec<_>>()
    );
    std::fs::write(&path, old.to_string()).unwrap();
    let damaged = "a".repeat(32);
    std::fs::write(
        directory.join(format!("{damaged}.json")),
        "broken private data",
    )
    .unwrap();
    assert!(store.load_session(&source.id).is_err());
    let inventory = store.session_inventory().unwrap();
    assert_eq!(inventory.len(), 2);
    assert_eq!(store.sessions().unwrap(), vec![source.id.clone()]);
    assert!(inventory.iter().any(|row| row["state"] == "unreadable"));
    assert!(
        !serde_json::to_string(&inventory)
            .unwrap()
            .contains("broken private data")
    );
    let bytes = std::fs::read(&path).unwrap();
    let mut offset = 0;
    let mut seen = 0;
    loop {
        let page = store.inspect_session(&source.id, offset).unwrap();
        assert_eq!(page["catalog_compatible"], false);
        assert!(serde_json::to_vec(&page["events"]).unwrap().len() <= 16384);
        seen += page["events"].as_array().unwrap().len();
        match page["next_offset"].as_u64() {
            Some(next) => {
                assert!(next as usize > offset);
                offset = next as usize;
            }
            None => break,
        }
    }
    assert_eq!(seen, 50);
    let page = store.inspect_session(&source.id, 0).unwrap();
    let recovered = store
        .recover_session(
            &source.id,
            page["ticket"].as_str().unwrap(),
            "Review current sources",
        )
        .unwrap();
    assert!(store.load_session(&recovered.id).is_ok());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
