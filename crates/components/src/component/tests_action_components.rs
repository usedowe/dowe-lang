#[test]
fn validates_button_events_and_alert_props() {
    let button = container_component_node(
        BuiltinComponent::Button,
        vec![string_prop("onClick", "saveBlog")],
        vec![text_node("Save").expect("text")],
        false,
    )
    .expect("button");
    match button {
        ViewNode::Button { props, .. } => {
            assert_eq!(props.element.on_click.as_deref(), Some("saveBlog"));
        }
        _ => panic!("button"),
    }

    let alert = container_component_node(
        BuiltinComponent::Alert,
        vec![
            string_prop("type", "success"),
            string_prop("message", "alert.message"),
            string_prop("visible", "alert.visible"),
            string_prop("onClose", "closeAlert"),
        ],
        Vec::new(),
        false,
    )
    .expect("alert");
    match alert {
        ViewNode::Alert { props } => {
            assert_eq!(props.kind.as_str(), "success");
            assert_eq!(props.message, "alert.message");
            assert_eq!(props.visible.as_deref(), Some("alert.visible"));
            assert_eq!(props.on_close.as_deref(), Some("closeAlert"));
        }
        _ => panic!("alert"),
    }
}

#[test]
fn resolves_icon_button_and_control_icon_regions() {
    let mut icon_button = container_component_node(
        BuiltinComponent::IconButton,
        vec![
            string_prop("icon", "settings"),
            string_prop("label", "Open settings"),
        ],
        Vec::new(),
        false,
    )
    .expect("icon button");
    super::apply_design_defaults_to_tree(
        &mut icon_button,
        &super::DesignDefaults::with_builtin_defaults(),
    );
    match icon_button {
        ViewNode::Button { props, children } => {
            assert!(props.icon_only);
            assert!(props.icon_start.is_some());
            assert_eq!(props.label.as_deref(), Some("Open settings"));
            assert!(children.is_empty());
            assert_eq!(
                props.style.sizing.w.expect("width").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(20))
            );
            assert_eq!(
                props.style.sizing.h.expect("height").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(20))
            );
            let icon = props.icon_start.expect("icon");
            assert_eq!(
                icon.props.style.sizing.w.expect("icon width").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(12))
            );
            assert_eq!(
                icon.props.style.sizing.h.expect("icon height").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(12))
            );
        }
        _ => panic!("icon button"),
    }

    let input = input_node(vec![
        string_prop("iconStart", "magnifier"),
        string_prop("iconEnd", "close-circle"),
    ])
    .expect("input icons");
    match input {
        ViewNode::Input { props } => {
            assert!(props.icon_start.is_some());
            assert!(props.icon_end.is_some());
        }
        _ => panic!("input"),
    }

    assert!(
        container_component_node(
            BuiltinComponent::IconButton,
            vec![string_prop("icon", "settings")],
            Vec::new(),
            false,
        )
        .is_err()
    );
}

#[test]
fn resolves_chip_icon_props_with_size_proportional_icons() {
    let chip = super::chip_component_node(
        vec![
            string_prop("size", "lg"),
            string_prop("startIcon", "settings"),
            string_prop("endIcon", "magnifier"),
        ],
        "Filters",
        None,
        None,
    )
    .expect("chip icons");

    let ViewNode::Chip { start, end, .. } = chip else {
        panic!("chip");
    };
    for icon in [start.expect("start icon"), end.expect("end icon")] {
        assert_eq!(
            icon.props.style.sizing.w.expect("icon width").entries[0].value,
            SizeValue::Scale(ScaleValue::from_half_steps(10))
        );
        assert_eq!(
            icon.props.style.sizing.h.expect("icon height").entries[0].value,
            SizeValue::Scale(ScaleValue::from_half_steps(10))
        );
    }

    assert!(
        super::chip_component_node(
            vec![string_prop("startIcon", "settings")],
            "Filters",
            Some(super::solar_control_icon("magnifier").expect("region icon")),
            None,
        )
        .is_err()
    );
}

#[test]
fn normalizes_button_visual_props() {
    let mut node = container_component_node(
        BuiltinComponent::Button,
        vec![string_prop("size", "lg"), number_prop("pl", 1)],
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
            assert_eq!(props.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.color, Some(ColorFamily::Primary));
            assert_eq!(props.style.motion().gesture, Some(ViewGesture::Press));
            assert_eq!(
                props.style.rounded.expect("rounded").entries[0].value,
                RoundedSize::Md
            );
            assert_eq!(props.size, Some(ButtonSize::Lg));
            assert_eq!(
                props.style.spacing.pl.expect("pl").entries[0].value,
                ScaleValue::from_half_steps(2)
            );
            assert_eq!(
                props.style.spacing.pr.expect("pr").entries[0].value,
                ScaleValue::from_half_steps(10)
            );
            assert_eq!(
                props.style.spacing.py.expect("py").entries[0].value,
                ScaleValue::from_half_steps(6)
            );
            assert_eq!(
                props.style.sizing.min_h.expect("minH").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(22))
            );
        }
        _ => panic!("button"),
    }
}

#[test]
fn defaults_press_feedback_for_action_controls_and_respects_opt_out() {
    let defaults = super::DesignDefaults::with_builtin_defaults();
    let mut icon_button = container_component_node(
        BuiltinComponent::IconButton,
        vec![
            string_prop("icon", "settings"),
            string_prop("label", "Open settings"),
        ],
        Vec::new(),
        false,
    )
    .expect("icon button");
    let mut fab = container_component_node(BuiltinComponent::Fab, Vec::new(), Vec::new(), false)
        .expect("fab");
    let mut opted_out = container_component_node(
        BuiltinComponent::Button,
        vec![string_prop("gesture", "none")],
        vec![text_node("No motion").expect("text")],
        false,
    )
    .expect("button");

    super::apply_design_defaults_to_tree(&mut icon_button, &defaults);
    super::apply_design_defaults_to_tree(&mut fab, &defaults);
    super::apply_design_defaults_to_tree(&mut opted_out, &defaults);

    assert!(matches!(
        icon_button,
        ViewNode::Button { ref props, .. } if props.style.motion().gesture == Some(ViewGesture::Press)
    ));
    assert!(matches!(
        fab,
        ViewNode::Fab { ref props, .. } if props.style.style.motion().gesture == Some(ViewGesture::Press)
    ));
    assert!(matches!(
        opted_out,
        ViewNode::Button { ref props, .. } if props.style.motion().gesture == Some(ViewGesture::None)
    ));
}
