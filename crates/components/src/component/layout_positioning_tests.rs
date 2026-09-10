#[test]
fn validates_relative_absolute_and_fixed_box_positioning() {
    let tree = container_component_node(
        BuiltinComponent::Box,
        vec![string_prop("position", "relative")],
        vec![
            container_component_node(
                BuiltinComponent::Box,
                vec![
                    string_prop("position", "absolute"),
                    number_prop("top", 4),
                    number_prop("right", 6),
                ],
                vec![text_node("Proof").expect("text")],
                false,
            )
            .expect("absolute box"),
        ],
        false,
    )
    .expect("relative box");

    validate_view_tree(&tree).expect("valid positioned tree");
    let ViewNode::Box { props, children } = &tree else {
        panic!("box");
    };
    assert_eq!(props.position().mode, BoxPosition::Relative);
    let ViewNode::Box { props, .. } = &children[0] else {
        panic!("absolute box");
    };
    assert_eq!(props.position().mode, BoxPosition::Absolute);
    assert_eq!(
        props.position().top.as_ref().expect("top").entries[0]
            .value
            .native_units(),
        16
    );
    assert_eq!(
        props.position().right.as_ref().expect("right").entries[0]
            .value
            .native_units(),
        24
    );

    let fixed = container_component_node(
        BuiltinComponent::Box,
        vec![
            string_prop("position", "fixed"),
            number_prop("bottom", 4),
            number_prop("right", 4),
        ],
        vec![text_node("Persistent").expect("text")],
        false,
    )
    .expect("fixed box");
    validate_view_tree(&fixed).expect("valid fixed box");
    assert_eq!(fixed_box_nodes(&fixed).len(), 1);
}

