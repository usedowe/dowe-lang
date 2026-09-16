use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse};
use serde_json::{Value, json};

#[tokio::test]
async fn approval_cannot_overwrite_a_concurrently_created_file() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let path = root.path().join(".agent/plans/feature.json");
    let mut host = host(true);
    host.competing_path = Some(path.clone());
    assert!(
        draft_workflow_plan(
            &store,
            &HarnessConfig::default(),
            &ModelSelection::new("openai", "gpt-5.5"),
            "feature",
            "Build login",
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!(std::fs::read_to_string(path).unwrap(), "user content");
}

#[tokio::test]
async fn draft_command_is_shared_and_scoped_calls_cannot_escalate() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let active = ModelSelection::new("openai", "gpt-5.5");
    let mut host = host(true);
    let mut task = HarnessTask::new("/draft-plan feature Build login", &active);
    task.role = HarnessRole::Research;
    assert!(
        run_agent_task(
            &store,
            &mut session,
            &HarnessConfig::default(),
            task,
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!(host.calls, 0);
    let result = run_agent_task(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask::new("/draft-plan feature Build login", &active),
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(result, HarnessOutcome::Completed));
    assert!(
        host.events
            .iter()
            .any(|e| e["event"] == "request_prepared" && e["role"] == "plan")
    );
    assert!(
        host.events
            .iter()
            .any(|e| e["event"] == "workflow_plan_drafted")
    );
}

#[tokio::test]
async fn ui_draft_negotiates_catalog_then_selected_contracts_before_planning() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = host(true);
    let result = draft_workflow_plan(
        &store,
        &HarnessConfig::default(),
        &ModelSelection::new("openai", "gpt-5.5"),
        "feature",
        "Build a landing page",
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(result, HarnessOutcome::Completed));
    assert_eq!(host.calls, 2);
    assert!(host.requests[0].contains("Complete catalog:"));
    assert!(host.requests[1].contains("selected these authoritative component contracts"));
    assert!(host.requests[1].contains("props"));
    assert!(host.requests[1].contains("example"));
    assert!(host.events.iter().any(|event| {
        event["event"] == "view_component_catalog_sent"
            && event["component_count"].as_u64().unwrap() > 10
    }));
    assert!(host.events.iter().any(|event| {
        event["event"] == "view_component_selection_received"
            && event["selected"] == json!(["Button", "Card"])
    }));
    let plan = read_workflow_plan(store.root(), ".agent/plans/feature.json").unwrap();
    assert_eq!(
        plan.view_components
            .iter()
            .map(|contract| contract.name.as_str())
            .collect::<Vec<_>>(),
        ["Button", "Card"]
    );
    assert!(
        plan.view_components
            .iter()
            .all(|contract| { !contract.props.is_empty() && !contract.example.is_empty() })
    );
}

#[tokio::test]
async fn exhausted_draft_budget_prevents_provider_and_save() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut host = host(true);
    let config = HarnessConfig {
        token_budget: 1,
        ..Default::default()
    };
    let outcome = draft_workflow_plan(
        &store,
        &config,
        &ModelSelection::new("openai", "gpt-5.5"),
        "feature",
        "Build login",
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(outcome, HarnessOutcome::BudgetExhausted));
    assert_eq!((host.calls, host.approvals), (0, 0));
    assert!(!root.path().join(".agent/plans/feature.json").exists());
}

struct Host {
    response: String,
    approve: bool,
    calls: usize,
    approvals: usize,
    events: Vec<Value>,
    requests: Vec<String>,
    competing_path: Option<std::path::PathBuf>,
}
impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.calls += 1;
        self.requests
            .push(serde_json::to_string(&request.messages).unwrap());
        let selection = serde_json::to_string(&request.messages)
            .unwrap()
            .contains("Complete catalog:");
        let payload = if selection {
            json!({"output_text":"{\"components\":[\"Button\",\"Card\"]}"})
        } else {
            json!({"output_text":self.response})
        };
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload,
        })
    }
    async fn approve(&mut self, approval: &Approval) -> AgentResult<Option<bool>> {
        assert_eq!(approval.call.name, "save_workflow_plan");
        self.approvals += 1;
        if let Some(path) = &self.competing_path {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "user content").unwrap();
        }
        Ok(Some(self.approve))
    }
    fn event(&mut self, event: &Value) -> AgentResult<()> {
        self.events.push(event.clone());
        Ok(())
    }
}
fn host(approve: bool) -> Host {
    Host { response: json!({"id":"feature","objective":"Build login","requirements":[
        {"id":"auth","question":"Which login method?","blocking":true,"answer":null}],
        "tasks":[{"id":"code","objective":"Implement login","dependencies":[],"write_scopes":["src"]}],
        "checks":[{"criterion":"works","test_paths":["tests/login.dowe"]}],"isolation":"isolated_workers"}).to_string(),
        approve, calls: 0, approvals: 0, events: vec![], requests: vec![], competing_path: None }
}

#[tokio::test]
async fn draft_is_saved_only_after_approval_and_remains_unexecuted() {
    let root = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let selection = ModelSelection::new("openai", "gpt-5.5");
    let mut denied = host(false);
    assert!(matches!(
        draft_workflow_plan(
            &store,
            &HarnessConfig::default(),
            &selection,
            "feature",
            "Build login",
            &mut denied
        )
        .await
        .unwrap(),
        HarnessOutcome::ApprovalRequired
    ));
    assert!(!root.path().join(".agent/plans/feature.json").exists());
    let mut accepted = host(true);
    assert!(matches!(
        draft_workflow_plan(
            &store,
            &HarnessConfig::default(),
            &selection,
            "feature",
            "Build login",
            &mut accepted
        )
        .await
        .unwrap(),
        HarnessOutcome::Completed
    ));
    let plan = read_workflow_plan(store.root(), ".agent/plans/feature.json").unwrap();
    assert!(plan.coordinator().is_err());
    assert_eq!((accepted.calls, accepted.approvals), (1, 1));
    assert!(!root.path().join(".agent/tasks/feature.json").exists());
    assert!(
        draft_workflow_plan(
            &store,
            &HarnessConfig::default(),
            &selection,
            "feature",
            "Build login",
            &mut accepted
        )
        .await
        .is_err()
    );
    assert_eq!(accepted.calls, 1);
}

#[tokio::test]
async fn invalid_output_never_reaches_save_approval() {
    for change in ["invalid", "id", "cycle", "path", "answer"] {
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let store = HarnessStore::new(home.path(), root.path()).unwrap();
        let mut host = host(true);
        let mut value: Value = serde_json::from_str(&host.response).unwrap();
        match change {
            "answer" => value["requirements"][0]["answer"] = json!("model invention"),
            "id" => value["id"] = json!("foreign"),
            "cycle" => value["tasks"][0]["dependencies"] = json!(["code"]),
            "path" => value["checks"][0]["test_paths"] = json!(["../outside"]),
            _ => {}
        }
        host.response = if change == "invalid" {
            "not JSON".into()
        } else {
            value.to_string()
        };
        assert!(
            draft_workflow_plan(
                &store,
                &HarnessConfig::default(),
                &ModelSelection::new("openai", "gpt-5.5"),
                "feature",
                "Build login",
                &mut host
            )
            .await
            .is_err()
        );
        assert_eq!(host.approvals, 0);
        assert!(!root.path().join(".agent/plans/feature.json").exists());
    }
}
