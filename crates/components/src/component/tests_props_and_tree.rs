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

#[test]
fn validates_button_visual_props() {
    let mut node = container_component_node(
        BuiltinComponent::Button,
        vec![
            string_prop("variant", "ghost"),
            string_prop("scheme", "warning"),
            string_prop("size", "xs"),
            string_prop("rounded", "full"),
        ],
        vec![text_node("Save").expect("text")],
        false,
    )
    .expect("button");
    super::apply_design_defaults_to_tree(
        &mut node,
        &super::DesignDefaults::with_builtin_defaults(),
    );

    match node {
        ViewNode::Button { props, .. } => {
            assert_eq!(props.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.color, Some(ColorFamily::Warning));
            assert_eq!(props.size, Some(ButtonSize::Xs));
            assert!(props.style.rounded.is_some());
            assert_eq!(
                props.style.spacing.px.expect("px").entries[0].value,
                ScaleValue::from_half_steps(5)
            );
            assert_eq!(
                props.style.spacing.py.expect("py").entries[0].value,
                ScaleValue::from_half_steps(3)
            );
        }
        _ => panic!("button"),
    }

    let error = container_component_node(
        BuiltinComponent::Button,
        vec![string_prop("size", "2xl")],
        vec![text_node("Save").expect("text")],
        false,
    )
    .expect_err("size error");
    assert_eq!(
        error,
        ComponentError::invalid_prop("size", "xs, sm, md, lg or xl")
    );

    let error = container_component_node(
        BuiltinComponent::Button,
        vec![string_prop("rounded", "circle")],
        vec![text_node("Save").expect("text")],
        false,
    )
    .expect_err("rounded error");
    assert_eq!(
        error,
        ComponentError::invalid_prop("rounded", "xs, sm, md, lg, xl or full")
    );
}

#[test]
fn validates_single_line_form_control_sizes() {
    let input = input_node(vec![
        string_prop("size", "sm"),
        boolean_prop("labelFloating", true),
    ])
    .expect("small floating input");
    let ViewNode::Input { props } = input else {
        panic!("input");
    };
    assert_eq!(props.size, Some(ButtonSize::Sm));
    assert_eq!(
        form_control_min_height(ButtonSize::Sm, false).native_units(),
        32
    );
    assert_eq!(
        form_control_min_height(ButtonSize::Sm, true).native_units(),
        40
    );
    assert_eq!(form_control_text_size(ButtonSize::Sm), TextSize::Sm);

    let select = select_node(
        vec![string_prop("size", "lg")],
        vec![super::SelectOption {
            value: "admin".to_string(),
            label: "Admin".to_string(),
            description: None,
        }],
    )
    .expect("large select");
    let ViewNode::Select { props, .. } = select else {
        panic!("select");
    };
    assert_eq!(props.size, Some(ButtonSize::Lg));
    assert_eq!(
        form_control_min_height(ButtonSize::Md, false).native_units(),
        40
    );
    assert_eq!(
        form_control_min_height(ButtonSize::Md, true).native_units(),
        48
    );
    assert_eq!(form_control_text_size(ButtonSize::Md), TextSize::Md);
    assert_eq!(
        form_control_min_height(ButtonSize::Lg, false).native_units(),
        48
    );
    assert_eq!(
        form_control_min_height(ButtonSize::Lg, true).native_units(),
        56
    );
    assert_eq!(form_control_text_size(ButtonSize::Lg), TextSize::Lg);

    let error = input_node(vec![string_prop("size", "xl")]).expect_err("size error");
    assert_eq!(error, ComponentError::invalid_prop("size", "sm, md or lg"));
}

#[test]
fn validates_brand_children_navigation_and_size() {
    let node = container_component_node(
        BuiltinComponent::Brand,
        vec![
            string_prop("href", "/"),
            string_prop("label", "Dowe home"),
            responsive_number_prop("w", &[("xs", 24), ("md", 32)]),
            number_prop("h", 8),
        ],
        vec![text_node("Dowe").expect("text")],
        false,
    )
    .expect("brand");

    match node {
        ViewNode::Brand { props, children } => {
            assert_eq!(props.label.as_deref(), Some("Dowe home"));
            assert!(matches!(
                props.navigation,
                Some(NavigationAction::Internal { ref path, .. }) if path == "/"
            ));
            assert_eq!(children.len(), 1);
            assert_eq!(
                props.style.sizing.w.expect("width").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(48))
            );
            assert_eq!(
                props.style.sizing.h.expect("height").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(16))
            );
        }
        _ => panic!("brand"),
    }

    let static_brand = container_component_node(
        BuiltinComponent::Brand,
        Vec::new(),
        vec![text_node("Dowe").expect("text")],
        false,
    )
    .expect("static brand");
    assert!(matches!(
        static_brand,
        ViewNode::Brand {
            props: BrandProps {
                navigation: None,
                ..
            },
            ..
        }
    ));

    let external_brand = container_component_node(
        BuiltinComponent::Brand,
        vec![string_prop("href", "https://dowe.dev")],
        vec![text_node("Dowe").expect("text")],
        false,
    )
    .expect("external brand");
    assert!(matches!(
        external_brand,
        ViewNode::Brand {
            props: BrandProps {
                navigation: Some(NavigationAction::External { .. }),
                ..
            },
            ..
        }
    ));

    assert!(
        container_component_node(BuiltinComponent::Brand, Vec::new(), Vec::new(), false).is_err()
    );
    assert!(
        container_component_node(
            BuiltinComponent::Brand,
            vec![string_prop("label", "")],
            vec![text_node("Dowe").expect("text")],
            false,
        )
        .is_err()
    );
    assert!(
        container_component_node(
            BuiltinComponent::Brand,
            vec![string_prop("href", "javascript:alert(1)")],
            vec![text_node("Dowe").expect("text")],
            false,
        )
        .is_err()
    );
    assert!(
        container_component_node(
            BuiltinComponent::Brand,
            vec![string_prop("variant", "solid")],
            vec![text_node("Dowe").expect("text")],
            false,
        )
        .is_err()
    );

    let brand_only_height = container_component_node(
        BuiltinComponent::Brand,
        vec![number_prop("h", 8)],
        vec![text_node("Dowe").expect("text")],
        false,
    )
    .expect("brand with automatic width");
    let ViewNode::Brand { props, .. } = brand_only_height else {
        panic!("brand with automatic width");
    };
    assert!(props.style.sizing.w.is_none());
    assert!(props.style.sizing.h.is_some());

    let brand_only_width = container_component_node(
        BuiltinComponent::Brand,
        vec![number_prop("w", 16)],
        vec![text_node("Dowe").expect("text")],
        false,
    )
    .expect("brand with automatic height");
    let ViewNode::Brand { props, .. } = brand_only_width else {
        panic!("brand with automatic height");
    };
    assert!(props.style.sizing.w.is_some());
    assert!(props.style.sizing.h.is_none());
}

#[test]
fn validates_banner_children_and_required_external_navigation() {
    let node = container_component_node(
        BuiltinComponent::Banner,
        vec![
            string_prop("href", "https://dowe.dev/cloud"),
            string_prop("label", "Explore Dowe Cloud"),
            responsive_number_prop("w", &[("xs", 24), ("md", 32)]),
            number_prop("h", 16),
        ],
        vec![text_node("Dowe Cloud").expect("text")],
        false,
    )
    .expect("banner");

    match node {
        ViewNode::Banner { props, children } => {
            assert_eq!(props.label.as_deref(), Some("Explore Dowe Cloud"));
            assert!(matches!(
                props.navigation,
                NavigationAction::External {
                    ref url,
                    web_target: WebTarget::Blank,
                    native_external_mode: NativeExternalMode::System,
                } if url == "https://dowe.dev/cloud"
            ));
            assert_eq!(children.len(), 1);
            assert_eq!(
                props.style.sizing.w.expect("width").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(48))
            );
            assert_eq!(
                props.style.sizing.h.expect("height").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(32))
            );
        }
        _ => panic!("banner"),
    }

    for props in [
        Vec::new(),
        vec![string_prop("href", "/pricing")],
        vec![string_prop("href", "#pricing")],
        vec![string_prop("href", "http://dowe.dev")],
        vec![string_prop("href", "javascript:alert(1)")],
        vec![
            string_prop("href", "https://dowe.dev"),
            string_prop("label", ""),
        ],
        vec![
            string_prop("href", "https://dowe.dev"),
            string_prop("variant", "solid"),
        ],
    ] {
        assert!(
            container_component_node(
                BuiltinComponent::Banner,
                props,
                vec![text_node("Dowe").expect("text")],
                false,
            )
            .is_err()
        );
    }
    assert!(
        container_component_node(
            BuiltinComponent::Banner,
            vec![string_prop("href", "https://dowe.dev")],
            Vec::new(),
            false,
        )
        .is_err()
    );
}

#[test]
fn validates_interactive_motion_props() {
    let box_node = container_component_node(
        BuiltinComponent::Box,
        vec![
            string_prop("animation", "fadeIn"),
            number_prop("rotate", -7),
            number_string_prop("scale", "1.05"),
            number_string_prop("translateX", "-1.5"),
            responsive_number_prop("translateY", &[("xs", 0), ("md", 2)]),
            string_prop("transition", "spring"),
            string_prop("gesture", "lift"),
            string_prop("onClick", "selectMobile"),
        ],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("box");
    match box_node {
        ViewNode::Box { props, .. } => {
            assert_eq!(props.animation(), Some(ViewAnimation::FadeIn));
            let motion = props.motion();
            assert_eq!(
                motion.rotate.as_ref().expect("rotate").entries[0].value,
                ViewRotation(-7)
            );
            assert_eq!(
                motion.scale.as_ref().expect("scale").entries[0].value,
                ViewScale(105)
            );
            assert_eq!(
                motion.translate_x.as_ref().expect("translate x").entries[0].value,
                ViewTranslation(-3)
            );
            assert_eq!(motion.transition, Some(ViewTransition::Spring));
            assert_eq!(motion.gesture, Some(ViewGesture::Lift));
            assert_eq!(props.element.on_click.as_deref(), Some("selectMobile"));
        }
        _ => panic!("box"),
    }

    let card_node = container_component_node(
        BuiltinComponent::Card,
        vec![string_prop("animation", "slideUp")],
        vec![text_node("Card").expect("text")],
        false,
    )
    .expect("card");
    match card_node {
        ViewNode::Card { props, .. } => {
            assert_eq!(props.style.animation(), Some(ViewAnimation::SlideUp));
        }
        _ => panic!("card"),
    }

    let error = container_component_node(
        BuiltinComponent::Box,
        vec![string_prop("animation", "bounce")],
        Vec::new(),
        false,
    )
    .expect_err("animation");
    assert_eq!(
        error,
        ComponentError::invalid_prop(
            "animation",
            "none, fadeIn, slideUp, slideDown, slideLeft, slideRight or scaleIn"
        )
    );

    let flex = container_component_node(
        BuiltinComponent::Flex,
        vec![string_prop("animation", "fadeIn")],
        Vec::new(),
        false,
    )
    .expect("flex");
    let ViewNode::Flex { props, .. } = flex else {
        panic!("flex");
    };
    assert_eq!(props.style.animation(), Some(ViewAnimation::FadeIn));

    for prop in [
        number_prop("rotate", 181),
        number_string_prop("scale", "2.01"),
        number_string_prop("translateX", "1.25"),
        string_prop("transition", "elastic"),
        string_prop("gesture", "swipe"),
    ] {
        assert!(
            container_component_node(BuiltinComponent::Chip, vec![prop], Vec::new(), false)
                .is_err()
        );
    }
}

#[test]
fn validates_text_title_typography_props() {
    let node = text_component_node(
        BuiltinComponent::Title,
        vec![
            string_prop("size", "4xl"),
            string_prop("align", "center"),
            string_prop("color", "primary"),
            string_prop("bg", "softPrimary"),
            string_prop("weight", "black"),
            string_prop("spacing", "tight"),
            string_prop("i18n", "home.hero.title"),
            string_prop("font", "poppins"),
            number_prop("p", 4),
            string_prop("rounded", "md"),
        ],
        "Welcome",
    )
    .expect("title");

    match node {
        ViewNode::Title { props, value } => {
            assert_eq!(value, "Welcome");
            assert!(props.size.is_some());
            assert_eq!(
                props.align.expect("align").entries[0].value,
                TextAlign::Center
            );
            assert!(props.style.text.is_some());
            assert!(props.style.bg.is_some());
            assert_eq!(
                props.style.font.expect("font").entries[0].value,
                FontFamily::Poppins
            );
            assert_eq!(
                props.weight.expect("weight").entries[0].value,
                TextWeight::Black
            );
            assert_eq!(
                props.letter_spacing.expect("spacing").entries[0].value,
                TextSpacing::Tight
            );
            assert_eq!(props.i18n.as_deref(), Some("home.hero.title"));
            assert!(props.style.spacing.p.is_some());
            assert!(props.style.rounded.is_some());
        }
        _ => panic!("title"),
    }

    let error = text_component_node(
        BuiltinComponent::Text,
        vec![string_prop("i18n", "home..title")],
        "Fallback",
    )
    .expect_err("i18n");
    assert_eq!(
        error,
        ComponentError::invalid_prop("i18n", "i18n key segments separated by dots")
    );

    let rich_text_error = rich_text_component_node(
        vec![string_prop("align", "center")],
        vec![RichTextMark {
            text: "Rich".to_string(),
            style: RichTextMarkStyle::Mark,
            color: ColorFamily::Primary,
        }],
    )
    .expect_err("RichText align");
    assert_eq!(
        rich_text_error,
        ComponentError::unknown_prop(BuiltinComponent::RichText, "align")
    );

    let invalid_align = text_component_node(
        BuiltinComponent::Text,
        vec![string_prop("align", "left")],
        "Invalid",
    )
    .expect_err("invalid align");
    assert_eq!(
        invalid_align,
        ComponentError::invalid_prop("align", "start, center, end or justify")
    );
}

#[test]
fn validates_svg_component_props_and_paths() {
    let path = svg_path_component(vec![
        string_prop("d", "M0 0h24v24H0z"),
        string_prop("fill", "currentColor"),
        string_prop("fillRule", "evenodd"),
        string_prop("transform", "matrix(1 0 0 1 4 6)"),
    ])
    .expect("path");
    assert_eq!(
        path.fill,
        SvgPathFill::Fill {
            color: None,
            opacity: 255,
            even_odd: true,
        }
    );
    assert_eq!(
        path.transform.as_ref().map(SvgTransform::as_str).as_deref(),
        Some("matrix(1 0 0 1 4 6)")
    );

    let node = svg_component_node(
        vec![
            string_prop("viewBox", "0 0 24 24"),
            string_prop("color", "tertiary"),
            number_prop("w", 8),
            number_prop("h", 8),
        ],
        vec![path],
    )
    .expect("svg");

    match node {
        ViewNode::Svg { props, paths } => {
            assert_eq!(props.view_box.as_str(), "0 0 24 24");
            assert!(props.style.text.is_some());
            assert!(props.style.sizing.w.is_some());
            assert_eq!(paths.len(), 1);
        }
        _ => panic!("svg"),
    }

    let svg_path = || {
        svg_path_component(vec![
            string_prop("d", "M0 0h24v24H0z"),
            string_prop("fill", "currentColor"),
        ])
        .expect("svg path")
    };
    let only_height = svg_component_node(
        vec![string_prop("viewBox", "0 0 120 60"), number_prop("h", 8)],
        vec![svg_path()],
    )
    .expect("svg with automatic width");
    let ViewNode::Svg {
        props: only_height_props,
        ..
    } = only_height
    else {
        panic!("svg with automatic width");
    };
    assert!(only_height_props.style.sizing.w.is_none());
    assert!(only_height_props.style.sizing.h.is_some());

    let only_width = svg_component_node(
        vec![string_prop("viewBox", "0 0 120 60"), number_prop("w", 16)],
        vec![svg_path()],
    )
    .expect("svg with automatic height");
    let ViewNode::Svg {
        props: only_width_props,
        ..
    } = only_width
    else {
        panic!("svg with automatic height");
    };
    assert!(only_width_props.style.sizing.w.is_some());
    assert!(only_width_props.style.sizing.h.is_none());

    let default_size = svg_component_node(
        vec![string_prop("viewBox", "0 0 24 24")],
        vec![svg_path()],
    )
    .expect("default svg size");
    let ViewNode::Svg {
        props: default_props,
        ..
    } = default_size
    else {
        panic!("default svg size");
    };
    let expected_default = SizeValue::Scale(ScaleValue::from_half_steps(12));
    assert_eq!(
        default_props.style.sizing.w.expect("default width").entries[0].value,
        expected_default
    );
    assert_eq!(
        default_props.style.sizing.h.expect("default height").entries[0].value,
        expected_default
    );

    let fill = svg_path_component(vec![
        string_prop("d", "M0 0L1 1"),
        string_prop("fill", "primary"),
    ])
    .expect("fill");
    assert_eq!(fill.fill, SvgPathFill::Color(super::ColorToken::Primary));
    let original = svg_path_component(vec![
        string_prop("d", "M0 0L1 1"),
        string_prop("fill", "#000000"),
        string_prop("fillRule", "evenodd"),
    ])
    .expect("original fill");
    assert!(matches!(
        original.fill,
        SvgPathFill::LiteralFill {
            red: 0,
            green: 0,
            blue: 0,
            opacity: 255,
            even_odd: true,
        }
    ));
    assert!(
        svg_path_component(vec![
            string_prop("d", "M0 0L1 1"),
            string_prop("transform", "translate(4 6)"),
        ])
        .is_err()
    );
    assert!(
        svg_path_component(vec![
            string_prop("d", "M0 0L1 1"),
            string_prop("fillRule", "inherit"),
        ])
        .is_err()
    );
}

#[test]
fn validates_runtime_svg_data_reference() {
    let node = svg_component_node(
        vec![
            string_prop("data", "icon.svg"),
            string_prop("color", "primary"),
            number_prop("w", 12),
            number_prop("h", 12),
        ],
        Vec::new(),
    )
    .expect("runtime svg");
    let ViewNode::Svg { props, paths } = node else {
        panic!("svg");
    };
    assert_eq!(props.data.as_deref(), Some("icon.svg"));
    assert!(paths.is_empty());

    assert!(
        svg_component_node(
            vec![
                string_prop("data", "icon.svg"),
                string_prop("viewBox", "0 0 24 24"),
            ],
            Vec::new(),
        )
        .is_err()
    );
    assert!(
        svg_component_node(
            vec![string_prop("data", "icon.svg")],
            vec![svg_path_component(vec![string_prop("d", "M0 0")]).expect("path")],
        )
        .is_err()
    );
}

#[test]
fn resolves_solar_icon_variant_names_and_paints() {
    for name in [
        "alt-arrow-down",
        "alt-arrow-down-broken",
        "alt-arrow-down-outline",
        "alt-arrow-down-bold",
        "alt-arrow-down-line-duotone",
        "alt-arrow-down-bold-duotone",
    ] {
        icon_component_node(vec![string_prop("name", name)]).expect("Solar variant");
    }

    let linear = icon_component_node(vec![
        string_prop("name", "alt-arrow-down"),
        string_prop("fill", "secondary"),
        string_prop("stroke", "tertiary"),
    ])
    .expect("linear icon");
    let ViewNode::Svg { props, paths } = linear else {
        panic!("icon svg");
    };
    assert_eq!(props.view_box.as_str(), "0 0 24 24");
    assert_eq!(
        props.style.sizing.w.expect("width").entries[0].value,
        SizeValue::Scale(ScaleValue::from_half_steps(12))
    );
    assert!(matches!(
        paths[0].fill,
        SvgPathFill::Stroke {
            color: Some(ColorToken::Tertiary),
            width: 150,
            line_cap: SvgLineCap::Round,
            line_join: SvgLineJoin::Round,
            ..
        }
    ));

    let duotone = icon_component_node(vec![string_prop("name", "alt-arrow-down-bold-duotone")])
        .expect("duotone icon");
    let ViewNode::Svg { paths, .. } = duotone else {
        panic!("icon svg");
    };
    assert!(
        paths
            .iter()
            .any(|path| matches!(path.fill, SvgPathFill::Fill { opacity: 128, .. }))
    );
}

#[test]
fn preserves_dynamic_icon_name_bindings_for_lowering() {
    let node = icon_component_node(vec![string_prop(
        "name",
        "@icon-binding:platform.icon",
    )])
    .expect("dynamic icon");
    let ViewNode::Svg { props, paths } = node else {
        panic!("icon svg");
    };
    assert_eq!(props.icon_name.as_deref(), Some("platform.icon"));
    assert_eq!(props.view_box.as_str(), "0 0 24 24");
    assert!(paths.is_empty());
}

#[test]
fn exposes_the_shared_side_nav_submenu_arrow_geometry() {
    let arrow = super::side_nav_submenu_arrow_icon();

    assert_eq!(arrow.props.view_box.as_str(), "0 0 24 24");
    assert_eq!(
        arrow.props.style.sizing.w.expect("width").entries[0].value,
        SizeValue::Scale(ScaleValue::from_half_steps(8))
    );
    assert_eq!(arrow.paths.len(), 2);
    assert_eq!(arrow.paths[1].data, super::SIDE_NAV_SUBMENU_ARROW_PATH);
    assert!(matches!(arrow.paths[0].fill, SvgPathFill::None));
    assert!(matches!(arrow.paths[1].fill, SvgPathFill::CurrentColor));
}

#[test]
fn rejects_unknown_solar_icon_names_and_removed_style_prop() {
    assert!(icon_component_node(vec![string_prop("name", "not-a-solar-icon")]).is_err());
    assert!(icon_component_node(vec![string_prop("name", "alt-arrow-down-linear")]).is_err());
    let error = icon_component_node(vec![
        string_prop("name", "alt-arrow-down"),
        string_prop("style", "bold"),
    ])
    .expect_err("removed style prop");
    assert!(
        error
            .to_string()
            .contains("include the Solar variant in name")
    );
}

#[test]
fn validates_every_bundled_solar_icon_variant() {
    assert_eq!(validate_solar_icon_catalog().expect("catalog"), 7476);
}

#[test]
fn exports_every_solar_variant_as_runtime_svg_data() {
    let catalog = super::solar_runtime_svg_catalog().expect("runtime catalog");
    assert_eq!(catalog.len(), 7476);
    assert_eq!(
        catalog
            .iter()
            .map(|entry| entry.category)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        37
    );
    let arrow = catalog
        .iter()
        .find(|entry| entry.name == "alt-arrow-down" && entry.style == "linear")
        .expect("linear arrow");
    assert_eq!(arrow.category, "arrows");
    assert!(
        arrow
            .svg
            .starts_with("{\"viewBox\":\"0 0 24 24\",\"paths\":[")
    );
    assert!(arrow.svg.contains("\"paint\":\"stroke\""));
}

#[test]
fn resolves_svg_spinner_icons_and_rejects_removed_style_prop() {
    let spinner = icon_component_node(vec![
        string_prop("name", "svg-spinners:3-dots-bounce"),
        string_prop("fill", "primary"),
    ])
    .expect("spinner icon");
    let ViewNode::Svg { props, paths } = spinner else {
        panic!("spinner svg");
    };
    assert_eq!(props.view_box.as_str(), "0 0 24 24");
    assert!(props.motion.is_some());
    assert_eq!(paths.len(), 3);
    assert!(paths.iter().all(|path| matches!(
        path.fill,
        SvgPathFill::Fill {
            color: Some(ColorToken::Primary),
            ..
        }
    )));
    assert!(
        icon_component_node(vec![
            string_prop("name", "svg-spinners:3-dots-bounce"),
            string_prop("style", "bold"),
        ])
        .is_err()
    );
    assert!(icon_component_node(vec![string_prop("name", "svg-spinners:not-a-spinner",)]).is_err());

    let pulse = icon_component_node(vec![string_prop("name", "svg-spinners:pulse")])
        .expect("pulse fallback");
    let ViewNode::Svg { paths, .. } = pulse else {
        panic!("pulse svg");
    };
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].data, "M12 3a9 9 0 1 1-6.364 2.636");

    let ring = icon_component_node(vec![string_prop("name", "svg-spinners:ring-resize")])
        .expect("ring resize fallback");
    let ViewNode::Svg { paths, .. } = ring else {
        panic!("ring resize svg");
    };
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].data, "M12 3a9 9 0 1 1-6.364 2.636");
}

#[test]
fn resolves_svg_logo_icons_with_bundled_source_and_native_paths() {
    let logo = icon_component_node(vec![string_prop("name", "svg-logos:github-icon")])
        .expect("SVG logo icon");
    let ViewNode::Svg { props, paths } = logo else {
        panic!("SVG logo");
    };
    let source = props.motion.expect("bundled SVG logo source");
    assert!(!source.animated);
    assert!(source.source.contains("<svg"));
    assert!(!paths.is_empty());
    assert!(paths.iter().any(|path| matches!(
        path.fill,
        SvgPathFill::LiteralFill { .. } | SvgPathFill::LiteralStroke { .. }
    )));

    assert!(
        icon_component_node(vec![
            string_prop("name", "svg-logos:github-icon"),
            string_prop("style", "bold"),
        ])
        .is_err()
    );
    assert!(icon_component_node(vec![string_prop("name", "svg-logos:not-a-logo",)]).is_err());
}

#[test]
fn validates_every_bundled_svg_logo() {
    assert_eq!(
        validate_svg_logo_catalog().expect("SVG Logos catalog"),
        1863
    );
}

#[test]
fn rejects_invalid_svg_component_usage() {
    let error = svg_component_node(vec![string_prop("viewBox", "0 0 24 24")], Vec::new())
        .expect_err("empty svg");
    assert_eq!(
        error,
        ComponentError::invalid_prop_combination("Svg requires at least one Path child")
    );

    let error = svg_component_node(
        vec![string_prop("viewBox", "0 0 0 24")],
        vec![svg_path_component(vec![string_prop("d", "M0 0")]).expect("path")],
    )
    .expect_err("viewbox");
    assert_eq!(
        error,
        ComponentError::invalid_prop("viewBox", "four numbers with positive width and height")
    );

    let error = svg_path_component(vec![string_prop("d", "M0 0 <script")]).expect_err("path data");
    assert_eq!(
        error,
        ComponentError::invalid_prop("d", "portable SVG path data")
    );

    let error = svg_path_component(vec![
        string_prop("d", "M0 0"),
        string_prop("fill", "url(#gradient)"),
    ])
    .expect_err("fill");
    assert_eq!(
        error,
        ComponentError::invalid_prop(
            "fill",
            "currentColor, none, hexadecimal color or color token"
        )
    );
}

#[test]
fn validates_responsive_text_typography_props() {
    let node = text_component_node(
        BuiltinComponent::Text,
        vec![
            responsive_string_prop("size", &[("xs", "sm"), ("md", "lg")]),
            responsive_string_prop("align", &[("xs", "start"), ("md", "justify")]),
            responsive_string_prop(
                "weight",
                &[("xs", "thin"), ("md", "extralight"), ("lg", "black")],
            ),
            responsive_string_prop("spacing", &[("xs", "normal"), ("md", "wide")]),
            responsive_string_prop("bg", &[("md", "softInfo")]),
            responsive_string_prop(
                "font",
                &[("xs", "inter"), ("md", "manrope"), ("lg", "lora")],
            ),
        ],
        "Login",
    )
    .expect("text");

    match node {
        ViewNode::Text { props, .. } => {
            assert_eq!(props.size.expect("size").entries.len(), 2);
            assert_eq!(
                props.align.expect("align").entries[1].value,
                TextAlign::Justify
            );
            assert_eq!(
                props.weight.expect("weight").entries[2].value,
                TextWeight::Black
            );
            assert_eq!(
                props.letter_spacing.expect("spacing").entries[1].value,
                TextSpacing::Wide
            );
            assert!(props.style.bg.is_some());
            let font = props.style.font.expect("font");
            assert_eq!(font.entries.len(), 3);
            assert_eq!(font.entries[1].value, FontFamily::Manrope);
            assert_eq!(font.entries[2].value, FontFamily::Lora);
        }
        _ => panic!("text"),
    }
}

#[test]
fn validates_show_visibility_props() {
    let node = container_component_node(
        BuiltinComponent::Box,
        vec![responsive_boolean_prop(
            "show",
            &[("xs", false), ("md", true)],
        )],
        vec![text_node("Ready").expect("text")],
        false,
    )
    .expect("box");

    match node {
        ViewNode::Box { props, .. } => match props.element.show.expect("show") {
            VisibilityCondition::Static(value) => {
                assert_eq!(value.entries.len(), 2);
                assert_eq!(value.entries[0].breakpoint, Breakpoint::Xs);
                assert!(!value.entries[0].value);
                assert_eq!(value.entries[1].breakpoint, Breakpoint::Md);
                assert!(value.entries[1].value);
            }
            VisibilityCondition::Signal(_) => panic!("static show"),
            VisibilityCondition::NumberComparison { .. } => panic!("static show"),
            VisibilityCondition::StringEquality { .. } => panic!("static show"),
        },
        _ => panic!("box"),
    }

    let node = text_component_node(
        BuiltinComponent::Text,
        vec![string_prop("show", "isReady")],
        "Ready",
    )
    .expect("text");

    match node {
        ViewNode::Text { props, .. } => {
            assert_eq!(
                props.style.element.show,
                Some(VisibilityCondition::Signal("isReady".to_string()))
            );
        }
        _ => panic!("text"),
    }

    let error = text_component_node(
        BuiltinComponent::Text,
        vec![responsive_string_prop("show", &[("xs", "false")])],
        "Ready",
    )
    .expect_err("invalid show");
    assert_eq!(error, ComponentError::invalid_prop("show", "boolean"));
}

#[test]
fn validates_side_nav_props_entries_and_icons() {
    let icon = super::side_nav_icon_component(
        svg_component_node(
            vec![string_prop("viewBox", "0 0 24 24")],
            vec![svg_path_component(vec![string_prop("d", "M3 11l9-8 9 8v10H3z")]).expect("path")],
        )
        .expect("svg"),
    )
    .expect("icon");
    let item = super::side_nav_item_component(
        vec![
            string_prop("label", "Home"),
            string_prop("description", "Overview"),
            string_prop("href", "/"),
        ],
        Some(icon),
    )
    .expect("item");
    let submenu = super::side_nav_submenu_component(
        vec![string_prop("label", "Content")],
        None,
        true,
        false,
        vec![super::SideNavItemProps {
            label: "Blogs".to_string(),
            i18n: None,
            description: None,
            description_i18n: None,
            status: None,
            status_i18n: None,
            icon: None,
            on_click: None,
            navigation: None,
        }],
    )
    .expect("submenu");
    let node = super::side_nav_component_node(
        vec![
            string_prop("variant", "ghost"),
            string_prop("scheme", "primary"),
            string_prop("size", "lg"),
            boolean_prop("wide", true),
        ],
        vec![item, submenu],
    )
    .expect("side nav");

    match node {
        ViewNode::SideNav { props, items } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Primary));
            assert_eq!(props.size, super::SideNavSize::Lg);
            assert!(props.wide);
            assert!(matches!(&items[0], super::SideNavItem::Item(props) if props.icon.is_some()));
            assert!(
                matches!(&items[1], super::SideNavItem::Submenu { open: true, bordered: false, items, .. } if items.len() == 1)
            );
        }
        _ => panic!("side nav"),
    }
    let structural_scheme = super::side_nav_component_node(
        vec![string_prop("scheme", "primary")],
        vec![super::SideNavItem::Divider],
    )
    .expect("primary scheme");
    let ViewNode::SideNav { props, .. } = structural_scheme else {
        panic!("side nav primary scheme");
    };
    assert_eq!(props.style.color, Some(ColorFamily::Primary));
}

#[test]
fn side_nav_memory_keys_use_ids_and_normalized_structure() {
    let items = vec![super::SideNavItem::Item(super::SideNavItemProps {
        label: "Home".to_string(),
        i18n: None,
        description: None,
        description_i18n: None,
        status: None,
        status_i18n: None,
        icon: None,
        on_click: None,
        navigation: None,
    })];
    let node = super::side_nav_component_node(Vec::new(), items.clone()).expect("side nav");
    let ViewNode::SideNav { mut props, .. } = node else {
        panic!("side nav");
    };
    assert_eq!(props.style.variant, None);
    assert_eq!(props.style.color, None);
    let structural = super::side_nav_memory_key(&props, &items);
    assert!(structural.starts_with("structure:"));
    assert_eq!(structural, super::side_nav_memory_key(&props, &items));
    props.style.element.id = Some("primary-navigation".to_string());
    assert_eq!(
        super::side_nav_memory_key(&props, &items),
        "id:primary-navigation"
    );
    props.style.element.id = Some("secondary-navigation".to_string());
    assert_eq!(
        super::side_nav_memory_key(&props, &items),
        "id:secondary-navigation"
    );
}

#[test]
fn validates_rail_nav_props_items_and_required_icons() {
    let item = super::rail_nav_item_component(vec![
        string_prop("label", "Home"),
        string_prop("icon", "home"),
        string_prop("href", "/"),
    ])
    .expect("item");
    let node = super::rail_nav_component_node(
        vec![
            string_prop("variant", "ghost"),
            string_prop("scheme", "primary"),
            string_prop("size", "lg"),
            boolean_prop("showLabels", true),
        ],
        vec![item, super::RailNavItem::Divider],
    )
    .expect("rail nav");

    match node {
        ViewNode::RailNav { props, items } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Primary));
            assert_eq!(props.size, super::SideNavSize::Lg);
            assert!(props.show_labels);
            assert!(
                matches!(&items[0], super::RailNavItem::Item(props) if props.navigation.is_some())
            );
            assert!(matches!(&items[1], super::RailNavItem::Divider));
        }
        _ => panic!("rail nav"),
    }

    let missing_icon = super::rail_nav_item_component(vec![string_prop("label", "Home")])
        .expect_err("missing icon");
    assert!(
        missing_icon
            .to_string()
            .contains("invalid value for prop `icon`")
    );

    let conflicting_action = super::rail_nav_item_component(vec![
        string_prop("label", "Home"),
        string_prop("icon", "home"),
        string_prop("href", "/"),
        string_prop("onClick", "openHome"),
    ])
    .expect_err("conflicting action");
    assert!(
        conflicting_action
            .to_string()
            .contains("`href` and `onClick` cannot be used on the same RailNav item")
    );
}

#[test]
fn validates_navigation_shell_components() {
    let nav_item = super::nav_menu_item_component(
        vec![string_prop("label", "Home"), string_prop("href", "/")],
        None,
    )
    .expect("nav item");
    let submenu = super::nav_menu_submenu_component(
        vec![string_prop("label", "Docs")],
        None,
        vec![super::NavMenuItemProps {
            label: "Guide".to_string(),
            i18n: Some("navigation.guide".to_string()),
            description: Some("Start here".to_string()),
            description_i18n: None,
            icon: None,
            on_click: None,
            navigation: None,
        }],
    )
    .expect("submenu");
    let megamenu = super::nav_menu_megamenu_component(
        vec![string_prop("label", "Resources")],
        None,
        vec![text_node("Resource hub").expect("text")],
        true,
    )
    .expect("megamenu");
    let nav_menu = super::nav_menu_component_node(
        vec![
            string_prop("variant", "ghost"),
            string_prop("scheme", "primary"),
            string_prop("size", "lg"),
        ],
        vec![nav_item, submenu, megamenu],
    )
    .expect("nav menu");

    match &nav_menu {
        ViewNode::NavMenu { props, items } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Primary));
            assert_eq!(props.size, super::SideNavSize::Lg);
            assert_eq!(items.len(), 3);
            assert!(
                matches!(&items[0], super::NavMenuItem::Item(props) if props.navigation.is_some())
            );
            assert!(
                matches!(&items[1], super::NavMenuItem::Submenu { items, .. } if items.len() == 1)
            );
            assert!(
                matches!(&items[2], super::NavMenuItem::Megamenu { content, .. } if content.len() == 1)
            );
        }
        _ => panic!("nav menu"),
    }

    let sidebar_item = super::side_nav_item_component(
        vec![string_prop("label", "Side Home"), string_prop("href", "/")],
        None,
    )
    .expect("sidebar item");
    let sidebar_nav = super::side_nav_component_node(
        vec![string_prop("size", "sm"), boolean_prop("wide", true)],
        vec![sidebar_item],
    )
    .expect("sidebar nav");
    let sidebar = super::sidebar_component_node(
        vec![
            string_prop("variant", "solid"),
            string_prop("scheme", "primary"),
        ],
        vec![text_node("Header").expect("header")],
        vec![sidebar_nav],
        vec![text_node("Footer").expect("footer")],
        false,
    )
    .expect("sidebar");

    match &sidebar {
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.style.color, Some(ColorFamily::Primary));
            assert_eq!(header.len(), 1);
            assert_eq!(body.len(), 1);
            assert_eq!(footer.len(), 1);
        }
        _ => panic!("sidebar"),
    }

    let scaffold = super::scaffold_component_node(
        vec![boolean_prop("boxed", true)],
        vec![nav_menu],
        vec![sidebar],
        vec![text_node("Main").expect("main")],
        Vec::new(),
        vec![text_node("Bottom").expect("bottom")],
        vec![text_node("Overlay").expect("overlay")],
        true,
    )
    .expect("scaffold");
    match scaffold {
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
        } => {
            assert!(props.boxed);
            assert_eq!(app_bar.len(), 1);
            assert_eq!(start.len(), 1);
            assert_eq!(main.len(), 1);
            assert!(end.is_empty());
            assert_eq!(bottom_bar.len(), 1);
            assert_eq!(overlays.len(), 1);
        }
        _ => panic!("scaffold"),
    }

    let error = super::nav_menu_component_node(
        vec![string_prop("color", "primary")],
        vec![
            super::nav_menu_item_component(vec![string_prop("label", "Home")], None).expect("item"),
        ],
    )
    .expect_err("color error");
    assert_eq!(
        error,
        ComponentError::new("unknown prop `color` on `NavMenu`; use `scheme` for visual family")
    );

    let error = super::sidebar_component_node(
        vec![string_prop("color", "primary")],
        Vec::new(),
        vec![text_node("Body").expect("body")],
        Vec::new(),
        false,
    )
    .expect_err("color error");
    assert_eq!(
        error,
        ComponentError::new("unknown prop `color` on `Sidebar`; use `scheme` for visual family")
    );
}

#[test]
fn validates_drawer_props_and_children() {
    let node = super::drawer_component_node(
        vec![
            string_prop("open", "drawerOpen"),
            string_prop("position", "end"),
            string_prop("variant", "ghost"),
            string_prop("scheme", "primary"),
            boolean_prop("disableOverlayClose", true),
            boolean_prop("hideCloseButton", true),
            responsive_boolean_prop("show", &[("xs", true), ("md", false)]),
        ],
        vec![text_node("Menu").expect("header")],
        vec![text_node("Navigation").expect("text")],
        vec![text_node("Footer").expect("footer")],
        false,
    )
    .expect("drawer");

    match node {
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            assert_eq!(props.open, "drawerOpen");
            assert_eq!(props.position, super::DrawerPosition::End);
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Primary));
            assert!(props.disable_overlay_close);
            assert!(props.hide_close_button);
            assert!(props.style.element.show.is_some());
            assert_eq!(header.len(), 1);
            assert_eq!(body.len(), 1);
            assert_eq!(footer.len(), 1);
        }
        _ => panic!("drawer"),
    }

    let error = super::drawer_component_node(
        vec![string_prop("open", "drawerOpen")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    )
    .expect_err("children");
    assert_eq!(
        error,
        ComponentError::invalid_prop_combination("Drawer requires body children")
    );
}

#[test]
fn validates_display_and_overlay_component_props() {
    let avatar = super::avatar_component_node(
        vec![
            string_prop("name", "Ada"),
            string_prop("scheme", "success"),
            string_prop("variant", "solid"),
            string_prop("size", "lg"),
            string_prop("status", "online"),
            boolean_prop("bordered", true),
        ],
        None,
    )
    .expect("avatar");
    match avatar {
        ViewNode::Avatar { props, .. } => {
            assert_eq!(props.style.color, Some(ColorFamily::Success));
            assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.size, ButtonSize::Lg);
            assert_eq!(props.status, Some(super::AvatarStatus::Online));
            assert!(props.bordered);
        }
        _ => panic!("avatar"),
    }

    let badge = super::badge_component_node(
        vec![
            string_prop("text", "3"),
            string_prop("position", "bottom-right"),
        ],
        vec![text_node("Inbox").expect("text")],
        false,
    )
    .expect("badge");
    assert!(matches!(
        badge,
        ViewNode::Badge {
            props: super::BadgeProps {
                position: super::OverlayCornerPosition::BottomRight,
                ..
            },
            ..
        }
    ));

    let chip = super::chip_component_node(
        vec![string_prop("onClose", "close"), string_prop("size", "sm")],
        "Filter",
        None,
        None,
    )
    .expect("chip");
    assert!(matches!(
        chip,
        ViewNode::Chip {
            props: super::ChipProps {
                on_close: Some(_),
                ..
            },
            ..
        }
    ));

    let skeleton = super::skeleton_component_node(vec![
        string_prop("variant", "rounded"),
        string_prop("animation", "pulse"),
    ])
    .expect("skeleton");
    assert!(matches!(
        skeleton,
        ViewNode::Skeleton {
            props: super::SkeletonProps {
                variant: super::SkeletonVariant::Rounded,
                animation: super::SkeletonAnimation::Pulse,
                ..
            }
        }
    ));

    let modal = super::modal_component_node(
        vec![
            string_prop("open", "open"),
            string_prop("scheme", "surface"),
        ],
        vec![text_node("Header").expect("text")],
        vec![text_node("Body").expect("text")],
        vec![text_node("Footer").expect("text")],
        false,
    )
    .expect("modal");
    assert!(matches!(
        modal,
        ViewNode::Modal {
            props: super::ModalProps { open, .. },
            header,
            body,
            footer,
        } if open == "open" && header.len() == 1 && body.len() == 1 && footer.len() == 1
    ));

    let dialog = super::alert_dialog_component_node(vec![
        string_prop("open", "open"),
        string_prop("title", "Delete?"),
        string_prop("description", "Cannot undo."),
        string_prop("onConfirm", "confirm"),
    ])
    .expect("dialog");
    assert!(matches!(
        dialog,
        ViewNode::AlertDialog {
            props: super::AlertDialogProps {
                open,
                on_confirm: Some(_),
                ..
            },
        } if open == "open"
    ));

    let tooltip = super::tooltip_component_node(
        vec![
            string_prop("label", "More"),
            string_prop("position", "end"),
            string_prop("scheme", "muted"),
        ],
        vec![text_node("Trigger").expect("text")],
        false,
    )
    .expect("tooltip");
    assert!(matches!(
        tooltip,
        ViewNode::Tooltip {
            props: super::TooltipProps {
                position: super::OverlayPosition::End,
                ..
            },
            ..
        }
    ));

    let toast = super::toast_component_node(vec![
        string_prop("type", "success"),
        string_prop("description", "Saved"),
        string_prop("position", "top-right"),
        string_prop("variant", "outlined"),
        string_prop("scheme", "surface"),
        boolean_prop("showIcon", true),
    ])
    .expect("toast");
    assert!(matches!(
        toast,
        ViewNode::Toast {
            props: super::ToastProps {
                kind: super::ToastKind::Success,
                position: super::OverlayCornerPosition::TopRight,
                show_icon: true,
                style: super::VariantProps {
                    variant: Some(super::ComponentVariant::Outlined),
                    color: Some(super::ColorFamily::Surface),
                    ..
                },
                ..
            },
        }
    ));

    let dropdown = super::dropdown_component_node(
        vec![string_prop("scheme", "surface")],
        vec![text_node("Menu").expect("text")],
        Vec::new(),
        vec![super::OverlayEntry::Item(
            super::overlay_item_component(
                BuiltinComponent::Dropdown,
                vec![string_prop("label", "Profile")],
                None,
            )
            .expect("item"),
        )],
        Vec::new(),
        false,
    )
    .expect("dropdown");
    assert!(matches!(dropdown, ViewNode::Dropdown { entries, .. } if entries.len() == 1));

    let command = super::command_component_node(
        vec![string_prop("open", "open"), string_prop("shortcut", "p")],
        vec![super::CommandEntry::Item(
            super::overlay_item_component(
                BuiltinComponent::Command,
                vec![string_prop("label", "Home")],
                None,
            )
            .expect("item"),
        )],
    )
    .expect("command");
    assert!(matches!(
        command,
        ViewNode::Command {
            props: super::CommandProps {
                open: Some(open),
                shortcut,
                ..
            },
            ..
        } if open == "open" && shortcut == "p"
    ));

    let error = super::avatar_component_node(vec![string_prop("color", "primary")], None)
        .expect_err("color");
    assert_eq!(
        error,
        ComponentError::new("unknown prop `color` on `Avatar`; use `scheme` for visual family")
    );
}

#[test]
fn rejects_invalid_design_props() {
    let error = container_component_node(
        BuiltinComponent::Flex,
        vec![number_prop("py", 13)],
        vec![text_node("Hello").expect("text")],
        false,
    )
    .expect_err("error");

    assert_eq!(
        error,
        ComponentError::invalid_prop("py", "Dowe scale value from 0 to 96")
    );

    let error = input_node(vec![string_prop("scheme", "primaryText")]).expect_err("error");
    assert_eq!(
        error,
        ComponentError::invalid_prop(
            "scheme",
            "primary, secondary, tertiary, muted, success, info, warning or danger"
        )
    );

    let error = input_node(vec![string_prop("color", "primary")]).expect_err("error");
    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Input, "color")
    );

    let error = container_component_node(
        BuiltinComponent::Card,
        vec![string_prop("color", "primary")],
        Vec::new(),
        false,
    )
    .expect_err("error");
    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Card, "color")
    );

    let error = container_component_node(
        BuiltinComponent::Alert,
        vec![
            string_prop("type", "success"),
            string_prop("message", "Saved"),
            string_prop("color", "primary"),
        ],
        Vec::new(),
        false,
    )
    .expect_err("error");
    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Alert, "color")
    );

    let error = text_component_node(
        BuiltinComponent::Text,
        vec![string_prop("font", "Inter")],
        "Hello",
    )
    .expect_err("error");
    assert_eq!(
        error,
        ComponentError::invalid_prop(
            "font",
            "system, inter, roboto, montserrat, lato, poppins, manrope, quicksand, lora, syne, jost or puritan"
        )
    );
}

#[test]
fn composes_children_with_page_tree() {
    let layout = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Text {
                props: Default::default(),
                value: "Before".to_string(),
            },
            ViewNode::Children,
            ViewNode::Text {
                props: Default::default(),
                value: "After".to_string(),
            },
        ],
    };
    let page = ViewNode::Box {
        props: Default::default(),
        children: vec![ViewNode::Text {
            props: Default::default(),
            value: "Login".to_string(),
        }],
    };

    assert_eq!(
        compose_tree(&layout, &page),
        ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Text {
                    props: Default::default(),
                    value: "Before".to_string()
                },
                page,
                ViewNode::Text {
                    props: Default::default(),
                    value: "After".to_string()
                }
            ]
        }
    );
}

#[test]
fn finds_only_fixed_fabs_in_nested_trees() {
    let fixed = ViewNode::Fab {
        props: FabProps {
            style: Default::default(),
            position: OverlayCornerPosition::BottomRight,
            fixed: true,
            offset_x: ScaleValue::from_half_steps(8),
            offset_y: ScaleValue::from_half_steps(8),
            icon: ViewIcon::Plus,
            label: "Open actions".to_string(),
        },
        actions: Vec::new(),
    };
    let inline = ViewNode::Fab {
        props: FabProps {
            fixed: false,
            ..match &fixed {
                ViewNode::Fab { props, .. } => props.clone(),
                _ => unreachable!(),
            }
        },
        actions: Vec::new(),
    };
    let tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Section {
                props: Default::default(),
                children: vec![fixed],
            },
            inline,
        ],
    };

    let fabs = fixed_fab_nodes(&tree);
    assert_eq!(fabs.len(), 1);
    assert!(matches!(fabs[0], ViewNode::Fab { props, .. } if props.fixed));
}

#[test]
fn finds_first_text() {
    let tree = ViewNode::Box {
        props: Default::default(),
        children: vec![ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Text {
                props: Default::default(),
                value: "Login".to_string(),
            }],
        }],
    };

    assert_eq!(first_text(&tree), Some("Login".to_string()));
}

fn string_prop(name: &str, value: &str) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::String(value.to_string()),
    }
}

fn number_prop(name: &str, value: i32) -> ComponentProp {
    number_string_prop(name, &value.to_string())
}

fn number_string_prop(name: &str, value: &str) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::Number(value.to_string()),
    }
}

fn boolean_prop(name: &str, value: bool) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::Boolean(value),
    }
}

fn responsive_number_prop(name: &str, entries: &[(&str, i32)]) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::Responsive(
            entries
                .iter()
                .map(|(breakpoint, value)| ResponsivePropEntry {
                    breakpoint: (*breakpoint).to_string(),
                    value: super::PropScalar::Number(value.to_string()),
                })
                .collect(),
        ),
    }
}

fn responsive_boolean_prop(name: &str, entries: &[(&str, bool)]) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::Responsive(
            entries
                .iter()
                .map(|(breakpoint, value)| ResponsivePropEntry {
                    breakpoint: (*breakpoint).to_string(),
                    value: super::PropScalar::Boolean(*value),
                })
                .collect(),
        ),
    }
}

fn responsive_string_prop(name: &str, entries: &[(&str, &str)]) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::Responsive(
            entries
                .iter()
                .map(|(breakpoint, value)| ResponsivePropEntry {
                    breakpoint: (*breakpoint).to_string(),
                    value: super::PropScalar::String((*value).to_string()),
                })
                .collect(),
        ),
    }
}

#[test]
fn parses_the_closed_form_validation_rule_set() {
    let cases = [
        ("required", super::FormValidationRuleKind::Required),
        ("email", super::FormValidationRuleKind::Email),
        ("min:3", super::FormValidationRuleKind::Min(3)),
        ("max:40", super::FormValidationRuleKind::Max(40)),
        ("url", super::FormValidationRuleKind::Url),
        ("phone", super::FormValidationRuleKind::Phone),
        (
            "pattern:^[A-Za-z]+$",
            super::FormValidationRuleKind::Pattern("^[A-Za-z]+$".to_string()),
        ),
        ("alphanumeric", super::FormValidationRuleKind::Alphanumeric),
        ("numeric", super::FormValidationRuleKind::Numeric),
        ("alpha", super::FormValidationRuleKind::Alpha),
        (
            "matches:profile.password",
            super::FormValidationRuleKind::Matches("profile.password".to_string()),
        ),
        (
            "strongPassword",
            super::FormValidationRuleKind::StrongPassword,
        ),
        ("creditCard", super::FormValidationRuleKind::CreditCard),
        ("date", super::FormValidationRuleKind::Date),
        ("minWords:2", super::FormValidationRuleKind::MinWords(2)),
        ("maxWords:8", super::FormValidationRuleKind::MaxWords(8)),
    ];

    for (source, expected) in cases {
        assert_eq!(
            super::form_validation_rule(source, "Invalid value")
                .expect("validation rule")
                .kind,
            expected
        );
    }
}

#[test]
fn rejects_invalid_form_validation_contracts() {
    for rule in [
        "custom",
        "min:0",
        "max:nope",
        "matches:profile..password",
        "pattern:(unclosed",
        "pattern:(?=lookahead)",
        "pattern:(a)\\1",
    ] {
        assert!(super::form_validation_rule(rule, "Invalid").is_err());
    }
    assert!(super::form_validation_rule("required", " ").is_err());
}

#[test]
fn attaches_validation_and_form_messages_to_supported_controls() {
    let rule = super::form_validation_rule("required", "Required").expect("rule");
    let input = super::input_node(vec![
        string_prop("helpText", "Use your work email"),
        string_prop("errorText", "Server rejected this value"),
    ])
    .expect("input");
    let input = super::attach_form_validation(input, vec![rule.clone()]).expect("validation");
    let ViewNode::Input { props } = input else {
        panic!("input");
    };
    let validation = props.element.form_validation().expect("form validation");
    assert_eq!(validation.help_text.as_deref(), Some("Use your work email"));
    assert_eq!(
        validation.error_text.as_deref(),
        Some("Server rejected this value")
    );
    assert_eq!(validation.rules, vec![rule.clone()]);

    let checkbox = super::checkbox_component_node(vec![
        string_prop("helpText", "Required to continue"),
        string_prop("errorText", "Accept the terms"),
    ])
    .expect("checkbox");
    let checkbox = super::attach_form_validation(checkbox, vec![rule.clone()]).expect("validation");
    let ViewNode::Checkbox { props } = checkbox else {
        panic!("checkbox");
    };
    let validation = props
        .style
        .element
        .form_validation()
        .expect("form validation");
    assert_eq!(
        validation.help_text.as_deref(),
        Some("Required to continue")
    );
    assert_eq!(validation.error_text.as_deref(), Some("Accept the terms"));
    assert_eq!(validation.rules, vec![rule]);

    assert!(
        super::attach_form_validation(super::text_node("Unsupported").expect("text"), Vec::new())
            .is_err()
    );
}
