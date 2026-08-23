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
            string_prop("color", "accent"),
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

    let default_size =
        svg_component_node(vec![string_prop("viewBox", "0 0 24 24")], vec![svg_path()])
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
        default_props
            .style
            .sizing
            .h
            .expect("default height")
            .entries[0]
            .value,
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
    assert!(svg_path_component(vec![
        string_prop("d", "M0 0L1 1"),
        string_prop("transform", "translate(4 6)"),
    ])
    .is_err());
    assert!(svg_path_component(vec![
        string_prop("d", "M0 0L1 1"),
        string_prop("fillRule", "inherit"),
    ])
    .is_err());
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

    assert!(svg_component_node(
        vec![
            string_prop("data", "icon.svg"),
            string_prop("viewBox", "0 0 24 24"),
        ],
        Vec::new(),
    )
    .is_err());
    assert!(svg_component_node(
        vec![string_prop("data", "icon.svg")],
        vec![svg_path_component(vec![string_prop("d", "M0 0")]).expect("path")],
    )
    .is_err());
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
        string_prop("stroke", "accent"),
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
            color: Some(ColorToken::Accent),
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
    assert!(paths
        .iter()
        .any(|path| matches!(path.fill, SvgPathFill::Fill { opacity: 128, .. })));
}

#[test]
fn preserves_dynamic_icon_name_bindings_for_lowering() {
    let node = icon_component_node(vec![string_prop("name", "@icon-binding:platform.icon")])
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
    assert!(error
        .to_string()
        .contains("include the Solar variant in name"));
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
    assert!(arrow
        .svg
        .starts_with("{\"viewBox\":\"0 0 24 24\",\"paths\":["));
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
    assert!(icon_component_node(vec![
        string_prop("name", "svg-spinners:3-dots-bounce"),
        string_prop("style", "bold"),
    ])
    .is_err());
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

    assert!(icon_component_node(vec![
        string_prop("name", "svg-logos:github-icon"),
        string_prop("style", "bold"),
    ])
    .is_err());
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
