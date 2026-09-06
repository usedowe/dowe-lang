fn parse_notification_statement(node: &SourceNode) -> DoweResult<ServerNotificationStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "`notify` must declare exactly one result binding",
        ));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`notify` result binding must be static"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(
        node,
        &[
            "user", "id", "category", "title", "body", "route", "data", "tag",
        ],
    )?;
    let user = node
        .prop("user")
        .ok_or_else(|| node_error(node, "`notify` requires `user:<value>`"))
        .and_then(|prop| Ok(store_literal(&prop.value)?))?;
    let id = node
        .prop("id")
        .map(|prop| store_literal(&prop.value))
        .transpose()?
        .unwrap_or_else(|| StoreLiteral::String("notification".to_string()));
    let title = required_notification_literal(node, "title")?;
    let body = required_notification_literal(node, "body")?;
    let category = node
        .prop("category")
        .map(|prop| store_literal(&prop.value))
        .transpose()?
        .unwrap_or_else(|| StoreLiteral::String("process".to_string()));
    let route = node
        .prop("route")
        .map(|prop| store_literal(&prop.value))
        .transpose()?
        .unwrap_or(StoreLiteral::Null);
    let data = node
        .prop("data")
        .map(|prop| store_literal(&prop.value))
        .transpose()?
        .unwrap_or(StoreLiteral::Object(Vec::new()));
    let tag = node
        .prop("tag")
        .map(|prop| store_literal(&prop.value))
        .transpose()?
        .unwrap_or(StoreLiteral::Null);
    Ok(ServerNotificationStatement {
        binding,
        user,
        payload: StoreLiteral::Object(vec![
            ("id".to_string(), id),
            ("title".to_string(), title),
            ("body".to_string(), body),
            ("category".to_string(), category),
            ("route".to_string(), route),
            ("data".to_string(), data),
            ("tag".to_string(), tag),
        ]),
    })
}

fn required_notification_literal(node: &SourceNode, name: &str) -> DoweResult<StoreLiteral> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("`notify` requires `{name}:<value>`")))?;
    Ok(store_literal(&prop.value)?)
}
