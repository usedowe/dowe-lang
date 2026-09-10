use dowe_agent::native_harness::{
    HarnessStore, SessionExtensionCall, SessionObserver,
    MAX_EXTENSION_CALLS,
};
use serde_json::json;
use tempfile::TempDir;

fn registry() -> (TempDir, TempDir, dowe_agent::native_harness::SessionExtensionRegistry, String) {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let store = HarnessStore::new(home.path(), project.path()).unwrap();
    let mut session = store.create_session().unwrap();
    session.events.push(json!({"event":"response", "prompt":"secret", "secret":"secret"}));
    store.save_session(&mut session).unwrap();
    let observer = SessionObserver::new(store, session.id.clone()).unwrap();
    let handle = observer.subscribe();
    let registry = observer.create_extension_registry(handle).unwrap();
    (home, project, registry, session.id)
}

#[test]
fn exposes_deterministic_fixed_read_only_registry_and_redacted_events() {
    let (_home, _project, registry, session_id) = registry();
    assert_eq!(registry.descriptors().len(), 1);
    assert_eq!(registry.fingerprint(), dowe_agent::native_harness::session_extension_descriptor_fingerprint());
    let result = registry.call(SessionExtensionCall {
        session_id,
        tool_id: "events.poll".into(),
        arguments: json!({"since": 0}),
    }).unwrap();
    let encoded = serde_json::to_string(&result).unwrap();
    assert!(encoded.contains("response"));
    assert!(!encoded.contains("secret"));
    assert!(!encoded.contains("prompt"));
}

#[test]
fn rejects_foreign_unknown_malformed_and_closed_calls() {
    let (_home, _project, registry, session_id) = registry();
    let call = |id: &str, args| registry.call(SessionExtensionCall { session_id: session_id.clone(), tool_id: id.into(), arguments: args });
    assert!(call("missing", json!({})).is_err());
    assert!(call("events.poll", json!({})).is_err());
    assert!(registry.call(SessionExtensionCall { session_id: "foreign".into(), tool_id: "events.poll".into(), arguments: json!({"since": 0}) }).is_err());
    registry.close();
    assert!(call("events.poll", json!({"since": 0})).is_err());
}

#[test]
fn enforces_call_bound() {
    let (_home, _project, registry, session_id) = registry();
    for _ in 0..MAX_EXTENSION_CALLS {
        let _ = registry.call(SessionExtensionCall { session_id: session_id.clone(), tool_id: "events.poll".into(), arguments: json!({"since": 0}) });
    }
    assert!(registry.call(SessionExtensionCall { session_id, tool_id: "events.poll".into(), arguments: json!({"since": 0}) }).is_err());
}

#[test]
fn observer_unsubscribe_disables_registry() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let store = HarnessStore::new(home.path(), project.path()).unwrap();
    let session = store.create_session().unwrap();
    let observer = SessionObserver::new(store, session.id.clone()).unwrap();
    let handle = observer.subscribe();
    let registry = observer.create_extension_registry(handle).unwrap();
    // A second subscription invalidates the registry's observer handle.
    let _replacement = observer.subscribe();
    assert!(registry.call(SessionExtensionCall { session_id: session.id, tool_id: "events.poll".into(), arguments: json!({"since": 0}) }).is_err());
}
