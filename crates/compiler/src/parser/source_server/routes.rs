fn parse_route(
    node: &SourceNode,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<Vec<Endpoint>> {
    let path = required_path_arg(node, "route")?;
    let middlewares = route_middlewares(node, imports)?;
    let mut endpoints = Vec::new();

    for child in &node.children {
        match child.name.as_str() {
            "response" => endpoints.push(Endpoint {
                method: HttpMethod::Get,
                path: path.clone(),
                behavior: EndpointBehavior::StaticText(required_text_prop(child)?),
                action: ServerAction::empty(),
                middlewares: middlewares.clone(),
            }),
            "handler" => {
                reject_explicit_handler_async(child)?;
                let action = parse_action(
                    child,
                    handler_action_context(child),
                    types,
                    environment,
                    imports,
                )?;
                endpoints.push(Endpoint {
                    method: HttpMethod::Get,
                    path: path.clone(),
                    behavior: handler_behavior(child, &path, &action)?,
                    action,
                    middlewares: middlewares.clone(),
                });
            }
            "method" => endpoints.push(parse_method(
                child,
                &path,
                imports,
                &middlewares,
                types,
                environment,
            )?),
            _ => return Err(node_error(child, "unsupported route block")),
        }
    }

    if endpoints.is_empty() {
        return Err(node_error(
            node,
            "route must declare a response, handler, or method",
        ));
    }

    Ok(endpoints)
}

fn route_middlewares(
    node: &SourceNode,
    imports: &ServerImports,
) -> DoweResult<Vec<ServerMiddleware>> {
    let Some(prop) = node.prop("middleware") else {
        return Ok(Vec::new());
    };
    let names = match &prop.value {
        SourceValue::Bareword(value) => vec![value.clone()],
        SourceValue::Array(values) => {
            let mut names = Vec::new();
            for value in values {
                let SourceValue::Bareword(name) = value else {
                    return Err(prop_error(prop, "`middleware` values must be references"));
                };
                names.push(name.clone());
            }
            names
        }
        _ => {
            return Err(prop_error(
                prop,
                "`middleware` must be a reference or array",
            ));
        }
    };
    let mut middlewares = Vec::new();
    for name in names {
        let middleware = imports
            .middlewares
            .get(&name)
            .ok_or_else(|| prop_error(prop, format!("unknown middleware import `{name}`")))?;
        middlewares.push(middleware.clone());
    }
    Ok(middlewares)
}

fn parse_method(
    node: &SourceNode,
    path: &str,
    imports: &ServerImports,
    middlewares: &[ServerMiddleware],
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<Endpoint> {
    let method_name = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "method must declare an HTTP method"))?;
    let method = HttpMethod::from_str(&method_name)
        .map_err(|_| node_error(node, format!("unsupported HTTP method `{method_name}`")))?;
    parse_endpoint_method(node, method, path, imports, middlewares, types, environment)
}

fn parse_endpoint_method(
    node: &SourceNode,
    method: HttpMethod,
    path: &str,
    imports: &ServerImports,
    middlewares: &[ServerMiddleware],
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
) -> DoweResult<Endpoint> {
    if let Some(handler_name) = optional_prop_string(node, "handler")? {
        let handler = imports
            .handlers
            .get(&handler_name)
            .ok_or_else(|| node_error(node, format!("unknown handler import `{handler_name}`")))?;
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior: handler.behavior.clone(),
            action: handler.action.clone(),
            middlewares: middlewares.to_vec(),
        });
    }
    let action = parse_action(
        node,
        handler_action_context(node),
        types,
        environment,
        imports,
    )?;
    if has_reference_log(&action)
        && let Some(behavior) = database_action_endpoint_behavior(
            &action,
            return_json_value(node),
            return_status(node)?,
        )?
    {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) = database_endpoint_behavior(&action, return_json_ref(node))? {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) =
        database_action_endpoint_behavior(&action, return_json_value(node), return_status(node)?)?
    {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) =
        kv_action_endpoint_behavior(&action, return_json_value(node), return_status(node)?)?
    {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) =
        vector_action_endpoint_behavior(&action, return_json_value(node), return_status(node)?)?
    {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    if let Some(behavior) = http_endpoint_behavior(node)? {
        return Ok(Endpoint {
            method,
            path: path.to_string(),
            behavior,
            action,
            middlewares: middlewares.to_vec(),
        });
    }
    let behavior = match method {
        HttpMethod::Get => EndpointBehavior::StaticText(
            return_text(node).unwrap_or_else(|| "List posts".to_string()),
        ),
        HttpMethod::Post => {
            if returns_created_json(node) {
                EndpointBehavior::CreatePostJson
            } else {
                return Err(node_error(
                    node,
                    "POST method must return supported JSON response",
                ));
            }
        }
        _ => return Err(node_error(node, "method behavior is not supported yet")),
    };

    Ok(Endpoint {
        method,
        path: path.to_string(),
        behavior,
        action,
        middlewares: middlewares.to_vec(),
    })
}

