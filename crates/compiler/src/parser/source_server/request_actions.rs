#[derive(Clone, Copy)]
enum ActionContext<'a> {
    Init,
    HttpHandler {
        async_handler: bool,
        request: Option<&'a str>,
    },
    Middleware,
    Function,
    WebSocket,
    Protocol {
        binding: &'a str,
    },
}

fn parse_request_json_const(
    node: &SourceNode,
    context: ActionContext,
    types: &TypeRegistry,
) -> DoweResult<ServerStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "request JSON uses `const <binding[:Type]> value:req.json`",
        ));
    }
    reject_unknown_props(node, &["value"])?;
    let binding = node.args[0].as_required_string().ok_or_else(|| {
        node_error(
            node,
            "request JSON uses `const <binding[:Type]> value:req.json`",
        )
    })?;
    let value = node
        .prop("value")
        .and_then(|prop| prop.value.as_string_like())
        .ok_or_else(|| {
            node_error(
                node,
                "request JSON uses `const <binding[:Type]> value:req.json`",
            )
        })?;
    if value != "req.json" {
        return Err(node_error(
            node,
            "request JSON uses `const <binding[:Type]> value:req.json`",
        ));
    }
    validate_request_usage(node, context, "req.json")?;
    let (binding, schema) = parse_binding_type(node, &binding, types)?;
    Ok(ServerStatement::RequestJson { binding, schema })
}

fn legacy_request_json_let(node: &SourceNode) -> bool {
    node.args.len() == 4
        && node.args[1].as_string_like().as_deref() == Some("=")
        && node.args[2].as_string_like().as_deref() == Some("await")
        && node.args[3].as_string_like().as_deref() == Some("req.json()")
}

fn legacy_spawn_let(node: &SourceNode) -> bool {
    assignment(node).is_some_and(|(_, expression)| expression == "dowe.spawn")
}

fn legacy_http_let(node: &SourceNode) -> bool {
    assignment(node).is_some_and(|(_, expression)| {
        matches!(
            expression.as_str(),
            "http.request" | "http.get" | "http.post"
        )
    })
}

fn legacy_crypto_let(node: &SourceNode) -> bool {
    assignment(node).is_some_and(|(_, expression)| {
        matches!(expression.as_str(), "crypto.aesCtr" | "crypto.cencAesCtr")
    })
}

fn parse_request_metadata_let(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<Option<ServerStatement>> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(None);
    };
    let statement = match expression.as_str() {
        "req.query" => {
            reject_unknown_props(node, &[])?;
            ServerStatement::RequestQuery { binding }
        }
        "req.rawQuery" => {
            reject_unknown_props(node, &[])?;
            ServerStatement::RequestRawQuery { binding }
        }
        "req.header" => {
            reject_unknown_props(node, &["name"])?;
            ServerStatement::RequestHeader {
                binding,
                name: required_header_name_prop(node, "name")?,
            }
        }
        "req.cookie" => {
            reject_unknown_props(node, &["name"])?;
            ServerStatement::RequestCookie {
                binding,
                name: required_cookie_name_prop(node, "name")?,
            }
        }
        _ => return Ok(None),
    };
    validate_request_usage(node, context, expression.as_str())?;
    if node.args.len() != 3 {
        return Err(node_error(
            node,
            "`req.query`, `req.rawQuery`, `req.header`, and `req.cookie` only accept named props",
        ));
    }
    match context {
        ActionContext::HttpHandler {
            request: Some("req"),
            ..
        } => {}
        _ => {
            return Err(node_error(
                node,
                "`req.query`, `req.rawQuery`, `req.header`, and `req.cookie` are only valid in HTTP handlers",
            ));
        }
    }
    Ok(Some(statement))
}

fn parse_request_declaration(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<ServerStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "request uses `request <binding> source:\"query|rawQuery|header|cookie\" [name:<name>]`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "request requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    let source = required_source_selector(node, "request")?;
    let statement = match source.as_str() {
        "query" => {
            reject_unknown_props(node, &["source"])?;
            ServerStatement::RequestQuery { binding }
        }
        "rawQuery" => {
            reject_unknown_props(node, &["source"])?;
            ServerStatement::RequestRawQuery { binding }
        }
        "header" => {
            reject_unknown_props(node, &["source", "name"])?;
            ServerStatement::RequestHeader {
                binding,
                name: required_header_name_prop(node, "name")?,
            }
        }
        "cookie" => {
            reject_unknown_props(node, &["source", "name"])?;
            ServerStatement::RequestCookie {
                binding,
                name: required_cookie_name_prop(node, "name")?,
            }
        }
        _ => {
            return Err(node_error(
                node,
                "request `source` must be `query`, `rawQuery`, `header`, or `cookie`",
            ));
        }
    };
    if !matches!(
        context,
        ActionContext::HttpHandler {
            request: Some("req"),
            ..
        }
    ) {
        return Err(node_error(
            node,
            "`request` declarations are only valid in HTTP handlers",
        ));
    }
    Ok(statement)
}

fn legacy_request_metadata_error(node: &SourceNode, statement: &ServerStatement) -> DoweError {
    let replacement = match statement {
        ServerStatement::RequestQuery { binding } => {
            format!("request {binding} source:\"query\"")
        }
        ServerStatement::RequestRawQuery { binding } => {
            format!("request {binding} source:\"rawQuery\"")
        }
        ServerStatement::RequestHeader { binding, name } => {
            format!("request {binding} source:\"header\" name:\"{name}\"")
        }
        ServerStatement::RequestCookie { binding, name } => {
            format!("request {binding} source:\"cookie\" name:\"{name}\"")
        }
        _ => unreachable!(),
    };
    node_error(
        node,
        format!("request metadata uses `{replacement}`; `let` is not supported"),
    )
}

fn parse_websocket_json_let(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<Option<WebSocketJsonStatement>> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(None);
    };
    if expression != "ws.json" {
        return Ok(None);
    }
    if node.args.len() != 3 {
        return Err(node_error(
            node,
            "`ws.json` does not accept positional values",
        ));
    }
    if !matches!(context, ActionContext::WebSocket) {
        return Err(node_error(
            node,
            "`ws.json` is only valid in WebSocket handlers",
        ));
    }
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &[])?;
    Ok(Some(WebSocketJsonStatement { binding }))
}

fn parse_websocket_json_declaration(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<WebSocketJsonStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "WebSocket JSON uses `ws <binding> source:\"json\"`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "WebSocket JSON requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &["source"])?;
    if required_source_selector(node, "ws")? != "json" {
        return Err(node_error(node, "ws only supports `source:\"json\"`"));
    }
    if !matches!(context, ActionContext::WebSocket) {
        return Err(node_error(
            node,
            "`ws ... source:\"json\"` is only valid in WebSocket handlers",
        ));
    }
    Ok(WebSocketJsonStatement { binding })
}

fn parse_agent_chat_let(node: &SourceNode) -> DoweResult<Option<AgentChatTransform>> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(None);
    };
    if expression != "agent.chat" {
        return Ok(None);
    }
    if node.args.len() != 4 {
        return Err(node_error(
            node,
            "`agent.chat` requires a source request binding",
        ));
    }
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &[])?;
    let source = node.args[3]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`agent.chat` source must be a reference"))?;
    Ok(Some(AgentChatTransform { binding, source }))
}

fn parse_agent_chat_declaration(node: &SourceNode) -> DoweResult<AgentChatTransform> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "Agent chat uses `agent <binding> source:\"chat\" request:<request>`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "Agent chat requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &["source", "request"])?;
    if required_source_selector(node, "agent")? != "chat" {
        return Err(node_error(node, "agent only supports `source:\"chat\"`"));
    }
    Ok(AgentChatTransform {
        binding,
        source: required_reference_prop(node, "request")?,
    })
}
