use dowe_agent::native_harness::{HarnessTurn, ToolResult};
use dowe_agent::{AgentMessage, AgentMessageContent, AgentMessagePart, AgentPrepareOptions, AgentProviderProtocol, prepare_agent_request};
use serde_json::json;

#[test]
fn captured_screenshot_projects_as_a_normal_image_bearing_user_turn() {
    let root = tempfile::tempdir().unwrap();
    let mut request = prepare_agent_request(root.path(), "inspect", AgentPrepareOptions::default()).unwrap().request;
    request.extra.insert("dowe_harness_turns".into(), serde_json::to_value(vec![HarnessTurn {
        message: Some(AgentMessage { role: "user".into(), content: AgentMessageContent::Parts(vec![
            AgentMessagePart::Text { text: "Captured web screenshot".into() },
            AgentMessagePart::ImageUrl { image_url: dowe_agent::ImageUrl { url: "data:image/png;base64,aGVsbG8=".into() } },
        ]) }), ..Default::default()
    }, HarnessTurn { results: vec![ToolResult { id: "shot".into(), name: "capture_web_screenshot".into(), failed: false, output: json!({"status":"captured","path":".dowe/visual-qa/s/rendered.png"}) }], ..Default::default() }]).unwrap());
    let mut body = json!({"messages":[]});
    dowe_agent::native_harness::apply_harness_turns_for_test(AgentProviderProtocol::OpenAiCompletions, &request, &mut body).unwrap();
    assert_eq!(body["messages"][0]["content"][1]["type"], "image_url");
}
