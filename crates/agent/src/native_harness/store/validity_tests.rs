use super::*;
use serde_json::json;

#[test]
fn derived_memory_requires_unchanged_catalog_and_source_summary() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let summary = json!({"decisions":["Theme uses semantic colors"]});
    session.summary = Some(summary.to_string());
    store.save_session(&mut session).unwrap();
    let id = store
        .propose_decisions(&session.id, &summary)
        .unwrap()
        .remove(0);
    store.confirm_memory(&id).unwrap();
    assert_eq!(store.recall("Theme").unwrap().len(), 1);
    session.summary = Some(json!({"decisions":["Theme decision corrected"]}).to_string());
    store.save_session(&mut session).unwrap();
    assert!(store.recall("Theme").unwrap().is_empty());
    assert_eq!(
        store.memory_status().unwrap()[0]["reason"],
        "source_summary_changed"
    );
    assert!(store.confirm_memory(&id).is_err());
    store.delete_session(&session.id).unwrap();
    assert_eq!(store.observations().unwrap().len(), 1);
    assert_eq!(
        store.memory_status().unwrap()[0]["reason"],
        "source_unavailable"
    );
    store
        .update_memory(&id, "Theme", "Theme decision reviewed locally", &[])
        .unwrap();
    assert_eq!(store.recall("Theme").unwrap().len(), 1);
    let path = store.memory_path().unwrap();
    let mut memories = store.memories().unwrap();
    memories.observations[0].validity.catalog = Some("previous-catalog".into());
    write_private_json(&path, &memories).unwrap();
    assert_eq!(
        store.memory_status().unwrap()[0]["reason"],
        "catalog_changed"
    );
    assert!(store.recall("Theme").unwrap().is_empty());
}

#[test]
fn invalidation_and_stale_relinking_require_review_not_implicit_confirmation() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    fs::write(root.path().join("theme.dowe"), "theme\n").unwrap();
    let id = store
        .remember("Theme", "Theme uses semantic colors", "user", true)
        .unwrap();
    store
        .update_memory(
            &id,
            "Theme",
            "Theme uses semantic colors",
            &["theme.dowe".into()],
        )
        .unwrap();
    fs::write(root.path().join("theme.dowe"), "theme changed\n").unwrap();
    assert!(store.recall("Theme").unwrap().is_empty());
    store.link_memory(&id, &["theme.dowe".into()]).unwrap();
    assert!(!store.observations().unwrap()[0].confirmed);
    assert!(store.recall("Theme").unwrap().is_empty());
    store.confirm_memory(&id).unwrap();
    store.invalidate_memory(&id, "Decision superseded").unwrap();
    assert!(store.recall("Theme").unwrap().is_empty());
    assert_eq!(store.memory_status().unwrap()[0]["reason"], "invalidated");
    assert!(store.confirm_memory(&id).is_err());
    store
        .update_memory(&id, "Theme", "Theme uses the revised palette", &[])
        .unwrap();
    assert_eq!(store.recall("Theme").unwrap().len(), 1);
}

#[test]
fn legacy_local_decisions_remain_usable_but_missing_derived_proof_is_not_invented() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    store
        .remember("Theme", "Theme local decision", "user", true)
        .unwrap();
    store
        .remember(
            "Theme",
            "Theme old derived decision",
            "session:unknown",
            true,
        )
        .unwrap();
    let path = store.memory_path().unwrap();
    let mut data = serde_json::to_value(store.memories().unwrap()).unwrap();
    for item in data["observations"].as_array_mut().unwrap() {
        item.as_object_mut().unwrap().remove("validity");
    }
    write_private_json(&path, &data).unwrap();
    assert_eq!(store.recall("Theme").unwrap().len(), 1);
    assert_eq!(
        store.memory_status().unwrap()[1]["reason"],
        "source_proof_missing"
    );
}
