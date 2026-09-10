include!("validate_node_structure_and_content.rs");
include!("validate_node_data_and_media.rs");
include!("validate_node_navigation_and_overlays.rs");

fn resolve_each_collection(
    collection: &str,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
) -> Option<ViewSignalValue> {
    let mut value = signals
        .get(collection.split('.').next()?)
        .cloned()
        .or_else(|| locals.get(collection.split('.').next()?).and_then(Clone::clone))?;
    for field in collection.split('.').skip(1) {
        let ViewSignalValue::Object(fields) = value else {
            return None;
        };
        value = fields
            .into_iter()
            .find_map(|(name, field_value)| (name == field).then_some(field_value))?;
    }
    Some(value)
}

fn validate_node_variant_references(
    path: &Path,
    node: &ViewNode,
    signals: &HashMap<String, ViewSignalValue>,
    writable_signals: &HashSet<String>,
    actions: &HashSet<String>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
) -> DoweResult<()> {
    if let Some(result) = validate_node_structure_and_content(
        path, node, signals, writable_signals, actions, locals,
    ) {
        return result;
    }
    if let Some(result) = validate_node_data_and_media(
        path, node, signals, writable_signals, actions, locals,
    ) {
        return result;
    }
    if let Some(result) = validate_node_navigation_and_overlays(
        path, node, signals, writable_signals, actions, locals,
    ) {
        return result;
    }
    for group in node_child_groups(node) {
        for child in group {
            validate_node_references(path, child, signals, writable_signals, actions, locals)?;
        }
    }
    Ok(())
}
