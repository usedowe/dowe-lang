#[test]
fn parses_server_notification_action_and_infers_result() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        r#"main
  server port:0
    route "/chat"
      handler
        notify sent id:"message-1" category:"chat" user:"user-1" title:"New message" body:"Hello" route:"/chat/1" data:{ chatId:"1" }
        return json:sent"#
            .to_string(),
    )
    .expect("source");
    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/chat")
        .expect("endpoint");
    assert!(matches!(
        endpoint.endpoint.action.statements.first(),
        Some(ServerStatement::Notification(statement))
            if statement.binding == "sent"
                && statement.user == StoreLiteral::String("user-1".to_string())
                && matches!(
                    &statement.payload,
                    StoreLiteral::Object(fields)
                        if fields.iter().any(|(name, value)| {
                            name == "id" && value == &StoreLiteral::String("message-1".to_string())
                        })
                )
    ));
}
