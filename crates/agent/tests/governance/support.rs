use dowe_agent::native_harness::{
    Approval, ChildExecutionRequest, HarnessHost, HarnessRole, HarnessStore, ModelSelection,
    SessionRecord,
};
use dowe_agent::{AgentError, AgentRequest, AgentResult, AgentServerResponse};
use dowe_agent_harness::{
    AgentRecord, AgentRole, AllowedEditSurface, CodeGraphBinding, TaskRecord, WorkerRecord,
    WorkerRole,
};
use dowe_codegraph::CodeGraphMode;
use serde_json::{Value, json};
use std::collections::VecDeque;

pub(crate) fn binding(revision: u64) -> CodeGraphBinding {
    CodeGraphBinding {
        generation: "g1".into(),
        revision,
        root: "/project".into(),
        mode: CodeGraphMode::Project,
    }
}

pub(crate) struct AdapterHost {
    pub(crate) responses: VecDeque<Value>,
    pub(crate) approvals: usize,
    pub(crate) events: Vec<Value>,
}

impl HarnessHost for AdapterHost {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.events
            .push(json!({"event":"provider_request","extra":request.extra}));
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: self
                .responses
                .pop_front()
                .ok_or_else(|| AgentError::new("unexpected provider call"))?,
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

pub(crate) fn attach_worker(
    store: &HarnessStore,
    session_id: &str,
    binding: CodeGraphBinding,
    role: WorkerRole,
    scope: &str,
) -> (String, String) {
    let mut session = store.load_session(session_id).unwrap();
    let worker_id = "worker-1".to_owned();
    let task_id = "task-1".to_owned();
    let mut record = SessionRecord::new(session_id, binding.clone()).unwrap();
    record
        .add_agent(AgentRecord::new(&worker_id, AgentRole::Worker).unwrap())
        .unwrap();
    let mut task = TaskRecord::new(
        &task_id,
        binding,
        vec![AllowedEditSurface::new(scope).unwrap()],
    )
    .unwrap();
    task.add_worker(
        WorkerRecord::new(
            &worker_id,
            role,
            vec![AllowedEditSurface::new(scope).unwrap()],
            true,
        )
        .unwrap(),
    )
    .unwrap();
    record.add_task(task).unwrap();
    session.attach_orchestration(record).unwrap();
    store.save_session(&mut session).unwrap();
    (task_id, worker_id)
}

pub(crate) fn attach_child(
    store: &HarnessStore,
    session_id: &str,
    binding: CodeGraphBinding,
    scope: &str,
) -> (String, String, String) {
    let mut session = store.load_session(session_id).unwrap();
    let parent_id = "coordinator-1".to_owned();
    let child_id = "child-1".to_owned();
    let task_id = "child-task-1".to_owned();
    let mut record = SessionRecord::new(session_id, binding.clone()).unwrap();
    record
        .add_agent(AgentRecord::new(&parent_id, AgentRole::Coordinator).unwrap())
        .unwrap();
    record
        .add_agent(AgentRecord::new_child(&child_id, &parent_id).unwrap())
        .unwrap();
    let mut task = TaskRecord::new(
        &task_id,
        binding,
        vec![AllowedEditSurface::new(scope).unwrap()],
    )
    .unwrap();
    task.add_worker(
        WorkerRecord::new(
            &child_id,
            WorkerRole::Mutating,
            vec![AllowedEditSurface::new(scope).unwrap()],
            true,
        )
        .unwrap(),
    )
    .unwrap();
    record.add_task(task).unwrap();
    session.attach_orchestration(record).unwrap();
    store.save_session(&mut session).unwrap();
    (task_id, parent_id, child_id)
}

pub(crate) fn child_request(
    session_id: &str,
    task_id: &str,
    parent_id: &str,
    child_id: &str,
    binding: CodeGraphBinding,
    scope: &str,
) -> ChildExecutionRequest {
    ChildExecutionRequest {
        session_id: session_id.into(),
        task_id: task_id.into(),
        worker_id: child_id.into(),
        parent_agent_id: parent_id.into(),
        child_agent_id: child_id.into(),
        codegraph_binding: binding,
        prompt: "child task".into(),
        role: HarnessRole::Plan,
        active: ModelSelection::new("openai", "gpt-5.5"),
        explicit: None,
        edit_scope: vec![AllowedEditSurface::new(scope).unwrap()],
    }
}
