fn http_endpoint_behavior(node: &SourceNode) -> DoweResult<Option<EndpointBehavior>> {
    if let Some(upstream) = return_reference_prop(node, "reverse")? {
        return Ok(Some(EndpointBehavior::HttpReverseProxy(
            HttpReverseProxyEndpoint {
                upstream,
                strategy: return_reverse_proxy_strategy(node)?,
                state: return_reference_prop(node, "state")?,
                loading_url: return_reference_prop(node, "loadingUrl")?,
                error_url: return_reference_prop(node, "errorUrl")?,
            },
        )));
    }
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
    let behavior = handler_behavior_without_task_timing(node, path, action)?;
    validate_response_headers_task_scope(node, action, &behavior)?;
    Ok(behavior)
}

fn handler_behavior_without_task_timing(
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
    if let Some(behavior) =
        queue_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = http_endpoint_behavior(node)? {
        validate_reverse_proxy_source(node, action, &behavior)?;
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
    let behavior = exported_handler_behavior_without_task_timing(node, action)?;
    validate_response_headers_task_scope(node, action, &behavior)?;
    Ok(behavior)
}

fn exported_handler_behavior_without_task_timing(
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
    if let Some(behavior) =
        queue_action_endpoint_behavior(action, return_json_value(node), return_status(node)?)?
    {
        return Ok(behavior);
    }
    if let Some(behavior) = http_endpoint_behavior(node)? {
        validate_reverse_proxy_source(node, action, &behavior)?;
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

fn validate_response_headers_task_scope(
    node: &SourceNode,
    action: &ServerAction,
    behavior: &EndpointBehavior,
) -> DoweResult<()> {
    if matches!(behavior, EndpointBehavior::HttpReverseProxy(_))
        && node
            .children
            .last()
            .is_some_and(|child| child.name == "return" && child.prop("reverse").is_some())
    {
        return Ok(());
    }
    for statement in &action.statements {
        let ServerStatement::Task(job) = statement else {
            continue;
        };
        if matches!(job.timing, crate::model::ServerTaskTiming::ResponseHeaders) {
            return Err(DoweError::at_path(
                &job.source_path,
                format!(
                    "{}: `after:\"headers\"` is only valid in an HTTP handler whose final response is `return reverse:...`",
                    job.source_line
                ),
            ));
        }
    }
    Ok(())
}

fn validate_reverse_proxy_source(
    node: &SourceNode,
    action: &ServerAction,
    behavior: &EndpointBehavior,
) -> DoweResult<()> {
    let EndpointBehavior::HttpReverseProxy(response) = behavior else {
        return Ok(());
    };
    let root = response.upstream.split('.').next().unwrap_or_default();
    let from_cache = action.statements.iter().any(|statement| {
        matches!(
            statement,
            ServerStatement::Kv(ServerKvStatement::Get {
                binding,
                required: true,
                ..
            }) if binding == root
        )
    });
    if !response.upstream.contains('.') || !from_cache {
        return Err(node_error(
            node,
            "reverse proxy upstream must reference a field from a required Cache.get binding",
        ));
    }
    for reference in [
        response.state.as_deref(),
        response.loading_url.as_deref(),
        response.error_url.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if reference.split('.').next().unwrap_or_default() != root || !reference.contains('.') {
            return Err(node_error(
                node,
                "reverse proxy state and fallback URLs must reference the same required Cache.get binding",
            ));
        }
    }
    Ok(())
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

fn return_reverse_proxy_strategy(node: &SourceNode) -> DoweResult<ReverseProxyStrategy> {
    let value = node
        .children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("strategy"));
    match value.map(|prop| (&prop.value, prop)) {
        None => Ok(ReverseProxyStrategy::Single),
        Some((SourceValue::String(value), _)) if value == "roundRobin" => {
            Ok(ReverseProxyStrategy::RoundRobin)
        }
        Some((_, prop)) => Err(prop_error(
            prop,
            "`strategy` must be the static string `roundRobin`",
        )),
    }
}
