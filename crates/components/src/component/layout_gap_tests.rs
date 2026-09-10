#[test]
fn parses_section_center_as_static_and_responsive_boolean() {
    let default = container_component_node(
        BuiltinComponent::Section,
        Vec::new(),
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("default section");
    let ViewNode::Section { props, .. } = default else {
        panic!("section");
    };
    assert!(props.center_x.is_none());

    let centered = container_component_node(
        BuiltinComponent::Section,
        vec![boolean_prop("centerX", true)],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("centered section");
    let ViewNode::Section { props, .. } = centered else {
        panic!("section");
    };
    assert_eq!(
        props.center_x.as_ref().expect("center").entries[0].value,
        true
    );

    let responsive = container_component_node(
        BuiltinComponent::Section,
        vec![responsive_boolean_prop(
            "centerX",
            &[("xs", false), ("md", true)],
        )],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("responsive section");
    let ViewNode::Section { props, .. } = responsive else {
        panic!("section");
    };
    assert_eq!(props.center_x.as_ref().expect("center").entries.len(), 2);
    assert_eq!(
        props.center_x.as_ref().expect("center").entries[1].value,
        true
    );
}

