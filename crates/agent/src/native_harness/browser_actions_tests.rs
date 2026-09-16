use super::*;
use crate::native_harness::HarnessConfig;

#[test]
fn browser_actions_reject_remote_targets_and_unbounded_input() {
    assert!(validate_action(&BrowserAction::Navigate { url: "https://example.com".into() }).is_err());
    assert!(validate_action(&BrowserAction::Fill { selector: "#email".into(), value: "x\n".into() }).is_err());
    assert!(validate_endpoint("http://example.com:9222").is_err());
    assert!(validate_endpoint("http://127.0.0.1:9222").is_ok());
}

#[test]
fn browser_commands_escape_dom_inputs_as_json_strings() {
    let (_, params, _) = command_for_action(&BrowserAction::Fill { selector: "#email".into(), value: "quoted ' value".into() }).unwrap();
    let expression = params["expression"].as_str().unwrap();
    assert!(expression.contains("querySelector(\"#email\")"));
    assert!(expression.contains("quoted ' value"));
}

#[test]
fn browser_action_tool_is_approval_gated_and_role_scoped() {
    let root = tempfile::tempdir().unwrap();
    let mut tools = HarnessTools::new(root.path(), "browser-gate", HarnessConfig::default()).unwrap();
    let call = ToolCall::new("browser", "execute_browser_actions", json!({
        "cdp_endpoint":"ws://127.0.0.1:9222",
        "actions":[{"navigate":{"url":"http://127.0.0.1:3000/"}}],
        "reason":"verify sign in"
    }));
    assert!(tools.prepare(&call, HarnessRole::Research).is_err());
    let approval = tools.prepare(&call, HarnessRole::Execute).unwrap().unwrap();
    assert_eq!(approval.call.name, "execute_browser_actions");
    assert!(tools.pending.contains_key(&approval.id));
    tools.reject(approval).unwrap();
}

#[test]
fn durable_use_case_actions_map_only_to_supported_dom_operations() {
    let fill = UseCaseAction { id: "email".into(), kind: "fill".into(), target: "#email".into(), value: Some(Value::String("user@example.test".into())) };
    assert!(matches!(browser_action(&fill), Ok(BrowserAction::Fill { .. })));
    let unsupported = UseCaseAction { id: "wait".into(), kind: "wait_for_network".into(), target: "network-idle".into(), value: None };
    assert!(browser_action(&unsupported).is_err());
}

#[tokio::test]
async fn executes_actions_and_records_dom_evidence_over_cdp() {
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();
        while let Some(Ok(Message::Text(text))) = socket.next().await {
            let request: Value = serde_json::from_str(&text).unwrap();
            let value = if request["method"] == "Runtime.evaluate" {
                json!({"id":request["id"],"result":{"result":{"value":"<html><body>done</body></html>"}}})
            } else { json!({"id":request["id"],"result":{}}) };
            socket.send(Message::Text(serde_json::to_string(&value).unwrap().into())).await.unwrap();
        }
    });
    let report = execute_browser_actions(&format!("ws://{address}"), &[
        BrowserAction::Navigate { url: "http://127.0.0.1:3000/".into() },
        BrowserAction::Click { selector: "#submit".into() },
        BrowserAction::Fill { selector: "#email".into(), value: "a@example.test".into() },
        BrowserAction::Press { key: "Enter".into() },
    ]).await.unwrap();
    assert!(report.passed);
    assert_eq!(report.results.len(), 4);
    assert_eq!(report.dom_bytes, Some(30));
    assert!(report.dom_sha256.is_some());
}
