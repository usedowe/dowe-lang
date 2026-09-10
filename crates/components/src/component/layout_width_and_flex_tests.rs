#[test]
fn parses_flex_item_values_on_layout_components() {
    let box_node = container_component_node(
        BuiltinComponent::Box,
        vec![ComponentProp {
            name: "flex".to_string(),
            value: PropValue::Responsive(vec![
                ResponsivePropEntry {
                    breakpoint: "xs".to_string(),
                    value: super::PropScalar::Number("1".to_string()),
                },
                ResponsivePropEntry {
                    breakpoint: "md".to_string(),
                    value: super::PropScalar::String("none".to_string()),
                },
            ]),
        }],
        Vec::new(),
        false,
    )
    .expect("box flex");
    let ViewNode::Box { props, .. } = box_node else {
        panic!("box");
    };
    let flex = props.flex.expect("flex");
    assert_eq!(flex.entries[0].value, FlexItem::Fill);
    assert_eq!(flex.entries[1].breakpoint, Breakpoint::Md);
    assert_eq!(flex.entries[1].value, FlexItem::None);

    for (component, value, expected) in [
        (BuiltinComponent::Section, "initial", FlexItem::Initial),
        (BuiltinComponent::Flex, "auto", FlexItem::Auto),
        (BuiltinComponent::Grid, "none", FlexItem::None),
        (BuiltinComponent::Card, "auto", FlexItem::Auto),
    ] {
        let node = container_component_node(
            component,
            vec![string_prop("flex", value)],
            Vec::new(),
            false,
        )
        .expect("flex item component");
        let flex = match node {
            ViewNode::Section { props, .. } => props.flex,
            ViewNode::Flex { props, .. } => props.style.flex,
            ViewNode::Grid { props, .. } => props.style.flex,
            ViewNode::Card { props, .. } => props.style.flex,
            _ => panic!("layout component"),
        }
        .expect("flex");
        assert_eq!(flex.entries[0].value, expected);
    }

    assert_eq!(
        container_component_node(
            BuiltinComponent::Box,
            vec![number_prop("flex", 2)],
            Vec::new(),
            false,
        )
        .expect_err("invalid flex item"),
        ComponentError::invalid_prop("flex", "initial, auto, none or 1")
    );
}

