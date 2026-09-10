fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.contains(&prop.name.as_str()) {
            return Err(prop_error(
                prop,
                format!("`{}` is not valid on `{}`", prop.name, node.name),
            ));
        }
    }
    Ok(())
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

fn prop_error(prop: &SourceProp, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &prop.location.path,
        format!(
            "{}:{}: {}",
            prop.location.line,
            prop.location.column,
            message.as_ref()
        ),
    )
}

fn required_static_string_prop(prop: &SourceProp) -> DoweResult<String> {
    required_static_string_value(prop, &prop.value)
}

fn required_static_string_value(prop: &SourceProp, value: &SourceValue) -> DoweResult<String> {
    match value {
        SourceValue::String(value) => Ok(value.clone()),
        _ => Err(quoted_static_string_error(prop)),
    }
}

fn quoted_static_string_error(prop: &SourceProp) -> DoweError {
    prop_error(
        prop,
        format!(
            "invalid value for prop `{}`: expected quoted static string literal",
            prop.name
        ),
    )
}
