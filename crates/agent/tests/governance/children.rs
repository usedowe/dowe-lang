use super::support::{AdapterHost, attach_child, binding, child_request};
use dowe_agent::native_harness::{
    HarnessConfig, HarnessOutcome, HarnessRole, HarnessStore, SessionRecord, run_child_turn,
};
use dowe_agent_harness::{AgentExecutionKind, AgentRecord, AgentRole, CodeGraphBinding};
use serde_json::json;
use std::collections::VecDeque;

#[test]
fn child_registration_requires_a_same_session_coordinator_and_rejects_recursive_children() {
    let mut session = SessionRecord::new("s1", binding(1)).unwrap();
    assert!(
        session
            .add_agent(AgentRecord::new_child("child", "missing").unwrap())
            .is_err()
    );
    session
        .add_agent(AgentRecord::new("coordinator", AgentRole::Coordinator).unwrap())
        .unwrap();
    let child = AgentRecord::new_child("child", "coordinator").unwrap();
    assert_eq!(child.execution_kind, AgentExecutionKind::Child);
    session.add_agent(child).unwrap();
    assert!(
        session
            .add_agent(AgentRecord::new_child("grandchild", "child").unwrap())
            .is_err()
    );
    assert!(
        session
            .add_agent(AgentRecord::new_child("child", "coordinator").unwrap())
            .is_err()
    );
}

#[tokio::test]
async fn child_turn_rejects_foreign_identity_and_exact_binding_mismatch_before_provider() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let (task_id, parent_id, child_id) = attach_child(&store, &session.id, binding(1), "src");
    let mut host = AdapterHost {
        responses: VecDeque::new(),
        approvals: 0,
        events: vec![],
    };
    let mut request = child_request(
        &session.id,
        &task_id,
        &parent_id,
        &child_id,
        binding(2),
        "src",
    );
    assert!(
        run_child_turn(
            &store,
            request.clone(),
            &HarnessConfig::default(),
            &mut host
        )
        .await
        .is_err()
    );
    request.codegraph_binding = binding(1);
    request.child_agent_id = "foreign-child".into();
    assert!(
        run_child_turn(&store, request, &HarnessConfig::default(), &mut host)
            .await
            .is_err()
    );
    assert!(host.events.is_empty());
    assert_eq!(host.approvals, 0);
}

#[tokio::test]
async fn child_turn_isolated_context_persists_identity_and_never_governs_delivery() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    let persisted = dowe_codegraph::ensure_persistent_codegraph(root.path()).unwrap();
    let graph_binding = CodeGraphBinding {
        generation: persisted.generation.unwrap(),
        revision: persisted.manifest.revision,
        root: persisted.manifest.root,
        mode: persisted.manifest.mode,
    };
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let (task_id, parent_id, child_id) =
        attach_child(&store, &session.id, graph_binding.clone(), "src");
    let mut stored = store.load_session(&session.id).unwrap();
    stored.summary = Some("parent-only context".into());
    let parent_turns = stored.turns.len();
    store.save_session(&mut stored).unwrap();
    let mut host = AdapterHost {
        responses: VecDeque::from([json!({"output_text":"child complete"})]),
        approvals: 0,
        events: vec![],
    };
    let request = child_request(
        &session.id,
        &task_id,
        &parent_id,
        &child_id,
        graph_binding,
        "src",
    );
    let outcome = run_child_turn(&store, request, &HarnessConfig::default(), &mut host)
        .await
        .unwrap();
    assert!(matches!(outcome, HarnessOutcome::Completed));
    let provider_request = host
        .events
        .iter()
        .find(|event| event["event"] == "provider_request")
        .unwrap();
    assert!(
        !provider_request["extra"]["dowe_harness_turns"]
            .to_string()
            .contains("parent-only context")
    );
    let reloaded = store.load_session(&session.id).unwrap();
    assert_eq!(reloaded.turns.len(), parent_turns);
    assert!(
        reloaded
            .events
            .iter()
            .any(|event| event["event"] == "response_received"
                && event["session_id"] == session.id
                && event["task_id"] == task_id
                && event["agent_id"] == child_id)
    );
    let task = &reloaded.orchestration().unwrap().tasks[0];
    assert_eq!(
        task.acknowledgement_state,
        dowe_agent_harness::AcknowledgementState::Pending
    );
    assert_eq!(
        task.delivery_state,
        dowe_agent_harness::DeliveryState::NotRequested
    );
}

#[tokio::test]
async fn child_turn_forwards_exact_scope_and_keeps_approval_host_owned() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("src")).unwrap();
    std::fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    let persisted = dowe_codegraph::ensure_persistent_codegraph(root.path()).unwrap();
    let graph_binding = CodeGraphBinding {
        generation: persisted.generation.unwrap(),
        revision: persisted.manifest.revision,
        root: persisted.manifest.root,
        mode: persisted.manifest.mode,
    };
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let (task_id, parent_id, child_id) =
        attach_child(&store, &session.id, graph_binding.clone(), "src");
    let write = json!({"output":[{"type":"function_call","call_id":"child-write","name":"write_file","arguments":json!({"path":"src/generated.dowe","content":"generated {}\n","skill":"core","reason":"child scope test"}).to_string()}]});
    let mut host = AdapterHost {
        responses: VecDeque::from([write]),
        approvals: 0,
        events: vec![],
    };
    let mut request = child_request(
        &session.id,
        &task_id,
        &parent_id,
        &child_id,
        graph_binding,
        "src",
    );
    request.role = HarnessRole::Execute;
    let outcome = run_child_turn(&store, request, &HarnessConfig::default(), &mut host)
        .await
        .unwrap();
    assert!(matches!(outcome, HarnessOutcome::ApprovalRequired));
    assert_eq!(host.approvals, 1);
    assert!(!root.path().join("src/generated.dowe").exists());
    let reloaded = store.load_session(&session.id).unwrap();
    let task = &reloaded.orchestration().unwrap().tasks[0];
    assert!(task.receipts.is_empty());
    assert_eq!(
        task.acknowledgement_state,
        dowe_agent_harness::AcknowledgementState::Pending
    );
    assert_eq!(
        task.delivery_state,
        dowe_agent_harness::DeliveryState::NotRequested
    );
}
