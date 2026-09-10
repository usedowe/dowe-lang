fn parse_server_config(
    node: &SourceNode,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    target: ServerTarget,
) -> DoweResult<ServerConfig> {
    let port = required_port(node)?;
    let mut databases = Vec::new();
    let mut databases_seen = false;
    let mut endpoints = Vec::new();
    let mut websockets = Vec::new();
    let mut transports = Vec::new();
    let mut rtp = None;
    let mut models = Vec::new();
    let mut init_action = ServerAction::empty();
    let mut cors = CorsConfig::default();
    let mut cors_seen = false;
    let mut tls = None;
    let mut database_service = false;
    let mut cache_service = false;
    let mut vector_service = false;
    let mut queue_service = false;

    for child in &node.children {
        match child.name.as_str() {
            "databases" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`databases` registry is only supported by `main.server`",
                    ));
                }
                if databases_seen {
                    return Err(node_error(child, "duplicate `databases` registry"));
                }
                databases_seen = true;
                reject_unknown_props(child, &["databases"])?;
                databases = parse_database_registry(child, imports)?;
            }
            "endpoints" => {
                for name in endpoint_group_references(child)? {
                    let group = imports.endpoint_groups.get(&name).ok_or_else(|| {
                        node_error(child, format!("missing endpoints import `{name}`"))
                    })?;
                    endpoints.extend(group.endpoints.clone());
                    websockets.extend(group.websockets.clone());
                }
            }
            "route" => endpoints.extend(parse_route(child, imports, types, environment)?),
            "endpoint" => {
                return Err(node_error(child, "`endpoint` has been renamed to `route`"));
            }
            "websocket" => websockets.push(parse_websocket(child, environment, imports, "", &[])?),
            "udp" => transports.push(parse_transport(
                child,
                ServerTransportProtocol::Udp,
                environment,
            )?),
            "tcp" => transports.push(parse_transport(
                child,
                ServerTransportProtocol::Tcp,
                environment,
            )?),
            "rtp" => {
                if rtp.is_some() {
                    return Err(node_error(child, "duplicate `rtp` block"));
                }
                rtp = Some(parse_rtp_config(child)?);
            }
            "model" => models.push(parse_server_model(child)?),
            "database" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`database service` is only supported by `main.server`",
                    ));
                }
                if database_service {
                    return Err(node_error(child, "duplicate `database service` block"));
                }
                parse_database_service(child)?;
                database_service = true;
            }
            "cache" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`cache service` is only supported by `main.server`",
                    ));
                }
                if cache_service {
                    return Err(node_error(child, "duplicate `cache service` block"));
                }
                parse_cache_service(child)?;
                cache_service = true;
            }
            "vector" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`vector service` is only supported by `main.server`",
                    ));
                }
                if vector_service {
                    return Err(node_error(child, "duplicate `vector service` block"));
                }
                parse_vector_service(child)?;
                vector_service = true;
            }
            "queue" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`queue service` is only supported by `main.server`",
                    ));
                }
                if queue_service {
                    return Err(node_error(child, "duplicate `queue service` block"));
                }
                parse_queue_service(child)?;
                queue_service = true;
            }
            "cors" => {
                if cors_seen {
                    return Err(node_error(child, "duplicate `cors` block"));
                }
                cors_seen = true;
                cors = match target {
                    ServerTarget::Server => parse_server_cors_config(child)?,
                    ServerTarget::Desktop => parse_desktop_cors_config(child)?,
                };
            }
            "tls" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`tls` is only supported by `main.server`",
                    ));
                }
                if tls.is_some() {
                    return Err(node_error(child, "duplicate `tls` block"));
                }
                tls = Some(parse_tls_config(child, environment, port)?);
            }
            "init" => {
                init_action = parse_action(child, ActionContext::Init, types, environment, imports)?
            }
            _ => return Err(node_error(child, "unsupported server block")),
        }
    }
    validate_unique_transport_names(node, &transports)?;
    validate_unique_model_names(node, &models)?;
    if database_service
        && (endpoints
            .iter()
            .any(|route| route.path == "/v1/databases/:name")
            || websockets
                .iter()
                .any(|route| route.path == "/v1/databases/:name"))
    {
        return Err(node_error(
            node,
            "`database service` reserves WebSocket path `/v1/databases/:name`",
        ));
    }
    if cache_service
        && (endpoints
            .iter()
            .any(|route| route.path == "/v1/caches/:name")
            || websockets
                .iter()
                .any(|route| route.path == "/v1/caches/:name"))
    {
        return Err(node_error(
            node,
            "`cache service` reserves WebSocket path `/v1/caches/:name`",
        ));
    }
    if vector_service
        && (endpoints
            .iter()
            .any(|route| route.path == "/v1/vectors/:name")
            || websockets
                .iter()
                .any(|route| route.path == "/v1/vectors/:name"))
    {
        return Err(node_error(
            node,
            "`vector service` reserves WebSocket path `/v1/vectors/:name`",
        ));
    }
    if queue_service
        && (endpoints
            .iter()
            .any(|route| route.path == "/v1/queues/:name")
            || websockets
                .iter()
                .any(|route| route.path == "/v1/queues/:name"))
    {
        return Err(node_error(
            node,
            "`queue service` reserves WebSocket path `/v1/queues/:name`",
        ));
    }

    Ok(ServerConfig {
        port,
        databases,
        tls,
        endpoints,
        websockets,
        transports,
        rtp,
        models,
        init_action,
        cors,
        database_service,
        cache_service,
        vector_service,
        queue_service,
    })
}

fn parse_native_ipc_functions(
    node: &SourceNode,
    imports: &ServerImports,
) -> DoweResult<(Vec<NativeIpcFunction>, Vec<StoreConnection>)> {
    reject_unknown_props(node, &["functions", "databases"])?;
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(node, "`ipc` accepts props only"));
    }
    let names = binding_array_prop(node, "functions")?;
    if names.is_empty() {
        return Err(node_error(node, "`ipc functions` must not be empty"));
    }
    let mut seen = HashSet::new();
    let functions = names
        .into_iter()
        .map(|name| {
            if !seen.insert(name.clone()) {
                return Err(node_error(node, format!("duplicate IPC function `{name}`")));
            }
            let callable = imports.callables.get(&name).ok_or_else(|| {
                node_error(node, format!("missing IPC function import `{name}`"))
            })?;
            Ok(NativeIpcFunction {
                name,
                action: callable.action.clone(),
            })
        })
        .collect::<DoweResult<Vec<_>>>()?;
    let databases = if node.prop("databases").is_some() {
        parse_database_registry(node, imports)?
    } else {
        Vec::new()
    };
    Ok((functions, databases))
}

