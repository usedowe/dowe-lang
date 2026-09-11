use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse};
use serde_json::{json, Value};
use std::collections::VecDeque;

struct Host {
    responses: VecDeque<Value>,
    validations: VecDeque<Value>,
    requests: usize,
    events: Vec<Value>,
}

impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.requests += 1;
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: self.responses.pop_front().expect("provider response"),
        })
    }

    async fn approve(&mut self, _: &Approval) -> AgentResult<Option<bool>> {
        Ok(Some(true))
    }

    fn validate_dowe_project(
        &mut self,
        _: &std::path::Path,
    ) -> impl std::future::Future<Output = AgentResult<Value>> {
        let result = self.validations.pop_front().expect("validation response");
        async move { Ok(result) }
    }

    fn event(&mut self, event: &Value) -> AgentResult<()> {
        self.events.push(event.clone());
        Ok(())
    }
}

#[tokio::test]
async fn failed_automatic_validation_returns_to_the_model_before_completion() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("theme.dowe"),
        "theme\n  design defaultTheme:\"light\"\n    Card variant:\"outlined\"\n",
    )
    .unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let write = json!({
        "output": [{
            "type": "function_call",
            "call_id": "write",
            "name": "write_file",
            "arguments": json!({
                "path": "main.dowe",
                "content": "main {}\n",
                "skill": "core",
                "reason": "create application root"
            }).to_string()
        }]
    });
    let mut host = Host {
        responses: VecDeque::from([
            write,
            json!({"output_text":"I will repair the diagnostics."}),
            json!({"output_text":"Validated."}),
        ]),
        validations: VecDeque::from([
            json!({"status":"failed","compiler":{"status":"failed","error":"invalid declaration"}}),
            json!({"status":"passed","compiler":{"status":"passed"}}),
        ]),
        requests: 0,
        events: Vec::new(),
    };

    let outcome = run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask::new(
            "Create the Dowe app",
            &ModelSelection::new("openai", "gpt-5.5"),
        ),
        &mut host,
    )
    .await
    .unwrap();

    assert!(matches!(outcome, HarnessOutcome::Completed));
    assert_eq!(host.requests, 3);
    assert!(host.events.iter().any(|event| {
        event["event"] == "validation_finished" && event["result"]["status"] == "failed"
    }));
    assert!(host.events.iter().any(|event| {
        event["event"] == "task_state"
            && event["status"] == "done"
            && event["validation"]["status"] == "passed"
    }));
}

#[tokio::test]
async fn explicit_failed_validation_cannot_be_hidden_by_a_no_tool_response() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let validate = json!({
        "output": [{
            "type": "function_call",
            "call_id": "validate",
            "name": "validate_dowe_project",
            "arguments": "{}"
        }]
    });
    let mut host = Host {
        responses: VecDeque::from([validate, json!({"output_text":"The project needs repair."})]),
        validations: VecDeque::from([json!({
            "status":"failed",
            "compiler":{"status":"failed","error":"invalid declaration"}
        })]),
        requests: 0,
        events: Vec::new(),
    };

    let outcome = run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask::new(
            "Validate the Dowe project",
            &ModelSelection::new("openai", "gpt-5.5"),
        ),
        &mut host,
    )
    .await
    .unwrap();

    assert!(matches!(outcome, HarnessOutcome::ValidationFailed));
    assert!(host.events.iter().any(|event| {
        event["event"] == "task_state"
            && event["status"] == "failed"
            && event["reason"] == "dowe_validation_failed"
    }));
}
