fn signal_type(node: &SourceNode, types: &crate::parser::TypeRegistry) -> Option<DoweType> {
    let name = prop_string(node, "type")?;
    types.resolve(node, &name).ok()
}

fn find_each_item_fields(
    nodes: &[SourceNode],
    root_nodes: &[SourceNode],
    types: &crate::parser::TypeRegistry,
    reference_root: &str,
) -> Option<Vec<String>> {
    for node in nodes {
        if node.name == "each"
            && node
                .prop("as")
                .and_then(|prop| prop.value.as_required_string())
                .is_some_and(|name| name == reference_root)
        {
            let collection = node
                .prop("in")
                .and_then(|prop| prop.value.as_required_string())?;
            let collection_type = find_signal_type(root_nodes, types, &collection)?;
            if let DoweType::Array(item) = collection_type {
                return Some(reference_fields_for_type(&item));
            }
        }
        if let Some(fields) =
            find_each_item_fields(&node.children, root_nodes, types, reference_root)
        {
            return Some(fields);
        }
    }
    None
}

fn find_signal_type(
    nodes: &[SourceNode],
    types: &crate::parser::TypeRegistry,
    signal: &str,
) -> Option<DoweType> {
    for node in nodes {
        if node.name == "signal"
            && node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|name| name == signal)
        {
            return signal_type(node, types);
        }
        if let Some(value) = find_signal_type(&node.children, types, signal) {
            return Some(value);
        }
    }
    None
}

fn assignment_expression(node: &SourceNode) -> Option<(String, String)> {
    if node.name != "let" || node.args.len() < 3 {
        return None;
    }
    let binding = node.args[0].as_string_like()?;
    if node.args[1].as_string_like()?.as_str() != "=" {
        return None;
    }
    Some((binding, node.args[2].as_string_like()?))
}

