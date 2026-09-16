use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse};
use dowe_agent_harness::coordinator::{DriveOutcome, Phase};
use serde_json::{Value, json};
use std::path::PathBuf;

struct Host {
    calls: usize,
    approve_plan: bool,
    approve_integration: bool,
    candidate: Option<PathBuf>,
    plans: usize,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            calls: 0,
            approve_plan: true,
            approve_integration: false,
            candidate: None,
            plans: 0,
        }
    }
}

impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.calls += 1;
        if self.calls == 1 {
            let candidate = self.candidate.as_ref().unwrap();
            std::fs::create_dir(candidate.join("src")).unwrap();
            std::fs::write(candidate.join("src/result.txt"), "retained result").unwrap();
        }
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: json!({"output_text":if self.calls == 1 {"Done."} else {"{\"approved\":true,\"findings\":[]}"}}),
        })
    }
    async fn approve(&mut self, approval: &Approval) -> AgentResult<Option<bool>> {
        Ok(Some(match approval.call.name.as_str() {
            "approve_plan" => {
                self.plans += 1;
                self.approve_plan
            }
            "integrate_worktrees" => self.approve_integration,
            other => panic!("unexpected approval {other}"),
        }))
    }
    fn event(&mut self, event: &Value) -> AgentResult<()> {
        if event["event"] == "workflow_worktree_created" {
            assert!(self.candidate.is_none(), "resume must reuse the candidate");
            self.candidate = Some(PathBuf::from(event["path"].as_str().unwrap()));
        }
        Ok(())
    }
}

fn fixture() -> (
    tempfile::TempDir,
    tempfile::TempDir,
    HarnessStore,
    WorkflowPlan,
) {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("tests")).unwrap();
    std::fs::write(
        root.path().join("tests/check.dowe"),
        "test \"works\"\n  assert true value:true\n",
    )
    .unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Test"],
        vec!["config", "user.email", "test@example.invalid"],
        vec!["add", "."],
        vec!["commit", "-qm", "initial"],
    ] {
        assert!(
            std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let plan = WorkflowPlan {
        id: "resume".into(),
        objective: "Build verified changes".into(),
        product_discovery: None,
        product_use_cases: Vec::new(),
        view_components: Vec::new(),
        risk: dowe_agent_harness::RiskLevel::Medium,
        autonomy: dowe_agent_harness::AutonomyLevel::Feature,
        visual_plan: None,
        backend_plan: None,
        verification_graph: None,
        specification: None,
        contracts: Vec::new(),
        codegraph_binding: None,
        requirements: vec![],
        tasks: vec![WorkflowTask {
            id: "code".into(),
            objective: "Implement changes".into(),
            dependencies: vec![],
            write_scopes: vec!["src".into()],
        }],
        checks: vec![WorkflowCheck {
            criterion: "works".into(),
            test_paths: vec!["tests/check.dowe".into()],
        }],
        isolation: WorkflowIsolation::DetachedWorktrees,
        cleanup_after_integration: false,
    };
    (root, home, store, plan)
}

fn model() -> ModelSelection {
    ModelSelection::new("openai", "gpt-5.5")
}

#[tokio::test]
async fn integration_pause_resumes_without_replaying_tasks_or_resetting_usage() {
    let (root, home, store, plan) = fixture();
    let config = HarnessConfig::default();
    let mut host = Host::default();
    assert_eq!(
        run_workflow(&store, &config, &model(), &plan, &mut host)
            .await
            .unwrap(),
        DriveOutcome::Blocked
    );
    let paused = store.load_workflow("resume").unwrap();
    assert!(paused.can_resume());
    assert_eq!(paused.coordinator.phase(), Phase::Deliverable);
    assert!(!root.path().join("src/result.txt").exists());
    host.approve_integration = true;
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    assert_eq!(
        resume_workflow(
            &store,
            &config,
            &model(),
            "resume",
            paused.sequence,
            &mut host
        )
        .await
        .unwrap(),
        DriveOutcome::Completed
    );
    assert_eq!(host.calls, 3);
    assert_eq!(host.plans, 2);
    assert_eq!(
        std::fs::read_to_string(root.path().join("src/result.txt")).unwrap(),
        "retained result"
    );
    let completed = store.load_workflow("resume").unwrap();
    let usage = completed.coordinator.summary().unwrap();
    assert_eq!(usage.task_attempts, 1);
    assert_eq!(usage.review_sessions, 2);
    assert!(usage.charged_tokens > paused.coordinator.summary().unwrap().charged_tokens);
    assert_eq!(
        resume_workflow(
            &store,
            &config,
            &model(),
            "resume",
            completed.sequence,
            &mut host
        )
        .await
        .unwrap(),
        DriveOutcome::Completed
    );
    assert_eq!(host.calls, 3);
    assert_eq!(
        store.load_workflow("resume").unwrap().sequence,
        completed.sequence
    );
}

#[tokio::test]
async fn stale_sequence_configuration_and_candidate_drift_fail_before_provider_calls() {
    let (_root, _home, store, plan) = fixture();
    let config = HarnessConfig::default();
    let mut host = Host::default();
    run_workflow(&store, &config, &model(), &plan, &mut host)
        .await
        .unwrap();
    let paused = store.load_workflow("resume").unwrap();
    assert!(
        resume_workflow(
            &store,
            &config,
            &model(),
            "resume",
            paused.sequence - 1,
            &mut host
        )
        .await
        .is_err()
    );
    let changed = HarnessConfig {
        token_budget: config.token_budget + 1,
        ..config.clone()
    };
    assert!(
        resume_workflow(
            &store,
            &changed,
            &model(),
            "resume",
            paused.sequence,
            &mut host
        )
        .await
        .is_err()
    );
    std::fs::write(
        host.candidate.as_ref().unwrap().join("src/result.txt"),
        "external edit",
    )
    .unwrap();
    assert!(
        resume_workflow(
            &store,
            &config,
            &model(),
            "resume",
            paused.sequence,
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!(host.calls, 2);
    assert_eq!(
        store.load_workflow("resume").unwrap().sequence,
        paused.sequence
    );
}

#[tokio::test]
async fn renewed_denial_and_legacy_recovery_never_execute_workers() {
    let (_root, _home, store, plan) = fixture();
    let config = HarnessConfig::default();
    let mut host = Host {
        approve_plan: false,
        ..Host::default()
    };
    assert_eq!(
        run_workflow(&store, &config, &model(), &plan, &mut host)
            .await
            .unwrap(),
        DriveOutcome::AwaitingApproval
    );
    let paused = store.load_workflow("resume").unwrap();
    assert_eq!(
        resume_workflow(
            &store,
            &config,
            &model(),
            "resume",
            paused.sequence,
            &mut host
        )
        .await
        .unwrap(),
        DriveOutcome::AwaitingApproval
    );
    assert_eq!(host.calls, 0);
    let current = store.load_workflow("resume").unwrap();
    let recovered = store.recover_workflow("resume", current.sequence).unwrap();
    assert!(!recovered.can_resume());
    assert!(
        resume_workflow(
            &store,
            &config,
            &model(),
            "resume",
            recovered.sequence,
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!(host.calls, 0);
}

#[tokio::test]
async fn interrupted_or_consumed_time_checkpoints_never_dispatch() {
    for exhausted_time in [false, true] {
        let (root, _home, store, plan) = fixture();
        let config = HarnessConfig::default();
        let mut host = Host {
            approve_plan: false,
            ..Host::default()
        };
        run_workflow(&store, &config, &model(), &plan, &mut host)
            .await
            .unwrap();
        let paused = store.load_workflow("resume").unwrap();
        let path = root.path().join(".agent/tasks/resume.json");
        let mut checkpoint: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        if exhausted_time {
            checkpoint["continuation"]["elapsed_ms"] = json!(config.duration_seconds * 1000);
        } else {
            checkpoint["continuation"]["resumable"] = json!(false);
        }
        std::fs::write(&path, serde_json::to_vec(&checkpoint).unwrap()).unwrap();
        let result = resume_workflow(
            &store,
            &config,
            &model(),
            "resume",
            paused.sequence,
            &mut host,
        )
        .await;
        if exhausted_time {
            assert_eq!(result.unwrap(), DriveOutcome::BudgetExhausted);
        } else {
            assert!(result.is_err());
        }
        assert_eq!(host.calls, 0);
        assert_eq!(host.plans, 1);
    }
}
