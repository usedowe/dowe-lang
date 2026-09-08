use dowe_agent::native_harness::*;
use dowe_agent::{AgentError, AgentRequest, AgentResult, AgentServerResponse};
use serde_json::{Value, json};
use std::collections::VecDeque;

struct Host {
    outcomes: VecDeque<Result<Value, String>>,
    requests: usize,
    events: Vec<Value>,
    decision: Option<bool>,
}

impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.requests += 1;
        match self.outcomes.pop_front().expect("unexpected provider call") {
            Ok(payload) => Ok(AgentServerResponse {
                request_id: request.request_id.clone(),
                request_type: request.request_type,
                model: request.model.clone(),
                payload,
            }),
            Err(message) => Err(AgentError::new(message)),
        }
    }

    async fn approve(&mut self, _: &Approval) -> AgentResult<Option<bool>> {
        Ok(self.decision)
    }

    fn event(&mut self, event: &Value) -> AgentResult<()> {
        self.events.push(event.clone());
        Ok(())
    }
}

fn setup(
    outcomes: Vec<Result<Value, String>>,
    decision: Option<bool>,
) -> (
    tempfile::TempDir,
    tempfile::TempDir,
    HarnessStore,
    HarnessSession,
    Host,
) {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let session = store.create_session().unwrap();
    (
        home,
        root,
        store,
        session,
        Host {
            outcomes: outcomes.into(),
            requests: 0,
            events: vec![],
            decision,
        },
    )
}

#[tokio::test]
async fn transient_transport_failure_succeeds_after_one_retry() {
    let (_home, _root, store, mut session, mut host) = setup(
        vec![
            Err("connection reset by peer".into()),
            Ok(json!({"output_text":"done"})),
        ],
        None,
    );
    let outcome = run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask::new("Run task", &ModelSelection::new("openai", "gpt-5.5")),
        &mut host,
    )
    .await
    .unwrap();
    assert!(matches!(outcome, HarnessOutcome::Completed));
    assert_eq!(host.requests, 2);
    assert_eq!(
        host.events
            .iter()
            .filter(|event| event["event"] == "provider_retry")
            .count(),
        1
    );
    assert_eq!(
        host.events
            .iter()
            .find(|event| event["event"] == "provider_retry")
            .unwrap()["classification"],
        "connection_reset"
    );
}

#[tokio::test]
async fn transient_transport_failure_is_exhausted_after_two_retries() {
    let (_home, _root, store, mut session, mut host) = setup(
        vec![
            Err("service unavailable".into()),
            Err("service unavailable".into()),
            Err("service unavailable".into()),
        ],
        None,
    );
    let error = run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask::new("Run task", &ModelSelection::new("openai", "gpt-5.5")),
        &mut host,
    )
    .await
    .unwrap_err();
    assert_eq!(error.to_string(), "service unavailable");
    assert_eq!(host.requests, 3);
    assert_eq!(
        host.events
            .iter()
            .filter(|event| event["event"] == "provider_retry")
            .count(),
        2
    );
}

#[tokio::test]
async fn provider_failure_after_tool_result_is_not_retried() {
    let tool_call = json!({"output":[{"type":"function_call","call_id":"write","name":"write_file","arguments":json!({"path":"main.dowe","content":"main {}\n","skill":"core","reason":"create root"}).to_string()}]});
    let (_home, _root, store, mut session, mut host) = setup(
        vec![Ok(tool_call), Err("connection reset by peer".into())],
        Some(false),
    );
    let _ = run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask::new("Create app", &ModelSelection::new("openai", "gpt-5.5")),
        &mut host,
    )
    .await
    .unwrap_err();
    assert_eq!(host.requests, 2);
    assert_eq!(
        host.events
            .iter()
            .filter(|event| event["event"] == "provider_retry")
            .count(),
        0
    );
}
