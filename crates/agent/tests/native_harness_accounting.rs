use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse, AgentUsage};
use serde_json::{Value, json};

#[test]
fn retries_and_compaction_are_counted_without_counting_final_usage_twice() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let usage = json!(AgentUsage {
        input: 10,
        output: 5,
        cache_read: 2,
        cache_write: 1,
        cost_usd: Some(0.2),
        estimated_cost: false
    });
    session.events = vec![
        json!({"event":"request_attempt","requestId":"first","attempt":0,"usage":null}),
        json!({"event":"request_attempt","requestId":"first","attempt":1,"usage":usage}),
        json!({"event":"response_received","requestId":"first","usage":usage}),
        json!({"event":"context_compacted","requestId":"compact","usage":usage}),
    ];
    let total = session.usage();
    assert_eq!(total.responses, 3);
    assert_eq!(total.input, 20);
    assert_eq!(total.output, 10);
    assert_eq!(total.cache_read, 4);
    assert_eq!(total.cost_usd, 0.4);
    assert!(total.incomplete_usage && total.incomplete_cost);
}

struct InterruptedHost {
    events: Vec<Value>,
}
impl HarnessHost for InterruptedHost {
    async fn send(&mut self, _: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.events
            .push(json!({"event":"request_attempt","attempt":0,"interrupted":true,"usage":null}));
        std::future::pending().await
    }
    async fn approve(&mut self, _: &Approval) -> AgentResult<Option<bool>> {
        panic!("no tools were returned")
    }
    fn event(&mut self, _: &Value) -> AgentResult<()> {
        Ok(())
    }
    fn take_request_events(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.events)
    }
}

#[tokio::test]
async fn an_active_session_cannot_start_a_second_task_or_be_deleted() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let mut other = session.clone();
    let mut host = InterruptedHost { events: vec![] };
    let mut second_host = InterruptedHost { events: vec![] };
    let config = HarnessConfig::default();
    let active = ModelSelection::new("openai", "gpt-5.5");
    let first = run_harness_turn(
        &store,
        &mut session,
        &config,
        HarnessTask::new("Explain theme", &active),
        &mut host,
    );
    tokio::pin!(first);
    tokio::select! {
        result = &mut first => panic!("unexpected completion: {result:?}"),
        _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {}
    }
    let error = run_harness_turn(
        &store,
        &mut other,
        &config,
        HarnessTask::new("Second task", &active),
        &mut second_host,
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("active task"));
    assert!(second_host.events.is_empty());
    assert!(store.delete_session(&other.id).is_err());
}

#[tokio::test]
async fn timeout_preserves_unknown_attempt_receipts_without_replay() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let mut host = InterruptedHost { events: vec![] };
    let config = HarnessConfig {
        duration_seconds: 1,
        ..Default::default()
    };
    let active = ModelSelection::new("openai", "gpt-5.5");
    assert!(
        run_harness_turn(
            &store,
            &mut session,
            &config,
            HarnessTask::new("Explain main.dowe", &active),
            &mut host
        )
        .await
        .is_err()
    );
    let restored = store.load_session(&session.id).unwrap();
    assert_eq!(restored.usage().responses, 1);
    assert!(restored.usage().incomplete_cost);
    assert!(
        restored
            .events
            .iter()
            .any(|event| event["event"] == "request_attempt" && event["interrupted"] == true)
    );
}
