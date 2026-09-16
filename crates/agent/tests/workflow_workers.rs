use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse};
use dowe_agent_harness::coordinator::{DriveOutcome, ExecutionState, FailureClass, TaskResult};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Default)]
struct Host {
    workers: BTreeMap<String, PathBuf>,
    current: String,
    candidate: Option<PathBuf>,
    calls: usize,
    approve_integration: bool,
    conflict: bool,
}

impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.calls += 1;
        let text = match self.current.as_str() {
            "a" => {
                let path = &self.workers["a"];
                std::fs::write(path.join("first.txt"), "first").unwrap();
                if self.conflict {
                    std::fs::write(path.join("common.txt"), "a").unwrap();
                }
                "Done."
            }
            "b" => {
                let path = &self.workers["b"];
                assert!(
                    !path.join("first.txt").exists(),
                    "independent worker must not see sibling changes"
                );
                std::fs::write(path.join("second.txt"), "second").unwrap();
                if self.conflict {
                    std::fs::write(path.join("common.txt"), "b").unwrap();
                }
                "Done."
            }
            "c" => {
                let path = &self.workers["c"];
                assert_eq!(
                    std::fs::read_to_string(path.join("first.txt")).unwrap(),
                    "first"
                );
                assert_eq!(
                    std::fs::read_to_string(path.join("second.txt")).unwrap(),
                    "second"
                );
                std::fs::write(path.join("joined.txt"), "joined").unwrap();
                "Done."
            }
            "review" => "{\"approved\":true,\"findings\":[]}",
            other => panic!("unexpected worker {other}"),
        };
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: json!({"output_text":text}),
        })
    }
    async fn approve(&mut self, approval: &Approval) -> AgentResult<Option<bool>> {
        Ok(Some(
            approval.call.name != "integrate_worktrees" || self.approve_integration,
        ))
    }
    fn event(&mut self, event: &Value) -> AgentResult<()> {
        if event["event"] == "workflow_worker_created" {
            let id = event["task"].as_str().unwrap().to_string();
            assert!(!self.workers.contains_key(&id));
            self.current = id.clone();
            self.workers
                .insert(id, PathBuf::from(event["path"].as_str().unwrap()));
        }
        if event["event"] == "workflow_candidate_created" {
            self.candidate = Some(PathBuf::from(event["path"].as_str().unwrap()));
            self.current = "review".into();
        }
        Ok(())
    }
}

fn fixture(
    dependent: bool,
) -> (
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
    let mut tasks = vec![
        WorkflowTask {
            id: "a".into(),
            objective: "First changes".into(),
            dependencies: vec![],
            write_scopes: vec!["first.txt".into(), "common.txt".into()],
        },
        WorkflowTask {
            id: "b".into(),
            objective: "Second changes".into(),
            dependencies: vec![],
            write_scopes: vec!["second.txt".into(), "common.txt".into()],
        },
    ];
    if dependent {
        tasks.push(WorkflowTask {
            id: "c".into(),
            objective: "Join dependencies".into(),
            dependencies: vec!["a".into(), "b".into()],
            write_scopes: vec!["joined.txt".into()],
        });
    }
    let plan = WorkflowPlan {
        id: "workers".into(),
        objective: "Isolated changes".into(),
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
        tasks,
        checks: vec![WorkflowCheck {
            criterion: "works".into(),
            test_paths: vec!["tests/check.dowe".into()],
        }],
        isolation: WorkflowIsolation::IsolatedWorkers,
        cleanup_after_integration: false,
    };
    (root, home, store, plan)
}

#[tokio::test]
async fn isolated_workers_inherit_only_dependencies_then_resume_common_candidate() {
    let (root, home, store, plan) = fixture(true);
    let config = HarnessConfig::default();
    let model = ModelSelection::new("openai", "gpt-5.5");
    let mut host = Host::default();
    assert_eq!(
        run_workflow(&store, &config, &model, &plan, &mut host)
            .await
            .unwrap(),
        DriveOutcome::Blocked
    );
    assert_eq!(host.workers.len(), 3);
    assert_eq!(host.calls, 4);
    assert!(!root.path().join("first.txt").exists());
    let candidate = host.candidate.as_ref().unwrap();
    assert!(candidate.join("first.txt").is_file());
    assert!(candidate.join("second.txt").is_file());
    assert!(candidate.join("joined.txt").is_file());
    let checkpoint = store.load_workflow("workers").unwrap();
    assert!(checkpoint.can_resume());
    let checkpoint_path = root.path().join(".agent/tasks/workers.json");
    let original = std::fs::read(&checkpoint_path).unwrap();
    let mut forged: Value = serde_json::from_slice(&original).unwrap();
    forged["continuation"]["worker_worktrees"]["a"]["path"] = json!(root.path());
    std::fs::write(&checkpoint_path, serde_json::to_vec(&forged).unwrap()).unwrap();
    assert!(
        resume_workflow(
            &store,
            &config,
            &model,
            "workers",
            checkpoint.sequence,
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!(host.calls, 4);
    std::fs::write(&checkpoint_path, original).unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    host.approve_integration = true;
    assert_eq!(
        resume_workflow(
            &store,
            &config,
            &model,
            "workers",
            checkpoint.sequence,
            &mut host
        )
        .await
        .unwrap(),
        DriveOutcome::Completed
    );
    assert_eq!(host.calls, 5);
    assert_eq!(
        std::fs::read_to_string(root.path().join("joined.txt")).unwrap(),
        "joined"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("first.txt")).unwrap(),
        "first"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("second.txt")).unwrap(),
        "second"
    );
    let current = std::fs::read_to_string(root.path().join(".dowe/codegraph/CURRENT")).unwrap();
    assert!(current.starts_with("generation-"));
}

#[tokio::test]
async fn conflicting_independent_results_block_before_review_or_host_writes() {
    let (root, _home, store, plan) = fixture(false);
    let mut host = Host {
        conflict: true,
        ..Default::default()
    };
    let outcome = run_workflow(
        &store,
        &HarnessConfig::default(),
        &ModelSelection::new("openai", "gpt-5.5"),
        &plan,
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::Blocked);
    assert_eq!(host.calls, 2);
    assert!(host.candidate.is_none());
    assert!(!root.path().join("common.txt").exists());
    let checkpoint = store.load_workflow("workers").unwrap();
    assert!(!checkpoint.can_resume());
    assert!(
        checkpoint
            .coordinator
            .schedule()
            .unwrap()
            .tasks()
            .any(|task| task.state()
                == ExecutionState::Finished(TaskResult::Failed(FailureClass::Conflict)))
    );
}
