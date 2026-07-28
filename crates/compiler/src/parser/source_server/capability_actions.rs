fn parse_jwt_statement(
    node: &SourceNode,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerJwtStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "JWT requires one result binding: `jwt <binding> ...`",
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "JWT requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(
        node,
        &[
            "secret",
            "key",
            "algorithm",
            "encryption",
            "token",
            "claims",
        ],
    )?;
    let token = node
        .prop("token")
        .map(|prop| {
            prop.value
                .as_string_like()
                .ok_or_else(|| prop_error(prop, "`token` must be a binding reference"))
        })
        .transpose()?;
    let claims = node
        .prop("claims")
        .map(|prop| store_literal(&prop.value))
        .transpose()?;
    if token.is_some() == claims.is_some() {
        return Err(node_error(
            node,
            "JWT requires exactly one of `token` or `claims`",
        ));
    }
    if node.prop("secret").is_some() {
        if node.prop("key").is_some() || node.prop("encryption").is_some() {
            return Err(node_error(
                node,
                "JWS JWT uses `secret` and `algorithm:\"HS256\"`",
            ));
        }
        let secret = required_secret_prop(node, "secret", environment)?;
        let algorithm = required_algorithm_prop(node, "algorithm", &["HS256"])?;
        return match (token, claims) {
            (Some(token), None) => Ok(ServerJwtStatement::Verify {
                binding,
                token,
                secret,
                algorithm,
            }),
            (None, Some(claims)) => Ok(ServerJwtStatement::Sign {
                binding,
                claims,
                secret,
                algorithm,
            }),
            _ => unreachable!(),
        };
    }
    if node.prop("key").is_some() {
        let key = required_secret_prop(node, "key", environment)?;
        let algorithm = required_algorithm_prop(node, "algorithm", &["dir"])?;
        let encryption = required_algorithm_prop(node, "encryption", &["A256GCM"])?;
        return match (token, claims) {
            (Some(token), None) => Ok(ServerJwtStatement::Decrypt {
                binding,
                token,
                key,
                algorithm,
                encryption,
            }),
            (None, Some(claims)) => Ok(ServerJwtStatement::Encrypt {
                binding,
                claims,
                key,
                algorithm,
                encryption,
            }),
            _ => unreachable!(),
        };
    }
    Err(node_error(
        node,
        "JWT requires server-only `secret` for JWS or `key` for JWE",
    ))
}

fn legacy_jwt_let(node: &SourceNode) -> bool {
    assignment(node).is_some_and(|(_, expression)| {
        matches!(
            expression.as_str(),
            "jwt.verify" | "jwt.sign" | "jwt.decrypt" | "jwt.encrypt"
        )
    })
}

fn parse_stdlib_let(node: &SourceNode) -> DoweResult<Option<ServerStdlibStatement>> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(None);
    };
    let Some(call) = parse_stdlib_call(node, &expression, StdlibSurface::Server, &[])? else {
        return Ok(None);
    };
    if node.args.len() != 3 {
        return Err(node_error(
            node,
            format!("`{expression}` only accepts named arguments"),
        ));
    }
    validate_binding_name(node, &binding)?;
    Ok(Some(ServerStdlibStatement { binding, call }))
}

fn parse_stdlib_declaration(node: &SourceNode) -> DoweResult<ServerStdlibStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            format!(
                "{} uses `{} <binding> source:\"<function>\" <props>`",
                node.name, node.name
            ),
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "stdlib requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    let source = required_source_selector(node, &node.name)?;
    let expression = format!("{}.{}", node.name, source);
    let call = parse_stdlib_call(node, &expression, StdlibSurface::Server, &["source"])?
        .ok_or_else(|| {
            node_error(
                node,
                format!("unsupported stdlib namespace `{}`", node.name),
            )
        })?;
    Ok(ServerStdlibStatement { binding, call })
}

fn legacy_stdlib_error(node: &SourceNode, statement: &ServerStdlibStatement) -> DoweError {
    node_error(
        node,
        format!(
            "stdlib uses `{} {} source:\"{}\" <props>`; `let` is not supported",
            statement.call.namespace, statement.binding, statement.call.function
        ),
    )
}

fn required_source_selector(node: &SourceNode, capability: &str) -> DoweResult<String> {
    let prop = node
        .prop("source")
        .ok_or_else(|| node_error(node, format!("`{capability}` requires `source:\"...\"`")))?;
    required_static_string_prop(prop)
}

fn parse_server_function_call(
    node: &SourceNode,
    context: ActionContext,
    callables: &HashMap<String, ServerCallable>,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<Option<ServerCallStatement>> {
    let Some(callable) = callables.get(&node.name) else {
        return Ok(None);
    };
    if !node.children.is_empty() {
        return Err(node_error(
            node,
            "server function calls do not accept child blocks",
        ));
    }
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            format!(
                "server function call requires one result binding: `{} <binding> [args:{{ ... }}]`",
                callable.name
            ),
        ));
    }
    let binding = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "server function call requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    reject_unknown_props(node, &["args"])?;
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
    let args = if let Some(prop) = node.prop("args") {
        match &prop.value {
            SourceValue::Object(_) => store_literal(&prop.value)?,
            _ => return Err(prop_error(prop, "`args` must be an object")),
        }
    } else {
        StoreLiteral::Object(Vec::new())
    };
    validate_server_function_args(node, &args, &callable.action.params, bindings)?;
    Ok(Some(ServerCallStatement {
        binding,
        target: callable.name.clone(),
        args,
        action: Box::new(callable.action.clone()),
    }))
}

fn push_server_function_call(
    node: &SourceNode,
    context: ActionContext,
    callables: &HashMap<String, ServerCallable>,
    bindings: &mut HashMap<String, DoweType>,
    statements: &mut Vec<ServerStatement>,
) -> DoweResult<bool> {
    let Some(statement) = parse_server_function_call(node, context, callables, bindings)? else {
        return Ok(false);
    };
    validate_store_literal_references(node, &statement.args, bindings)?;
    bindings.insert(
        statement.binding.clone(),
        statement
            .action
            .return_type
            .as_ref()
            .map(|return_type| return_type.schema.clone())
            .unwrap_or(DoweType::Unknown),
    );
    statements.push(ServerStatement::Call(statement));
    Ok(true)
}

fn reject_legacy_server_function_call(
    node: &SourceNode,
    callables: &HashMap<String, ServerCallable>,
) -> DoweResult<()> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(());
    };
    let Some(callable) = callables.get(&expression) else {
        return Ok(());
    };
    let args = if callable.action.params.is_empty() {
        ""
    } else {
        " args:{ ... }"
    };
    Err(node_error(
        node,
        format!(
            "server function calls use `{} {}{args}`; `let {} = {}` is not supported",
            callable.name, binding, binding, callable.name
        ),
    ))
}

