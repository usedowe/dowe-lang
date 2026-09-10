use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse};
use serde_json::{Value, json};
use std::collections::VecDeque;

struct Host {
    responses: VecDeque<Value>,
    requests: Vec<AgentRequest>,
    events: Vec<Value>,
    decision: Option<bool>,
}

impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.requests.push(request.clone());
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: self
                .responses
                .pop_front()
                .expect("unexpected provider call"),
        })
    }
    async fn approve(&mut self, _: &Approval) -> AgentResult<Option<bool>> {
        Ok(self.decision)
    }
    fn event(&mut self, event: &Value) -> AgentResult<()> {
        self.events.push(event.clone());
        Ok(())
    }
}

fn write_call() -> Value {
    json!({"output":[{"type":"function_call","call_id":"write","name":"write_file","arguments":json!({"path":"main.dowe","content":"main {}\n","skill":"core","reason":"create application root"}).to_string()}]})
}

fn read_call(id: &str, path: &str) -> Value {
    json!({"output":[{"type":"function_call","call_id":id,"name":"read_file","arguments":json!({"path":path}).to_string()}]})
}

#[tokio::test]
async fn token_safety_bound_applies_per_request_not_to_accumulated_history() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("one.txt"), "one\n").unwrap();
    std::fs::write(root.path().join("two.txt"), "two\n").unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let mut host = Host {
        responses: VecDeque::from([
            read_call("one", "one.txt"),
            read_call("two", "two.txt"),
            json!({"output_text":"finished"}),
        ]),
        requests: vec![],
        events: vec![],
        decision: None,
    };
    let config = HarnessConfig {
        token_budget: 30_000,
        ..Default::default()
    };
    let outcome = run_harness_turn(
        &store,
        &mut session,
        &config,
        HarnessTask::new("Read both files", &ModelSelection::new("openai", "gpt-5.5")),
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(outcome, HarnessOutcome::Completed));
    assert_eq!(host.requests.len(), 3);
}

#[tokio::test]
async fn approved_tools_continue_with_native_results_and_persist_evidence() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let mut host = Host {
        responses: VecDeque::from([
            write_call(),
            json!({"output_text":"Applied main.dowe; validation not run."}),
        ]),
        requests: vec![],
        events: vec![],
        decision: Some(true),
    };
    let result = run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask::new(
            "Create a Dowe app",
            &ModelSelection::new("openai", "gpt-5.5"),
        ),
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(result, HarnessOutcome::Completed));
    assert_eq!(
        std::fs::read_to_string(root.path().join("main.dowe")).unwrap(),
        "main {}\n"
    );
    assert_eq!(host.requests.len(), 2);
    let replay = host.requests[1].extra["dowe_harness_turns"].to_string();
    assert!(replay.contains("applied") && replay.contains("write"));
    assert!(!store.load_session(&session.id).unwrap().interrupted);
    assert!(
        host.events
            .iter()
            .any(|event| event["event"] == "operation_finished"
                    && event["receipt"].get("codegraphBinding").is_some()
                    && event["receipt"]["operation"] == "write_file"
                    && event["receipt"]["status"] == "succeeded"
                    && event["receipt"]["beforeFingerprint"].is_null()
                    && event["receipt"]["afterFingerprint"].is_string())
    );
}

#[tokio::test]
async fn failed_operation_records_failed_receipt_status() {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let store = HarnessStore::new(home.path(), root.path()).unwrap();
        let mut session = store.create_session().unwrap();
        let mut host = Host {
            responses: VecDeque::from([
                json!({"output":[{"type":"function_call","call_id":"image","name":"generate_image","arguments":json!({"prompt":"texture","destination":"public/assets/texture.png","reason":"test"}).to_string()}]}),
                json!({"output_text":"generation failed"}),
            ]),
            requests: vec![], events: vec![], decision: Some(true),
        };
        run_harness_turn(&store, &mut session, &HarnessConfig::default(),
            HarnessTask::new("Generate asset", &ModelSelection::new("openai", "gpt-5.5")), &mut host).await.unwrap();
        assert!(host.events.iter().any(|event| {
            event["event"] == "operation_finished" && event["failed"] == true
                && event["receipt"]["status"] == "failed"
        }));
    }

    #[tokio::test]
    async fn noninteractive_approval_returns_without_execution_or_another_model_call() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let mut host = Host {
        responses: VecDeque::from([write_call()]),
        requests: vec![],
        events: vec![],
        decision: None,
    };
    let result = run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask::new("Create app", &ModelSelection::new("openai", "gpt-5.5")),
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(result, HarnessOutcome::ApprovalRequired));
    assert!(!root.path().join("main.dowe").exists());
    assert_eq!(host.requests.len(), 1);
    assert!(!session.interrupted);
    assert!(
        host.events
            .iter()
            .any(|event| event["event"] == "approval_required")
    );
}

#[tokio::test]
async fn read_only_role_cannot_obtain_approval_for_writes() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let mut host = Host {
        responses: VecDeque::from([write_call(), json!({"output_text":"Plan only."})]),
        requests: vec![],
        events: vec![],
        decision: Some(true),
    };
    run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask {
            role: HarnessRole::Plan,
            ..HarnessTask::new("Plan app", &ModelSelection::new("openai", "gpt-5.5"))
        },
        &mut host,
    )
    .await
    .unwrap();
    assert!(!root.path().join("main.dowe").exists());
    assert!(
        !host
            .events
            .iter()
            .any(|event| event["event"] == "approval_required")
    );
    assert!(
        host.requests[0]
            .tools
            .iter()
            .all(|tool| tool.function.name != "shell" && tool.function.name != "write_file")
    );
}

#[tokio::test]
async fn unknown_cost_stops_tools_when_a_cost_budget_is_required() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let config = HarnessConfig {
        cost_budget_usd: Some(1.0),
        ..Default::default()
    };
    let mut host = Host {
        responses: VecDeque::from([write_call()]),
        requests: vec![],
        events: vec![],
        decision: Some(true),
    };
    let outcome = run_harness_turn(
        &store,
        &mut session,
        &config,
        HarnessTask::new("Create app", &ModelSelection::new("openai", "gpt-5.5")),
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(outcome, HarnessOutcome::BudgetExhausted));
    assert!(!root.path().join("main.dowe").exists());
    assert!(
        !host
            .events
            .iter()
            .any(|event| event["event"] == "approval_required")
    );
    assert!(!session.interrupted);
    assert!(session.usage().incomplete_cost);
}

#[test]
fn public_environment_templates_allow_names_and_placeholders_not_credentials() {
    let root = tempfile::tempdir().unwrap();
    let mut tools = HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    let call = ToolCall::new(
        "template",
        "write_file",
        json!({"path":".env.example","content":"API_KEY=\nTOKEN=<YOUR_TOKEN>\n","skill":"core/configuration","reason":"create public template"}),
    );
    let approval = tools.prepare(&call, HarnessRole::Execute).unwrap().unwrap();
    tools.apply_write(approval).unwrap();
    assert_eq!(
        std::fs::read_to_string(root.path().join(".env.example")).unwrap(),
        "API_KEY=\nTOKEN=<YOUR_TOKEN>\n"
    );
    let secret = ToolCall::new(
        "bad",
        "write_file",
        json!({"path":".env.example","content":"API_KEY=must-not-send-this\n","skill":"core/configuration","reason":"attempt credential disclosure"}),
    );
    assert!(
        tools
            .prepare(&secret, HarnessRole::Execute)
            .unwrap_err()
            .to_string()
            .contains("sensitive file change")
    );
}

#[test]
fn bounded_file_pages_never_skip_unseen_lines() {
    let root = tempfile::tempdir().unwrap();
    let source = (0..100)
        .map(|index| format!("line {index}: quoted \\\"value\\\" and \\\\ paths\n"))
        .collect::<String>();
    std::fs::write(root.path().join("README.md"), &source).unwrap();
    let tools = HarnessTools::new(
        root.path(),
        "session",
        HarnessConfig {
            max_output_bytes: 1024,
            ..Default::default()
        },
    )
    .unwrap();
    let mut offset = 1;
    let mut observed = String::new();
    loop {
        let page = tools.read("README.md", offset, 1000).unwrap();
        assert!(page.to_string().len() <= 1024);
        observed.push_str(page["content"].as_str().unwrap());
        if page["truncated"] == false {
            break;
        }
        let next = page["next_offset"].as_u64().unwrap() as usize;
        assert!(next > offset);
        offset = next;
    }
    assert_eq!(observed, source);
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[tokio::test]
async fn shell_uses_clean_environment_and_redacted_text_not_raw_byte_arrays() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join(".env"), "SECRET=private-test-value\n").unwrap();
    let config = HarnessConfig {
        shell: Some("/bin/sh".into()),
        ..Default::default()
    };
    let mut tools = HarnessTools::new(root.path(), "session", config).unwrap();
    let call = ToolCall::new(
        "shell",
        "shell",
        json!({"command":"printf private-test-value; printf error >&2; exit 3","cwd":".","reason":"test output capture"}),
    );
    let mut approval = tools.prepare(&call, HarnessRole::Execute).unwrap().unwrap();
    approval.details["command"] = json!("different command");
    assert!(tools.run_shell(approval).await.is_err());
    let approval = tools.prepare(&call, HarnessRole::Execute).unwrap().unwrap();
    let output = tools.run_shell(approval).await.unwrap();
    assert_eq!(output["exit_code"], 3);
    assert_eq!(output["stderr"], "error");
    assert!(!output.to_string().contains("private-test-value"));
    assert_eq!(output["stdout"], "[REDACTED]");
}

#[test]
fn protected_local_env_edit_preserves_other_values_and_is_never_model_input() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join(".gitignore"), ".env\n.dowe/\n").unwrap();
    std::fs::write(
        root.path().join(".env"),
        "OTHER='preserve-byte-for-byte'\nTOKEN=old\n",
    )
    .unwrap();
    set_local_environment_value(root.path(), ".env", "TOKEN", "new-secret").unwrap();
    let text = std::fs::read_to_string(root.path().join(".env")).unwrap();
    assert_eq!(
        text,
        "OTHER='preserve-byte-for-byte'\nTOKEN=\"new-secret\"\n"
    );
    let tools = HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    let projection = tools.read(".env", 1, 20).unwrap().to_string();
    assert!(!projection.contains("new-secret") && !projection.contains("preserve-byte-for-byte"));
    assert!(set_local_environment_value(root.path(), "../.env", "TOKEN", "bad").is_err());
    assert!(set_local_environment_value(root.path(), ".env.live", "TOKEN", "bad").is_err());
    std::fs::write(root.path().join(".gitignore"), ".env\n.dowe/\n!*\n").unwrap();
    assert!(set_local_environment_value(root.path(), ".env", "TOKEN", "bad").is_err());
    assert_eq!(
        std::fs::read_to_string(root.path().join(".env")).unwrap(),
        text
    );
    assert!(
        std::fs::read_dir(root.path().join(".dowe/agent-env"))
            .unwrap()
            .next()
            .is_none()
    );
}
