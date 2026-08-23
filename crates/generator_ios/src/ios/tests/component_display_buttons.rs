#[test]
fn generates_loading_button_with_animated_spinner_and_disabled_state() {
    let route = ViewRoute {
        id: "loading-button".to_string(),
        route_path: "/loading-button".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Button {
            props: VariantProps {
                loading_icon: Some(
                    svg_spinner_control_icon("3-dots-move").expect("button spinner"),
                ),
                reactive: ReactiveVariantProps {
                    loading: Some("saving".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Save")],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains(".disabled(state.bool(\"saving\", fallback: true))"));
    assert!(views.contains("if state.bool(\"saving\", fallback: true)"));
    assert!(views.contains("animated: true"));
}

#[test]
fn generates_disabled_button_opacity_for_swiftui() {
    let route = ViewRoute {
        id: "disabled-button".to_string(),
        route_path: "/disabled-button".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Button {
            props: VariantProps {
                variant: Some(ComponentVariant::Soft),
                color: Some(ColorFamily::Secondary),
                reactive: ReactiveVariantProps {
                    disabled: Some("formInvalid".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Submit")],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains(".textSelection(.disabled)"));
    assert!(views.contains(".disabled(state.bool(\"formInvalid\", fallback: true))"));
    assert!(views.contains(".background(DoweDesign.secondary.opacity(state.bool(\"formInvalid\", fallback: true) ? 0.5 : 1))"));
    assert_eq!(
        views.matches(".opacity(state.bool(\"formInvalid\", fallback: true) ? 0.5 : 1)").count(),
        1
    );
}

#[test]
fn generates_full_hit_targets_for_icon_and_text_buttons() {
    let route = ViewRoute {
        id: "button-hit-targets".to_string(),
        route_path: "/button-hit-targets".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Button {
                    props: VariantProps {
                        style: StyleProps {
                            sizing: SizingProps {
                                w: Some(ResponsiveValue::scalar(SizeValue::Scale(
                                    ScaleValue::from_half_steps(20),
                                ))),
                                h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                                    ScaleValue::from_half_steps(20),
                                ))),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        icon_start: Some(solar_control_icon("settings").expect("settings icon")),
                        icon_only: true,
                        label: Some("Open settings".to_string()),
                        navigation: Some(NavigationAction::Internal {
                            path: "/settings".to_string(),
                            fragment: None,
                            operation: NavigationOperation::Push,
                        }),
                        ..Default::default()
                    },
                    children: Vec::new(),
                },
                ViewNode::Button {
                    props: VariantProps {
                        style: StyleProps {
                            spacing: SpacingProps {
                                px: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                                py: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(5))),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        navigation: Some(NavigationAction::Internal {
                            path: "/save".to_string(),
                            fragment: None,
                            operation: NavigationOperation::Push,
                        }),
                        ..Default::default()
                    },
                    children: vec![text("Save")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    let icon_start = views
        .find("Button(action: { navigate(\"push\", \"/settings\", nil) })")
        .expect("icon button");
    let text_start = views
        .find("Button(action: { navigate(\"push\", \"/save\", nil) })")
        .expect("text button");
    let icon_output = &views[icon_start..text_start];
    let text_output = &views[text_start..];
    let icon_width = icon_output.find(".frame(width:").expect("icon width");
    let icon_height = icon_output.find(".frame(height:").expect("icon height");
    let icon_hit_target = icon_output
        .find(".contentShape(Rectangle())")
        .expect("icon hit target");
    let icon_background = icon_output.find(".background(").expect("icon background");
    assert!(icon_width < icon_hit_target);
    assert!(icon_height < icon_hit_target);
    assert!(icon_hit_target < icon_background);
    assert!(icon_output.contains(".accessibilityLabel(Text(\"Open settings\"))"));
    let text_padding = text_output
        .find(".padding(EdgeInsets(")
        .expect("text button padding");
    let text_hit_target = text_output
        .find(".contentShape(Rectangle())")
        .expect("text button hit target");
    let text_background = text_output
        .find(".background(")
        .expect("text button background");
    let text_line_limit = text_output
        .find(".lineLimit(1)")
        .expect("single-line label");
    let text_intrinsic_width = text_output
        .find(".fixedSize(horizontal: true, vertical: false)")
        .expect("intrinsic label width");
    assert!(text_line_limit < text_intrinsic_width);
    assert!(text_intrinsic_width < text_padding);
    assert!(text_padding < text_hit_target);
    assert!(text_hit_target < text_background);
}

#[test]
fn generates_swiftui_percentage_widths() {
    let route = ViewRoute {
        id: "percentage".to_string(),
        route_path: "/percentage".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                sizing: SizingProps {
                    w: Some(ResponsiveValue::scalar(SizeValue::Percent(30))),
                    min_w: Some(ResponsiveValue::scalar(SizeValue::Percent(60))),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("DoweSize.percent(CGFloat(0.3))"));
    assert!(views.contains("DoweSize.percent(CGFloat(0.6))"));
    assert!(views.contains(".dowePercentageWidth(width:"));
    assert!(views.contains("minimumWidthFraction: dowePercentage(minWidth)"));
}

#[test]
fn generates_swiftui_viewport_minus_height() {
    let route = ViewRoute {
        id: "viewport".to_string(),
        route_path: "/viewport".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                sizing: dowe_components::SizingProps {
                    h: Some(ResponsiveValue::scalar(
                        dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(32)),
                    )),
                    min_h: Some(ResponsiveValue::scalar(
                        dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(40)),
                    )),
                    max_w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        ScaleValue::from_half_steps(128),
                    ))),
                    max_h: Some(ResponsiveValue::scalar(
                        dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(48)),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets), viewportHeight: doweSafeAreaHeight(geometry, safeAreaInsets))"));
    assert!(views.contains("DoweSize.viewportMinus(CGFloat(64))"));
    assert!(views.contains("DoweSize.viewportMinus(CGFloat(80))"));
    assert!(views.contains(
        ".frame(height: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.viewportMinus(CGFloat(64))), viewportHeight: viewportHeight))"
    ));
    assert!(views.contains(
        ".frame(minHeight: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.viewportMinus(CGFloat(80))), viewportHeight: viewportHeight))"
    ));
    assert!(views.contains(
        ".frame(maxWidth: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(256)))))"
    ));
    assert!(views.contains(
        ".frame(maxHeight: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.viewportMinus(CGFloat(96))), viewportHeight: viewportHeight))"
    ));
}

