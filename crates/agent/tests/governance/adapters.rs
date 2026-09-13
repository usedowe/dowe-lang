use super::support::{attach_worker, binding, AdapterHost};
use dowe_agent::native_harness::{
    run_orchestrated_turn, HarnessConfig, HarnessOutcome, HarnessRole, HarnessStore, ModelSelection,
};
use dowe_agent_harness::{CodeGraphBinding, WorkerRole};
use serde_json::json;
use std::collections::VecDeque;

#[tokio::test]
async fn adapter_rejects_foreign_session_task_and_worker() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let (task_id, worker_id) =
        attach_worker(&store, &session.id, binding(1), WorkerRole::Mutating, "src");
    let selection = ModelSelection::new("openai", "gpt-5.5");
    let mut host = AdapterHost {
        responses: VecDeque::new(),
        approvals: 0,
        events: vec![],
    };
    assert!(
        run_orchestrated_turn(
            &store,
            "foreign",
            &task_id,
            &worker_id,
            binding(1),
            "x",
            HarnessRole::Plan,
            &selection,
            None,
            &[],
            &HarnessConfig::default(),
            &mut host
        )
        .await
        .is_err()
    );
    assert!(
        run_orchestrated_turn(
            &store,
            &session.id,
            "foreign-task",
            &worker_id,
            binding(1),
            "x",
            HarnessRole::Plan,
            &selection,
            None,
            &[],
            &HarnessConfig::default(),
            &mut host
        )
        .await
        .is_err()
    );
    assert!(
        run_orchestrated_turn(
            &store,
            &session.id,
            &task_id,
            "foreign-worker",
            binding(1),
            "x",
            HarnessRole::Plan,
            &selection,
            None,
            &[],
            &HarnessConfig::default(),
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!(host.approvals, 0);
}

#[tokio::test]
async fn adapter_rejects_expected_binding_mismatch_before_provider_execution() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let (task_id, worker_id) =
        attach_worker(&store, &session.id, binding(1), WorkerRole::Mutating, "src");
    let selection = ModelSelection::new("openai", "gpt-5.5");
    let mut host = AdapterHost {
        responses: VecDeque::new(),
        approvals: 0,
        events: vec![],
    };
    let error = run_orchestrated_turn(
        &store,
        &session.id,
        &task_id,
        &worker_id,
        binding(1),
        "x",
        HarnessRole::Plan,
        &selection,
        None,
        &[],
        &HarnessConfig::default(),
        &mut host,
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("persisted CodeGraphBinding"));
    assert_eq!(host.approvals, 0);
}

#[tokio::test]
async fn adapter_forwards_worker_scope_and_keeps_approval_and_delivery_host_owned() {
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
    let (task_id, worker_id) = attach_worker(
        &store,
        &session.id,
        graph_binding.clone(),
        WorkerRole::Mutating,
        "src",
    );
    let selection = ModelSelection::new("openai", "gpt-5.5");
    let write = json!({"output":[{"type":"function_call","call_id":"write-1","name":"write_file","arguments":json!({"path":"src/generated.dowe","content":"generated {}\n","skill":"core","reason":"bounded test"}).to_string()}]});
    let mut host = AdapterHost {
        responses: VecDeque::from([write]),
        approvals: 0,
        events: vec![],
    };
    let outcome = run_orchestrated_turn(
        &store,
        &session.id,
        &task_id,
        &worker_id,
        graph_binding,
        "write",
        HarnessRole::Execute,
        &selection,
        None,
        &[],
        &HarnessConfig::default(),
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(outcome, HarnessOutcome::ApprovalRequired));
    assert_eq!(host.approvals, 1);
    assert!(!root.path().join("src/generated.dowe").exists());
    let reloaded = store.load_session(&session.id).unwrap();
    let task = &reloaded.orchestration().unwrap().tasks[0];
    assert_eq!(
        task.workers[0].state,
        dowe_agent_harness::WorkerState::Running
    );
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
