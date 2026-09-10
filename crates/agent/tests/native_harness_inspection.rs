use dowe_agent::native_harness::{HarnessStore, SessionEventLimits, SessionObserver, SessionRecord};
use dowe_agent_harness::CodeGraphBinding;
use dowe_codegraph::CodeGraphMode;

fn binding() -> CodeGraphBinding {
    CodeGraphBinding { generation: "g1".into(), revision: 7, root: "/project".into(), mode: CodeGraphMode::Project }
}

#[test]
fn inspection_is_session_scoped_and_cursor_is_after_cursor() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut first = store.create_session().unwrap();
    let second = store.create_session().unwrap();
    first.events = vec![serde_json::json!({"event":"safe","secret":"omit"}), serde_json::json!({"event":"next"})];
    store.save_session(&mut first).unwrap();
    assert_eq!(store.inspect_session_view(&first.id).unwrap().event_cursor, 2);
    let page = store.inspect_events(&first.id, 1).unwrap();
    assert_eq!(page.since, 1);
    assert_eq!(page.cursor, 2);
    assert_eq!(page.events.len(), 1);
    assert_eq!(page.events[0].kind, "next");
    assert!(store.inspect_events(&first.id, 3).is_err());
    assert!(store.inspect_tasks(&second.id).unwrap().tasks.is_empty());
}

#[test]
fn observer_replacement_and_unsubscribe_are_idempotent() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let observer = SessionObserver::new(store.clone(), session.id.clone()).unwrap();
    let first = observer.subscribe();
    let second = observer.subscribe();
    assert!(!first.is_active());
    assert!(second.is_active());
    assert!(observer.poll(&second, 0, SessionEventLimits::default()).is_ok());
    observer.unsubscribe(&second);
    observer.unsubscribe(&second);
    assert!(!second.is_active());
}

#[test]
fn governance_projection_shows_binding_without_authority() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    session.attach_orchestration(SessionRecord::new(session.id.clone(), binding()).unwrap()).unwrap();
    store.save_session(&mut session).unwrap();
    let view = store.inspect_governance(&session.id).unwrap();
    assert_eq!(view.codegraph_binding, binding());
    assert_eq!(view.authority, "read_only_observation");
    assert!(view.tasks.is_empty());
}
