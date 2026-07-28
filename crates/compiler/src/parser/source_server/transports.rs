fn parse_transport(
    node: &SourceNode,
    protocol: ServerTransportProtocol,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerTransport> {
    let name = required_name_prop(node)?;
    let bind = optional_bind_prop(node)?;
    let port = required_transport_port(node)?;
    let expected = match protocol {
        ServerTransportProtocol::Udp => "packet",
        ServerTransportProtocol::Tcp => "connection",
    };
    let mut action = ServerAction::empty();
    let mut binding = expected.to_string();
    let mut seen = false;

    for child in &node.children {
        if child.name != expected {
            return Err(node_error(
                child,
                format!(
                    "{} transport only accepts `{expected}` block",
                    protocol.as_str()
                ),
            ));
        }
        if seen {
            return Err(node_error(child, format!("duplicate `{expected}` block")));
        }
        seen = true;
        binding = child
            .args
            .first()
            .and_then(SourceValue::as_string_like)
            .unwrap_or_else(|| expected.to_string());
        validate_binding_name(child, &binding)?;
        let imports = ServerImports::default();
        action = parse_action(
            child,
            ActionContext::Protocol { binding: &binding },
            &TypeRegistry::empty(),
            environment,
            &imports,
        )?;
    }

    Ok(ServerTransport {
        name,
        protocol,
        bind,
        port,
        action,
        binding,
    })
}

fn parse_rtp_config(node: &SourceNode) -> DoweResult<RtpConfig> {
    reject_unknown_props(node, &["bind", "min", "max"])?;
    let bind = optional_bind_prop(node)?;
    let min = required_port_prop(node, "min")?;
    let max = required_port_prop(node, "max")?;
    if min > max {
        return Err(node_error(
            node,
            "`rtp` min must be less than or equal to max",
        ));
    }
    Ok(RtpConfig { bind, min, max })
}

fn parse_server_model(node: &SourceNode) -> DoweResult<ServerModel> {
    reject_unknown_props(
        node,
        &["name", "kind", "engine", "format", "source", "sampleRates"],
    )?;
    let name = required_name_prop(node)?;
    let kind = required_model_kind_prop(node)?;
    let engine = required_model_engine_prop(node)?;
    let format = required_model_format_prop(node)?;
    let source = optional_model_source_prop(node, format)?;
    let sample_rates = optional_sample_rates_prop(node)?;
    match (engine, format) {
        (ServerModelEngine::Candle, ServerModelFormat::Onnx)
        | (ServerModelEngine::Energy, ServerModelFormat::Builtin) => {}
        (ServerModelEngine::Candle, _) => {
            return Err(node_error(node, "candle models must use format:\"onnx\""));
        }
        (ServerModelEngine::Energy, _) => {
            return Err(node_error(
                node,
                "energy models must use format:\"builtin\"",
            ));
        }
    }
    Ok(ServerModel {
        name,
        kind,
        engine,
        format,
        source,
        sample_rates,
    })
}

fn validate_unique_transport_names(
    node: &SourceNode,
    transports: &[ServerTransport],
) -> DoweResult<()> {
    let mut names = Vec::<&str>::new();
    for transport in transports {
        if names.iter().any(|name| *name == transport.name) {
            return Err(node_error(
                node,
                format!("duplicate transport `{}`", transport.name),
            ));
        }
        names.push(&transport.name);
    }
    Ok(())
}

fn validate_unique_model_names(node: &SourceNode, models: &[ServerModel]) -> DoweResult<()> {
    let mut names = Vec::<&str>::new();
    for model in models {
        if names.iter().any(|name| *name == model.name) {
            return Err(node_error(
                node,
                format!("duplicate model `{}`", model.name),
            ));
        }
        names.push(&model.name);
    }
    Ok(())
}

fn parse_websocket(
    node: &SourceNode,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
    parent_path: &str,
    inherited_middlewares: &[ServerMiddleware],
) -> DoweResult<WebSocketRoute> {
    let path = if let Some(prop) = node.prop("path") {
        if !node.args.is_empty() {
            return Err(node_error(
                node,
                "`websocket` uses either `path:\"/...\"` or one path argument",
            ));
        }
        endpoint_path_value(prop, false)?
    } else {
        required_path_arg(node, "websocket")?
    };
    let path = join_endpoint_paths(parent_path, &path);
    let middlewares = merge_middlewares(inherited_middlewares, route_middlewares(node, imports)?);
    let mut handlers = WebSocketHandlers::default();

    for child in &node.children {
        let imports = ServerImports::default();
        let action = parse_action(
            child,
            ActionContext::WebSocket,
            &TypeRegistry::empty(),
            environment,
            &imports,
        )?;
        match child.name.as_str() {
            "open" => handlers.open = action,
            "message" => handlers.message = action,
            "close" => handlers.close = action,
            "drain" => handlers.drain = action,
            _ => return Err(node_error(child, "unsupported WebSocket handler")),
        }
    }

    Ok(WebSocketRoute {
        path,
        handlers,
        middlewares,
    })
}

