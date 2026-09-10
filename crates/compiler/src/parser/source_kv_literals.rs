fn optional_string_prop(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    match &prop.value {
        SourceValue::String(value) => Ok(Some(value.clone())),
        _ => Err(node_error(
            node,
            format!("`{name}` must be a quoted string"),
        )),
    }
}

fn optional_bool_prop(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::Boolean(value) => Ok(*value),
            _ => Err(node_error(node, format!("`{name}` must be boolean"))),
        })
        .transpose()
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|allowed| *allowed == prop.name) {
            return Err(node_error(
                node,
                format!("Cache does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

fn validate_kv_literal_references(
    node: &SourceNode,
    value: &StoreLiteral,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match value {
        StoreLiteral::Reference(reference) => validate_reference_path(node, reference, bindings),
        StoreLiteral::Array(values) => {
            for value in values {
                validate_kv_literal_references(node, value, bindings)?;
            }
            Ok(())
        }
        StoreLiteral::Object(entries) => {
            for (_, value) in entries {
                validate_kv_literal_references(node, value, bindings)?;
            }
            Ok(())
        }
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn assignment(node: &SourceNode) -> Option<(String, String)> {
    if node.args.len() < 3 {
        return None;
    }
    let binding = node.args[0].as_string_like()?;
    let equals = node.args[1].as_string_like()?;
    let expression = node.args[2].as_string_like()?;
    (equals == "=").then_some((binding, expression))
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}
