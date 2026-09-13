use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse};
use serde_json::{Value, json};
use std::collections::VecDeque;

struct Host {
    responses: VecDeque<Value>,
    validations: VecDeque<Value>,
    requests: usize,
    events: Vec<Value>,
}

fn reference_skill_calls() -> Value {
    let root = tempfile::tempdir().unwrap();
    let tools = HarnessTools::new(root.path(), "skill-fixture", HarnessConfig::default()).unwrap();
    let mut output = Vec::new();
    let mut index = 0;
    for id in [
        "theme",
        "views",
        "views/layouts",
        "views/pages",
        "views/components",
        "views/catalog",
        "views/shape",
        "views/reference-ui",
        "views/assets",
        "views/svg",
    ] {
        let mut offset = 1_usize;
        let mut hash = None::<String>;
        loop {
            let mut arguments = json!({"id": id, "offset": offset});
            if let Some(value) = &hash {
                arguments["hash"] = json!(value);
            }
            let page = tools
                .execute_skill(&ToolCall::new("fixture", "get_skill", arguments.clone()))
                .unwrap();
            output.push(json!({
                "type": "function_call",
                "call_id": format!("skill-{index}"),
                "name": "get_skill",
                "arguments": arguments.to_string(),
            }));
            index += 1;
            hash = page["hash"].as_str().map(str::to_owned);
            if !page["truncated"].as_bool().unwrap_or(false) {
                break;
            }
            offset = page["next_offset"].as_u64().unwrap() as usize;
        }
    }
    json!({"output": output})
}

fn ui_write_call() -> Value {
    json!({
        "output": [{
            "type": "function_call",
            "call_id": "write-page",
            "name": "write_file",
            "arguments": json!({
                "path": "views/page.dowe",
                "content": "page Home\n  Text\n    \"Hello\"\n",
                "skill": "views/pages",
                "reason": "create the reference UI page"
            }).to_string()
        }]
    })
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
    let repair_write = json!({
        "output": [{
            "type": "function_call",
            "call_id": "repair-write",
            "name": "write_file",
            "arguments": json!({
                "path": "views/page.dowe",
                "content": "page Home\n  Text\n    \"Repaired\"\n",
                "skill": "core",
                "reason": "repair the validation diagnostics"
            }).to_string()
        }]
    });
    let mut host = Host {
        responses: VecDeque::from([
            write,
            json!({"output_text":"I will repair the diagnostics."}),
            repair_write,
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
    assert_eq!(host.requests, 4);
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

#[tokio::test]
async fn ui_validation_returns_compiler_diagnostics_with_quality_findings() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("views")).unwrap();
    std::fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    std::fs::write(
        root.path().join("theme.dowe"),
        "theme\n  design defaultTheme:\"light\"\n    Card variant:\"solid\"\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("views/page.dowe"),
        "page Home\n  Text color:\"muted\"\n    \"Caption\"\n",
    )
    .unwrap();
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
            "status":"passed",
            "compiler":{"status":"passed","marker":"compiler-ran"}
        })]),
        requests: 0,
        events: Vec::new(),
    };

    let outcome = run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask::new(
            "Validate this reference UI project",
            &ModelSelection::new("openai", "gpt-5.5"),
        ),
        &mut host,
    )
    .await
    .unwrap();

    assert!(matches!(outcome, HarnessOutcome::ValidationFailed));
    let validation = session
        .turns
        .iter()
        .flat_map(|turn| turn.results.iter())
        .find(|result| result.name == "validate_dowe_project")
        .expect("validation result");
    assert_eq!(validation.output["status"], "failed");
    assert_eq!(validation.output["quality"]["status"], "failed");
    assert_eq!(validation.output["compiler"]["status"], "passed");
    assert_eq!(
        validation.output["compiler"]["compiler"]["marker"],
        "compiler-ran"
    );
}

#[tokio::test]
async fn attached_reference_ui_requires_a_capture_attempt_before_completion() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("views")).unwrap();
    std::fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    std::fs::write(
        root.path().join("theme.dowe"),
        "theme\n  design defaultTheme:\"light\"\n    Card variant:\"solid\"\n",
    )
    .unwrap();
    let reference = root.path().join("reference.png");
    std::fs::write(&reference, b"visual evidence").unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let mut host = Host {
        responses: VecDeque::from([
            reference_skill_calls(),
            ui_write_call(),
            json!({"output_text":"The page is ready."}),
            json!({"output_text":"No further changes are needed."}),
        ]),
        validations: VecDeque::from([json!({"status":"passed","compiler":{"status":"passed"}})]),
        requests: 0,
        events: Vec::new(),
    };

    let outcome = run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask {
            image_paths: &[reference],
            ..HarnessTask::new(
                "Implement this landing page from the attached reference image",
                &ModelSelection::new("openai", "gpt-5.5"),
            )
        },
        &mut host,
    )
    .await
    .unwrap();

    assert!(matches!(outcome, HarnessOutcome::ValidationFailed));
    assert!(host.events.iter().any(|event| {
        event["event"] == "task_state"
            && event["status"] == "failed"
            && event["reason"] == "visual_qa_required"
            && event["visualQA"]["status"] == "not_run"
    }));
}

#[tokio::test]
async fn attached_reference_ui_preserves_not_run_after_a_capture_attempt() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("views")).unwrap();
    std::fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
    std::fs::write(
        root.path().join("theme.dowe"),
        "theme\n  design defaultTheme:\"light\"\n    Card variant:\"solid\"\n",
    )
    .unwrap();
    let reference = root.path().join("reference.png");
    std::fs::write(&reference, b"visual evidence").unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let capture = json!({
        "output": [{
            "type": "function_call",
            "call_id": "capture",
            "name": "capture_web_screenshot",
            "arguments": json!({
                "url": "http://127.0.0.1:65536/",
                "reason": "compare the final page with the attached reference"
            }).to_string()
        }]
    });
    let mut host = Host {
        responses: VecDeque::from([
            reference_skill_calls(),
            ui_write_call(),
            json!({"output_text":"The page is ready."}),
            capture,
            json!({"output_text":"The capture was attempted; comparison is unavailable."}),
        ]),
        validations: VecDeque::from([json!({"status":"passed","compiler":{"status":"passed"}})]),
        requests: 0,
        events: Vec::new(),
    };

    let outcome = run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask {
            image_paths: &[reference],
            ..HarnessTask::new(
                "Implement this landing page from the attached reference image",
                &ModelSelection::new("openai", "gpt-5.5"),
            )
        },
        &mut host,
    )
    .await
    .unwrap();

    assert!(matches!(outcome, HarnessOutcome::ValidationFailed));
    assert!(host.events.iter().any(|event| {
        event["event"] == "task_state"
            && event["status"] == "failed"
            && event["reason"] == "visual_qa_not_run"
            && event["visualQA"]["status"] == "not_run"
            && event["visualQA"]["attempted"] == true
    }));
}
