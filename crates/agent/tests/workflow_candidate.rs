use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse};
use dowe_agent_harness::coordinator::DriveOutcome;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Default)]
struct Host {
    candidate: Option<PathBuf>,
    calls: usize,
    events: Vec<Value>,
}

impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.calls += 1;
        let payload = match self.calls {
            1 => {
                let path = self.candidate.as_ref().unwrap().join("src/result.txt");
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, "candidate-evidence-93817").unwrap();
                json!({"output_text":"Implemented."})
            }
            2 | 4 => {
                json!({"output":[{"type":"function_call","call_id":format!("read-{}", self.calls),"name":"read_file","arguments":"{\"path\":\"src/result.txt\"}"}]})
            }
            3 => {
                assert!(
                    serde_json::to_string(request)
                        .unwrap()
                        .contains("candidate-evidence-93817")
                );
                json!({"output_text":"Dependency verified."})
            }
            5 => {
                assert!(
                    serde_json::to_string(request)
                        .unwrap()
                        .contains("candidate-evidence-93817")
                );
                json!({"output_text":"{\"approved\":true,\"findings\":[]}"})
            }
            _ => panic!("unexpected provider request {}", self.calls),
        };
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload,
        })
    }

    async fn approve(&mut self, approval: &Approval) -> AgentResult<Option<bool>> {
        if approval.call.name == "integrate_worktrees" {
            assert_eq!(
                approval.details["manifest"]["newFiles"][0]["path"],
                "src/result.txt"
            );
            assert_eq!(
                approval.details["manifest"]["patchSha256"]
                    .as_str()
                    .unwrap()
                    .len(),
                64
            );
        }
        Ok(Some(true))
    }

    fn event(&mut self, event: &Value) -> AgentResult<()> {
        if event["event"] == "workflow_worktree_created" {
            assert!(self.candidate.is_none());
            self.candidate = Some(PathBuf::from(event["path"].as_str().unwrap()));
        }
        self.events.push(event.clone());
        Ok(())
    }
}

#[tokio::test]
async fn dependency_and_reviewer_read_the_combined_candidate() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("tests")).unwrap();
    std::fs::write(
        root.path().join("tests/check.dowe"),
        "test \"verified\"\n  assert true value:true\n",
    )
    .unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Dowe Test"],
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
        id: "candidate-test".into(),
        objective: "Implement verified changes".into(),
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
                id: "first".into(),
                objective: "Implement changes".into(),
                dependencies: vec![],
                write_scopes: vec!["src".into()],
            },
            WorkflowTask {
                id: "second".into(),
                objective: "Inspect dependency result".into(),
                dependencies: vec!["first".into()],
                write_scopes: vec!["src".into()],
            },
        ],
        checks: vec![WorkflowCheck {
            criterion: "works".into(),
            test_paths: vec!["tests/check.dowe".into()],
        }],
        isolation: WorkflowIsolation::DetachedWorktrees,
        cleanup_after_integration: true,
    };
    let mut host = Host::default();
    let outcome = run_workflow(
        &store,
        &HarnessConfig::default(),
        &ModelSelection::new("openai", "gpt-5.5"),
        &plan,
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(outcome, DriveOutcome::Completed);
    assert_eq!(host.calls, 5);
    assert_eq!(
        std::fs::read_to_string(root.path().join("src/result.txt")).unwrap(),
        "candidate-evidence-93817"
    );
    assert!(host.candidate.unwrap().join("src/result.txt").is_file());
    assert!(
        host.events
            .iter()
            .any(|event| event["event"] == "workflow_worktrees_retained")
    );
    assert_eq!(
        host.events
            .iter()
            .filter(|event| event["event"] == "workflow_verification")
            .count(),
        2
    );
    assert!(!home.path().join("harness/harness").exists());
}
