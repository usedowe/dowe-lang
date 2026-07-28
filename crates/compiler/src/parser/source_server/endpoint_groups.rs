fn parse_endpoint_group_node(
    node: &SourceNode,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<(String, EndpointGroup)> {
    let name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, "`endpoints` must declare an export name"))?;
    validate_binding_name(node, &name)?;
    if node.children.is_empty() {
        return Err(node_error(
            node,
            "`endpoints` must declare at least one route",
        ));
    }
    let group = parse_endpoint_group_children(
        &node.children,
        EndpointScope::default(),
        false,
        imports,
        types,
        environment,
    )?;
    if group.endpoints.is_empty() && group.websockets.is_empty() {
        return Err(node_error(
            node,
            "`endpoints` must declare at least one HTTP method or WebSocket",
        ));
    }
    Ok((name, group))
}

#[derive(Clone, Default)]
struct EndpointScope {
    path: String,
    middlewares: Vec<ServerMiddleware>,
}

fn parse_endpoint_group_children(
    nodes: &[SourceNode],
    scope: EndpointScope,
    inside_group: bool,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<EndpointGroup> {
    let mut group = EndpointGroup::default();
    for node in nodes {
        match node.name.as_str() {
            "group" => {
                if inside_group {
                    return Err(node_error(
                        node,
                        "`endpoints` groups cannot contain another `group`; put middleware on the group or its HTTP method",
                    ));
                }
                let child_scope = endpoint_group_scope(node, &scope, imports)?;
                let child_group = parse_endpoint_group_children(
                    &node.children,
                    child_scope,
                    true,
                    imports,
                    types,
                    environment,
                )?;
                group.endpoints.extend(child_group.endpoints);
                group.websockets.extend(child_group.websockets);
            }
            "get" | "post" | "put" | "patch" | "delete" => {
                group.endpoints.push(parse_declared_endpoint_method(
                    node,
                    declared_http_method(node)?,
                    &scope,
                    imports,
                    types,
                    environment,
                )?)
            }
            "websocket" => group.websockets.push(parse_websocket(
                node,
                environment,
                imports,
                &scope.path,
                &scope.middlewares,
            )?),
            "route" | "method" => {
                return Err(node_error(
                    node,
                    "endpoint modules use `group` with lowercase HTTP declarations such as `get path:\"/status\" handler:status`",
                ));
            }
            _ => {
                return Err(node_error(
                    node,
                    "`endpoints` only accepts `group`, lowercase HTTP declarations, or `websocket` blocks",
                ));
            }
        }
    }
    Ok(group)
}

fn endpoint_group_scope(
    node: &SourceNode,
    parent: &EndpointScope,
    imports: &ServerImports,
) -> DoweResult<EndpointScope> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "`group` does not accept positional arguments",
        ));
    }
    reject_unknown_props(node, &["path", "middleware"])?;
    let path = node
        .prop("path")
        .map(|prop| endpoint_path_value(prop, false))
        .transpose()?
        .unwrap_or_default();
    Ok(EndpointScope {
        path: join_endpoint_paths(&parent.path, &path),
        middlewares: merge_middlewares(&parent.middlewares, route_middlewares(node, imports)?),
    })
}

fn declared_http_method(node: &SourceNode) -> DoweResult<HttpMethod> {
    match node.name.as_str() {
        "get" => Ok(HttpMethod::Get),
        "post" => Ok(HttpMethod::Post),
        "put" => Ok(HttpMethod::Put),
        "patch" => Ok(HttpMethod::Patch),
        "delete" => Ok(HttpMethod::Delete),
        _ => Err(node_error(node, "unsupported HTTP method declaration")),
    }
}

fn parse_declared_endpoint_method(
    node: &SourceNode,
    method: HttpMethod,
    scope: &EndpointScope,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<Endpoint> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "HTTP declarations use `path:\"/...\"` props",
        ));
    }
    reject_unknown_props(
        node,
        &["path", "handler", "middleware", "json", "status", "text"],
    )?;
    let path = node
        .prop("path")
        .ok_or_else(|| node_error(node, "HTTP declarations must declare `path`"))
        .and_then(|prop| endpoint_path_value(prop, true))?;
    let path = join_endpoint_paths(&scope.path, &path);
    if !path.starts_with('/') {
        return Err(node_error(
            node,
            "HTTP route path must resolve to a slash-prefixed path",
        ));
    }
    let middlewares = merge_middlewares(&scope.middlewares, route_middlewares(node, imports)?);
    parse_endpoint_method(
        node,
        method,
        &path,
        imports,
        &middlewares,
        types,
        environment,
    )
}

fn endpoint_path_value(prop: &SourceProp, allow_empty: bool) -> DoweResult<String> {
    let SourceValue::String(path) = &prop.value else {
        return Err(prop_error(prop, "`path` must be a quoted string"));
    };
    let path = path.clone();
    if path.is_empty() && allow_empty {
        return Ok(path);
    }
    if !path.starts_with('/') {
        return Err(prop_error(prop, "`path` must start with `/`"));
    }
    Ok(path)
}

fn join_endpoint_paths(parent: &str, child: &str) -> String {
    match (parent, child) {
        ("", "") => String::new(),
        ("", child) => child.to_string(),
        (parent, "") => parent.to_string(),
        (parent, child) => format!(
            "{}/{}",
            parent.trim_end_matches('/'),
            child.trim_start_matches('/')
        ),
    }
}

fn merge_middlewares(
    parent: &[ServerMiddleware],
    own: Vec<ServerMiddleware>,
) -> Vec<ServerMiddleware> {
    let mut middlewares = parent.to_vec();
    middlewares.extend(own);
    middlewares
}

fn endpoint_group_references(node: &SourceNode) -> DoweResult<Vec<String>> {
    let value = node
        .prop("endpoints")
        .map(|prop| &prop.value)
        .or_else(|| node.args.first())
        .ok_or_else(|| node_error(node, "`endpoints` must reference an imported route group"))?;
    let values = match value {
        SourceValue::Array(values) => {
            if values.is_empty() {
                return Err(node_error(
                    node,
                    "`endpoints` route module list must not be empty",
                ));
            }
            values
                .iter()
                .map(|value| {
                    let SourceValue::Bareword(value) = value else {
                        return Err(node_error(
                            node,
                            "`endpoints` list values must be imported symbols",
                        ));
                    };
                    (!value.is_empty()).then(|| value.clone()).ok_or_else(|| {
                        node_error(node, "`endpoints` list values must be imported symbols")
                    })
                })
                .collect::<DoweResult<Vec<_>>>()?
        }
        value => vec![value.as_required_string().ok_or_else(|| {
            node_error(node, "`endpoints` must reference an imported route group")
        })?],
    };
    let mut seen = HashSet::new();
    for reference in &values {
        if !seen.insert(reference.clone()) {
            return Err(node_error(
                node,
                format!("duplicate endpoints reference `{reference}`"),
            ));
        }
    }
    Ok(values)
}

