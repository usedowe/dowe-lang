fn parse_handler_node(
    node: &SourceNode,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<(String, ServerHandler)> {
    reject_explicit_handler_async(node)?;
    let name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, "handler must declare a name"))?;
    let action = parse_action(
        node,
        handler_action_context(node),
        types,
        environment,
        imports,
    )?;
    let behavior = exported_handler_behavior(node, &action)?;
    Ok((name, ServerHandler { action, behavior }))
}

fn parse_middleware_node(
    _file: &SourceFile,
    node: &SourceNode,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<(String, ServerMiddleware)> {
    let name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, "middleware must declare a name"))?;
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "middleware declarations use `middleware <name> [params:{ ... }]`; `req` and `next` are implicit",
        ));
    }
    reject_unknown_props(node, &["params"])?;
    let params = middleware_params(node)?;
    let action = parse_middleware_action(node, environment, imports)?;
    Ok((
        name.clone(),
        ServerMiddleware {
            name,
            params,
            action,
        },
    ))
}

fn middleware_params(node: &SourceNode) -> DoweResult<StoreLiteral> {
    let Some(prop) = node.prop("params") else {
        return Ok(StoreLiteral::Object(Vec::new()));
    };
    let params = store_literal(&prop.value)?;
    if matches!(params, StoreLiteral::Object(_)) {
        Ok(params)
    } else {
        Err(prop_error(prop, "middleware params must be an object"))
    }
}

fn parse_server_function_node(
    node: &SourceNode,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<(String, ServerCallable)> {
    let name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, "`fn` must declare a name"))?;
    if node.args.len() != 1 {
        return Err(node_error(node, "`fn` must declare one name"));
    }
    validate_binding_name(node, &name)?;
    let action = parse_server_function_action(node, types, environment, imports)?;
    Ok((name.clone(), ServerCallable { name, action }))
}

