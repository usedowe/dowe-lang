#[test]
fn validates_children_scope() {
    assert_eq!(
        children_node(false).expect_err("children error"),
        ComponentError::children_outside_layout()
    );

    assert_eq!(children_node(true).expect("children"), ViewNode::Children);
}

#[test]
fn validates_design_props() {
    let node = container_component_node(
        BuiltinComponent::Box,
        vec![
            string_prop("bg", "primary"),
            string_prop("font", "roboto"),
            number_string_prop("px", "0.5"),
            number_prop("p", 8),
            responsive_string_prop("h", &[("xs", "full"), ("md", "auto")]),
            string_prop("minH", "vh-16"),
            responsive_number_prop("maxW", &[("xs", 64), ("md", 80)]),
            string_prop("maxH", "vh-24"),
        ],
        vec![text_node("Hello").expect("text")],
        false,
    )
    .expect("box");

    match node {
        ViewNode::Box { props, .. } => {
            assert!(props.bg.is_some());
            assert_eq!(
                props.font.expect("font").entries[0].value,
                FontFamily::Roboto
            );
            assert_eq!(
                props.spacing.p.expect("p").entries[0].value,
                ScaleValue::from_half_steps(16)
            );
            assert_eq!(
                props.spacing.px.expect("px").entries[0].value,
                ScaleValue::from_half_steps(1)
            );
            assert_eq!(props.sizing.h.expect("h").entries[1].value, SizeValue::Auto);
            assert_eq!(
                props.sizing.min_h.expect("minH").entries[0].value,
                SizeValue::ViewportMinus(ScaleValue::from_half_steps(32))
            );
            assert_eq!(
                props.sizing.max_w.expect("maxW").entries[1].value,
                SizeValue::Scale(ScaleValue::from_half_steps(160))
            );
            assert_eq!(
                props.sizing.max_h.expect("maxH").entries[0].value,
                SizeValue::ViewportMinus(ScaleValue::from_half_steps(48))
            );
        }
        _ => panic!("box"),
    }

    assert_eq!(
        container_component_node(
            BuiltinComponent::Box,
            vec![string_prop("h", "vh-nope")],
            vec![text_node("Hello").expect("text")],
            false,
        )
        .expect_err("invalid viewport height"),
        ComponentError::invalid_prop("h", "Dowe scale value, full, auto or vh-<scale>")
    );

    assert_eq!(
        container_component_node(
            BuiltinComponent::Box,
            vec![string_prop("w", "vh-16")],
            vec![text_node("Hello").expect("text")],
            false,
        )
        .expect_err("viewport height as width"),
        ComponentError::invalid_prop(
            "w",
            "Dowe scale value, container size, percentage from 10% to 100% in 10% increments or full",
        )
    );

    assert_eq!(
        container_component_node(
            BuiltinComponent::Box,
            vec![string_prop("maxW", "vh-16")],
            vec![text_node("Hello").expect("text")],
            false,
        )
        .expect_err("viewport height as max width"),
        ComponentError::invalid_prop("maxW", "Dowe scale value, container size or full")
    );
}

#[test]
fn validates_container_width_values_for_all_width_props() {
    for prop in ["w", "minW", "maxW"] {
        for value in ContainerSize::all() {
            let node = container_component_node(
                BuiltinComponent::Box,
                vec![string_prop(prop, value.as_str())],
                vec![text_node("Hello").expect("text")],
                false,
            )
            .expect("container width");

            let sizing = match node {
                ViewNode::Box { props, .. } => props.sizing,
                _ => panic!("box"),
            };
            let parsed = match prop {
                "w" => sizing.w,
                "minW" => sizing.min_w,
                "maxW" => sizing.max_w,
                _ => unreachable!(),
            }
            .expect("width prop")
            .entries[0]
                .value;
            assert_eq!(parsed, SizeValue::Container(*value));
        }
    }

    for prop in ["h", "minH", "maxH"] {
        let error = container_component_node(
            BuiltinComponent::Box,
            vec![string_prop(prop, "2xl")],
            vec![text_node("Hello").expect("text")],
            false,
        )
        .expect_err("container height");
        assert_eq!(
            error,
            ComponentError::invalid_prop(prop, "Dowe scale value, full, auto or vh-<scale>")
        );
    }
}

#[test]
fn validates_percentage_width_values() {
    for prop in ["w", "minW"] {
        for percentage in (10..=100).step_by(10) {
            let value = format!("{percentage}%");
            let node = container_component_node(
                BuiltinComponent::Box,
                vec![string_prop(prop, &value)],
                vec![text_node("Hello").expect("text")],
                false,
            )
            .expect("percentage width");

            let sizing = match node {
                ViewNode::Box { props, .. } => props.sizing,
                _ => panic!("box"),
            };
            let parsed = match prop {
                "w" => sizing.w,
                "minW" => sizing.min_w,
                _ => unreachable!(),
            }
            .expect("width prop")
            .entries[0]
                .value;
            assert_eq!(parsed, SizeValue::Percent(percentage));
        }
    }

    for (prop, expected) in [
        ("maxW", "Dowe scale value, container size or full"),
        ("h", "Dowe scale value, full, auto or vh-<scale>"),
        ("minH", "Dowe scale value, full, auto or vh-<scale>"),
        ("maxH", "Dowe scale value, full, auto or vh-<scale>"),
    ] {
        let error = container_component_node(
            BuiltinComponent::Box,
            vec![string_prop(prop, "50%")],
            vec![text_node("Hello").expect("text")],
            false,
        )
        .expect_err("unsupported percentage");
        assert_eq!(error, ComponentError::invalid_prop(prop, expected));
    }

    for value in ["0%", "15%", "110%", "%10", "10%%"] {
        let error = container_component_node(
            BuiltinComponent::Box,
            vec![string_prop("w", value)],
            vec![text_node("Hello").expect("text")],
            false,
        )
        .expect_err("invalid percentage");
        assert_eq!(
            error,
            ComponentError::invalid_prop(
                "w",
                "Dowe scale value, container size, percentage from 10% to 100% in 10% increments or full",
            )
        );
    }
}

#[test]
fn validates_flex_alignment_contract() {
    for value in [
        "start", "end", "end-safe", "center", "center-safe", "between", "around",
        "evenly", "stretch", "normal",
    ] {
        let node = container_component_node(
            BuiltinComponent::Flex,
            vec![string_prop("justify", value)],
            Vec::new(),
            false,
        )
        .expect("valid justify value");
        let ViewNode::Flex { props, .. } = node else {
            panic!("flex");
        };
        assert_eq!(props.justify.expect("justify").entries[0].value.as_str(), value);
    }

    for value in [
        "start", "end", "end-safe", "center", "center-safe", "baseline", "baseline-last",
        "stretch",
    ] {
        let node = container_component_node(
            BuiltinComponent::Flex,
            vec![string_prop("align", value)],
            Vec::new(),
            false,
        )
        .expect("valid align value");
        let ViewNode::Flex { props, .. } = node else {
            panic!("flex");
        };
        assert_eq!(props.align.expect("align").entries[0].value.as_str(), value);
    }
}

#[test]
fn validates_grid_alignment_contract() {
    for value in [
        "start", "end", "end-safe", "center", "center-safe", "between", "around",
        "evenly", "stretch", "normal",
    ] {
        let node = container_component_node(
            BuiltinComponent::Grid,
            vec![string_prop("justify", value)],
            Vec::new(),
            false,
        )
        .expect("valid grid justify value");
        let ViewNode::Grid { props, .. } = node else {
            panic!("grid");
        };
        assert_eq!(props.justify.expect("justify").entries[0].value.as_str(), value);
    }

    for value in [
        "start", "end", "end-safe", "center", "center-safe", "baseline", "baseline-last",
        "stretch",
    ] {
        let node = container_component_node(
            BuiltinComponent::Grid,
            vec![string_prop("align", value)],
            Vec::new(),
            false,
        )
        .expect("valid grid align value");
        let ViewNode::Grid { props, .. } = node else {
            panic!("grid");
        };
        assert_eq!(props.align.expect("align").entries[0].value.as_str(), value);
    }
}

#[test]
fn rejects_grid_alignment_values_for_the_wrong_axis() {
    for (prop, value) in [("justify", "baseline"), ("align", "between")] {
        let error = container_component_node(
            BuiltinComponent::Grid,
            vec![string_prop(prop, value)],
            Vec::new(),
            false,
        )
        .expect_err("invalid grid alignment value");
        assert!(error.message.contains(prop));
    }
}

#[test]
fn validates_container_refactor_props() {
    let flex = container_component_node(
        BuiltinComponent::Flex,
        vec![
            responsive_string_prop("direction", &[("xs", "column"), ("md", "row")]),
            boolean_prop("wrap", true),
            string_prop("justify", "space-between"),
            string_prop("gap", "20px"),
        ],
        vec![text_node("Hello").expect("text")],
        false,
    )
    .expect("flex");

    match flex {
        ViewNode::Flex { props, .. } => {
            assert_eq!(props.direction.entries[0].value, FlexDirection::Column);
            assert_eq!(props.direction.entries[1].breakpoint, Breakpoint::Md);
            assert_eq!(props.direction.entries[1].value, FlexDirection::Row);
            assert!(props.wrap);
            assert_eq!(
                props.justify.expect("justify").entries[0].value.as_str(),
                "between"
            );
            assert!(matches!(
                props.gap.expect("gap").entries[0].value,
                GapValue::Single(_)
            ));
        }
        _ => panic!("flex"),
    }

    let default_flex = container_component_node(
        BuiltinComponent::Flex,
        Vec::new(),
        vec![text_node("Default").expect("text")],
        false,
    )
    .expect("default flex");
    match default_flex {
        ViewNode::Flex { props, .. } => {
            assert_eq!(props.direction.entries[0].breakpoint, Breakpoint::Xs);
            assert_eq!(props.direction.entries[0].value, FlexDirection::Row);
            assert!(!props.wrap);
        }
        _ => panic!("flex"),
    }

    assert_eq!(
        container_component_node(
            BuiltinComponent::Flex,
            vec![string_prop("direction", "row-reverse")],
            Vec::new(),
            false,
        )
        .expect_err("invalid flex direction"),
        ComponentError::invalid_prop("direction", "row or column")
    );

    assert_eq!(
        container_component_node(
            BuiltinComponent::Flex,
            vec![string_prop("wrap", "true")],
            Vec::new(),
            false,
        )
        .expect_err("invalid flex wrap"),
        ComponentError::invalid_prop("wrap", "boolean")
    );

    let default_grid =
        container_component_node(BuiltinComponent::Grid, Vec::new(), Vec::new(), false)
            .expect("default grid");
    match default_grid {
        ViewNode::Grid { props, .. } => {
            assert_eq!(
                props.columns.expect("columns").entries[0].value,
                GridTracks::Count(1)
            );
            assert_eq!(
                props.justify.expect("justify").entries[0].value,
                GridAlignment::Stretch
            );
            assert_eq!(
                props.align.expect("align").entries[0].value,
                GridAlignment::Stretch
            );
            assert_eq!(
                props.style.sizing.w.expect("width").entries[0].value,
                SizeValue::Full
            );
            assert!(props.style.sizing.h.is_none());
        }
        _ => panic!("grid"),
    }

    let grid = container_component_node(
        BuiltinComponent::Grid,
        vec![
            number_prop("columns", 3),
            number_prop("rows", 2),
            string_prop("justify", "center"),
            string_prop("gap", "10px 20px"),
        ],
        vec![
            container_component_node(
                BuiltinComponent::Box,
                vec![number_prop("colSpan", 2)],
                vec![text_node("Wide").expect("text")],
                false,
            )
            .expect("box"),
            container_component_node(
                BuiltinComponent::Card,
                vec![
                    string_prop("scheme", "surface"),
                    string_prop("rounded", "full"),
                    string_prop("cover", "/images/card.jpg"),
                    boolean_prop("overlay", true),
                ],
                vec![text_node("Card").expect("text")],
                false,
            )
            .expect("card"),
        ],
        false,
    )
    .expect("grid");

    validate_view_tree(&grid).expect("valid grid tree");

    match grid {
        ViewNode::Grid { props, children } => {
            assert_eq!(
                props.columns.expect("columns").entries[0].value,
                GridTracks::Count(3)
            );
            assert_eq!(
                props.justify.expect("justify").entries[0].value,
                GridAlignment::Center
            );
            assert_eq!(children.len(), 2);
        }
        _ => panic!("grid"),
    }

    let fractional_grid = container_component_node(
        BuiltinComponent::Grid,
        vec![string_prop("columns", "1fr 2fr 1fr")],
        Vec::new(),
        false,
    )
    .expect("fractional grid columns");
    match fractional_grid {
        ViewNode::Grid { props, .. } => assert_eq!(
            props.columns.expect("columns").entries[0].value,
            GridTracks::Fractions(vec![1, 2, 1])
        ),
        _ => panic!("grid"),
    }

    assert_eq!(
        container_component_node(
            BuiltinComponent::Grid,
            vec![string_prop("columns", "5fr 0fr")],
            Vec::new(),
            false,
        )
        .expect_err("invalid fractional grid columns"),
        ComponentError::invalid_prop(
            "columns",
            "positive integer from 1 to 12 or space-separated positive fr tracks"
        )
    );

    assert_eq!(
        container_component_node(
            BuiltinComponent::Grid,
            vec![string_prop("rows", "100px auto")],
            Vec::new(),
            false,
        )
        .expect_err("grid row template"),
        ComponentError::invalid_prop("rows", "positive integer or auto")
    );

    assert_eq!(
        container_component_node(
            BuiltinComponent::Grid,
            vec![number_prop("columns", 13)],
            Vec::new(),
            false,
        )
        .expect_err("too many grid columns"),
        ComponentError::invalid_prop(
            "columns",
            "positive integer from 1 to 12 or space-separated positive fr tracks"
        )
    );
}

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

#[test]
fn rejects_grid_spans_outside_direct_grid_children() {
    let tree = container_component_node(
        BuiltinComponent::Box,
        Vec::new(),
        vec![
            container_component_node(
                BuiltinComponent::Box,
                vec![number_prop("colSpan", 2)],
                vec![text_node("Wide").expect("text")],
                false,
            )
            .expect("box"),
        ],
        false,
    )
    .expect("tree");

    assert!(validate_view_tree(&tree).is_err());
}

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

#[test]
fn validates_section_background_props() {
    let node = container_component_node(
        BuiltinComponent::Section,
        vec![
            string_prop("background", "aurora"),
            string_prop("color", "backgroundText"),
            string_prop("animation", "fadeIn"),
            boolean_prop("boxed", true),
        ],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("section");

    match node {
        ViewNode::Section { props, .. } => {
            assert_eq!(
                props.background.as_ref().expect("background").entries[0].value,
                SectionBackground::Aurora
            );
            assert!(props.text.is_some());
            assert_eq!(props.animation(), Some(ViewAnimation::FadeIn));
            assert!(props.boxed);
        }
        _ => panic!("section"),
    }
}

#[test]
fn rejects_invalid_section_background_props() {
    let invalid_background = container_component_node(
        BuiltinComponent::Section,
        vec![string_prop("background", "custom")],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("background");
    assert_eq!(
        invalid_background,
        ComponentError::invalid_prop(
            "background",
            "aurora, sunrise, ocean, meadow or slate"
        )
    );

    let combined_layers = container_component_node(
        BuiltinComponent::Section,
        vec![
            string_prop("background", "aurora"),
            string_prop("cover", "/hero.jpg"),
        ],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("layers");
    assert_eq!(
        combined_layers,
        ComponentError::invalid_prop_combination(
            "`cover` and `background` cannot be used together on `Section`"
        )
    );
}

#[test]
fn rejects_non_boolean_section_boxed_prop() {
    let error = container_component_node(
        BuiltinComponent::Section,
        vec![string_prop("boxed", "true")],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("boxed");

    assert_eq!(error, ComponentError::invalid_prop("boxed", "boolean"));
}

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

#[test]
fn rejects_invalid_section_center_values() {
    let string_value = container_component_node(
        BuiltinComponent::Section,
        vec![string_prop("centerX", "true")],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("center string");
    assert_eq!(
        string_value,
        ComponentError::invalid_prop("centerX", "boolean")
    );

    let invalid_breakpoint = container_component_node(
        BuiltinComponent::Section,
        vec![responsive_boolean_prop("centerX", &[("xxl", true)])],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("center breakpoint");
    assert_eq!(
        invalid_breakpoint,
        ComponentError::invalid_prop("centerX", "valid breakpoint")
    );
}

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

#[test]
fn rejects_invalid_section_gap_values() {
    let error = container_component_node(
        BuiltinComponent::Section,
        vec![boolean_prop("gap", true)],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("gap boolean");
    assert_eq!(
        error,
        ComponentError::invalid_prop("gap", "Dowe scale value or px value")
    );
}

#[test]
fn rejects_overlay_without_cover() {
    let error = container_component_node(
        BuiltinComponent::Box,
        vec![boolean_prop("overlay", true)],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("overlay error");

    assert_eq!(
        error,
        ComponentError::invalid_prop_combination("`overlay` requires `cover` on `Box`")
    );
}

#[test]
fn parses_overlay_forms() {
    let rgba = container_component_node(
        BuiltinComponent::Box,
        vec![
            string_prop("cover", "/images/hero.jpg"),
            string_prop("overlay", "rgba(0,0,0,0.5)"),
        ],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("rgba");

    match rgba {
        ViewNode::Box { props, .. } => {
            assert!(matches!(
                props.overlay.expect("overlay").entries[0].value,
                OverlayPaint::Rgba(_)
            ));
        }
        _ => panic!("box"),
    }

    assert!(
        container_component_node(
            BuiltinComponent::Box,
            vec![
                string_prop("cover", "/images/hero.jpg"),
                string_prop("overlay", "blur(4px)"),
            ],
            vec![text_node("Hero").expect("text")],
            false,
        )
        .is_err()
    );
}

#[test]
fn validates_variant_props() {
    let node = input_node(vec![
        string_prop("variant", "ghost"),
        string_prop("scheme", "danger"),
        string_prop("bind", "blog.title"),
        string_prop("label", "Title"),
        string_prop("placeholder", "Write a title"),
        boolean_prop("labelFloating", true),
    ])
    .expect("input");

    match node {
        ViewNode::Input { props } => {
            assert_eq!(props.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.color, Some(ColorFamily::Danger));
            assert_eq!(props.element.bind.as_deref(), Some("blog.title"));
            assert_eq!(props.label.as_deref(), Some("Title"));
            assert_eq!(props.placeholder.as_deref(), Some("Write a title"));
            assert!(props.label_floating);
        }
        _ => panic!("input"),
    }
}

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

#[test]
fn normalizes_card_padding_default_and_author_override() {
    let default_card = container_component_node(
        BuiltinComponent::Card,
        Vec::new(),
        vec![text_node("Card").expect("text")],
        false,
    )
    .expect("card");

    match default_card {
        ViewNode::Card { props, .. } => {
            let padding = props.style.spacing.p.expect("default padding");
            assert_eq!(padding.entries.len(), 2);
            assert_eq!(padding.entries[0].breakpoint, Breakpoint::Xs);
            assert_eq!(padding.entries[0].value, ScaleValue::from_half_steps(8));
            assert_eq!(padding.entries[1].breakpoint, Breakpoint::Lg);
            assert_eq!(padding.entries[1].value, ScaleValue::from_half_steps(10));
            assert!(props.style.spacing.px.is_none());
            assert!(props.style.spacing.py.is_none());
        }
        _ => panic!("card"),
    }

    let padded_card = container_component_node(
        BuiltinComponent::Card,
        vec![number_prop("p", 4)],
        vec![text_node("Card").expect("text")],
        false,
    )
    .expect("padded card");

    match padded_card {
        ViewNode::Card { props, .. } => {
            assert_eq!(
                props.style.spacing.p.expect("p").entries[0].value,
                ScaleValue::from_half_steps(8)
            );
        }
        _ => panic!("card"),
    }

    let vertical_card = container_component_node(
        BuiltinComponent::Card,
        vec![number_prop("py", 6)],
        vec![text_node("Card").expect("text")],
        false,
    )
    .expect("vertical card");

    match vertical_card {
        ViewNode::Card { props, .. } => {
            assert!(props.style.spacing.p.is_none());
            assert_eq!(
                props.style.spacing.py.expect("py").entries[0].value,
                ScaleValue::from_half_steps(12)
            );
            let horizontal = props.style.spacing.px.expect("default px");
            assert_eq!(horizontal.entries[0].value, ScaleValue::from_half_steps(8));
            assert_eq!(horizontal.entries[1].value, ScaleValue::from_half_steps(10));
        }
        _ => panic!("card"),
    }
}

#[test]
fn derives_section_axis_padding_defaults_and_preserves_overrides() {
    let default_spacing = section_content_spacing(&SpacingProps::default());
    let horizontal = default_spacing.px.expect("default horizontal padding");
    let vertical = default_spacing.py.expect("default vertical padding");
    assert_eq!(horizontal.entries[0].value, ScaleValue::from_half_steps(8));
    assert_eq!(horizontal.entries[1].value, ScaleValue::from_half_steps(12));
    assert_eq!(vertical.entries[0].value, ScaleValue::from_half_steps(20));
    assert_eq!(vertical.entries[1].value, ScaleValue::from_half_steps(32));

    let authored = SpacingProps {
        py: Some(super::ResponsiveValue::scalar(ScaleValue::from_half_steps(
            12,
        ))),
        ..Default::default()
    };
    let effective = section_content_spacing(&authored);
    assert_eq!(
        effective.py.expect("authored vertical padding").entries[0].value,
        ScaleValue::from_half_steps(12)
    );
    assert_eq!(
        effective.px.expect("default horizontal padding").entries[1].value,
        ScaleValue::from_half_steps(12)
    );
}
