fn export_tree(
    node: &SourceNode,
    allow_children: bool,
    environment: &EnvironmentConfig,
    types: &TypeRegistry,
) -> DoweResult<ViewNode> {
    export_tree_with_stores(node, allow_children, environment, types, &[])
}

fn export_tree_with_stores(
    node: &SourceNode,
    allow_children: bool,
    environment: &EnvironmentConfig,
    types: &TypeRegistry,
    stores: &[ImportedViewStore],
) -> DoweResult<ViewNode> {
    for store in stores {
        if !node_uses_reference(node, &store.name) {
            return Err(DoweError::at_path(
                &node.location.path,
                format!(
                    "View Store import `{}` is not used by the view module",
                    store.name
                ),
            ));
        }
    }
    let mut tree = lower_export_tree_with_stores(node, allow_children, types, stores)?;
    validate_view_tree(&tree).map_err(|error| node_error(node, error.to_string()))?;
    validate_reactive_view_tree(&node.location.path, &tree, environment)?;
    resolve_dynamic_icon_fallbacks(&mut tree);
    Ok(tree)
}

fn node_uses_reference(node: &SourceNode, name: &str) -> bool {
    value_uses_reference(&SourceValue::Bareword(node.name.clone()), name)
        || node
            .args
            .iter()
            .any(|value| value_uses_reference(value, name))
        || node
            .props
            .iter()
            .any(|prop| value_uses_reference(&prop.value, name))
        || node
            .children
            .iter()
            .any(|child| node_uses_reference(child, name))
}

