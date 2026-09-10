fn parse_task(
    node: &SourceNode,
    context: ActionContext,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<ServerBackgroundJob> {
    let timing = parse_task_timing(node, context)?;
    let inline = parse_task_mode(node, false)?;
    if node.prop("fn").is_none() {
        if !node.args.is_empty() {
            return Err(node_error(
                node,
                "task does not accept positional targets; use `fn:<imported-fn>`",
            ));
        }
        if node.children.is_empty() {
            return Err(node_error(
                node,
                "task must declare one imported target or a non-empty inline body",
            ));
        }
        let args = parse_background_args(node, context, bindings, false)?;
        let action = parse_inline_task_action(node, types, environment, imports)?;
        return Ok(ServerBackgroundJob {
            id: background_job_id(node, "task", "inline"),
            target: None,
            args,
            action: Box::new(action),
            schedule: None,
            timing,
            inline,
            source_path: node.location.relative_path.clone(),
            source_line: node.location.line,
        });
    }
    if !node.children.is_empty() {
        return Err(node_error(node, "named task does not accept child blocks"));
    }
    parse_target_background_job(node, context, &imports.callables, bindings, false, timing)
}

fn parse_cron_job(
    node: &SourceNode,
    context: ActionContext,
    callables: &HashMap<String, ServerCallable>,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<ServerBackgroundJob> {
    if let Some(prop) = node.prop("after") {
        return Err(prop_error(
            prop,
            "`after` is only valid on a direct reverse-proxy task",
        ));
    }
    parse_target_background_job(
        node,
        context,
        callables,
        bindings,
        true,
        crate::model::ServerTaskTiming::Immediate,
    )
}

fn parse_target_background_job(
    node: &SourceNode,
    context: ActionContext,
    callables: &HashMap<String, ServerCallable>,
    bindings: &HashMap<String, DoweType>,
    cron: bool,
    timing: crate::model::ServerTaskTiming,
) -> DoweResult<ServerBackgroundJob> {
    if cron && !matches!(context, ActionContext::Init) {
        return Err(node_error(node, "`cron` is only valid inside server init"));
    }
    if !node.children.is_empty() {
        return Err(node_error(
            node,
            if cron {
                "cron does not accept child blocks"
            } else {
                "named task does not accept child blocks"
            },
        ));
    }
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            if cron {
                "cron does not accept positional targets; use `fn:<imported-fn>`"
            } else {
                "task does not accept positional targets; use `fn:<imported-fn>`"
            },
        ));
    }
    let target_prop = node.prop("fn").ok_or_else(|| {
        node_error(
            node,
            if cron {
                "cron must declare `fn:<imported-fn>`"
            } else {
                "task must declare `fn:<imported-fn>`"
            },
        )
    })?;
    let target = match &target_prop.value {
        SourceValue::Bareword(value) if !value.is_empty() => value.clone(),
        _ => {
            return Err(prop_error(
                target_prop,
                "`fn` must reference an imported server function",
            ));
        }
    };
    let callable = callables
        .get(&target)
        .ok_or_else(|| node_error(node, format!("missing server function import `{target}`")))?;
    reject_unknown_props(
        node,
        if cron {
            &["args", "fn", "schedule", "mode"]
        } else {
            &["args", "after", "fn", "mode"]
        },
    )?;
    let args = parse_background_args(node, context, bindings, cron)?;
    validate_server_function_args(node, &args, &callable.action.params, bindings)?;
    let inline = parse_task_mode(node, cron)?;
    let schedule = if cron {
        let prop = node
            .prop("schedule")
            .ok_or_else(|| node_error(node, "cron must declare `schedule`"))?;
        let SourceValue::String(value) = &prop.value else {
            return Err(prop_error(prop, "`schedule` must be a quoted string"));
        };
        CronSchedule::parse(value).map_err(|error| prop_error(prop, error.to_string()))?;
        Some(value.clone())
    } else {
        None
    };
    Ok(ServerBackgroundJob {
        id: background_job_id(node, if cron { "cron" } else { "task" }, &target),
        target: Some(callable.name.clone()),
        args,
        action: Box::new(callable.action.clone()),
        schedule,
        timing,
        inline,
        source_path: node.location.relative_path.clone(),
        source_line: node.location.line,
    })
}

fn parse_task_mode(node: &SourceNode, cron: bool) -> DoweResult<bool> {
    let Some(prop) = node.prop("mode") else {
        return Ok(false);
    };
    let SourceValue::String(value) = &prop.value else {
        return Err(prop_error(prop, "`mode` must be `process` or `inline`"));
    };
    if cron && value == "inline" {
        return Err(prop_error(prop, "cron jobs must use process mode"));
    }
    match value.as_str() {
        "process" => Ok(false),
        "inline" => Ok(true),
        _ => Err(prop_error(prop, "`mode` must be `process` or `inline`")),
    }
}

fn parse_task_timing(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<crate::model::ServerTaskTiming> {
    reject_unknown_props(node, &["args", "after", "fn", "mode"])?;
    let Some(prop) = node.prop("after") else {
        return Ok(crate::model::ServerTaskTiming::Immediate);
    };
    let SourceValue::String(value) = &prop.value else {
        return Err(prop_error(
            prop,
            "`after` must be the quoted string \"headers\"",
        ));
    };
    if value != "headers" {
        return Err(prop_error(prop, "`after` must be \"headers\""));
    }
    if !matches!(context, ActionContext::HttpHandler { .. }) {
        return Err(prop_error(
            prop,
            "`after:\"headers\"` is only valid directly in an HTTP handler that returns `reverse`",
        ));
    }
    validate_response_headers_task_event(node)?;
    Ok(crate::model::ServerTaskTiming::ResponseHeaders)
}

fn validate_response_headers_task_event(node: &SourceNode) -> DoweResult<()> {
    let Some(prop) = node.prop("args") else {
        return Err(node_error(
            node,
            "`after:\"headers\"` requires `args:{ event:{ ... } }`",
        ));
    };
    let SourceValue::Object(entries) = &prop.value else {
        return Err(prop_error(prop, "`args` must be an object"));
    };
    let event = entries.iter().find_map(|entry| match entry {
        SourceObjectEntry::KeyValue { key, value } if key == "event" => Some(value),
        _ => None,
    });
    if !matches!(event, Some(SourceValue::Object(_))) {
        return Err(node_error(
            node,
            "`after:\"headers\"` requires `args.event` to be an object",
        ));
    }
    Ok(())
}

fn parse_background_args(
    node: &SourceNode,
    context: ActionContext,
    bindings: &HashMap<String, DoweType>,
    static_only: bool,
) -> DoweResult<StoreLiteral> {
    let Some(prop) = node.prop("args") else {
        return Ok(StoreLiteral::Object(Vec::new()));
    };
    let SourceValue::Object(_) = &prop.value else {
        return Err(prop_error(prop, "`args` must be an object"));
    };
    let value = store_literal(&prop.value)?;
    if static_only
        || !matches!(
            context,
            ActionContext::HttpHandler { .. } | ActionContext::Function
        )
    {
        reject_background_references(node, &value)?;
    } else {
        validate_store_literal_references(node, &value, bindings)?;
    }
    Ok(value)
}

fn parse_inline_task_action(
    node: &SourceNode,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<ServerFunctionAction> {
    validate_inline_task_body(node, imports)?;
    let mut action_node = node.clone();
    action_node.name = "fn".to_string();
    action_node.args.clear();
    action_node.props.clear();
    action_node.children.push(SourceNode {
        location: node.location.clone(),
        name: "return".to_string(),
        args: Vec::new(),
        props: vec![SourceProp {
            name: "value".to_string(),
            value: SourceValue::Null,
            location: node.location.clone(),
        }],
        children: Vec::new(),
    });
    parse_server_function_action(&action_node, types, environment, imports)
}

fn validate_inline_task_body(node: &SourceNode, imports: &ServerImports) -> DoweResult<()> {
    let mut bindings = imports
        .config_bindings
        .keys()
        .cloned()
        .collect::<HashSet<_>>();
    bindings.insert("args".to_string());
    for child in &node.children {
        validate_inline_task_node(child, &bindings)?;
        if let Some(binding) = inline_task_binding(child, imports) {
            bindings.insert(binding);
        }
    }
    Ok(())
}

fn validate_inline_task_node(node: &SourceNode, bindings: &HashSet<String>) -> DoweResult<()> {
    match node.name.as_str() {
        "return" | "task" | "cron" | "response" | "next" | "send" | "bridge" | "ws" => {
            return Err(node_error(
                node,
                format!("inline task body cannot use `{}`", node.name),
            ));
        }
        _ => {}
    }
    if matches!(node.name.as_str(), "log" | "info" | "warn" | "error") {
        for value in &node.args {
            validate_inline_task_value(node, value, bindings)?;
        }
    }
    for prop in &node.props {
        validate_inline_task_value(node, &prop.value, bindings)?;
    }
    for child in &node.children {
        validate_inline_task_node(child, bindings)?;
    }
    Ok(())
}

