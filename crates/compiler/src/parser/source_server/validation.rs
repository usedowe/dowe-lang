fn parse_binding_type(
    node: &SourceNode,
    value: &str,
    types: &TypeRegistry,
) -> DoweResult<(String, Option<DoweType>)> {
    let Some((binding, type_name)) = value.split_once(':') else {
        validate_binding_name(node, value)?;
        return Ok((value.to_string(), None));
    };
    if binding.is_empty() || type_name.is_empty() {
        return Err(node_error(node, "typed binding must use `name:Type`"));
    }
    validate_binding_name(node, binding)?;
    let schema = types.resolve(node, type_name)?;
    Ok((binding.to_string(), Some(schema)))
}

fn validate_binding_name(node: &SourceNode, value: &str) -> DoweResult<()> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(node_error(node, "binding name must not be empty"));
    };
    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
    {
        return Err(node_error(
            node,
            format!("binding `{value}` must be an ASCII identifier"),
        ));
    }
    Ok(())
}

fn validate_return(node: &SourceNode, context: ActionContext) -> DoweResult<()> {
    let source = node
        .args
        .iter()
        .map(SourceValue::to_source)
        .chain(
            node.props
                .iter()
                .map(|prop| format!("{}:{}", prop.name, prop.value.to_source())),
        )
        .collect::<Vec<_>>()
        .join(" ");
    validate_request_usage(node, context, &source)?;
    if source.contains("await") && !context_allows_await(context) {
        return Err(node_error(
            node,
            "`await` is only valid inside async handlers",
        ));
    }
    if node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .as_deref()
        == Some("response")
    {
        return Err(node_error(
            node,
            "HTTP returns use `return <props>`; remove `response`",
        ));
    }
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "HTTP returns do not accept positional values",
        ));
    }
    reject_unknown_props(
        node,
        &[
            "status",
            "text",
            "json",
            "proxy",
            "agent",
            "bytes",
            "contentType",
            "headers",
            "cookies",
            "request",
        ],
    )?;
    let body_count = ["text", "json", "proxy", "agent", "bytes"]
        .iter()
        .filter(|name| node.prop(name).is_some())
        .count();
    if body_count == 0 {
        return Err(node_error(
            node,
            "return must declare text, json, proxy, agent, or bytes",
        ));
    }
    if body_count > 1 {
        return Err(node_error(
            node,
            "return must declare exactly one of text, json, proxy, agent, or bytes",
        ));
    }
    if let Some(prop) = node.prop("text") {
        required_static_string_prop(prop)?;
    }
    if node.prop("agent").is_some() && node.prop("request").is_none() {
        return Err(node_error(
            node,
            "agent response must declare `request` binding",
        ));
    }
    Ok(())
}

fn validate_request_usage(
    node: &SourceNode,
    context: ActionContext,
    source: &str,
) -> DoweResult<()> {
    if source.contains("req.params")
        && !matches!(
            context,
            ActionContext::HttpHandler {
                request: Some("req"),
                ..
            }
        )
    {
        return Err(node_error(
            node,
            "`req.params` is only valid in HTTP handlers",
        ));
    }
    let uses_request_metadata = source.contains("req.query")
        || source.contains("req.rawQuery")
        || source.contains("req.header")
        || source.contains("req.cookie");
    if uses_request_metadata
        && !matches!(
            context,
            ActionContext::HttpHandler {
                request: Some("req"),
                ..
            }
        )
    {
        return Err(node_error(
            node,
            "`req.query`, `req.rawQuery`, `req.header`, and `req.cookie` are only valid in HTTP handlers",
        ));
    }
    if source.contains("req.json") {
        match context {
            ActionContext::HttpHandler {
                async_handler: true,
                request: Some("req"),
            } => {}
            ActionContext::HttpHandler { .. } => {
                return Err(node_error(
                    node,
                    "`req.json` requires an async request handler",
                ));
            }
            ActionContext::Init
            | ActionContext::Middleware
            | ActionContext::Function
            | ActionContext::WebSocket
            | ActionContext::Protocol { .. } => {
                return Err(node_error(
                    node,
                    "`req.json` is only valid in HTTP handlers",
                ));
            }
        }
    }
    Ok(())
}

fn context_allows_await(context: ActionContext) -> bool {
    matches!(
        context,
        ActionContext::HttpHandler {
            async_handler: true,
            ..
        }
    )
}

