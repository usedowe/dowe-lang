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
                    number_prop("overlay", 1),
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

