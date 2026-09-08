use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse};
use serde_json::{Value, json};
use std::collections::VecDeque;

fn asset(path: &str, bytes: &[u8]) -> ToolCall {
    ToolCall::new(
        "asset",
        "write_asset",
        json!({
            "path": path,
            "content_base64": STANDARD.encode(bytes),
            "skill": "views/pages",
            "reason": "install binary asset"
        }),
    )
}

#[test]
fn binary_assets_round_trip_with_private_metadata() {
    let root = tempfile::tempdir().unwrap();
    let mut tools = HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    let bytes = [0, 0xff, 0x80, b'\n'];
    let approval = tools
        .prepare(
            &asset("public/assets/icon.bin", &bytes),
            HarnessRole::Execute,
        )
        .unwrap()
        .unwrap();
    assert_eq!(approval.details["byte_count"], 4);
    assert_eq!(
        approval.details["sha256"],
        "6d6f7836f1e146dc0204afb5133dae52fdc05603d8ac2dc793b481b0e0829fd1"
    );
    assert!(
        !serde_json::to_string(&approval)
            .unwrap()
            .contains(&STANDARD.encode(bytes))
    );
    tools.apply_write(approval).unwrap();
    assert_eq!(
        std::fs::read(root.path().join("public/assets/icon.bin")).unwrap(),
        bytes
    );
}

#[tokio::test]
async fn binary_asset_calls_are_redacted_from_session_and_provider_projection() {
    struct Host {
        responses: VecDeque<Value>,
        requests: Vec<AgentRequest>,
    }

    impl HarnessHost for Host {
        async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
            self.requests.push(request.clone());
            Ok(AgentServerResponse {
                request_id: request.request_id.clone(),
                request_type: request.request_type,
                model: request.model.clone(),
                payload: self.responses.pop_front().unwrap(),
            })
        }

        async fn approve(&mut self, _: &Approval) -> AgentResult<Option<bool>> {
            Ok(Some(true))
        }

        fn event(&mut self, _: &Value) -> AgentResult<()> {
            Ok(())
        }
    }

    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let bytes = [0, 0xff, 0x80, b'\n'];
    let encoded = STANDARD.encode(bytes);
    let response = json!({"output":[{"type":"function_call","call_id":"asset","name":"write_asset","arguments":json!({"path":"public/assets/icon.bin","content_base64":encoded,"skill":"views/pages","reason":"install binary asset"}).to_string()}]});
    let mut host = Host {
        responses: VecDeque::from([response, json!({"output_text":"done"})]),
        requests: Vec::new(),
    };

    run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask::new("Install an icon", &ModelSelection::new("openai", "gpt-5.5")),
        &mut host,
    )
    .await
    .unwrap();

    let persisted = serde_json::to_vec(&store.load_session(&session.id).unwrap()).unwrap();
    let projected = serde_json::to_vec(&host.requests[1]).unwrap();
    for visible in [&persisted, &projected] {
        assert!(!String::from_utf8_lossy(visible).contains(&encoded));
        assert!(!visible.windows(bytes.len()).any(|window| window == bytes));
    }
    assert_eq!(
        std::fs::read(root.path().join("public/assets/icon.bin")).unwrap(),
        bytes
    );
}

#[test]
fn binary_assets_reject_malformed_input_and_base_drift() {
    let root = tempfile::tempdir().unwrap();
    let mut tools = HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    let mut bad = asset("public/assets/icon.bin", b"bytes");
    bad.arguments["content_base64"] = json!("not base64!");
    assert!(tools.prepare(&bad, HarnessRole::Execute).is_err());
    let oversized = asset("public/assets/large.bin", &vec![0u8; 8 * 1024 * 1024 + 1]);
    assert!(tools.prepare(&oversized, HarnessRole::Execute).is_err());
    let path = root.path().join("public/assets/icon.bin");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"old").unwrap();
    let approval = tools
        .prepare(
            &asset("public/assets/icon.bin", b"new"),
            HarnessRole::Execute,
        )
        .unwrap()
        .unwrap();
    std::fs::write(&path, b"drift").unwrap();
    assert!(tools.apply_write(approval).is_err());
}
