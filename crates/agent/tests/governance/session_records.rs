use super::support::binding;
use dowe_agent::native_harness::{HarnessStore, SessionRecord};
use serde_json::Value;

#[test]
fn legacy_session_json_decodes_without_orchestration() {
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
    let mut json = serde_json::to_value(&session).unwrap();
    json.as_object_mut().unwrap().remove("orchestration");
    std::fs::write(path, serde_json::to_vec(&json).unwrap()).unwrap();

    assert!(
        store
            .load_session(&session.id)
            .unwrap()
            .orchestration()
            .is_none()
    );
}

#[test]
fn orchestration_attaches_updates_and_reloads_through_native_session_storage() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let record = SessionRecord::new(session.id.clone(), binding(1)).unwrap();
    session.attach_orchestration(record.clone()).unwrap();
    store.save_session(&mut session).unwrap();

    let mut reloaded = store.load_session(&session.id).unwrap();
    assert_eq!(reloaded.orchestration(), Some(&record));
    let updated = SessionRecord::new(reloaded.id.clone(), binding(2)).unwrap();
    reloaded.update_orchestration(updated.clone()).unwrap();
    store.save_session(&mut reloaded).unwrap();
    assert_eq!(
        store.load_session(&session.id).unwrap().orchestration(),
        Some(&updated)
    );
}

#[test]
fn orchestration_rejects_mismatched_session_and_replacement_identity() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let foreign = SessionRecord::new("foreign", binding(1)).unwrap();
    assert!(session.attach_orchestration(foreign).is_err());

    session
        .attach_orchestration(SessionRecord::new(session.id.clone(), binding(1)).unwrap())
        .unwrap();
    assert!(
        session
            .update_orchestration(SessionRecord::new("foreign", binding(2)).unwrap())
            .is_err()
    );
}

#[test]
fn independent_native_sessions_keep_independent_orchestration_records() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut first = store.create_session().unwrap();
    let mut second = store.create_session().unwrap();
    let first_record = SessionRecord::new(first.id.clone(), binding(1)).unwrap();
    let second_record = SessionRecord::new(second.id.clone(), binding(2)).unwrap();
    first.attach_orchestration(first_record.clone()).unwrap();
    second.attach_orchestration(second_record.clone()).unwrap();
    store.save_session(&mut first).unwrap();
    store.save_session(&mut second).unwrap();

    assert_eq!(
        store.load_session(&first.id).unwrap().orchestration(),
        Some(&first_record)
    );
    assert_eq!(
        store.load_session(&second.id).unwrap().orchestration(),
        Some(&second_record)
    );

    let raw: Value = serde_json::from_slice(
        &std::fs::read(
            home.path()
                .join("harness")
                .join(&first.project)
                .join("sessions")
                .join(format!("{}.json", first.id)),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(raw.get("orchestration").is_some());
}
