#[test]
fn applies_footer_padding_defaults_and_preserves_overrides() {
    let default_footer = bar_component_node(
        BuiltinComponent::Footer,
        Vec::new(),
        vec![text_node("Directory").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
        false,
    )
    .expect("default footer");

    let ViewNode::Footer { props, .. } = default_footer else {
        panic!("footer");
    };
    let horizontal = props.style.style.spacing.px.expect("default px");
    assert_eq!(horizontal.entries.len(), 2);
    assert_eq!(horizontal.entries[0].breakpoint, Breakpoint::Xs);
    assert_eq!(horizontal.entries[0].value, ScaleValue::from_half_steps(8));
    assert_eq!(horizontal.entries[1].breakpoint, Breakpoint::Md);
    assert_eq!(horizontal.entries[1].value, ScaleValue::from_half_steps(12));
    let top = props.style.style.spacing.pt.expect("default pt");
    assert_eq!(top.entries.len(), 2);
    assert_eq!(top.entries[0].breakpoint, Breakpoint::Xs);
    assert_eq!(top.entries[0].value, ScaleValue::from_half_steps(20));
    assert_eq!(top.entries[1].breakpoint, Breakpoint::Md);
    assert_eq!(top.entries[1].value, ScaleValue::from_half_steps(32));
    let bottom = props.style.style.spacing.pb.expect("default pb");
    assert_eq!(bottom.entries.len(), 2);
    assert_eq!(bottom.entries[0].breakpoint, Breakpoint::Xs);
    assert_eq!(bottom.entries[0].value, ScaleValue::from_half_steps(8));
    assert_eq!(bottom.entries[1].breakpoint, Breakpoint::Md);
    assert_eq!(bottom.entries[1].value, ScaleValue::from_half_steps(12));

    let authored_footer = bar_component_node(
        BuiltinComponent::Footer,
        vec![number_prop("px", 2), number_prop("py", 3)],
        vec![text_node("Directory").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
        false,
    )
    .expect("authored footer");

    let ViewNode::Footer { props, .. } = authored_footer else {
        panic!("footer");
    };
    let horizontal = props.style.style.spacing.px.expect("authored px");
    assert_eq!(horizontal.entries.len(), 1);
    assert_eq!(horizontal.entries[0].value, ScaleValue::from_half_steps(4));
    let vertical = props.style.style.spacing.py.expect("authored py");
    assert_eq!(vertical.entries.len(), 1);
    assert_eq!(vertical.entries[0].value, ScaleValue::from_half_steps(6));
    assert!(props.style.style.spacing.pt.is_none());
    assert!(props.style.style.spacing.pb.is_none());
}

