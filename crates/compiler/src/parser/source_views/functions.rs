fn parse_view_function(
    node: &SourceNode,
    scope_kind: &str,
    scope_name: &str,
    types: &TypeRegistry,
) -> DoweResult<ViewAction> {
    if node.args.len() != 1 {
        return Err(node_error(node, "`fn` must declare one name"));
    }
    let name = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`fn` must declare a name"))?;
    if name == "init" {
        return Err(node_error(
            node,
            "`init` is a reserved view hook; declare `init` without `fn`",
        ));
    }
    if node.children.is_empty() {
        return Err(node_error(node, "`fn` must contain at least one statement"));
    }
    for prop in &node.props {
        if !matches!(prop.name.as_str(), "params" | "return") {
            return Err(prop_error(
                prop,
                format!("`fn` does not support `{}`", prop.name),
            ));
        }
    }
    let params = parse_function_params(node, types)?;
    let return_type = parse_function_return(node, types)?;
    let kind = if node.children.len() == 1 {
        let operation = &node.children[0];
        match operation.name.as_str() {
            "request"
                if operation
                    .args
                    .first()
                    .and_then(SourceValue::as_required_string)
                    .is_some_and(|value| ViewRequestMethod::from_name(&value).is_some()) =>
            {
                ViewActionKind::Request(parse_legacy_request_action(operation)?)
            }
            "set" => ViewActionKind::Assign(parse_set_action(operation)?),
            "reset" => ViewActionKind::Reset(parse_reset_action(operation)?),
            _ => ViewActionKind::Sequence(parse_function_statements(&node.children)?),
        }
    } else {
        ViewActionKind::Sequence(parse_function_statements(&node.children)?)
    };
    Ok(ViewAction {
        id: reactive_id("fn", scope_kind, scope_name, node, &name),
        name,
        params,
        return_type,
        kind,
    })
}

fn parse_view_init(
    node: &SourceNode,
    scope_kind: &str,
    scope_name: &str,
) -> DoweResult<ViewAction> {
    if !node.args.is_empty() || !node.props.is_empty() {
        return Err(node_error(
            node,
            "`init` does not accept a name, arguments, or props",
        ));
    }
    if node.children.is_empty() {
        return Err(node_error(
            node,
            "`init` must contain at least one statement",
        ));
    }
    Ok(ViewAction::init(
        reactive_id("init", scope_kind, scope_name, node, "init"),
        parse_function_statements(&node.children)?,
    ))
}

fn parse_function_statements(nodes: &[SourceNode]) -> DoweResult<Vec<ViewFunctionStatement>> {
    let mut statements = Vec::new();
    let mut index = 0;
    while index < nodes.len() {
        let node = &nodes[index];
        match node.name.as_str() {
            "request" => statements.push(parse_request_statement(node)?),
            "set" => statements.push(ViewFunctionStatement::Assign(parse_set_action(node)?)),
            "reset" => statements.push(ViewFunctionStatement::Reset(parse_reset_action(node)?)),
            "toast" => statements.push(ViewFunctionStatement::Toast(parse_toast_statement(node)?)),
            "redirect" => statements.push(parse_redirect_statement(node)?),
            "if" => {
                let else_node = nodes.get(index + 1).filter(|next| next.name == "else");
                statements.push(parse_function_if(node, else_node)?);
                index += usize::from(else_node.is_some());
            }
            "else" => {
                return Err(node_error(
                    node,
                    "`else` must follow an `if` at the same indentation level",
                ));
            }
            "onSuccess" | "onError" => {
                return Err(node_error(
                    node,
                    "request callbacks were replaced by `request result ...` followed by `if result.ok`",
                ));
            }
            "assign" => {
                return Err(node_error(
                    node,
                    "`assign` was replaced by `set target value:<value>`",
                ));
            }
            _ => {
                return Err(node_error(
                    node,
                    "view function statements must be `request`, `if`, `set`, `reset`, `toast`, or `redirect`",
                ));
            }
        }
        index += 1;
    }
    Ok(statements)
}

fn parse_redirect_statement(node: &SourceNode) -> DoweResult<ViewFunctionStatement> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`redirect` does not accept arguments or children",
        ));
    }
    for prop in &node.props {
        if prop.name != "path" {
            return Err(prop_error(
                prop,
                format!("`redirect` does not support `{}`", prop.name),
            ));
        }
    }
    let path = required_static_string_prop(node, "path")?;
    if !path.starts_with('/') {
        return Err(node_error(node, "`redirect` path must start with `/`"));
    }
    Ok(ViewFunctionStatement::Redirect { path })
}

fn parse_request_statement(node: &SourceNode) -> DoweResult<ViewFunctionStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`request` must use `request result method:\"METHOD\" route:\"/path\"`",
        ));
    }
    let result = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`request` result must be a name"))?;
    if result.contains('.') {
        return Err(node_error(node, "`request` result must be one name"));
    }
    for prop in &node.props {
        if !matches!(
            prop.name.as_str(),
            "method" | "base" | "headers" | "route" | "path" | "body" | "autoload"
        ) {
            return Err(node_error(
                node,
                format!(
                    "`request {}` was replaced by sequential `set`, `reset`, and `if {result}.ok` statements",
                    prop.name
                ),
            ));
        }
    }
    let method_name = required_static_string_prop(node, "method")?;
    let method = ViewRequestMethod::from_name(&method_name).ok_or_else(|| {
        node_error(
            node,
            "`request method` must be GET, POST, PUT, PATCH or DELETE",
        )
    })?;
    let path = request_path(node)?;
    if !path.starts_with('/') {
        return Err(node_error(node, "`request` path must start with `/`"));
    }
    let base_env = optional_env_ref_prop(node, "base")?
        .or_else(|| is_api_route(&path).then(|| "BACKEND_URL".to_string()));
    Ok(ViewFunctionStatement::Request {
        result,
        action: ViewRequestAction {
            method,
            path,
            base_env,
            headers: request_headers(node)?,
            body: optional_prop_string(node, "body")?,
            update: None,
            reset: None,
            success_alert: None,
            success_message: None,
            error_alert: None,
            error_message: None,
            autoload: optional_prop_bool(node, "autoload")?.unwrap_or(false),
        },
    })
}
