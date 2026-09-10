#[test]
fn rejects_invalid_box_positioning_contracts() {
    let static_offset_error = container_component_node(
        BuiltinComponent::Box,
        vec![number_prop("top", 4)],
        Vec::new(),
        false,
    )
    .expect_err("static offset");
    assert!(
        static_offset_error
            .to_string()
            .contains("require `position:\"absolute\"` or `position:\"fixed\"`"),
        "{static_offset_error}"
    );
    assert!(
        container_component_node(
            BuiltinComponent::Box,
            vec![
                string_prop("position", "absolute"),
                number_prop("left", 2),
                number_prop("right", 2),
            ],
            Vec::new(),
            false,
        )
        .expect_err("ambiguous horizontal axis")
        .to_string()
        .contains("`left` and `right`")
    );

    let orphan = container_component_node(
        BuiltinComponent::Box,
        vec![string_prop("position", "absolute")],
        Vec::new(),
        false,
    )
    .expect("absolute box");
    assert!(
        validate_view_tree(&orphan)
            .expect_err("orphan absolute box")
            .to_string()
            .contains("direct child of `Box position:\"relative\"`")
    );

    let fixed_in_each = ViewNode::Each {
        item: "item".to_string(),
        collection: "items".to_string(),
        key: "item.id".to_string(),
        children: vec![
            container_component_node(
                BuiltinComponent::Box,
                vec![string_prop("position", "fixed")],
                Vec::new(),
                false,
            )
            .expect("fixed box"),
        ],
    };
    assert!(
        validate_view_tree(&fixed_in_each)
            .expect_err("fixed inside each")
            .to_string()
            .contains("cannot be nested inside `each` or `Splash`")
    );
}

