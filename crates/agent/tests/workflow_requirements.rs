use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse, ClarificationQuestion};
use dowe_agent_harness::coordinator::DriveOutcome;
use serde_json::{Value, json};
use std::collections::VecDeque;

#[derive(Default)]
struct Host {
    answers: VecDeque<Option<String>>,
    questions: Vec<String>,
    approvals: Vec<Value>,
    execute: bool,
    calls: usize,
}
impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        assert!(
            self.execute && !self.approvals.is_empty(),
            "no provider before approval"
        );
        self.calls += 1;
        let serialized = serde_json::to_string(request).unwrap();
        assert!(serialized.contains("Europe") && serialized.contains("EUR"));
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: json!({"output_text": if self.calls == 1 { "Done." } else { "{\"approved\":true,\"findings\":[]}" }}),
        })
    }
    async fn approve(&mut self, approval: &Approval) -> AgentResult<Option<bool>> {
        self.approvals.push(approval.details.clone());
        Ok(Some(self.execute))
    }
    async fn ask_clarification(
        &mut self,
        question: &ClarificationQuestion,
    ) -> AgentResult<Option<String>> {
        self.questions.push(question.text.clone());
        Ok(self.answers.pop_front().flatten())
    }
    fn event(&mut self, _: &Value) -> AgentResult<()> {
        Ok(())
    }
    fn secrets(&self) -> Vec<String> {
        vec!["fixture-secret-value".into()]
    }
}
fn plan() -> WorkflowPlan {
    serde_json::from_value(json!({"id":"requirements", "objective":"Create feature", "requirements":[
        {"id":"region","question":"Which region?","blocking":true,"answer":null},
        {"id":"currency","question":"Which currency?","blocking":true,"answer":null}
    ], "tasks":[{"id":"build","objective":"Implement feature","dependencies":[],"write_scopes":["src"]}],
    "checks":[{"criterion":"works","test_paths":["tests/check.dowe"]}]})).unwrap()
}

#[tokio::test]
async fn missing_answers_suspend_and_repeat_reuses_durable_decisions() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = Host {
        answers: VecDeque::from([Some("Europe".into()), None]),
        ..Default::default()
    };
    let selection = ModelSelection::new("openai", "gpt-5.5");
    assert_eq!(
        run_workflow(
            &store,
            &HarnessConfig::default(),
            &selection,
            &plan(),
            &mut host
        )
        .await
        .unwrap(),
        DriveOutcome::AwaitingRequirements
    );
    assert!(host.approvals.is_empty());
    assert!(!root.path().join(".agent/tasks/requirements.json").exists());
    let reopened = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut second = Host {
        answers: VecDeque::from([Some("EUR".into())]),
        ..Default::default()
    };
    assert_eq!(
        run_workflow(
            &reopened,
            &HarnessConfig::default(),
            &selection,
            &plan(),
            &mut second
        )
        .await
        .unwrap(),
        DriveOutcome::AwaitingApproval
    );
    assert_eq!(second.questions, ["Which currency?"]);
    assert_eq!(
        second.approvals[0]["plan"]["requirements"][0]["answer"],
        "Europe"
    );
    assert_eq!(
        second.approvals[0]["plan"]["requirements"][1]["answer"],
        "EUR"
    );
    assert!(
        run_workflow(
            &reopened,
            &HarnessConfig::default(),
            &selection,
            &plan(),
            &mut second
        )
        .await
        .is_err()
    );
    assert_eq!(second.questions.len(), 1);
}

#[tokio::test]
async fn changed_draft_and_secret_answers_cannot_be_accepted() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let selection = ModelSelection::new("openai", "gpt-5.5");
    let mut host = Host::default();
    assert_eq!(
        run_workflow(
            &store,
            &HarnessConfig::default(),
            &selection,
            &plan(),
            &mut host
        )
        .await
        .unwrap(),
        DriveOutcome::AwaitingRequirements
    );
    let mut changed = plan();
    changed.objective = "Different feature".into();
    assert!(
        run_workflow(
            &store,
            &HarnessConfig::default(),
            &selection,
            &changed,
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!(host.questions.len(), 1);
    host.answers.push_back(Some("fixture-secret-value".into()));
    assert!(
        run_workflow(
            &store,
            &HarnessConfig::default(),
            &selection,
            &plan(),
            &mut host
        )
        .await
        .is_err()
    );
    let packet = std::fs::read_to_string(
        root.path()
            .join(".agent/tasks/requirements.requirements.json"),
    )
    .unwrap();
    assert!(!packet.contains("fixture-secret-value"));
    assert!(host.approvals.is_empty());
}

#[tokio::test]
async fn invalid_drafts_and_invalid_answers_never_reach_approval() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let selection = ModelSelection::new("openai", "gpt-5.5");
    let mut host = Host::default();
    let mut cyclic = plan();
    cyclic.tasks[0].dependencies.push("build".into());
    assert!(
        run_workflow(
            &store,
            &HarnessConfig::default(),
            &selection,
            &cyclic,
            &mut host
        )
        .await
        .is_err()
    );
    assert!(host.questions.is_empty());
    let mut invalid_path = plan();
    invalid_path.checks[0].test_paths = vec!["../outside.dowe".into()];
    assert!(
        run_workflow(
            &store,
            &HarnessConfig::default(),
            &selection,
            &invalid_path,
            &mut host
        )
        .await
        .is_err()
    );
    assert!(host.questions.is_empty());
    for answer in [" ".to_string(), "a".repeat(8193), "bad\0answer".into()] {
        host.answers.push_back(Some(answer));
        assert!(
            run_workflow(
                &store,
                &HarnessConfig::default(),
                &selection,
                &plan(),
                &mut host
            )
            .await
            .is_err()
        );
    }
    assert!(host.approvals.is_empty());
    let path = root
        .path()
        .join(".agent/tasks/requirements.requirements.json");
    let mut record: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    record["answers"]["foreign"] = json!("injected");
    std::fs::write(&path, serde_json::to_vec(&record).unwrap()).unwrap();
    assert!(
        run_workflow(
            &store,
            &HarnessConfig::default(),
            &selection,
            &plan(),
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!(host.questions.len(), 3);
}

#[tokio::test]
async fn build_command_maps_unanswered_questions_to_clarification() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("plan.json"),
        serde_json::to_vec(&plan()).unwrap(),
    )
    .unwrap();
    assert!(read_workflow_plan(&root.path().canonicalize().unwrap(), "plan.json").is_ok());
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let selection = ModelSelection::new("openai", "gpt-5.5");
    let mut host = Host::default();
    let outcome = run_agent_task(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask {
            prompt: "/build-plan plan.json",
            role: HarnessRole::Execute,
            active: &selection,
            explicit: None,
            image_paths: &[],
            edit_scope: None,
            expected_codegraph_binding: None,
            permission_mode: HarnessPermissionMode::Confirm,
        },
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(outcome, HarnessOutcome::ClarificationRequired));
    assert!(host.approvals.is_empty());
}

#[tokio::test]
async fn optional_unanswered_requirements_do_not_prompt() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut draft = plan();
    for requirement in &mut draft.requirements {
        requirement.blocking = false;
    }
    let mut host = Host::default();
    assert_eq!(
        run_workflow(
            &store,
            &HarnessConfig::default(),
            &ModelSelection::new("openai", "gpt-5.5"),
            &draft,
            &mut host
        )
        .await
        .unwrap(),
        DriveOutcome::AwaitingApproval
    );
    assert!(host.questions.is_empty());
    assert!(
        !root
            .path()
            .join(".agent/tasks/requirements.requirements.json")
            .exists()
    );
}

#[tokio::test]
async fn resolved_decisions_reach_workers_and_reviewer_after_approval() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("tests")).unwrap();
    std::fs::write(
        root.path().join("tests/check.dowe"),
        "test \"works\"\n  assert true value:true\n",
    )
    .unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = Host {
        answers: VecDeque::from([Some("Europe".into()), Some("EUR".into())]),
        execute: true,
        ..Default::default()
    };
    assert_eq!(
        run_workflow(
            &store,
            &HarnessConfig::default(),
            &ModelSelection::new("openai", "gpt-5.5"),
            &plan(),
            &mut host
        )
        .await
        .unwrap(),
        DriveOutcome::Completed
    );
    assert_eq!(host.calls, 2);
}

#[cfg(unix)]
#[tokio::test]
async fn symlinked_requirement_records_are_not_followed() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    std::fs::create_dir_all(root.path().join(".agent/tasks")).unwrap();
    let target = outside.path().join("record.json");
    std::fs::write(&target, "preserve").unwrap();
    std::os::unix::fs::symlink(
        &target,
        root.path()
            .join(".agent/tasks/requirements.requirements.json"),
    )
    .unwrap();
    let mut host = Host::default();
    assert!(
        run_workflow(
            &store,
            &HarnessConfig::default(),
            &ModelSelection::new("openai", "gpt-5.5"),
            &plan(),
            &mut host
        )
        .await
        .is_err()
    );
    assert!(host.questions.is_empty());
    assert_eq!(std::fs::read_to_string(target).unwrap(), "preserve");
}
