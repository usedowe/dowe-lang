use dowe_ipc::*;
use serde_json::{Value, json};

#[tokio::test]
async fn ipc_draft_rejects_non_json_without_saving_or_approval() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = AgentHarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = Host::default();
    assert!(
        draft_agent_workflow_plan(
            &store,
            &AgentHarnessConfig::default(),
            &AgentModelSelection::new("openai", "gpt-5.5"),
            "draft",
            "Build login",
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!((host.calls, host.approvals), (1, 0));
    assert!(!root.path().join(".agent/plans/draft.json").exists());
}

#[derive(Default)]
struct Host {
    calls: usize,
    approvals: usize,
    events: Vec<Value>,
}

impl AgentHarnessHost for Host {
    async fn send(
        &mut self,
        request: &AgentRequest,
    ) -> dowe_agent::AgentResult<AgentServerResponse> {
        self.calls += 1;
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: json!({"output_text":"Read-only answer."}),
        })
    }
    async fn approve(&mut self, _: &AgentApproval) -> dowe_agent::AgentResult<Option<bool>> {
        self.approvals += 1;
        Ok(Some(false))
    }
    fn event(&mut self, event: &Value) -> dowe_agent::AgentResult<()> {
        self.events.push(event.clone());
        Ok(())
    }
}

#[tokio::test]
async fn ipc_dispatches_plan_roles_without_mutation_authority() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = AgentHarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let mut host = Host::default();
    let active = AgentModelSelection::new("openai", "gpt-5.5");
    let result = run_agent_task(
        &store,
        &mut session,
        &AgentHarnessConfig::default(),
        AgentHarnessTask::new("/plan Build login", &active),
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(result, AgentHarnessOutcome::Completed));
    assert_eq!(host.approvals, 0);
    assert!(
        host.events
            .iter()
            .any(|e| e["event"] == "request_prepared" && e["role"] == "plan")
    );
}

#[tokio::test]
async fn ipc_unanswered_requirement_never_requests_approval_or_provider() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = AgentHarnessStore::new(home.path(), root.path()).unwrap();
    let plan: WorkflowPlan = serde_json::from_value(json!({"id":"ipc-question","objective":"Build feature",
        "requirements":[{"id":"region","question":"Which region?","blocking":true,"answer":null}],
        "tasks":[{"id":"code","objective":"Implement feature","dependencies":[],"write_scopes":["src"]}],
        "checks":[{"criterion":"works","test_paths":["tests/check.dowe"]}]})).unwrap();
    let mut host = Host::default();
    let result = run_agent_workflow(
        &store,
        &AgentHarnessConfig::default(),
        &AgentModelSelection::new("openai", "gpt-5.5"),
        &plan,
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(
        result,
        dowe_agent_harness::coordinator::DriveOutcome::AwaitingRequirements
    );
    assert_eq!((host.calls, host.approvals), (0, 0));
    assert!(
        root.path()
            .join(".agent/tasks/ipc-question.requirements.json")
            .is_file()
    );
}

#[tokio::test]
async fn ipc_build_plan_uses_shared_approval_and_checkpoint() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = AgentHarnessStore::new(home.path(), root.path()).unwrap();
    std::fs::write(root.path().join("plan.json"), json!({"id":"ipc-feature","objective":"Build feature",
        "requirements":[],"tasks":[{"id":"code","objective":"Implement feature","dependencies":[],"write_scopes":["src"]}],
        "checks":[{"criterion":"works","test_paths":["tests/check.dowe"]}]}).to_string()).unwrap();
    let mut session = store.create_session().unwrap();
    let mut host = Host::default();
    let active = AgentModelSelection::new("openai", "gpt-5.5");
    let result = run_agent_task(
        &store,
        &mut session,
        &AgentHarnessConfig::default(),
        AgentHarnessTask::new("/build-plan plan.json", &active),
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(result, AgentHarnessOutcome::ApprovalRequired));
    assert_eq!(host.calls, 0);
    assert_eq!(host.approvals, 1);
    let checkpoint = inspect_agent_workflow(&store, "ipc-feature").unwrap();
    assert_eq!(
        checkpoint.coordinator.phase(),
        dowe_agent_harness::coordinator::Phase::AwaitingApproval
    );
    assert!(checkpoint.can_resume());
    let result = resume_agent_workflow(
        &store,
        &AgentHarnessConfig::default(),
        &active,
        "ipc-feature",
        checkpoint.sequence,
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(
        result,
        dowe_agent_harness::coordinator::DriveOutcome::AwaitingApproval
    );
    let current = inspect_agent_workflow(&store, "ipc-feature").unwrap();
    let prompt = format!("/resume-workflow ipc-feature {}", current.sequence);
    let result = run_agent_task(
        &store,
        &mut session,
        &AgentHarnessConfig::default(),
        AgentHarnessTask::new(&prompt, &active),
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(result, AgentHarnessOutcome::ApprovalRequired));
    assert_eq!(host.calls, 0);
    assert_eq!(host.approvals, 3);
}

#[tokio::test]
async fn readonly_role_cannot_escalate_through_build_command() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = AgentHarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let mut host = Host::default();
    let active = AgentModelSelection::new("openai", "gpt-5.5");
    let mut task = AgentHarnessTask::new("/build-plan plan.json", &active);
    task.role = AgentHarnessRole::Research;
    assert!(
        run_agent_task(
            &store,
            &mut session,
            &AgentHarnessConfig::default(),
            task,
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!(host.calls, 0);
    assert_eq!(host.approvals, 0);
    let mut task = AgentHarnessTask::new("/resume-workflow target 1", &active);
    task.role = AgentHarnessRole::Research;
    assert!(
        run_agent_task(
            &store,
            &mut session,
            &AgentHarnessConfig::default(),
            task,
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!(host.calls, 0);
    assert_eq!(host.approvals, 0);
}
