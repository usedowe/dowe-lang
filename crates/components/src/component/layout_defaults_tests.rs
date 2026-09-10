#[test]
fn validates_layout_bar_props_and_regions() {
    let node = bar_component_node(
        BuiltinComponent::AppBar,
        vec![
            string_prop("variant", "ghost"),
            string_prop("scheme", "surface"),
            boolean_prop("bordered", true),
            boolean_prop("blurred", true),
            boolean_prop("boxed", true),
            boolean_prop("floating", true),
            string_prop("position", "fixed"),
            boolean_prop("dockOnScroll", true),
        ],
        Vec::new(),
        vec![text_node("Menu").expect("text")],
        vec![text_node("Brand").expect("text")],
        vec![children_node(true).expect("children")],
        Vec::new(),
        None,
        true,
    )
    .expect("appbar");

    match node {
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
            top,
            bottom,
            ..
        } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
            assert!(props.bordered);
            assert!(props.blurred);
            assert!(props.boxed);
            assert!(props.floating);
            assert_eq!(props.position, BarPosition::Fixed);
            assert!(props.dock_on_scroll);
            assert_eq!(start.len(), 1);
            assert_eq!(center.len(), 1);
            assert_eq!(end, vec![ViewNode::Children]);
            assert!(top.is_empty());
            assert!(bottom.is_empty());
        }
        _ => panic!("appbar"),
    }

    let footer = bar_component_node(
        BuiltinComponent::Footer,
        vec![boolean_prop("boxed", true)],
        vec![text_node("Directory").expect("text")],
        Vec::new(),
        vec![text_node("Navigation").expect("text")],
        Vec::new(),
        vec![text_node("Legal").expect("text")],
        None,
        false,
    )
    .expect("footer");

    let ViewNode::Footer {
        props,
        top,
        center,
        bottom,
        ..
    } = footer
    else {
        panic!("footer");
    };
    assert!(props.boxed);
    assert_eq!(top.len(), 1);
    assert_eq!(center.len(), 1);
    assert_eq!(bottom.len(), 1);

    let error = bar_component_node(
        BuiltinComponent::Footer,
        vec![boolean_prop("floating", true)],
        Vec::new(),
        vec![text_node("Footer").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
        false,
    )
    .expect_err("footer floating");

    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Footer, "floating")
    );

    let error = bar_component_node(
        BuiltinComponent::AppBar,
        vec![string_prop("position", "absolute")],
        Vec::new(),
        vec![text_node("Menu").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
        false,
    )
    .expect_err("appbar position");
    assert_eq!(
        error,
        ComponentError::invalid_prop("position", "static, sticky or fixed")
    );

    let error = bar_component_node(
        BuiltinComponent::BottomBar,
        vec![string_prop("position", "fixed")],
        Vec::new(),
        vec![text_node("Menu").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
        false,
    )
    .expect_err("bottom bar position");
    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::BottomBar, "position")
    );

    let error = bar_component_node(
        BuiltinComponent::AppBar,
        vec![boolean_prop("dockOnScroll", true)],
        Vec::new(),
        vec![text_node("Menu").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
        false,
    )
    .expect_err("dock without fixed floating AppBar");
    assert_eq!(
        error,
        ComponentError::invalid_prop_combination(
            "`dockOnScroll:true` requires `floating:true` and `position:\"fixed\"` on `AppBar`"
        )
    );

    let error = bar_component_node(
        BuiltinComponent::Footer,
        vec![boolean_prop("dockOnScroll", true)],
        Vec::new(),
        vec![text_node("Footer").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
        false,
    )
    .expect_err("footer dock on scroll");
    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Footer, "dockOnScroll")
    );
}

