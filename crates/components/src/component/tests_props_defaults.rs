#[test]
fn recognizes_complete_braced_text_bindings_only() {
    assert_eq!(text_binding_path("{blog.title}"), Some("blog.title"));
    assert_eq!(text_binding_path("blog.title"), None);
    assert_eq!(text_binding_path("Hello {blog.title}"), None);
    assert_eq!(text_binding_path("{}"), None);
    assert_eq!(text_binding_path("{blog..title}"), None);
}

#[test]
fn normalizes_accordion_single_open_defaults() {
    let default_open_items = || {
        ["first", "second"]
            .into_iter()
            .map(|id| {
                super::accordion_item_component(
                    vec![
                        string_prop("id", id),
                        string_prop("label", id),
                        boolean_prop("defaultOpen", true),
                    ],
                    vec![text_node(id).expect("text")],
                )
                .expect("accordion item")
            })
            .collect::<Vec<_>>()
    };
    let node =
        super::accordion_component_node(Vec::new(), default_open_items()).expect("accordion");

    match node {
        ViewNode::Accordion { props, items } => {
            assert!(!props.multiple);
            assert!(items[0].default_open);
            assert!(!items[1].default_open);
        }
        _ => panic!("accordion"),
    }

    let node =
        super::accordion_component_node(vec![boolean_prop("multiple", true)], default_open_items())
            .expect("multiple accordion");
    let ViewNode::Accordion { props, items } = node else {
        panic!("accordion");
    };
    assert!(props.multiple);
    assert!(items.iter().all(|item| item.default_open));
}

#[test]
fn hides_image_controls_by_default_and_allows_explicit_enablement() {
    let default_node = super::image_component_node(vec![string_prop("src", "/assets/photo.jpg")])
        .expect("default image");
    let ViewNode::Image { props } = default_node else {
        panic!("image");
    };
    assert!(props.hide_controls);

    let enabled_node = super::image_component_node(vec![
        string_prop("src", "/assets/photo.jpg"),
        boolean_prop("hideControls", false),
    ])
    .expect("enabled image");
    let ViewNode::Image { props } = enabled_node else {
        panic!("image");
    };
    assert!(!props.hide_controls);
}

#[test]
fn exposes_the_normative_component_visual_defaults() {
    let defaults = super::DesignDefaults::with_builtin_defaults();
    let expected = [
        (
            super::DesignComponentSlot::Button,
            Some(super::ColorFamily::Primary),
            Some(super::ComponentVariant::Solid),
            Some(super::RoundedSize::Md),
        ),
        (
            super::DesignComponentSlot::IconButton,
            Some(super::ColorFamily::Primary),
            Some(super::ComponentVariant::Solid),
            Some(super::RoundedSize::Md),
        ),
        (
            super::DesignComponentSlot::Card,
            Some(super::ColorFamily::Surface),
            Some(super::ComponentVariant::Solid),
            Some(super::RoundedSize::Md),
        ),
        (
            super::DesignComponentSlot::Drawer,
            Some(super::ColorFamily::Surface),
            Some(super::ComponentVariant::Solid),
            None,
        ),
        (
            super::DesignComponentSlot::Toast,
            Some(super::ColorFamily::Info),
            Some(super::ComponentVariant::Solid),
            Some(super::RoundedSize::Md),
        ),
        (
            super::DesignComponentSlot::Accordion,
            None,
            Some(super::ComponentVariant::Ghost),
            None,
        ),
        (
            super::DesignComponentSlot::Checkbox,
            Some(super::ColorFamily::Primary),
            None,
            None,
        ),
        (
            super::DesignComponentSlot::Input,
            Some(super::ColorFamily::Primary),
            Some(super::ComponentVariant::Outlined),
            None,
        ),
        (
            super::DesignComponentSlot::Date,
            Some(super::ColorFamily::Primary),
            Some(super::ComponentVariant::Outlined),
            None,
        ),
        (
            super::DesignComponentSlot::Password,
            Some(super::ColorFamily::Primary),
            Some(super::ComponentVariant::Outlined),
            None,
        ),
        (
            super::DesignComponentSlot::Select,
            Some(super::ColorFamily::Primary),
            Some(super::ComponentVariant::Outlined),
            None,
        ),
        (
            super::DesignComponentSlot::Pin,
            Some(super::ColorFamily::Primary),
            Some(super::ComponentVariant::Outlined),
            None,
        ),
        (
            super::DesignComponentSlot::AppBar,
            Some(super::ColorFamily::Surface),
            Some(super::ComponentVariant::Solid),
            None,
        ),
        (
            super::DesignComponentSlot::Footer,
            Some(super::ColorFamily::Surface),
            Some(super::ComponentVariant::Solid),
            None,
        ),
        (
            super::DesignComponentSlot::Modal,
            Some(super::ColorFamily::Surface),
            Some(super::ComponentVariant::Solid),
            None,
        ),
        (
            super::DesignComponentSlot::Dropdown,
            Some(super::ColorFamily::Surface),
            Some(super::ComponentVariant::Solid),
            None,
        ),
        (
            super::DesignComponentSlot::Tooltip,
            Some(super::ColorFamily::Surface),
            Some(super::ComponentVariant::Solid),
            None,
        ),
    ];

    for (slot, scheme, variant, rounded) in expected {
        assert_eq!(defaults.scheme.get(&slot).copied(), scheme);
        assert_eq!(defaults.variant.get(&slot).copied(), variant);
        assert_eq!(defaults.radius.get(&slot).copied(), rounded);
    }
    assert_eq!(
        defaults.scheme.get(&super::DesignComponentSlot::Tabs),
        Some(&super::ColorFamily::Primary)
    );
    assert_eq!(
        defaults.tabs_variant.get(&super::DesignComponentSlot::Tabs),
        Some(&super::TabsVariant::Pills)
    );
    assert!(defaults.border.is_empty());
    assert!(defaults.shadow.is_empty());

    let mut section =
        container_component_node(BuiltinComponent::Section, Vec::new(), Vec::new(), true)
            .expect("section");
    super::apply_design_defaults_to_tree(&mut section, &defaults);
    let ViewNode::Section { props, .. } = section else {
        panic!("section");
    };
    assert_eq!(
        props.bg.expect("section scheme").entries[0].value,
        super::ColorToken::Background
    );
}

#[test]
fn resolves_function_toast_variant_with_design_precedence() {
    let toast = |variant: Option<&str>| {
        super::ViewFunctionStatement::Toast(super::ViewToastAction {
            kind: "success".to_string(),
            title: "Saved".to_string(),
            message: "Changes saved".to_string(),
            duration: None,
            scheme: None,
            variant: variant.map(str::to_string),
            position: None,
        })
    };
    let action = |variant| super::ViewAction {
        id: "save".to_string(),
        name: "save".to_string(),
        params: Vec::new(),
        return_type: None,
        kind: super::ViewActionKind::Sequence(vec![super::ViewFunctionStatement::If {
            result: "request".to_string(),
            success: vec![toast(variant)],
            error: vec![toast(Some("outlined"))],
        }]),
    };

    let mut built_in = vec![action(None)];
    super::apply_design_defaults_to_actions(
        &mut built_in,
        &super::DesignDefaults::with_builtin_defaults(),
    );
    let super::ViewActionKind::Sequence(statements) = &built_in[0].kind else {
        panic!("action sequence");
    };
    let super::ViewFunctionStatement::If { success, error, .. } = &statements[0] else {
        panic!("if statement");
    };
    let super::ViewFunctionStatement::Toast(toast) = &success[0] else {
        panic!("success toast");
    };
    assert_eq!(toast.variant.as_deref(), Some("solid"));
    let super::ViewFunctionStatement::Toast(toast) = &error[0] else {
        panic!("error toast");
    };
    assert_eq!(toast.variant.as_deref(), Some("outlined"));

    let mut themed_defaults = super::DesignDefaults::with_builtin_defaults();
    themed_defaults.variant.insert(
        super::DesignComponentSlot::Toast,
        super::ComponentVariant::Soft,
    );
    let mut themed = vec![action(None)];
    super::apply_design_defaults_to_actions(&mut themed, &themed_defaults);
    let super::ViewActionKind::Sequence(statements) = &themed[0].kind else {
        panic!("action sequence");
    };
    let super::ViewFunctionStatement::If { success, .. } = &statements[0] else {
        panic!("if statement");
    };
    let super::ViewFunctionStatement::Toast(toast) = &success[0] else {
        panic!("themed toast");
    };
    assert_eq!(toast.variant.as_deref(), Some("soft"));

    let mut explicit = vec![action(Some("ghost"))];
    super::apply_design_defaults_to_actions(&mut explicit, &themed_defaults);
    let super::ViewActionKind::Sequence(statements) = &explicit[0].kind else {
        panic!("action sequence");
    };
    let super::ViewFunctionStatement::If { success, .. } = &statements[0] else {
        panic!("if statement");
    };
    let super::ViewFunctionStatement::Toast(toast) = &success[0] else {
        panic!("explicit toast");
    };
    assert_eq!(toast.variant.as_deref(), Some("ghost"));
}
