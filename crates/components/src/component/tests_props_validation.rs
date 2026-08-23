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
    assert!(container_component_node(
        BuiltinComponent::Brand,
        vec![string_prop("label", "")],
        vec![text_node("Dowe").expect("text")],
        false,
    )
    .is_err());
    assert!(container_component_node(
        BuiltinComponent::Brand,
        vec![string_prop("href", "javascript:alert(1)")],
        vec![text_node("Dowe").expect("text")],
        false,
    )
    .is_err());
    assert!(container_component_node(
        BuiltinComponent::Brand,
        vec![string_prop("variant", "solid")],
        vec![text_node("Dowe").expect("text")],
        false,
    )
    .is_err());

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
        assert!(container_component_node(
            BuiltinComponent::Banner,
            props,
            vec![text_node("Dowe").expect("text")],
            false,
        )
        .is_err());
    }
    assert!(container_component_node(
        BuiltinComponent::Banner,
        vec![string_prop("href", "https://dowe.dev")],
        Vec::new(),
        false,
    )
    .is_err());
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
            string_prop("as", "h1"),
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
            assert_eq!(props.as_tag.as_deref(), Some("h1"));
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
