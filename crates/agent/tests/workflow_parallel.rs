use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse, GeneratedImage};
use dowe_agent_harness::coordinator::DriveOutcome;
use serde_json::{Value, json};
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

struct Host {
    active: Arc<AtomicUsize>,
    peak: Arc<AtomicUsize>,
}

struct Worker {
    active: Arc<AtomicUsize>,
    peak: Arc<AtomicUsize>,
}

impl HarnessHost for Host {
    fn parallel_worker(&mut self) -> AgentResult<Option<ParallelHarnessHost>> {
        Ok(Some(ParallelHarnessHost::new(Worker {
            active: self.active.clone(),
            peak: self.peak.clone(),
        })))
    }

    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: json!({"output_text":"{\"approved\":true,\"findings\":[]}"}),
        })
    }

    async fn approve(&mut self, _: &Approval) -> AgentResult<Option<bool>> {
        Ok(Some(true))
    }

    async fn validate_dowe_project(&mut self, _: &std::path::Path) -> AgentResult<Value> {
        Ok(json!({"status":"passed"}))
    }

    fn event(&mut self, _: &Value) -> AgentResult<()> {
        Ok(())
    }
}

impl ParallelWorker for Worker {
    fn open_terminal(&mut self) -> AgentResult<Box<dyn HarnessTerminal>> {
        Err(dowe_agent::AgentError::new(
            "terminal unavailable in test worker",
        ))
    }

    fn send<'a>(
        &'a mut self,
        request: &'a AgentRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = AgentResult<AgentServerResponse>> + Send + 'a>,
    > {
        Box::pin(async move {
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.peak.fetch_max(active, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(80)).await;
            self.active.fetch_sub(1, Ordering::SeqCst);
            Ok(AgentServerResponse {
                request_id: request.request_id.clone(),
                request_type: request.request_type,
                model: request.model.clone(),
                payload: json!({"output_text":"Done."}),
            })
        })
    }

    fn approve<'a>(
        &'a mut self,
        _: &'a Approval,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AgentResult<Option<bool>>> + Send + 'a>>
    {
        Box::pin(async { Ok(Some(true)) })
    }

    fn generate_image<'a>(
        &'a mut self,
        _: &'a ModelSelection,
        _: &'a Approval,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AgentResult<GeneratedImage>> + Send + 'a>>
    {
        Box::pin(async { Err(dowe_agent::AgentError::new("image unavailable")) })
    }

    fn ask_clarification<'a>(
        &'a mut self,
        _: &'a ClarificationQuestion,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AgentResult<Option<String>>> + Send + 'a>>
    {
        Box::pin(async { Ok(None) })
    }

    fn validate_dowe_project<'a>(
        &'a mut self,
        _: &'a std::path::Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AgentResult<Value>> + Send + 'a>> {
        Box::pin(async { Ok(json!({"status":"not_run"})) })
    }

    fn event(&mut self, _: &Value) -> AgentResult<()> {
        Ok(())
    }
}

fn plan() -> WorkflowPlan {
    WorkflowPlan {
        id: "parallel".into(),
        objective: "parallel feature".into(),
        product_discovery: None,
        product_use_cases: Vec::new(),
        view_components: Vec::new(),
        risk: dowe_agent_harness::RiskLevel::Medium,
        autonomy: dowe_agent_harness::AutonomyLevel::Feature,
        visual_plan: None,
        backend_plan: None,
        verification_graph: None,
        specification: None,
        contracts: vec![],
        codegraph_binding: None,
        requirements: vec![],
        tasks: ["a", "b"]
            .into_iter()
            .map(|id| WorkflowTask {
                id: id.into(),
                objective: format!("implement {id}"),
                dependencies: vec![],
                write_scopes: vec![format!("src/{id}")],
            })
            .collect(),
        checks: vec![],
        isolation: WorkflowIsolation::SharedCheckout,
        cleanup_after_integration: false,
    }
}

#[tokio::test]
async fn independent_shared_checkout_workers_use_concurrent_provider_futures() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let mut host = Host {
        active,
        peak: peak.clone(),
    };
    let mut workflow = plan();
    workflow.checks = vec![WorkflowCheck {
        criterion: "provider work completed".into(),
        test_paths: vec![],
    }];
    let outcome = run_workflow(
        &store,
        &HarnessConfig::default(),
        &ModelSelection::new("openai", "gpt-5.5"),
        &workflow,
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::Completed);
    assert_eq!(peak.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn independent_isolated_workers_overlap_before_capture() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(root.path())
        .status()
        .unwrap();
    std::fs::create_dir_all(root.path().join("src")).unwrap();
    std::fs::write(root.path().join("src/.keep"), "tracked").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(root.path())
        .status()
        .unwrap();
    Command::new("git")
        .args([
            "-c",
            "user.name=Dowe",
            "-c",
            "user.email=dowe@example.invalid",
            "commit",
            "-qm",
            "initial",
        ])
        .current_dir(root.path())
        .status()
        .unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let mut host = Host {
        active,
        peak: peak.clone(),
    };
    let mut workflow = plan();
    workflow.id = "isolated-parallel".into();
    workflow.isolation = WorkflowIsolation::IsolatedWorkers;
    workflow.checks = vec![WorkflowCheck {
        criterion: "provider work completed".into(),
        test_paths: vec![],
    }];
    let outcome = run_workflow(
        &store,
        &HarnessConfig::default(),
        &ModelSelection::new("openai", "gpt-5.5"),
        &workflow,
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::Completed);
    assert_eq!(peak.load(Ordering::SeqCst), 2);
    assert_eq!(
        std::fs::read_to_string(root.path().join("src/.keep")).unwrap(),
        "tracked"
    );
}
