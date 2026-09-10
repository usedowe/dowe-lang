use dowe_agent::native_harness::{
    run_child_turn, run_orchestrated_turn, Approval, ChildExecutionRequest, HarnessConfig, HarnessHost, HarnessOutcome, HarnessRole,
    HarnessStore, ModelSelection, SessionRecord,
};
use dowe_agent::{AgentError, AgentRequest, AgentResult, AgentServerResponse};
use dowe_agent_harness::{AgentExecutionKind, AgentRecord, AgentRole, AllowedEditSurface, CodeGraphBinding, TaskRecord, WorkerRecord, WorkerRole};
use dowe_codegraph::CodeGraphMode;
use serde_json::{Value, json};
use std::collections::VecDeque;

fn binding(revision: u64) -> CodeGraphBinding {
    CodeGraphBinding {
        generation: "g1".into(),
        revision,
        root: "/project".into(),
        mode: CodeGraphMode::Project,
    }
}

struct AdapterHost {
    responses: VecDeque<Value>,
    approvals: usize,
    events: Vec<Value>,
}

impl HarnessHost for AdapterHost {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.events.push(json!({"event":"provider_request","extra":request.extra}));
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: self.responses.pop_front().ok_or_else(|| AgentError::new("unexpected provider call"))?,
        })
    }

    async fn approve(&mut self, _: &Approval) -> AgentResult<Option<bool>> {
        self.approvals += 1;
        Ok(None)
    }

    fn event(&mut self, event: &Value) -> AgentResult<()> {
        self.events.push(event.clone());
        Ok(())
    }
}

fn attach_worker(store: &HarnessStore, session_id: &str, binding: CodeGraphBinding, role: WorkerRole, scope: &str) -> (String, String) {
    let mut session = store.load_session(session_id).unwrap();
    let worker_id = "worker-1".to_owned();
    let task_id = "task-1".to_owned();
    let mut record = SessionRecord::new(session_id, binding.clone()).unwrap();
    record.add_agent(AgentRecord::new(&worker_id, AgentRole::Worker).unwrap()).unwrap();
    let mut task = TaskRecord::new(&task_id, binding, vec![AllowedEditSurface::new(scope).unwrap()]).unwrap();
    task.add_worker(WorkerRecord::new(&worker_id, role, vec![AllowedEditSurface::new(scope).unwrap()], true).unwrap()).unwrap();
    record.add_task(task).unwrap();
    session.attach_orchestration(record).unwrap();
    store.save_session(&mut session).unwrap();
    (task_id, worker_id)
}

fn attach_child(store: &HarnessStore, session_id: &str, binding: CodeGraphBinding, scope: &str) -> (String, String, String) {
    let mut session = store.load_session(session_id).unwrap();
    let parent_id = "coordinator-1".to_owned();
    let child_id = "child-1".to_owned();
    let task_id = "child-task-1".to_owned();
    let mut record = SessionRecord::new(session_id, binding.clone()).unwrap();
    record.add_agent(AgentRecord::new(&parent_id, AgentRole::Coordinator).unwrap()).unwrap();
    record.add_agent(AgentRecord::new_child(&child_id, &parent_id).unwrap()).unwrap();
    let mut task = TaskRecord::new(&task_id, binding, vec![AllowedEditSurface::new(scope).unwrap()]).unwrap();
    task.add_worker(WorkerRecord::new(&child_id, WorkerRole::Mutating, vec![AllowedEditSurface::new(scope).unwrap()], true).unwrap()).unwrap();
    record.add_task(task).unwrap();
    session.attach_orchestration(record).unwrap();
    store.save_session(&mut session).unwrap();
    (task_id, parent_id, child_id)
}

fn child_request(session_id: &str, task_id: &str, parent_id: &str, child_id: &str, binding: CodeGraphBinding, scope: &str) -> ChildExecutionRequest {
    ChildExecutionRequest {
        session_id: session_id.into(), task_id: task_id.into(), worker_id: child_id.into(),
        parent_agent_id: parent_id.into(), child_agent_id: child_id.into(),
        codegraph_binding: binding, prompt: "child task".into(), role: HarnessRole::Plan,
        active: ModelSelection::new("openai", "gpt-5.5"), explicit: None,
        edit_scope: vec![AllowedEditSurface::new(scope).unwrap()],
    }
}

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

    assert!(store.load_session(&session.id).unwrap().orchestration().is_none());
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
    assert_eq!(store.load_session(&session.id).unwrap().orchestration(), Some(&updated));
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
    assert!(session
        .update_orchestration(SessionRecord::new("foreign", binding(2)).unwrap())
        .is_err());
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

    assert_eq!(store.load_session(&first.id).unwrap().orchestration(), Some(&first_record));
    assert_eq!(store.load_session(&second.id).unwrap().orchestration(), Some(&second_record));

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

#[tokio::test]
async fn adapter_rejects_foreign_session_task_and_worker() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let (task_id, worker_id) = attach_worker(&store, &session.id, binding(1), WorkerRole::Mutating, "src");
    let selection = ModelSelection::new("openai", "gpt-5.5");
    let mut host = AdapterHost { responses: VecDeque::new(), approvals: 0, events: vec![] };
    assert!(run_orchestrated_turn(&store, "foreign", &task_id, &worker_id, binding(1), "x", HarnessRole::Plan, &selection, None, &[], &HarnessConfig::default(), &mut host).await.is_err());
    assert!(run_orchestrated_turn(&store, &session.id, "foreign-task", &worker_id, binding(1), "x", HarnessRole::Plan, &selection, None, &[], &HarnessConfig::default(), &mut host).await.is_err());
    assert!(run_orchestrated_turn(&store, &session.id, &task_id, "foreign-worker", binding(1), "x", HarnessRole::Plan, &selection, None, &[], &HarnessConfig::default(), &mut host).await.is_err());
    assert_eq!(host.approvals, 0);
}

#[tokio::test]
async fn adapter_rejects_expected_binding_mismatch_before_provider_execution() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let (task_id, worker_id) = attach_worker(&store, &session.id, binding(1), WorkerRole::Mutating, "src");
    let selection = ModelSelection::new("openai", "gpt-5.5");
    let mut host = AdapterHost { responses: VecDeque::new(), approvals: 0, events: vec![] };
    let error = run_orchestrated_turn(&store, &session.id, &task_id, &worker_id, binding(1), "x", HarnessRole::Plan, &selection, None, &[], &HarnessConfig::default(), &mut host).await.unwrap_err();
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
    let graph_binding = CodeGraphBinding { generation: persisted.generation.unwrap(), revision: persisted.manifest.revision, root: persisted.manifest.root, mode: persisted.manifest.mode };
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let (task_id, worker_id) = attach_worker(&store, &session.id, graph_binding.clone(), WorkerRole::Mutating, "src");
    let selection = ModelSelection::new("openai", "gpt-5.5");
    let write = json!({"output":[{"type":"function_call","call_id":"write-1","name":"write_file","arguments":json!({"path":"src/generated.dowe","content":"generated {}\n","skill":"core","reason":"bounded test"}).to_string()}]});
    let mut host = AdapterHost { responses: VecDeque::from([write]), approvals: 0, events: vec![] };
    let outcome = run_orchestrated_turn(&store, &session.id, &task_id, &worker_id, graph_binding, "write", HarnessRole::Execute, &selection, None, &[], &HarnessConfig::default(), &mut host).await.unwrap();
    assert!(matches!(outcome, HarnessOutcome::ApprovalRequired));
    assert_eq!(host.approvals, 1);
    assert!(!root.path().join("src/generated.dowe").exists());
    let reloaded = store.load_session(&session.id).unwrap();
    let task = &reloaded.orchestration().unwrap().tasks[0];
    assert_eq!(task.workers[0].state, dowe_agent_harness::WorkerState::Running);
    assert!(task.receipts.is_empty());
    assert_eq!(task.acknowledgement_state, dowe_agent_harness::AcknowledgementState::Pending);
    assert_eq!(task.delivery_state, dowe_agent_harness::DeliveryState::NotRequested);
    }


#[test]
fn child_registration_requires_a_same_session_coordinator_and_rejects_recursive_children() {
    let mut session = SessionRecord::new("s1", binding(1)).unwrap();
    assert!(session.add_agent(AgentRecord::new_child("child", "missing").unwrap()).is_err());
    session.add_agent(AgentRecord::new("coordinator", AgentRole::Coordinator).unwrap()).unwrap();
    let child = AgentRecord::new_child("child", "coordinator").unwrap();
    assert_eq!(child.execution_kind, AgentExecutionKind::Child);
    session.add_agent(child).unwrap();
    assert!(session.add_agent(AgentRecord::new_child("grandchild", "child").unwrap()).is_err());
    assert!(session.add_agent(AgentRecord::new_child("child", "coordinator").unwrap()).is_err());
}

#[tokio::test]
async fn child_turn_rejects_foreign_identity_and_exact_binding_mismatch_before_provider() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let (task_id, parent_id, child_id) = attach_child(&store, &session.id, binding(1), "src");
    let mut host = AdapterHost { responses: VecDeque::new(), approvals: 0, events: vec![] };
    let mut request = child_request(&session.id, &task_id, &parent_id, &child_id, binding(2), "src");
    assert!(run_child_turn(&store, request.clone(), &HarnessConfig::default(), &mut host).await.is_err());
    request.codegraph_binding = binding(1);
    request.child_agent_id = "foreign-child".into();
    assert!(run_child_turn(&store, request, &HarnessConfig::default(), &mut host).await.is_err());
    assert!(host.events.is_empty());
    assert_eq!(host.approvals, 0);
}

#[tokio::test]
async fn child_turn_isolated_context_persists_identity_and_never_governs_delivery() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    let persisted = dowe_codegraph::ensure_persistent_codegraph(root.path()).unwrap();
    let graph_binding = CodeGraphBinding { generation: persisted.generation.unwrap(), revision: persisted.manifest.revision, root: persisted.manifest.root, mode: persisted.manifest.mode };
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let (task_id, parent_id, child_id) = attach_child(&store, &session.id, graph_binding.clone(), "src");
    let mut stored = store.load_session(&session.id).unwrap();
    stored.summary = Some("parent-only context".into());
    let parent_turns = stored.turns.len();
    store.save_session(&mut stored).unwrap();
    let mut host = AdapterHost { responses: VecDeque::from([json!({"output_text":"child complete"})]), approvals: 0, events: vec![] };
    let request = child_request(&session.id, &task_id, &parent_id, &child_id, graph_binding, "src");
    let outcome = run_child_turn(&store, request, &HarnessConfig::default(), &mut host).await.unwrap();
    assert!(matches!(outcome, HarnessOutcome::Completed));
    let provider_request = host.events.iter().find(|event| event["event"] == "provider_request").unwrap();
    assert!(!provider_request["extra"]["dowe_harness_turns"].to_string().contains("parent-only context"));
    let reloaded = store.load_session(&session.id).unwrap();
    assert_eq!(reloaded.turns.len(), parent_turns);
    assert!(reloaded.events.iter().any(|event| event["event"] == "response_received" && event["session_id"] == session.id && event["task_id"] == task_id && event["agent_id"] == child_id));
    let task = &reloaded.orchestration().unwrap().tasks[0];
    assert_eq!(task.acknowledgement_state, dowe_agent_harness::AcknowledgementState::Pending);
    assert_eq!(task.delivery_state, dowe_agent_harness::DeliveryState::NotRequested);
}

#[tokio::test]
async fn child_turn_forwards_exact_scope_and_keeps_approval_host_owned() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("src")).unwrap();
    std::fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    let persisted = dowe_codegraph::ensure_persistent_codegraph(root.path()).unwrap();
    let graph_binding = CodeGraphBinding { generation: persisted.generation.unwrap(), revision: persisted.manifest.revision, root: persisted.manifest.root, mode: persisted.manifest.mode };
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    let (task_id, parent_id, child_id) = attach_child(&store, &session.id, graph_binding.clone(), "src");
    let write = json!({"output":[{"type":"function_call","call_id":"child-write","name":"write_file","arguments":json!({"path":"src/generated.dowe","content":"generated {}\n","skill":"core","reason":"child scope test"}).to_string()}]});
    let mut host = AdapterHost { responses: VecDeque::from([write]), approvals: 0, events: vec![] };
    let mut request = child_request(&session.id, &task_id, &parent_id, &child_id, graph_binding, "src");
    request.role = HarnessRole::Execute;
    let outcome = run_child_turn(&store, request, &HarnessConfig::default(), &mut host).await.unwrap();
    assert!(matches!(outcome, HarnessOutcome::ApprovalRequired));
    assert_eq!(host.approvals, 1);
    assert!(!root.path().join("src/generated.dowe").exists());
    let reloaded = store.load_session(&session.id).unwrap();
    let task = &reloaded.orchestration().unwrap().tasks[0];
    assert!(task.receipts.is_empty());
    assert_eq!(task.acknowledgement_state, dowe_agent_harness::AcknowledgementState::Pending);
    assert_eq!(task.delivery_state, dowe_agent_harness::DeliveryState::NotRequested);
}

#[test]
fn project_skill_loading_is_bounded_untrusted_and_hash_paged() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join(".agents/skills/local");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("SKILL.md"), "first\nsecond\nthird\n").unwrap();
    let tools = dowe_agent::native_harness::HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    let first = tools.execute_read(&dowe_agent::native_harness::ToolCall::new("skill", "get_skill", json!({"id":"local","source":"project"}))).unwrap();
    assert_eq!(first["source"], "project");
    assert_eq!(first["relative_path"], ".agents/skills/local/SKILL.md");
    assert_eq!(first["untrusted"], true);
    let hash = first["hash"].as_str().unwrap();
    let next = tools.execute_read(&dowe_agent::native_harness::ToolCall::new("skill", "get_skill", json!({"id":"local","source":"project","offset":2,"hash":hash}))).unwrap();
    assert_eq!(next["content"], "second\nthird\n");
    assert!(tools.execute_read(&dowe_agent::native_harness::ToolCall::new("skill", "get_skill", json!({"id":"local","source":"project","offset":2}))).is_err());
    assert!(tools.execute_read(&dowe_agent::native_harness::ToolCall::new("skill", "get_skill", json!({"id":"local","source":"project","offset":2,"hash":"stale"}))).is_err());
}

#[test]
fn project_skill_selection_rejects_collisions_resources_and_invalid_ids() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join(".agents/skills/core");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("SKILL.md"), "shadow\n").unwrap();
    let tools = dowe_agent::native_harness::HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    let collision = tools.execute_read(&dowe_agent::native_harness::ToolCall::new("skill", "get_skill", json!({"id":"core","source":"project"}))).unwrap_err();
    assert!(collision.to_string().contains("collides"));
    for args in [json!({"id":"core","source":"project","resource":"SKILL.md"}), json!({"id":"../escape","source":"project"}), json!({"id":"missing","source":"project"})] {
        assert!(tools.execute_read(&dowe_agent::native_harness::ToolCall::new("skill", "get_skill", args)).is_err());
    }
}
