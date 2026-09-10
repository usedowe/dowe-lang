fn collect_media_form_node_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    collect_media_node_classes(node, classes);
    collect_form_node_classes(node, classes);
    collect_special_node_classes(node, classes);
}
