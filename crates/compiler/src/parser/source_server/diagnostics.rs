fn single_root<'a>(
    path: &Path,
    nodes: &'a [SourceNode],
    expected: &str,
) -> DoweResult<&'a SourceNode> {
    let mut roots = nodes.iter().filter(|node| node.name == expected);
    let root = roots
        .next()
        .ok_or_else(|| DoweError::at_path(path, format!("missing `{expected}` block")))?;
    if roots.next().is_some() {
        return Err(DoweError::at_path(
            path,
            format!("multiple `{expected}` blocks are not supported"),
        ));
    }
    Ok(root)
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
    match &prop.value {
        SourceValue::String(value) => Ok(value.clone()),
        _ => Err(DoweError::at_path(
            &prop.location.path,
            format!(
                "{}:{}: invalid value for prop `{}`: expected quoted static string literal",
                prop.location.line, prop.location.column, prop.name
            ),
        )),
    }
}

