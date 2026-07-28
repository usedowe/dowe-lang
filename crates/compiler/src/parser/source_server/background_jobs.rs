fn parse_background_job(
    node: &SourceNode,
    context: ActionContext,
    callables: &HashMap<String, ServerCallable>,
    cron: bool,
) -> DoweResult<ServerBackgroundJob> {
    if cron && !matches!(context, ActionContext::Init) {
        return Err(node_error(node, "`cron` is only valid inside server init"));
    }
    if !node.children.is_empty() {
        return Err(node_error(
            node,
            "background jobs do not accept child blocks",
        ));
    }
    let target = node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .ok_or_else(|| node_error(node, "background job must declare a target"))?;
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "background jobs accept one target and named props",
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
            &["args"]
        },
    )?;
    let args = if let Some(prop) = node.prop("args") {
        let SourceValue::Object(_) = &prop.value else {
            return Err(prop_error(prop, "`args` must be an object"));
        };
        let value = store_literal(&prop.value)?;
        reject_background_references(node, &value)?;
        value
    } else {
        StoreLiteral::Object(Vec::new())
    };
    validate_server_function_args(node, &args, &callable.action.params, &HashMap::new())?;
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
        id: format!(
            "{}:{}:{}:{}",
            node.location.relative_path.display(),
            node.location.line,
            node.name,
            target
        ),
        target: callable.name.clone(),
        args,
        action: Box::new(callable.action.clone()),
        schedule,
    })
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

