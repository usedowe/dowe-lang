fn static_value_has_bareword(value: &SourceValue) -> bool {
    match value {
        SourceValue::Bareword(_) => true,
        SourceValue::Object(entries) => entries.iter().any(|entry| match entry {
            SourceObjectEntry::KeyValue { value, .. } => static_value_has_bareword(value),
            SourceObjectEntry::Spread(_) => false,
        }),
        SourceValue::Array(values) => values.iter().any(static_value_has_bareword),
        SourceValue::String(_)
        | SourceValue::Number(_)
        | SourceValue::Boolean(_)
        | SourceValue::Null => false,
    }
}

fn prop_value(prop: &SourceProp) -> DoweResult<PropValue> {
    match &prop.value {
        SourceValue::String(value) | SourceValue::Bareword(value) => {
            Ok(PropValue::String(value.clone()))
        }
        SourceValue::Number(value) => Ok(PropValue::Number(value.clone())),
        SourceValue::Boolean(value) => Ok(PropValue::Boolean(*value)),
        SourceValue::Object(entries) => {
            Ok(PropValue::Responsive(responsive_entries(prop, entries)?))
        }
        SourceValue::Null | SourceValue::Array(_) => Err(DoweError::at_path(
            &prop.location.path,
            format!(
                "{}:{}: prop `{}` has unsupported value",
                prop.location.line, prop.location.column, prop.name
            ),
        )),
    }
}

fn responsive_entries(
    prop: &SourceProp,
    entries: &[SourceObjectEntry],
) -> DoweResult<Vec<ResponsivePropEntry>> {
    entries
        .iter()
        .map(|entry| match entry {
            SourceObjectEntry::KeyValue { key, value } => Ok(ResponsivePropEntry {
                breakpoint: key.clone(),
                value: prop_scalar(prop, value)?,
            }),
            SourceObjectEntry::Spread(_) => Err(DoweError::at_path(
                &prop.location.path,
                format!(
                    "{}:{}: prop `{}` cannot use object spread",
                    prop.location.line, prop.location.column, prop.name
                ),
            )),
        })
        .collect()
}

fn prop_scalar(prop: &SourceProp, value: &SourceValue) -> DoweResult<PropScalar> {
    match value {
        SourceValue::String(value) | SourceValue::Bareword(value) => {
            Ok(PropScalar::String(value.clone()))
        }
        SourceValue::Number(value) => Ok(PropScalar::Number(value.clone())),
        SourceValue::Boolean(value) => Ok(PropScalar::Boolean(*value)),
        SourceValue::Null | SourceValue::Array(_) | SourceValue::Object(_) => {
            Err(DoweError::at_path(
                &prop.location.path,
                format!(
                    "{}:{}: responsive prop `{}` has unsupported value",
                    prop.location.line, prop.location.column, prop.name
                ),
            ))
        }
    }
}

fn required_text_child(node: &SourceNode, component: BuiltinComponent) -> DoweResult<String> {
    text_child_value(node)?.ok_or_else(|| {
        node_error(
            node,
            format!("{} requires a text child", component.as_str()),
        )
    })
}

fn reject_text_prop(node: &SourceNode, component: BuiltinComponent) -> DoweResult<()> {
    if let Some(prop) = node.prop("text") {
        Err(prop_error(
            prop,
            ComponentError::unknown_prop(component, "text").to_string(),
        ))
    } else {
        Ok(())
    }
}

fn text_child_value(node: &SourceNode) -> DoweResult<Option<String>> {
    if node.children.is_empty() {
        return Ok(None);
    }
    if node.children.len() != 1 {
        return Err(node_error(node, "text components accept one text child"));
    }
    let child = &node.children[0];
    if !child.children.is_empty() || !child.props.is_empty() {
        return Err(quoted_text_child_error(child));
    }
    if !child.args.is_empty() {
        return Err(quoted_text_child_error(child));
    }
    let SourceValue::String(value) = parse_value(
        &child.location.path,
        child.location.line,
        child.location.column,
        &child.name,
    )?
    else {
        return Err(quoted_text_child_error(child));
    };
    Ok(Some(value))
}

fn text_child_line(node: &SourceNode) -> DoweResult<String> {
    if !node.children.is_empty() || !node.props.is_empty() {
        return Err(node_error(node, "text child must be plain text"));
    }
    let mut parts = Vec::new();
    parts.push(text_token(&node.name));
    parts.extend(node.args.iter().map(SourceValue::to_source).map(|value| {
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value[1..value.len() - 1].to_string()
        } else {
            value
        }
    }));
    Ok(parts.join(" "))
}

fn text_token(value: &str) -> String {
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn quoted_text_child_error(node: &SourceNode) -> DoweError {
    node_error(
        node,
        format!(
            "text child `{}` must be a quoted static string literal",
            text_child_source(node)
        ),
    )
}

fn text_child_source(node: &SourceNode) -> String {
    let mut parts = vec![node.name.clone()];
    parts.extend(node.args.iter().map(SourceValue::to_source));
    parts.join(" ")
}

fn reject_children(node: &SourceNode) -> DoweResult<()> {
    if node.children.is_empty() {
        Ok(())
    } else {
        Err(node_error(
            node,
            "children are not valid for this component",
        ))
    }
}

fn single_export(file: &SourceFile) -> DoweResult<&SourceNode> {
    let exports = file
        .nodes
        .iter()
        .filter(|node| matches!(node.name.as_str(), "layout" | "page" | "component"))
        .collect::<Vec<_>>();
    if exports.len() != 1
        || file
            .nodes
            .iter()
            .any(|node| !matches!(node.name.as_str(), "type" | "layout" | "page" | "component"))
    {
        return Err(DoweError::at_path(
            &file.path,
            "view modules must declare one export",
        ));
    }
    Ok(exports[0])
}

fn single_tree(path: &Path, mut nodes: Vec<ViewNode>) -> DoweResult<ViewNode> {
    if nodes.len() != 1 {
        return Err(DoweError::at_path(
            path,
            "view source must contain one root view node",
        ));
    }
    Ok(nodes.remove(0))
}

fn required_prop_string(node: &SourceNode, name: &str) -> DoweResult<String> {
    node.prop(name)
        .and_then(|prop| prop.value.as_required_string())
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))
}

fn required_prop_bareword(node: &SourceNode, name: &str) -> DoweResult<String> {
    match node.prop(name).map(|prop| &prop.value) {
        Some(SourceValue::Bareword(value)) => Ok(value.clone()),
        Some(_) => Err(node_error(node, format!("`{name}` must be a reference"))),
        None => Err(node_error(node, format!("missing `{name}`"))),
    }
}

fn required_prop_number(node: &SourceNode, name: &str) -> DoweResult<String> {
    match node.prop(name).map(|prop| &prop.value) {
        Some(SourceValue::Number(value)) => Ok(value.clone()),
        Some(_) => Err(node_error(node, format!("`{name}` must be a number"))),
        None => Err(node_error(node, format!("missing `{name}`"))),
    }
}

fn required_path_prop(node: &SourceNode) -> DoweResult<String> {
    let prop = node
        .prop("path")
        .ok_or_else(|| node_error(node, "missing `path`"))?;
    match &prop.value {
        SourceValue::String(value) => Ok(value.clone()),
        _ => Err(quoted_static_string_error(prop)),
    }
}

fn used_components(declaration: &ViewDeclaration) -> Vec<String> {
    let mut used = vec![declaration.component.clone()];
    for child in &declaration.children {
        used.extend(used_components(child));
    }
    used
}

