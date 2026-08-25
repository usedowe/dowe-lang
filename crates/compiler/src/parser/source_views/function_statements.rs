fn parse_function_if(
    node: &SourceNode,
    else_node: Option<&SourceNode>,
) -> DoweResult<ViewFunctionStatement> {
    if !node.props.is_empty() || node.args.len() != 1 || node.children.is_empty() {
        return Err(node_error(
            node,
            "`if` must use `if result.ok` with statements",
        ));
    }
    let condition = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`if` must use `if result.ok`"))?;
    let Some(result) = condition.strip_suffix(".ok") else {
        return Err(node_error(
            node,
            "`if` only supports `if result.ok` in a view function",
        ));
    };
    if result.is_empty() {
        return Err(node_error(node, "`if` must name a request result"));
    }
    let else_node =
        else_node.ok_or_else(|| node_error(node, "`if result.ok` must be followed by `else`"))?;
    if !else_node.args.is_empty() || !else_node.props.is_empty() || else_node.children.is_empty() {
        return Err(node_error(
            else_node,
            "`else` must contain function statements",
        ));
    }
    Ok(ViewFunctionStatement::If {
        result: result.to_string(),
        success: parse_function_statements(&node.children)?,
        error: parse_function_statements(&else_node.children)?,
    })
}

fn parse_toast_statement(node: &SourceNode) -> DoweResult<ViewToastAction> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(node, "`toast` only accepts props"));
    }
    for prop in &node.props {
        if !matches!(
            prop.name.as_str(),
            "value" | "duration" | "scheme" | "variant" | "position"
        ) {
            return Err(node_error(
                node,
                format!("`toast` does not support {}", prop.name),
            ));
        }
    }
    let value = node
        .prop("value")
        .ok_or_else(|| node_error(node, "`toast` requires `value`"))?;
    let SourceValue::Object(entries) = &value.value else {
        return Err(node_error(node, "`toast value` must be an object"));
    };
    let mut kind = "info".to_string();
    let mut title = String::new();
    let mut message = String::new();
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(node_error(
                node,
                "`toast value` does not support object spread",
            ));
        };
        match key.as_str() {
            "type" => {
                kind = value
                    .as_required_string()
                    .ok_or_else(|| node_error(node, "`toast value.type` must be a string"))?
            }
            "title" => {
                title = value
                    .as_required_string()
                    .ok_or_else(|| node_error(node, "`toast value.title` must be a string"))?
            }
            "message" => {
                message = value
                    .as_required_string()
                    .ok_or_else(|| node_error(node, "`toast value.message` must be a string"))?
            }
            "visible" => match value {
                SourceValue::Boolean(true) => {}
                SourceValue::Boolean(false) => {
                    return Err(node_error(node, "`toast value.visible` must be true"));
                }
                _ => return Err(node_error(node, "`toast value.visible` must be a boolean")),
            },
            "duration" => {}
            _ => {
                return Err(node_error(
                    node,
                    format!("`toast value` does not support {key}"),
                ));
            }
        }
    }
    if !matches!(kind.as_str(), "success" | "info" | "warning" | "error") || message.is_empty() {
        return Err(node_error(
            node,
            "`toast value` requires a valid type and non-empty message",
        ));
    }
    let scheme = optional_static_string_prop(node, "scheme")?;
    if scheme
        .as_deref()
        .is_some_and(|value| ColorFamily::from_name(value).is_none())
    {
        return Err(node_error(
            node,
            "`toast scheme` must be a Card design family",
        ));
    }
    let variant = optional_static_string_prop(node, "variant")?;
    let variant = match variant {
        Some(value) => match ComponentVariant::from_name(&value) {
            Some(parsed @ (ComponentVariant::Solid | ComponentVariant::Outlined | ComponentVariant::Ghost)) =>
                Some(parsed.as_str().to_string()),
            _ => {
                return Err(node_error(
                    node,
                    "`toast variant` must be solid, outlined, or ghost",
                ));
            }
        },
        None => None,
    };
    let position = optional_static_string_prop(node, "position")?;
    if position
        .as_deref()
        .is_some_and(|value| OverlayCornerPosition::from_name(value).is_none())
    {
        return Err(node_error(
            node,
            "`toast position` must be top-left, top-right, bottom-left, or bottom-right",
        ));
    }
    Ok(ViewToastAction {
        kind,
        title,
        message,
        duration: node.prop("duration").and_then(|prop| match &prop.value {
            SourceValue::Number(value) => value.parse().ok(),
            _ => None,
        }),
        scheme,
        variant,
        position,
    })
}

fn parse_function_params(
    node: &SourceNode,
    types: &TypeRegistry,
) -> DoweResult<Vec<ViewFunctionParameter>> {
    let Some(prop) = node.prop("params") else {
        return Ok(Vec::new());
    };
    let SourceValue::Object(entries) = &prop.value else {
        return Err(prop_error(prop, "`fn params` must be an object"));
    };
    let mut params = Vec::new();
    let mut names = HashSet::new();
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(
                prop,
                "`fn params` does not support object spread",
            ));
        };
        if !names.insert(key.clone()) {
            return Err(prop_error(prop, format!("duplicate fn parameter `{key}`")));
        }
        let type_name = value
            .as_required_string()
            .ok_or_else(|| prop_error(prop, "`fn params` values must be type names"))?;
        let schema = view_schema_value(&types.resolve(node, &type_name)?);
        params.push(ViewFunctionParameter {
            name: key.clone(),
            type_name,
            schema,
        });
    }
    Ok(params)
}

fn parse_function_return(
    node: &SourceNode,
    types: &TypeRegistry,
) -> DoweResult<Option<ViewFunctionReturn>> {
    let Some(prop) = node.prop("return") else {
        return Ok(None);
    };
    let type_name = prop
        .value
        .as_required_string()
        .ok_or_else(|| prop_error(prop, "`fn return` must be a quoted type name"))?;
    let schema = view_schema_value(&types.resolve(node, &type_name)?);
    Ok(Some(ViewFunctionReturn { type_name, schema }))
}

fn request_headers(node: &SourceNode) -> DoweResult<Vec<ViewRequestHeader>> {
    let Some(prop) = node.prop("headers") else {
        return Ok(Vec::new());
    };
    let SourceValue::Object(entries) = &prop.value else {
        return Err(node_error(node, "`request headers` must be an object"));
    };
    let mut headers = Vec::new();
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(node_error(node, "`request headers` must use named fields"));
        };
        let header_value = match value {
            SourceValue::String(value) => ViewRequestHeaderValue::Static(value.clone()),
            SourceValue::Bareword(value) => ViewRequestHeaderValue::Signal(value.clone()),
            _ => {
                return Err(node_error(
                    node,
                    "`request headers` values must be strings or Signal references",
                ));
            }
        };
        headers.push(ViewRequestHeader {
            name: key.clone(),
            value: header_value,
        });
    }
    Ok(headers)
}

fn request_path(node: &SourceNode) -> DoweResult<String> {
    let positional = match node.args.get(1) {
        Some(value) => Some(
            value
                .as_required_string()
                .ok_or_else(|| node_error(node, "`request` path must be a string"))?,
        ),
        None => None,
    };
    let route = optional_static_string_prop(node, "route")?;
    let path = optional_static_string_prop(node, "path")?;
    let count = usize::from(positional.is_some())
        + usize::from(route.is_some())
        + usize::from(path.is_some());
    if count == 0 {
        return Err(node_error(
            node,
            "`request` must declare a route with a positional path, `route`, or `path`",
        ));
    }
    if count > 1 {
        return Err(node_error(
            node,
            "`request` must declare only one route path",
        ));
    }
    Ok(positional.or(route).or(path).unwrap_or_default())
}

fn is_api_route(path: &str) -> bool {
    path == "/api" || path.starts_with("/api/")
}

fn optional_env_ref_prop(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::Bareword(value) => parse_env_ref(node, name, value),
            _ => Err(node_error(node, format!("`{name}` must be `env.NAME`"))),
        })
        .transpose()
}

fn parse_env_ref(node: &SourceNode, name: &str, value: &str) -> DoweResult<String> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() != 2 || parts[0] != "env" || parts[1].is_empty() {
        return Err(node_error(node, format!("`{name}` must be `env.NAME`")));
    }
    Ok(parts[1].to_string())
}

fn parse_set_action(node: &SourceNode) -> DoweResult<ViewAssignAction> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(node, "`set` must use `set target value:value`"));
    }
    let target = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`set` target must be a Signal or View Store name"))?;
    let (source, literal, call) = if let Some(source) = node.prop("source") {
        let source = source
            .value
            .as_required_string()
            .ok_or_else(|| node_error(node, "`set source` must be a reference"))?;
        (
            source.clone(),
            None,
            parse_stdlib_call(node, &source, StdlibSurface::Views, &["source"])?,
        )
    } else {
        if node.props.len() != 1 || node.prop("value").is_none() {
            return Err(node_error(node, "`set` only supports the `value` prop"));
        }
        let value = &node.prop("value").expect("value prop").value;
        let (source, literal) = match value {
            SourceValue::Bareword(value) if value.starts_with('!') && value.len() > 1 => {
                (value.clone(), None)
            }
            SourceValue::Bareword(value) => (value.clone(), None),
            SourceValue::Boolean(value) => (format!("$dowe:bool:{value}"), None),
            SourceValue::String(value) => (format!("$dowe:string:{value}"), None),
            SourceValue::Null
            | SourceValue::Number(_)
            | SourceValue::Array(_)
            | SourceValue::Object(_) => (
                "$dowe:literal".to_string(),
                Some(signal_value(value, node)?),
            ),
        };
        (source, literal, None)
    };
    Ok(ViewAssignAction {
        target,
        source,
        literal,
        call,
    })
}

fn parse_reset_action(node: &SourceNode) -> DoweResult<ViewResetAction> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(node, "`reset` must use `reset target`"));
    }
    Ok(ViewResetAction {
        target: node.args[0]
            .as_required_string()
            .ok_or_else(|| node_error(node, "`reset` target must be a signal name"))?,
    })
}

fn optional_prop_string(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    node.prop(name)
        .map(|prop| {
            prop.value
                .as_required_string()
                .ok_or_else(|| node_error(node, format!("`{name}` must be a string")))
        })
        .transpose()
}

fn optional_static_string_prop(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::String(value) if !value.is_empty() => Ok(value.clone()),
            SourceValue::String(_) => Err(node_error(node, format!("`{name}` must be a string"))),
            _ => Err(quoted_static_string_error(prop)),
        })
        .transpose()
}

fn required_static_string_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    optional_static_string_prop(node, name)?
        .ok_or_else(|| node_error(node, format!("`{name}` must be a quoted string")))
}

fn optional_prop_bool(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::Boolean(value) => Ok(*value),
            _ => Err(node_error(node, format!("`{name}` must be a boolean"))),
        })
        .transpose()
}
