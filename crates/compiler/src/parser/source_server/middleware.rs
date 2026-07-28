fn parse_middleware_action(
    node: &SourceNode,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<ServerMiddlewareAction> {
    let statements = parse_middleware_statements(&node.children, environment, imports)?;
    if !middleware_returns(&statements) {
        return Err(node_error(
            node,
            "middleware must call `next` or return a response",
        ));
    }
    Ok(ServerMiddlewareAction { statements })
}

fn parse_middleware_statements(
    nodes: &[SourceNode],
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<Vec<ServerMiddlewareStatement>> {
    let mut statements = Vec::new();
    for node in nodes {
        match node.name.as_str() {
            "let" => {
                reject_middleware_let(node, imports)?;
                unreachable!();
            }
            "request" => statements.push(parse_middleware_request_declaration(node)?),
            "bearer" => statements.push(parse_bearer_declaration(node)?),
            "session" => statements.push(parse_session_verify_declaration(node, imports)?),
            "jwt" => statements.push(ServerMiddlewareStatement::Jwt(parse_jwt_statement(
                node,
                environment,
            )?)),
            "const" => {
                return Err(node_error(
                    node,
                    "JWT results use `jwt <binding> ... token:<value>` or `jwt <binding> ... claims:<value>` without `const`",
                ));
            }
            "if" => statements.push(parse_middleware_if(node, environment, imports)?),
            "return" => statements.push(parse_middleware_return(node)?),
            "log" | "info" | "warn" | "error" => {
                statements.push(ServerMiddlewareStatement::Log(parse_log(node)?));
            }
            "next" => statements.push(parse_middleware_next(node)?),
            "continue" => return Err(node_error(node, "middleware continuation uses `next`")),
            _ => {
                let statement = parse_server_function_call(
                    node,
                    ActionContext::Middleware,
                    &imports.callables,
                    &HashMap::new(),
                )?
                .ok_or_else(|| node_error(node, "unsupported middleware action"))?;
                statements.push(ServerMiddlewareStatement::Call(statement));
            }
        }
    }
    Ok(statements)
}

fn reject_middleware_let(node: &SourceNode, imports: &ServerImports) -> DoweResult<()> {
    let (binding, expression) =
        assignment(node).ok_or_else(|| node_error(node, "middleware let must assign a value"))?;
    match expression.as_str() {
        "req.header" => {
            let name = required_header_name_prop(node, "name")?;
            Err(node_error(
                node,
                format!(
                    "request headers use `request {binding} source:\"header\" name:\"{name}\"`"
                ),
            ))
        }
        "bearer" => Err(node_error(
            node,
            "bearer extraction uses `bearer <binding> value:req.header.Authorization`",
        )),
        "jwt.verify" | "jwt.decrypt" | "jwt.sign" | "jwt.encrypt" => Err(node_error(
            node,
            "JWT expressions use `jwt <binding> ... token:<value>` or `jwt <binding> ... claims:<value>`",
        )),
        "session.verify" => Err(node_error(
            node,
            "session verification uses `session <binding> cache:<cache> database:<database> token:<token> [maxAge:<seconds>]`",
        )),
        _ => {
            reject_legacy_server_function_call(node, &imports.callables)?;
            Err(node_error(
                node,
                "`let` assignments are not supported; use `<capability> <binding> <props>`",
            ))
        }
    }
}

fn parse_middleware_request_declaration(
    node: &SourceNode,
) -> DoweResult<ServerMiddlewareStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "middleware request headers use `request <binding> source:\"header\" name:<header>`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "request requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_middleware_props(node, &["source", "name"])?;
    let source = required_source_selector(node, "request")?;
    if source != "header" {
        return Err(node_error(
            node,
            "middleware request only supports `source:\"header\"`",
        ));
    }
    Ok(ServerMiddlewareStatement::Header {
        binding,
        name: required_header_name_prop(node, "name")?,
    })
}

fn parse_bearer_declaration(node: &SourceNode) -> DoweResult<ServerMiddlewareStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "bearer requires one result binding: `bearer <binding> value:req.header.Authorization`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "bearer requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &["value"])?;
    let source = node
        .prop("value")
        .and_then(|prop| prop.value.as_string_like())
        .ok_or_else(|| node_error(node, "bearer requires `value:req.header.Authorization`"))?;
    let Some(header) = source.strip_prefix("req.header.") else {
        return Err(node_error(
            node,
            "bearer `value` must read a request header such as `req.header.Authorization`",
        ));
    };
    if normalize_http_header_name(header).is_none() {
        return Err(node_error(
            node,
            "bearer `value` uses an invalid header name",
        ));
    }
    Ok(ServerMiddlewareStatement::Bearer { binding, source })
}

fn parse_middleware_if(
    node: &SourceNode,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<ServerMiddlewareStatement> {
    let condition = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "middleware if must declare a condition"))?;
    let Some(binding) = condition.strip_suffix(".valid") else {
        return Err(node_error(
            node,
            "middleware if only supports validation checks",
        ));
    };
    let statements = parse_middleware_statements(&node.children, environment, imports)?;
    Ok(ServerMiddlewareStatement::IfValid {
        binding: binding.to_string(),
        statements,
    })
}

fn parse_session_verify_declaration(
    node: &SourceNode,
    imports: &ServerImports,
) -> DoweResult<ServerMiddlewareStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "session verification uses `session <binding> cache:<cache> database:<database> token:<token> [maxAge:<seconds>]`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "session verification requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_middleware_props(node, &["cache", "database", "token", "maxAge"])?;
    let cache_name = required_middleware_reference(node, "cache")?;
    let database_name = required_middleware_reference(node, "database")?;
    let token = required_middleware_reference(node, "token")?;
    let max_age_seconds = node
        .prop("maxAge")
        .map(|prop| match &prop.value {
            SourceValue::Number(value) => value.parse::<u64>().map_err(|_| {
                node_error(node, "session verify `maxAge` must be a positive integer")
            }),
            _ => Err(node_error(
                node,
                "session verify `maxAge` must be a positive integer",
            )),
        })
        .transpose()?
        .unwrap_or(2_592_000);
    if max_age_seconds == 0 {
        return Err(node_error(
            node,
            "session verify `maxAge` must be a positive integer",
        ));
    }
    let cache = imported_cache_connection(node, imports, &cache_name)?;
    let database = imported_database_connection(node, imports, &database_name)?;
    Ok(ServerMiddlewareStatement::SessionVerify {
        binding,
        cache,
        database,
        token,
        max_age_seconds,
    })
}

fn required_middleware_reference(node: &SourceNode, name: &str) -> DoweResult<String> {
    let value = node
        .prop(name)
        .and_then(|prop| prop.value.as_string_like())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| node_error(node, format!("session verify must declare `{name}`")))?;
    Ok(value)
}

fn reject_unknown_middleware_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|allowed| *allowed == prop.name) {
            return Err(node_error(
                node,
                format!("session verify does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

fn imported_cache_connection(
    node: &SourceNode,
    imports: &ServerImports,
    name: &str,
) -> DoweResult<CacheConnection> {
    match imports
        .config_bindings
        .get(name)
        .map(|binding| &binding.statement)
    {
        Some(ServerStatement::Kv(crate::model::ServerKvStatement::Handle { connection })) => {
            Ok(connection.clone())
        }
        Some(_) => Err(node_error(
            node,
            format!("`{name}` must reference a Cache connection"),
        )),
        None => Err(node_error(
            node,
            format!("Cache connection `{name}` is not imported"),
        )),
    }
}

fn imported_database_connection(
    node: &SourceNode,
    imports: &ServerImports,
    name: &str,
) -> DoweResult<StoreConnection> {
    match imports
        .config_bindings
        .get(name)
        .map(|binding| &binding.statement)
    {
        Some(ServerStatement::Store(crate::model::ServerStoreStatement::Handle { connection })) => {
            Ok(connection.clone())
        }
        Some(_) => Err(node_error(
            node,
            format!("`{name}` must reference a Database connection"),
        )),
        None => Err(node_error(
            node,
            format!("Database connection `{name}` is not imported"),
        )),
    }
}

fn parse_middleware_return(node: &SourceNode) -> DoweResult<ServerMiddlewareStatement> {
    if node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .as_deref()
        == Some("continue")
    {
        return Err(node_error(node, "middleware continuation uses `next`"));
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
            "middleware HTTP returns use `return <props>`; remove `response`",
        ));
    }
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "middleware HTTP returns do not accept positional values",
        ));
    }
    reject_unknown_props(node, &["status", "text", "json"])?;
    let status = return_status_from_node(node)?;
    match (node.prop("text"), node.prop("json")) {
        (Some(prop), None) => Ok(ServerMiddlewareStatement::Response {
            status,
            body: ServerMiddlewareResponseBody::Text(required_static_string_prop(prop)?),
        }),
        (None, Some(prop)) => Ok(ServerMiddlewareStatement::Response {
            status,
            body: ServerMiddlewareResponseBody::Json(store_literal(&prop.value)?),
        }),
        (None, None) => Err(node_error(node, "return must declare text or json")),
        (Some(_), Some(_)) => Err(node_error(
            node,
            "return must declare exactly one of text or json",
        )),
    }
}

fn middleware_returns(statements: &[ServerMiddlewareStatement]) -> bool {
    statements.iter().any(|statement| {
        matches!(
            statement,
            ServerMiddlewareStatement::Next { .. } | ServerMiddlewareStatement::Response { .. }
        )
    })
}

fn parse_middleware_next(node: &SourceNode) -> DoweResult<ServerMiddlewareStatement> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "`next` does not accept positional arguments",
        ));
    }
    reject_unknown_props(node, &["context"])?;
    Ok(ServerMiddlewareStatement::Next {
        context: node
            .prop("context")
            .map(|prop| store_literal(&prop.value))
            .transpose()?,
    })
}

