fn http_endpoint_behavior(node: &SourceNode) -> DoweResult<Option<EndpointBehavior>> {
    if let Some(binding) = return_reference_prop(node, "proxy")? {
        return Ok(Some(EndpointBehavior::HttpProxy(HttpProxyEndpoint {
            binding,
        })));
    }
    if let Some(binding) = return_reference_prop(node, "bytes")? {
        return Ok(Some(EndpointBehavior::HttpBytes(HttpBytesEndpoint {
            status: return_status(node)?,
            binding,
            content_type: return_content_type(node)?,
            headers: return_headers(node)?,
            cookies: return_cookies(node)?,
        })));
    }
    if let Some(upstream) = return_reference_prop(node, "agent")? {
        let request = return_reference_prop(node, "request")?
            .ok_or_else(|| node_error(node, "agent response must declare `request` binding"))?;
        return Ok(Some(EndpointBehavior::AgentResponse(
            AgentResponseEndpoint { upstream, request },
        )));
    }
    if let Some(value) = return_json_value(node) {
        if !returns_created_json(node) {
            return Ok(Some(EndpointBehavior::HttpActionJson(
                HttpActionJsonEndpoint {
                    status: return_status(node)?,
                    value: store_literal(value)?,
                },
            )));
        }
    }
    Ok(None)
}

fn handler_behavior(
    node: &SourceNode,
    path: &str,
    action: &ServerAction,
) -> DoweResult<EndpointBehavior> {
    if has_reference_log(action)
        && let Some(behavior) = database_action_endpoint_behavior(
            action,
            return_json_value(node),
            return_status(node)?,
        )?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = database_endpoint_behavior(action, return_json_ref(node))? {
        return Ok(behavior);
    }
    if let Some(behavior) =
        database_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) =
        kv_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) =
        vector_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = http_endpoint_behavior(node)? {
        return Ok(behavior);
    }
    if return_text(node).is_some_and(|value| value.contains("req.context")) {
        Ok(EndpointBehavior::TextTemplate(return_text(node).unwrap()))
    } else if path.contains("/:")
        && return_text(node).is_some_and(|value| value.contains("req.params"))
    {
        Ok(EndpointBehavior::UserGreeting)
    } else if let Some(text) = return_text(node) {
        Ok(EndpointBehavior::StaticText(text))
    } else {
        Err(node_error(
            node,
            "handler must return supported text response",
        ))
    }
}

fn exported_handler_behavior(
    node: &SourceNode,
    action: &ServerAction,
) -> DoweResult<EndpointBehavior> {
    if has_reference_log(action)
        && let Some(behavior) = database_action_endpoint_behavior(
            action,
            return_json_value(node),
            return_status(node)?,
        )?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = database_endpoint_behavior(action, return_json_ref(node))? {
        return Ok(behavior);
    }
    if let Some(behavior) =
        database_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) =
        kv_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) =
        vector_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = http_endpoint_behavior(node)? {
        return Ok(behavior);
    }
    if let Some(text) = return_text(node)
        && text.contains("req.context")
    {
        return Ok(EndpointBehavior::TextTemplate(text));
    }
    if let Some(text) = return_text(node) {
        return Ok(EndpointBehavior::StaticText(text));
    }
    if returns_created_json(node) {
        return Ok(EndpointBehavior::CreatePostJson);
    }
    Err(node_error(
        node,
        "external handler must return supported response behavior",
    ))
}

fn has_reference_log(action: &ServerAction) -> bool {
    action.statements.iter().any(|statement| {
        matches!(
            statement,
            ServerStatement::Log(ServerLog { values, .. })
                if values
                    .iter()
                    .any(|value| matches!(value, ServerLogValue::Reference(_)))
        )
    })
}

fn return_text(node: &SourceNode) -> Option<String> {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("text"))
        .and_then(|prop| match &prop.value {
            SourceValue::String(value) => Some(value.clone()),
            _ => None,
        })
}

fn returns_created_json(node: &SourceNode) -> bool {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("json"))
        .is_some_and(|prop| match &prop.value {
            SourceValue::Object(entries) => entries.iter().any(|entry| {
                matches!(
                    entry,
                    SourceObjectEntry::KeyValue {
                        key,
                        value: SourceValue::Boolean(true)
                    } if key == "created"
                )
            }),
            _ => false,
        })
}

fn return_json_ref(node: &SourceNode) -> Option<String> {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("json"))
        .and_then(|prop| match &prop.value {
            SourceValue::Bareword(value) => Some(value.clone()),
            _ => None,
        })
}

fn return_json_value(node: &SourceNode) -> Option<&SourceValue> {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("json"))
        .map(|prop| &prop.value)
}

fn return_reference_prop(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop(name))
        .map(|prop| {
            prop.value
                .as_string_like()
                .ok_or_else(|| prop_error(prop, format!("`{name}` must be a binding reference")))
        })
        .transpose()
}

