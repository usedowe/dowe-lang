#[test]
fn handles_mcp_initialize_tools_and_resources() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    let initialize = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#,
    )
    .expect("initialize")
    .expect("response");
    let tools = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    )
    .expect("tools")
    .expect("response");
    let resources = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":3,"method":"resources/list","params":{}}"#,
    )
    .expect("resources")
    .expect("response");
    let initialized: Value = serde_json::from_str(&initialize).expect("initialize json");
    let listed_tools: Value = serde_json::from_str(&tools).expect("tools json");
    let listed_resources: Value = serde_json::from_str(&resources).expect("resources json");

    assert_eq!(initialized["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(initialized["result"]["serverInfo"]["name"], "dowe-agent");
    assert_eq!(
        listed_tools["result"]["tools"]
            .as_array()
            .expect("tools array")
            .len(),
        4
    );
    assert!(
        listed_resources["result"]["resources"]
            .as_array()
            .expect("resources array")
            .iter()
            .any(|resource| resource["uri"] == "dowe://skills/views")
    );
    assert!(
        listed_resources["result"]["resources"]
            .as_array()
            .expect("resources array")
            .iter()
            .any(|resource| { resource["uri"] == "dowe://skills/views/references/styles.md" })
    );
    assert!(
        listed_resources["result"]["resources"]
            .as_array()
            .expect("resources array")
            .iter()
            .all(|resource| !resource["uri"]
                .as_str()
                .is_some_and(|uri| uri.ends_with("/full")))
    );
}

#[test]
fn handles_mcp_tool_calls_and_notifications() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    let response = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":"search","method":"tools/call","params":{"name":"dowe_examples_search","arguments":{"query":"dashboard sidebar form","limit":3}}}"#,
    )
    .expect("call")
    .expect("response");
    let notification = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    )
    .expect("notification");
    let payload: Value = serde_json::from_str(&response).expect("response json");

    assert_eq!(payload["id"], "search");
    assert_eq!(payload["result"]["isError"], false);
    assert!(
        payload["result"]["structuredContent"]["results"]
            .as_array()
            .expect("results")
            .iter()
            .any(|result| result["id"] == "dashboard-layout")
    );
    assert!(notification.is_none());
}

#[test]
fn handles_mcp_public_skill_resource_tool_call() {
    let temp = tempfile::tempdir().expect("tempdir");
    let response = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":"resource","method":"tools/call","params":{"name":"dowe_skills_get","arguments":{"id":"views","resource":"references/styles.md"}}}"#,
    )
    .expect("call")
    .expect("response");
    let payload: Value = serde_json::from_str(&response).expect("response json");

    assert_eq!(payload["result"]["isError"], false);
    assert_eq!(
        payload["result"]["structuredContent"]["path"],
        "references/styles.md"
    );
    assert!(
        payload["result"]["structuredContent"]["content"]
            .as_str()
            .expect("content")
            .contains("# Style and design-system reference")
    );
    assert!(
        !payload["result"]["structuredContent"]["content"]
            .as_str()
            .expect("content")
            .contains("# Canvas reference")
    );
}

#[test]
fn reads_mcp_skill_resource_and_recovers_from_invalid_json() {
    let temp = tempfile::tempdir().expect("tempdir");
    let resource = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":4,"method":"resources/read","params":{"uri":"dowe://skills/views/full"}}"#,
    )
    .expect("resource")
    .expect("response");
    let invalid = handle_mcp_message(temp.path(), "{")
        .expect("invalid")
        .expect("parse response");
    let resource_payload: Value = serde_json::from_str(&resource).expect("resource json");
    let invalid_payload: Value = serde_json::from_str(&invalid).expect("invalid json");

    assert!(
        resource_payload["result"]["contents"][0]["text"]
            .as_str()
            .expect("text")
            .contains("## Resource: references/views.md")
    );
    assert!(
        resource_payload["result"]["contents"][0]["text"]
            .as_str()
            .expect("text")
            .contains("Every page starts with `Section`")
    );
    assert_eq!(invalid_payload["error"]["code"], -32700);
}
