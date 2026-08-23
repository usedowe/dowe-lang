#[test]
fn fills_request_path_placeholders_from_signal_names() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = swift_content(&output);

    assert!(generated.contains("NSRegularExpression(pattern: \":[A-Za-z_][A-Za-z0-9_]*\")"));
    assert!(generated.contains("signals.reversed().first(where: { $0.value.name == name })"));
    assert!(!generated.contains("signals.last(where:"));
}

#[test]
fn generates_fixed_fab_as_route_overlay_with_dowe_icons() {
    let mut fab_route = route();
    fab_route.page_tree = fixed_fab_page();
    let output = generate_ios(
        &[fab_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = swift_content(&output);

    assert!(generated.contains("ZStack(alignment: .topLeading)"));
    assert!(generated.contains("private func fixedFab0() -> some View"));
    assert!(generated.contains("if doweFixedFabOpen0"));
    assert!(generated.contains("doweFixedFabOpen0.toggle()"));
    assert!(generated.contains("VStack(alignment: .trailing, spacing: CGFloat(12))"));
    assert!(generated.contains("HStack(spacing: CGFloat(12))"));
    assert!(generated.contains(".clipShape(Capsule())"));
    assert!(generated.contains(".contentShape(Capsule())"));
    assert!(generated.contains("ZStack {"));
    assert!(generated.contains(
        ".frame(maxWidth: .infinity, maxHeight: .infinity)\n                .contentShape(Circle())"
    ));
    assert!(generated.contains(".rotationEffect(.degrees(doweFixedFabOpen0 ? 45 : 0))"));
    let rotation = generated
        .find(".rotationEffect(.degrees(doweFixedFabOpen0 ? 45 : 0))")
        .expect("Fab rotation");
    let gesture = generated
        .find(".modifier(DoweGestureModifier(preset: .press, transition: .smooth))")
        .expect("Fab press gesture");
    assert!(rotation < gesture);
    assert!(generated.contains("DoweSvgView(viewBox:"));
    assert!(generated.contains("maxHeight: .infinity, alignment: .bottomTrailing"));
    assert!(!generated.contains("Image(systemName: \"plus\")"));
}

#[test]
fn generates_relative_absolute_and_fixed_boxes_as_swiftui_overlays() {
    let mut positioned_route = route();
    positioned_route.page_tree = positioned_box_page();
    let generated = swift_content(&generate_ios(
        &[positioned_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(generated.contains("ZStack(alignment: .topLeading)"));
    assert!(generated.contains("alignment: .topTrailing"));
    assert!(generated.contains(".padding(.top, doweResponsive(viewportWidth, xs: CGFloat(16)))"));
    assert!(
        generated.contains(".padding(.trailing, doweResponsive(viewportWidth, xs: CGFloat(24)))")
    );
    assert!(generated.contains("alignment: .bottomTrailing"));
    assert!(generated.contains("private func fixedBox0() -> some View"));
}

fn positioned_box_page() -> ViewNode {
    ViewNode::Box {
        props: StyleProps {
            extras: Some(Box::new(dowe_components::StyleExtras {
                position: dowe_components::PositionProps {
                    mode: BoxPosition::Relative,
                    ..Default::default()
                },
                ..Default::default()
            })),
            sizing: SizingProps {
                min_h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                    ScaleValue::from_half_steps(64),
                ))),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![
            text("Flow content"),
            ViewNode::Box {
                props: StyleProps {
                    extras: Some(Box::new(dowe_components::StyleExtras {
                        position: dowe_components::PositionProps {
                            mode: BoxPosition::Absolute,
                            top: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                            right: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(12))),
                            ..Default::default()
                        },
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                children: vec![text("Proof")],
            },
            ViewNode::Box {
                props: StyleProps {
                    extras: Some(Box::new(dowe_components::StyleExtras {
                        position: dowe_components::PositionProps {
                            mode: BoxPosition::Fixed,
                            right: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                            bottom: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                            ..Default::default()
                        },
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                children: vec![text("Persistent")],
            },
        ],
    }
}

fn fixed_fab_page() -> ViewNode {
    let mut style = VariantProps {
        color: Some(ColorFamily::Primary),
        variant: Some(ComponentVariant::Solid),
        size: Some(ButtonSize::Lg),
        ..Default::default()
    };
    style.style.motion_mut().gesture = Some(ViewGesture::Press);
    ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![
            text("Scrollable content"),
            ViewNode::Fab {
                props: FabProps {
                    style,
                    position: OverlayCornerPosition::BottomRight,
                    fixed: true,
                    offset_x: ScaleValue::from_half_steps(8),
                    offset_y: ScaleValue::from_half_steps(8),
                    icon: ViewIcon::Plus,
                    label: "Open actions".to_string(),
                },
                actions: vec![FabAction {
                    label: "Edit".to_string(),
                    icon: ViewIcon::Edit,
                    color: ColorFamily::Info,
                    on_click: None,
                    navigation: Some(NavigationAction::Internal {
                        path: "/edit".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                }],
            },
        ],
    }
}

