fn parse_task(
    node: &SourceNode,
    context: ActionContext,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<ServerBackgroundJob> {
    let timing = parse_task_timing(node, context)?;
    if node.args.is_empty() {
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
    let target = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| {
            node_error(
                node,
                if cron {
                    "cron must declare one imported target"
                } else {
                    "task must declare one imported target"
                },
            )
        })?;
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            if cron {
                "cron accepts exactly one target and named props"
            } else {
                "task accepts exactly one target and named props"
            },
        ));
    }
    let callable = callables
        .get(&target)
        .ok_or_else(|| node_error(node, format!("missing server function import `{target}`")))?;
    reject_unknown_props(
        node,
        if cron {
            &["args", "schedule"]
        } else {
            &["args", "after"]
        },
    )?;
    let args = parse_background_args(node, context, bindings, cron)?;
    validate_server_function_args(node, &args, &callable.action.params, bindings)?;
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
        source_path: node.location.relative_path.clone(),
        source_line: node.location.line,
    })
}

fn parse_task_timing(
    node: &SourceNode,
    context: ActionContext,
) -> DoweResult<crate::model::ServerTaskTiming> {
    reject_unknown_props(node, &["args", "after"])?;
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

fn validate_inline_task_value(
    node: &SourceNode,
    value: &SourceValue,
    bindings: &HashSet<String>,
) -> DoweResult<()> {
    match value {
        SourceValue::Bareword(reference) => {
            let binding = reference.split('.').next().unwrap_or(reference);
            if !bindings.contains(binding) {
                return Err(node_error(
                    node,
                    format!(
                        "inline task body cannot capture outer binding `{binding}`; pass it through `args`"
                    ),
                ));
            }
        }
        SourceValue::Array(values) => {
            for value in values {
                validate_inline_task_value(node, value, bindings)?;
            }
        }
        SourceValue::Object(entries) => {
            for entry in entries {
                match entry {
                    SourceObjectEntry::KeyValue { value, .. } => {
                        validate_inline_task_value(node, value, bindings)?;
                    }
                    SourceObjectEntry::Spread(_) => {
                        return Err(node_error(
                            node,
                            "inline task values do not support object spread",
                        ));
                    }
                }
            }
        }
        SourceValue::String(_)
        | SourceValue::Number(_)
        | SourceValue::Boolean(_)
        | SourceValue::Null => {}
    }
    Ok(())
}

fn inline_task_binding(node: &SourceNode, imports: &ServerImports) -> Option<String> {
    let creates_binding = matches!(
        node.name.as_str(),
        "database"
            | "db"
            | "cache"
            | "kv"
            | "query"
            | "vector"
            | "emb"
            | "spawn"
            | "file"
            | "password"
            | "http"
            | "crypto"
            | "jwt"
            | "agent"
    ) || dowe_stdlib::is_stdlib_namespace(&node.name)
        || imports.callables.contains_key(&node.name);
    creates_binding
        .then(|| node.args.first().and_then(SourceValue::as_required_string))
        .flatten()
        .map(|binding| binding.split(':').next().unwrap_or(&binding).to_string())
}

fn background_job_id(node: &SourceNode, kind: &str, target: &str) -> String {
    format!(
        "{}:{}:{kind}:{target}",
        node.location.relative_path.display(),
        node.location.line,
    )
}

fn legacy_task_error(node: &SourceNode) -> DoweError {
    let mut repair = "task".to_string();
    for argument in &node.args {
        repair.push(' ');
        repair.push_str(&argument.to_source());
    }
    for prop in &node.props {
        repair.push(' ');
        repair.push_str(&prop.name);
        repair.push(':');
        repair.push_str(&prop.value.to_source());
    }
    node_error(node, format!("`go` was renamed to `task`; use `{repair}`"))
}

fn reject_background_references(node: &SourceNode, value: &StoreLiteral) -> DoweResult<()> {
    match value {
        StoreLiteral::Reference(reference) => Err(node_error(
            node,
            format!("background args must be static JSON; found reference `{reference}`"),
        )),
        StoreLiteral::Array(values) => {
            for value in values {
                reject_background_references(node, value)?;
            }
            Ok(())
        }
        StoreLiteral::Object(entries) => {
            for (_, value) in entries {
                reject_background_references(node, value)?;
            }
            Ok(())
        }
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn parse_server_function_params(
    node: &SourceNode,
    types: &TypeRegistry,
) -> DoweResult<Vec<ServerFunctionParameter>> {
    let Some(prop) = node.prop("params") else {
        return Ok(Vec::new());
    };
    let SourceValue::Object(entries) = &prop.value else {
        return Err(prop_error(prop, "fn params must be an object"));
    };
    if entries.is_empty() {
        return Err(prop_error(prop, "fn params must be a non-empty object"));
    }
    let mut params = Vec::new();
    let mut names = HashSet::new();
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "fn params does not support object spread"));
        };
        if !names.insert(key.clone()) {
            return Err(prop_error(prop, format!("duplicate fn parameter `{key}`")));
        }
        validate_binding_name(node, key)?;
        let type_name = value
            .as_required_string()
            .ok_or_else(|| prop_error(prop, "fn params values must be type names"))?;
        let schema = types.resolve(node, &type_name)?;
        params.push(ServerFunctionParameter {
            name: key.clone(),
            type_name,
            schema,
        });
    }
    Ok(params)
}

fn parse_server_function_return(
    node: &SourceNode,
    types: &TypeRegistry,
) -> DoweResult<Option<ServerFunctionReturn>> {
    let Some(prop) = node.prop("return") else {
        return Ok(None);
    };
    let type_name = prop
        .value
        .as_required_string()
        .ok_or_else(|| prop_error(prop, "fn return must be a quoted type name"))?;
    let schema = types.resolve(node, &type_name)?;
    Ok(Some(ServerFunctionReturn { type_name, schema }))
}

fn validate_server_function_args(
    node: &SourceNode,
    args: &StoreLiteral,
    params: &[ServerFunctionParameter],
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    let StoreLiteral::Object(entries) = args else {
        return Err(node_error(node, "function args must be an object"));
    };
    if params.is_empty() {
        if entries.is_empty() {
            return Ok(());
        }
        return Err(node_error(node, "function does not declare params"));
    }
    for parameter in params {
        let value = entries
            .iter()
            .find(|(name, _)| name == &parameter.name)
            .map(|(_, value)| value)
            .ok_or_else(|| {
                node_error(
                    node,
                    format!(
                        "function call is missing required argument `{}`",
                        parameter.name
                    ),
                )
            })?;
        let actual = server_literal_type(value, bindings);
        if !server_type_assignable(&actual, &parameter.schema) {
            return Err(node_error(
                node,
                format!(
                    "argument `{}` is incompatible with function parameter type `{}`",
                    parameter.name, parameter.type_name
                ),
            ));
        }
    }
    for (name, _) in entries {
        if !params.iter().any(|parameter| parameter.name == *name) {
            return Err(node_error(
                node,
                format!("function call does not declare argument `{name}`"),
            ));
        }
    }
    Ok(())
}

fn server_literal_type(value: &StoreLiteral, bindings: &HashMap<String, DoweType>) -> DoweType {
    match value {
        StoreLiteral::Reference(reference) => {
            server_reference_type(reference, bindings).unwrap_or(DoweType::Unknown)
        }
        StoreLiteral::Array(values) => DoweType::Array(Box::new(
            values
                .first()
                .map(|value| server_literal_type(value, bindings))
                .unwrap_or(DoweType::Unknown),
        )),
        StoreLiteral::Object(entries) => DoweType::Object(
            entries
                .iter()
                .map(|(name, value)| DoweTypeField {
                    name: name.clone(),
                    value: server_literal_type(value, bindings),
                    optional: false,
                })
                .collect(),
        ),
        _ => type_from_store_literal(value),
    }
}

fn server_reference_type(
    reference: &str,
    bindings: &HashMap<String, DoweType>,
) -> Option<DoweType> {
    let (binding, path) = reference
        .split_once('.')
        .map_or((reference, ""), |(binding, path)| (binding, path));
    let mut value = bindings.get(binding)?.clone();
    for segment in path.split('.').filter(|segment| !segment.is_empty()) {
        value = match value {
            DoweType::Unknown => return Some(DoweType::Unknown),
            DoweType::Object(fields) => fields
                .into_iter()
                .find(|field| field.name == segment)
                .map(|field| field.value)?,
            _ => return None,
        };
    }
    Some(value)
}

fn server_type_assignable(actual: &DoweType, expected: &DoweType) -> bool {
    match (actual, expected) {
        (_, DoweType::Unknown) | (DoweType::Unknown, _) => true,
        (DoweType::Null, DoweType::Null)
        | (DoweType::Bool, DoweType::Bool)
        | (DoweType::Number, DoweType::Number)
        | (DoweType::String, DoweType::String) => true,
        (DoweType::Array(actual), DoweType::Array(expected)) => {
            server_type_assignable(actual, expected)
        }
        (DoweType::Object(actual), DoweType::Object(expected)) => expected.iter().all(|field| {
            actual
                .iter()
                .find(|candidate| candidate.name == field.name)
                .is_some_and(|candidate| server_type_assignable(&candidate.value, &field.value))
                || field.optional
        }),
        _ => false,
    }
}

fn parse_server_function_return_value(node: &SourceNode) -> DoweResult<StoreLiteral> {
    if node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .as_deref()
        == Some("response")
    {
        return Err(node_error(
            node,
            "server fn return must use `return value:<value>`",
        ));
    }
    if !node.args.is_empty() {
        return Err(node_error(node, "return value must use `value:<value>`"));
    }
    if node.props.iter().any(|prop| prop.name != "value") {
        return Err(node_error(
            node,
            "server fn return must use `return value:<value>`",
        ));
    }
    reject_unknown_props(node, &["value"])?;
    required_store_literal_prop(node, "value")
}
