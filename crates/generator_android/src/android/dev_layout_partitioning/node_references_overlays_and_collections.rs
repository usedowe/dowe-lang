fn dev_node_references_overlays_and_collections(
    node: &ViewNode,
    bindings: &DevLayoutBindings,
) -> Option<bool> {
    match node {
        ViewNode::NavMenu { props, items } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_nav_menu_items_reference_layout_bindings(items, bindings)
        }),
        ViewNode::Each {
            children,
            collection,
            ..
        } => Some({
            bindings.references_signal(collection)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::Avatar { props, .. } => Some(dev_variant_references_layout_bindings(&props.style, bindings)),
        ViewNode::Children => Some(false),
        _ => None,
    }
}
