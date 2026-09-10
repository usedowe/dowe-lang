fn ios_node_references_layout_bindings(node: &ViewNode, bindings: &IosLayoutBindings) -> bool {
    ios_structure_node_references_layout_bindings(node, bindings)
        || ios_media_node_references_layout_bindings(node, bindings)
        || ios_data_node_references_layout_bindings(node, bindings)
        || ios_navigation_node_references_layout_bindings(node, bindings)
        || ios_rich_node_references_layout_bindings(node, bindings)
}
