use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse};
use dowe_agent_harness::coordinator::{DriveOutcome, Phase};
use serde_json::{Value, json};

struct Host {
    approved: bool,
    calls: usize,
    events: Vec<Value>,
}

impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.calls += 1;
        let text = if self.calls == 1 {
            "Implemented."
        } else {
            "{\"approved\":true,\"findings\":[]}"
        };
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: json!({"output_text":text}),
        })
    }
    async fn approve(&mut self, approval: &Approval) -> AgentResult<Option<bool>> {
        assert!(matches!(
            approval.call.name.as_str(),
            "approve_plan" | "integrate_worktrees"
        ));
        Ok(Some(self.approved))
    }
    fn event(&mut self, event: &Value) -> AgentResult<()> {
        self.events.push(event.clone());
        Ok(())
    }
}

fn plan() -> WorkflowPlan {
    WorkflowPlan {
        id: "feature".into(),
        objective: "Build a verified feature".into(),
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
            id: "implement".into(),
            objective: "Implement feature".into(),
            dependencies: vec![],
            write_scopes: vec!["src".into()],
        }],
        checks: vec![WorkflowCheck {
            criterion: "works".into(),
            test_paths: vec!["tests/check.dowe".into()],
        }],
        isolation: WorkflowIsolation::SharedCheckout,
        cleanup_after_integration: false,
    }
}

#[tokio::test]
async fn denied_plan_never_calls_a_provider() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = Host {
        approved: false,
        calls: 0,
        events: vec![],
    };
    let outcome = run_workflow(
        &store,
        &HarnessConfig::default(),
        &ModelSelection::new("openai", "gpt-5.5"),
        &plan(),
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::AwaitingApproval);
    assert_eq!(host.calls, 0);
    assert_eq!(
        store.load_workflow("feature").unwrap().coordinator.phase(),
        Phase::AwaitingApproval
    );
}

#[tokio::test]
async fn execution_verification_review_and_checkpoints_are_connected() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("tests")).unwrap();
    std::fs::write(
        root.path().join("tests/check.dowe"),
        "test \"verified\"\n  assert true value:true\n",
    )
    .unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = Host {
        approved: true,
        calls: 0,
        events: vec![],
    };
    let outcome = run_workflow(
        &store,
        &HarnessConfig::default(),
        &ModelSelection::new("openai", "gpt-5.5"),
        &plan(),
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::Completed);
    assert_eq!(host.calls, 2);
    assert_eq!(
        store.load_workflow("feature").unwrap().coordinator.phase(),
        Phase::Completed
    );
    assert!(
        host.events
            .iter()
            .any(|e| e["event"] == "workflow_verification")
    );
}

#[tokio::test]
async fn empty_test_discovery_cannot_pass_verification() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("tests")).unwrap();
    std::fs::write(root.path().join("tests/check.dowe"), "").unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = Host {
        approved: true,
        calls: 0,
        events: vec![],
    };
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
    assert_eq!(host.calls, 1);
    assert_eq!(
        store.load_workflow("feature").unwrap().coordinator.phase(),
        Phase::Verification
    );
}

#[tokio::test]
async fn detached_workflow_verifies_in_isolation_and_blocks_delivery() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("tests")).unwrap();
    std::fs::write(
        root.path().join("tests/check.dowe"),
        "test \"verified\"\n  assert true value:true\n",
    )
    .unwrap();
    for args in [
        &["init", "-q"][..],
        &["config", "user.email", "test@example.invalid"][..],
        &["config", "user.name", "Dowe Test"][..],
        &["add", "."][..],
        &["commit", "-qm", "initial"][..],
    ] {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .unwrap();
        assert!(output.status.success(), "git failed: {:?}", output);
    }
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = Host {
        approved: true,
        calls: 0,
        events: vec![],
    };
    let mut isolated = plan();
    isolated.isolation = WorkflowIsolation::DetachedWorktrees;
    let outcome = run_workflow(
        &store,
        &HarnessConfig::default(),
        &ModelSelection::new("openai", "gpt-5.5"),
        &isolated,
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::Completed);
    let checkpoint = store.load_workflow("feature").unwrap();
    assert!(
        checkpoint
            .coordinator
            .integration()
            .is_some_and(|summary| { summary.ready && summary.applied })
    );
    assert!(checkpoint.coordinator.summary().is_some_and(|summary| {
        summary.task_count == 1
            && summary.task_attempts == 1
            && summary.retry_count == 0
            && summary.review_sessions == 1
            && summary.provider_attempts >= 2
    }));
    assert!(
        host.events
            .iter()
            .any(|event| event["event"] == "workflow_integration_applied")
    );
    assert!(
        host.events
            .iter()
            .any(|event| event["event"] == "workflow_worktree_created")
    );
}
