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
