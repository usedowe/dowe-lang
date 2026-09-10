#[test]
fn generates_diffuse_semantic_shadows_for_portable_components() {
    let shadow_style = |size, color| StyleProps {
        shadow: Some(ResponsiveValue::scalar(size)),
        shadow_color: Some(color),
        ..Default::default()
    };
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Card {
                props: VariantProps {
                    style: StyleProps {
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
                        ..shadow_style(ShadowSize::Md, ColorFamily::Primary)
                    },
                    ..Default::default()
                },
                children: vec![text("Card")],
            },
            ViewNode::Button {
                props: VariantProps {
                    style: shadow_style(ShadowSize::Sm, ColorFamily::Secondary),
                    ..Default::default()
                },
                children: vec![text("Button")],
            },
            ViewNode::Avatar {
                props: AvatarProps {
                    style: VariantProps {
                        style: StyleProps {
                            spacing: SpacingProps {
                                p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                                ..Default::default()
                            },
                            border: Some(ResponsiveValue::ordered(vec![ResponsiveEntry {
                                breakpoint: Breakpoint::Md,
                                value: BorderWidth(2),
                            }])),
                            border_color: Some(ColorFamily::Warning),
                            ..shadow_style(ShadowSize::Lg, ColorFamily::Accent)
                        },
                        ..Default::default()
                    },
                    src: None,
                    name: Some("Dowe".to_string()),
                    name_binding: None,
                    alt: "Dowe".to_string(),
                    alt_binding: None,
                    size: AvatarSize::Md,
                    size_binding: None,
                    status: None,
                    bordered: true,
                },
                icon: None,
            },
            ViewNode::Chip {
                props: ChipProps {
                    style: VariantProps {
                        style: StyleProps {
                            spacing: SpacingProps {
                                p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                                ..Default::default()
                            },
                            rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
                            border: Some(ResponsiveValue::ordered(vec![ResponsiveEntry {
                                breakpoint: Breakpoint::Md,
                                value: BorderWidth(2),
                            }])),
                            border_color: Some(ColorFamily::Danger),
                            ..shadow_style(ShadowSize::Xs, ColorFamily::Success)
                        },
                        variant: Some(ComponentVariant::Outlined),
                        ..Default::default()
                    },
                    on_close: None,
                },
                value: "Chip".to_string(),
                start: None,
                end: None,
            },
            ViewNode::Input {
                props: VariantProps {
                    style: StyleProps {
                        border: Some(ResponsiveValue::ordered(vec![ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: BorderWidth(2),
                        }])),
                        border_color: Some(ColorFamily::Danger),
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
                        ..shadow_style(ShadowSize::Md, ColorFamily::Info)
                    },
                    variant: Some(ComponentVariant::Outlined),
                    color: Some(ColorFamily::Info),
                    label: Some("Workspace".to_string()),
                    placeholder: Some("dowe-app".to_string()),
                    ..Default::default()
                },
            },
        ],
    };

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    for expected in [
        "DoweShadowSpec(color: DoweDesign.primary.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(24)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(10)) ?? CGFloat(0)), cornerRadius: doweResponsive(viewportWidth, xs: CGFloat(8)) ?? DoweDesign.radius",
        "DoweShadowSpec(color: DoweDesign.secondary.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(12)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(4)) ?? CGFloat(0)), cornerRadius: DoweDesign.radius",
        "shadow: Optional(DoweShadowSpec(color: DoweDesign.accent.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(44)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(18)) ?? CGFloat(0)))",
        "shadow: Optional(DoweShadowSpec(color: DoweDesign.success.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(2)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(1)) ?? CGFloat(0)))",
        "shadow: Optional(DoweShadowSpec(color: DoweDesign.info.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(24)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(10)) ?? CGFloat(0)))",
    ] {
        assert!(views.contains(expected), "missing {expected}");
    }
    assert!(views.contains("struct DoweShadowSurface: View"));
    assert!(views.contains("options: .shadowOnly"));
    assert!(views.contains("context.blendMode = .destinationOut"));
    assert!(views.contains("DoweShadowSurface(shadow: shadow, cornerRadius: CGFloat(9999))"));
    assert!(views.contains("DoweShadowSurface(shadow: shadow, cornerRadius: radius)"));
    assert!(
        views.contains(
            ".overlay(Circle().stroke(borderColor ?? Color.clear, lineWidth: borderWidth))"
        )
    );
    assert!(views.contains(".overlay(RoundedRectangle(cornerRadius: radius).stroke(borderColor ?? Color.clear, lineWidth: borderWidth))"));
    assert!(
        views.contains(
            "radius: doweResponsive(viewportWidth, xs: CGFloat(999)) ?? DoweDesign.radius"
        )
    );
    assert!(views.contains(".clipShape(RoundedRectangle(cornerRadius: radius))"));
    assert_eq!(views.matches("DoweDesign.info.opacity(0.28)").count(), 1);

    let input_runtime_start = views
        .find("struct DoweInputField: View")
        .expect("input runtime");
    let input_runtime = &views[input_runtime_start..];
    let input_border = input_runtime
        .find(".stroke(borderColor ?? Color.clear")
        .expect("input border");
    let input_shadow = input_runtime
        .find("DoweShadowSurface(shadow: shadow, cornerRadius: radius)")
        .expect("input field shadow");
    assert!(input_border < input_shadow);

    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("page");
    let button_start = page
        .content
        .find("Text(verbatim: \"Button\")")
        .expect("button");
    let button_output = &page.content[button_start..];
    let button_background = button_output
        .find(".background(DoweDesign.primary)")
        .expect("button background");
    let button_clip = button_output
        .find(".clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))")
        .expect("button clip");
    let button_shadow = button_output
        .find("DoweShadowSpec(color: DoweDesign.secondary.opacity(0.28)")
        .expect("button shadow");
    assert!(button_background < button_clip);
    assert!(button_clip < button_shadow);

    let avatar_start = page.content.find("DoweAvatar(").expect("avatar");
    let chip_start = page.content.find("DoweChip(").expect("chip");
    let input_start = page.content.find("DoweInputField(").expect("input field");
    let avatar_output = &page.content[avatar_start..chip_start];
    let chip_output = &page.content[chip_start..input_start];
    assert!(
        avatar_output
            .find("shadow: Optional(DoweShadowSpec")
            .expect("avatar shadow")
            < avatar_output
                .find(".padding(")
                .expect("avatar outer padding")
    );
    assert!(
        chip_output
            .find("shadow: Optional(DoweShadowSpec")
            .expect("chip shadow")
            < chip_output.find(".padding(").expect("chip outer padding")
    );
    assert!(!avatar_output.contains(".clipShape("));
    assert!(!chip_output.contains(".clipShape("));
    assert!(avatar_output.contains("borderColor: (doweResponsive(viewportWidth, md: CGFloat(2))) == nil ? Optional(DoweDesign.primaryText) : Optional(DoweDesign.warning)"));
    assert!(
        avatar_output
            .contains("borderWidth: doweResponsive(viewportWidth, md: CGFloat(2)) ?? CGFloat(3)")
    );
    assert!(chip_output.contains("borderColor: (doweResponsive(viewportWidth, md: CGFloat(2))) == nil ? Optional(DoweDesign.primary) : Optional(DoweDesign.danger)"));
    assert!(
        chip_output
            .contains("borderWidth: doweResponsive(viewportWidth, md: CGFloat(2)) ?? CGFloat(1)")
    );

    let input = page
        .content
        .lines()
        .find(|line| line.contains("DoweInputField") && line.contains("label: \"Workspace\""))
        .expect("input");
    assert!(input.contains("shadow: Optional(DoweShadowSpec"));
    assert!(input.contains("borderColor: (doweResponsive(viewportWidth, md: CGFloat(2))) == nil ? Optional(DoweDesign.muted) : Optional(DoweDesign.danger)"));
    assert!(
        input.contains("borderWidth: doweResponsive(viewportWidth, md: CGFloat(2)) ?? CGFloat(1)")
    );
}

