#[test]
fn rejects_grid_spans_outside_direct_grid_children() {
    let tree = container_component_node(
        BuiltinComponent::Box,
        Vec::new(),
        vec![
            container_component_node(
                BuiltinComponent::Box,
                vec![number_prop("colSpan", 2)],
                vec![text_node("Wide").expect("text")],
                false,
            )
            .expect("box"),
        ],
        false,
    )
    .expect("tree");

    assert!(validate_view_tree(&tree).is_err());
}

