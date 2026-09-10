#[test]
fn parses_section_gap_with_zero_default_and_responsive_values() {
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
    assert!(props.gap.is_none());

    let scalar = container_component_node(
        BuiltinComponent::Section,
        vec![number_prop("gap", 3)],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("scalar gap");
    let ViewNode::Section { props, .. } = scalar else {
        panic!("section");
    };
    assert_eq!(
        props.gap.expect("gap").entries[0].value,
        GapValue::Single(GapSize::Scale(ScaleValue(6)))
    );

    let pixels = container_component_node(
        BuiltinComponent::Section,
        vec![string_prop("gap", "8px")],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("pixel gap");
    let ViewNode::Section { props, .. } = pixels else {
        panic!("section");
    };
    assert_eq!(
        props.gap.expect("gap").entries[0].value,
        GapValue::Single(GapSize::Px(8))
    );

    let responsive = container_component_node(
        BuiltinComponent::Section,
        vec![responsive_number_prop("gap", &[("xs", 2), ("md", 4)])],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("responsive gap");
    let ViewNode::Section { props, .. } = responsive else {
        panic!("section");
    };
    let gap = props.gap.expect("gap");
    assert_eq!(gap.entries.len(), 2);
    assert_eq!(
        gap.entries[1].value,
        GapValue::Single(GapSize::Scale(ScaleValue(8)))
    );
}

