include!("node_references_structure_and_media.rs");
include!("node_references_forms_and_data.rs");
include!("node_references_navigation_shell.rs");
include!("node_references_overlays_and_collections.rs");

fn dev_node_references_layout_bindings(node: &ViewNode, bindings: &DevLayoutBindings) -> bool {
    if let Some(value) = dev_node_references_structure_and_media(node, bindings) {
        return value;
    }
    if let Some(value) = dev_node_references_forms_and_data(node, bindings) {
        return value;
    }
    if let Some(value) = dev_node_references_navigation_shell(node, bindings) {
        return value;
    }
    if let Some(value) = dev_node_references_overlays_and_collections(node, bindings) {
        return value;
    }
    unreachable!("all ViewNode variants are covered by layout reference dispatchers");
}
