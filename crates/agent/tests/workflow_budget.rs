use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse};
use dowe_agent_harness::coordinator::{DriveOutcome, Phase};
use serde_json::{Value, json};

#[derive(Default)]
struct Host {
    calls: usize,
    events: Vec<Value>,
    delayed: bool,
    mutate: bool,
    small_usage: bool,
}

impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.calls += 1;
        if self.delayed {
            std::future::pending::<()>().await;
        }
        let mut payload = json!({"output_text":"Implemented.",
            "usage":{"input_tokens":190000,"output_tokens":1000,"cost":0.9}});
        if self.small_usage {
            payload["usage"] = json!({"input_tokens":100,"output_tokens":10,"cost":0.001});
        }
        if self.mutate && self.calls == 1 {
            payload["output"] = json!([{"type":"function_call","call_id":"write","name":"write_file",
                "arguments":json!({"path":"docs/result.md","content":"must not be written","skill":"core","reason":"test budget gate"}).to_string()}]);
        }
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload,
        })
    }
    async fn approve(&mut self, _: &Approval) -> AgentResult<Option<bool>> {
        Ok(Some(true))
    }
    fn event(&mut self, event: &Value) -> AgentResult<()> {
        self.events.push(event.clone());
        Ok(())
    }
}

fn plan() -> WorkflowPlan {
    WorkflowPlan {
        id: "budgeted".into(),
        objective: "Build feature".into(),
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
        tasks: vec![
            WorkflowTask {
                id: "code".into(),
                objective: "Implement feature".into(),
                dependencies: vec![],
                write_scopes: vec!["src".into()],
            },
            WorkflowTask {
                id: "next".into(),
                objective: "Implement next feature".into(),
                dependencies: vec!["code".into()],
                write_scopes: vec!["src".into()],
            },
        ],
        checks: vec![WorkflowCheck {
            criterion: "works".into(),
            test_paths: vec!["tests.dowe".into()],
        }],
        isolation: WorkflowIsolation::SharedCheckout,
        cleanup_after_integration: false,
    }
}

#[tokio::test]
async fn task_sessions_share_one_token_budget() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = Host::default();
    let config = HarnessConfig {
        token_budget: 195000,
        ..Default::default()
    };
    let outcome = run_workflow(
        &store,
        &config,
        &ModelSelection::new("openai", "gpt-5.5"),
        &plan(),
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::BudgetExhausted);
    assert_eq!(host.calls, 1);
    assert_ne!(
        store.load_workflow("budgeted").unwrap().coordinator.phase(),
        Phase::Completed
    );
    assert!(host.events.iter().any(|e| e["event"] == "workflow_budget"));
}

#[tokio::test]
async fn duration_timeout_preserves_uncertain_work_without_replay() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = Host {
        delayed: true,
        ..Default::default()
    };
    let config = HarnessConfig {
        duration_seconds: 1,
        ..Default::default()
    };
    let outcome = run_workflow(
        &store,
        &config,
        &ModelSelection::new("openai", "gpt-5.5"),
        &plan(),
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::BudgetExhausted);
    let checkpoint = store.load_workflow("budgeted").unwrap();
    assert_eq!(checkpoint.coordinator.phase(), Phase::Planning);
    assert!(
        checkpoint
            .coordinator
            .schedule()
            .unwrap()
            .tasks()
            .any(|t| t.state()
                == dowe_agent_harness::coordinator::ExecutionState::Finished(
                    dowe_agent_harness::coordinator::TaskResult::Failed(
                        dowe_agent_harness::coordinator::FailureClass::UncertainEffect
                    )
                ))
    );
    assert_eq!(host.calls, 1);
}

#[tokio::test]
async fn reviewer_does_not_receive_a_fresh_budget() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    std::fs::write(
        root.path().join("tests.dowe"),
        "test \"works\"\n  assert true value:true\n",
    )
    .unwrap();
    let mut plan = plan();
    plan.tasks.truncate(1);
    let mut host = Host::default();
    let config = HarnessConfig {
        token_budget: 195000,
        ..Default::default()
    };
    let outcome = run_workflow(
        &store,
        &config,
        &ModelSelection::new("openai", "gpt-5.5"),
        &plan,
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::BudgetExhausted);
    assert_eq!(host.calls, 1);
    assert!(
        host.events
            .iter()
            .any(|e| e["event"] == "workflow_verification")
    );
}

#[tokio::test]
async fn insufficient_cost_reservation_prevents_first_provider_call() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = Host::default();
    let config = HarnessConfig {
        cost_budget_usd: Some(0.000001),
        ..Default::default()
    };
    let outcome = run_workflow(
        &store,
        &config,
        &ModelSelection::new("openai", "gpt-5.5"),
        &plan(),
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::BudgetExhausted);
    assert_eq!(host.calls, 0);
}

#[tokio::test]
async fn oversized_provider_response_cannot_apply_its_write() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = Host {
        mutate: true,
        ..Default::default()
    };
    let mut plan = plan();
    plan.tasks[0].write_scopes = vec!["docs".into()];
    let config = HarnessConfig {
        token_budget: 100000,
        ..Default::default()
    };
    let outcome = run_workflow(
        &store,
        &config,
        &ModelSelection::new("openai", "gpt-5.5"),
        &plan,
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::BudgetExhausted);
    assert_eq!(host.calls, 1);
    assert!(!root.path().join("docs/result.md").exists());
}

#[tokio::test]
async fn budget_gate_allows_the_same_write_when_usage_is_within_limits() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    std::fs::write(
        root.path().join("tests.dowe"),
        "test \"works\"\n  assert true value:true\n",
    )
    .unwrap();
    let mut host = Host {
        mutate: true,
        small_usage: true,
        ..Default::default()
    };
    let mut plan = plan();
    plan.tasks.truncate(1);
    plan.tasks[0].write_scopes = vec!["docs".into()];
    let result = run_workflow(
        &store,
        &HarnessConfig::default(),
        &ModelSelection::new("openai", "gpt-5.5"),
        &plan,
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(result, DriveOutcome::ReviewRejected);
    assert_eq!(
        std::fs::read_to_string(root.path().join("docs/result.md")).unwrap(),
        "must not be written"
    );
}
