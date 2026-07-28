fn parse_http_declaration(
    node: &SourceNode,
    context: ActionContext,
    environment: &EnvironmentConfig,
) -> DoweResult<OutboundHttpRequest> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "http uses `http <binding> method:\"get\" base:<url> path:\"/...\"`",
        ));
    }
    let binding = node.args[0].as_required_string().ok_or_else(|| {
        node_error(
            node,
            "http uses `http <binding> method:\"get\" base:<url> path:\"/...\"`",
        )
    })?;
    let method = required_http_method_prop(node)?;
    match context {
        ActionContext::HttpHandler {
            async_handler: true,
            ..
        }
        | ActionContext::WebSocket
        | ActionContext::Function => {}
        ActionContext::HttpHandler { .. } => {
            return Err(node_error(node, "http requires an async request handler"));
        }
        ActionContext::Init => {
            return Err(node_error(node, "http is not valid in server init"));
        }
        ActionContext::Middleware => {
            return Err(node_error(
                node,
                "http is only valid in async handlers, server functions, and WebSocket handlers",
            ));
        }
        ActionContext::Protocol { .. } => {
            return Err(node_error(node, "http is not valid in protocol handlers"));
        }
    }
    validate_binding_name(node, &binding)?;
    reject_unknown_props(
        node,
        &[
            "method",
            "base",
            "path",
            "bearer",
            "headers",
            "json",
            "mode",
            "redirect",
            "maxRedirects",
            "timeoutMs",
        ],
    )?;
    let base = required_http_base_prop(node, environment)?;
    let path = required_http_path_prop(node)?;
    let bearer = if node.prop("bearer").is_some() {
        Some(required_secret_prop(node, "bearer", environment)?)
    } else {
        None
    };
    let headers = optional_http_headers_prop(node, environment)?;
    let json = node
        .prop("json")
        .map(|prop| store_literal(&prop.value))
        .transpose()?;
    let mode = optional_http_mode_prop(node)?;
    let redirect = optional_http_redirect_prop(node)?;
    let max_redirects = optional_positive_u32_prop(node, "maxRedirects")?;
    if max_redirects.is_some() && redirect != HttpRedirectPolicy::Follow {
        return Err(node_error(
            node,
            "`maxRedirects` is only valid with redirect:\"follow\"",
        ));
    }
    let timeout_ms = optional_positive_u64_prop(node, "timeoutMs")?;
    Ok(OutboundHttpRequest {
        binding,
        method,
        base,
        path,
        bearer,
        headers,
        json,
        mode,
        redirect,
        max_redirects,
        timeout_ms,
    })
}

fn parse_spawn_declaration(node: &SourceNode) -> DoweResult<ServerSpawnStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "spawn uses `spawn <binding> command:<value> [args:<array>]`",
        ));
    }
    let binding = node.args[0].as_required_string().ok_or_else(|| {
        node_error(
            node,
            "spawn uses `spawn <binding> command:<value> [args:<array>]`",
        )
    })?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(
        node,
        &[
            "command",
            "args",
            "cwd",
            "timeoutMs",
            "maxOutputBytes",
            "background",
        ],
    )?;
    let command = required_store_literal_prop(node, "command")?;
    let args = node
        .prop("args")
        .map(|prop| store_literal(&prop.value))
        .transpose()?
        .unwrap_or_else(|| StoreLiteral::Array(Vec::new()));
    let cwd = node
        .prop("cwd")
        .map(|prop| store_literal(&prop.value))
        .transpose()?;
    let timeout_ms = optional_positive_u64_prop(node, "timeoutMs")?;
    let max_output_bytes = optional_positive_usize_prop(node, "maxOutputBytes")?;
    let background = optional_bool_prop(node, "background")?.unwrap_or(false);
    Ok(ServerSpawnStatement {
        binding,
        command,
        args,
        cwd,
        timeout_ms,
        max_output_bytes,
        background,
    })
}

fn parse_crypto_declaration(node: &SourceNode) -> DoweResult<ServerStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "crypto uses `crypto <binding> encryption:\"cencAesCtr\" data:<reference> key:<value> iv:<value>`",
        ));
    }
    let binding = node.args[0].as_required_string().ok_or_else(|| {
        node_error(
            node,
            "crypto uses `crypto <binding> encryption:\"cencAesCtr\" data:<reference> key:<value> iv:<value>`",
        )
    })?;
    validate_binding_name(node, &binding)?;
    let encryption_prop = node
        .prop("encryption")
        .ok_or_else(|| node_error(node, "missing `encryption`"))?;
    let encryption = required_static_string_prop(encryption_prop)?;
    match encryption.as_str() {
        "aesCtr" => {
            reject_unknown_props(node, &["encryption", "data", "key", "iv"])?;
            Ok(ServerStatement::CryptoAesCtr(ServerCryptoAesCtrStatement {
                binding,
                data: required_reference_prop(node, "data")?,
                key: required_store_literal_prop(node, "key")?,
                iv: required_store_literal_prop(node, "iv")?,
            }))
        }
        "cencAesCtr" => {
            reject_unknown_props(node, &["encryption", "data", "key", "iv", "subsamples"])?;
            let subsamples = node
                .prop("subsamples")
                .map(|prop| store_literal(&prop.value))
                .transpose()?;
            Ok(ServerStatement::CryptoCencAesCtr(
                ServerCryptoCencAesCtrStatement {
                    binding,
                    data: required_reference_prop(node, "data")?,
                    key: required_store_literal_prop(node, "key")?,
                    iv: required_store_literal_prop(node, "iv")?,
                    subsamples,
                },
            ))
        }
        _ => Err(prop_error(
            encryption_prop,
            "`encryption` must be aesCtr or cencAesCtr",
        )),
    }
}

fn parse_websocket_send_json(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<WebSocketSendJsonStatement> {
    if !matches!(context, ActionContext::WebSocket) {
        return Err(node_error(
            node,
            "`send ws` is only valid in WebSocket handlers",
        ));
    }
    if node.args.len() != 1 || node.args[0].as_string_like().as_deref() != Some("ws") {
        return Err(node_error(node, "`send` must target `ws`"));
    }
    reject_unknown_props(node, &["json"])?;
    let value = required_store_literal_prop(node, "json")?;
    Ok(WebSocketSendJsonStatement { value })
}

fn parse_websocket_sse_bridge(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<WebSocketSseBridgeStatement> {
    if !matches!(context, ActionContext::WebSocket) {
        return Err(node_error(
            node,
            "`bridge sse` is only valid in WebSocket handlers",
        ));
    }
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "`bridge` does not accept positional values",
        ));
    }
    reject_unknown_props(node, &["sse", "to", "requestId", "requestType", "model"])?;
    let upstream = required_reference_prop(node, "sse")?;
    let target = node
        .prop("to")
        .and_then(|prop| prop.value.as_string_like())
        .ok_or_else(|| node_error(node, "`bridge` must declare `to:ws`"))?;
    if target != "ws" {
        return Err(node_error(node, "`bridge` only supports `to:ws`"));
    }
    Ok(WebSocketSseBridgeStatement {
        upstream,
        request_id: required_reference_prop(node, "requestId")?,
        request_type: required_reference_prop(node, "requestType")?,
        model: required_reference_prop(node, "model")?,
    })
}

