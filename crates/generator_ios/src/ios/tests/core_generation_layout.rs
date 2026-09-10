#[test]
fn generates_wrapped_flex_flow_layout() {
    let mut flex_route = route();
    flex_route.layout_tree = ViewNode::Children;
    flex_route.page_tree = ViewNode::Flex {
        props: dowe_components::LayoutProps {
            wrap: true,
            gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                ScaleValue::from_half_steps(6),
            )))),
            ..Default::default()
        },
        children: vec![text("First"), text("Second")],
    };
    let output = generate_ios(
        &[flex_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains("DoweFlowLayout(justify: nil, align: nil, gap:"));
    assert!(views.contains("struct DoweFlowLayout: Layout"));
    assert!(views.contains("var contentWidth: CGFloat = 0"));
    assert!(!views.contains("rows.map { row in row.map"));
}

#[test]
fn keeps_swiftui_box_background_and_foreground_across_nested_boxes() {
    let mut nested = route();
    nested.layout_tree = ViewNode::Children;
    nested.page_tree = ViewNode::Box {
        props: StyleProps {
            bg: Some(ResponsiveValue::scalar(ColorToken::Surface)),
            text: Some(ResponsiveValue::scalar(ColorToken::SurfaceText)),
            spacing: dowe_components::SpacingProps {
                p: Some(responsive_scale(&[
                    (Breakpoint::Xs, 5),
                    (Breakpoint::Md, 7),
                ])),
                ..Default::default()
            },
            sizing: dowe_components::SizingProps {
                min_h: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                    ScaleValue::from_half_steps(72),
                ))),
                ..Default::default()
            },
            rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
            border: Some(ResponsiveValue::scalar(dowe_components::BorderWidth(1))),
            ..Default::default()
        },
        children: vec![ViewNode::Grid {
            props: GridProps {
                columns: Some(ResponsiveValue::ordered(vec![
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Xs,
                        value: GridTracks::Count(1),
                    },
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: GridTracks::Count(3),
                    },
                ])),
                gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                    ScaleValue::from_half_steps(10),
                )))),
                ..Default::default()
            },
            children: vec![
                ViewNode::Box {
                    props: Default::default(),
                    children: vec![ViewNode::Title {
                        props: Default::default(),
                        value: "Dowe Source Format".to_string(),
                    }],
                },
                ViewNode::Box {
                    props: Default::default(),
                    children: vec![ViewNode::Title {
                        props: Default::default(),
                        value: "Compiler-owned output".to_string(),
                    }],
                },
            ],
        }],
    };

    let output = generate_ios(
        &[nested],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    let grid_start = views
        .find("DoweGridLayout(tracks: doweResponsive(viewportWidth, xs: [CGFloat(1)], md: [CGFloat(1), CGFloat(1), CGFloat(1)]) ?? [CGFloat(1)]")
        .expect("nested grid");
    let surface_section = &views[grid_start..];
    let padding = surface_section
        .find(".padding(EdgeInsets(top: doweResponsive(viewportWidth, xs: CGFloat(20), md: CGFloat(28)) ?? CGFloat(0)")
        .expect("box padding");
    let min_height = surface_section
        .find(".frame(minHeight: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(144))), viewportHeight: viewportHeight))")
        .expect("box min height");
    let background = surface_section
        .find(".background(doweResponsive(viewportWidth, xs: DoweDesign.surface) ?? Color.clear)")
        .expect("box background");
    let foreground = surface_section
        .find(".foregroundStyle(doweResponsive(viewportWidth, xs: DoweDesign.surfaceText) ?? DoweDesign.backgroundText)")
        .expect("box foreground");
    let border = surface_section
        .find(".overlay(RoundedRectangle(cornerRadius: doweResponsive(viewportWidth, xs: CGFloat(12)) ?? DoweDesign.radius).stroke(DoweDesign.backgroundText, lineWidth: doweResponsive(viewportWidth, xs: CGFloat(1)) ?? CGFloat(0)))")
        .expect("box border");

    assert!(padding < background);
    assert!(min_height < background);
    assert!(background < foreground);
    assert!(foreground < border);
    assert!(views.contains("Text(verbatim: \"Dowe Source Format\")"));
    assert!(views.contains("Text(verbatim: \"Compiler-owned output\")"));
}

#[test]
fn keeps_card_shadow_after_swiftui_card_shape() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Card {
        props: VariantProps {
            style: StyleProps {
                shadow: Some(ResponsiveValue::scalar(ShadowSize::Lg)),
                shadow_color: Some(ColorFamily::Primary),
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
                extras: Some(Box::new(StyleExtras {
                    motion: ViewMotionStyle {
                        animation: Some(ViewAnimation::FadeIn),
                        rotate: Some(ResponsiveValue::scalar(ViewRotation(-7))),
                        scale: Some(ResponsiveValue::scalar(ViewScale(105))),
                        translate_x: Some(ResponsiveValue::scalar(ViewTranslation(-3))),
                        transition: Some(ViewTransition::Spring),
                        gesture: Some(ViewGesture::Lift),
                        ..Default::default()
                    },
                    ..Default::default()
                })),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Raised")],
    };

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    let card_start = views
        .find("VStack(alignment: .leading, spacing: 0)")
        .expect("card");
    let card_output = &views[card_start..];
    let clip = card_output
        .find(".clipShape(RoundedRectangle(cornerRadius: doweResponsive(viewportWidth, xs: CGFloat(8)) ?? DoweDesign.radius))")
        .expect("card clip");
    let shadow = card_output
        .find(".background(DoweShadowSurface(shadow: DoweShadowSpec(color: DoweDesign.primary.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(44)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(18)) ?? CGFloat(0)), cornerRadius: doweResponsive(viewportWidth, xs: CGFloat(8)) ?? DoweDesign.radius))")
        .expect("card shadow");
    let animation = shadow
        + card_output[shadow..]
            .find(".modifier(DoweAnimationModifier(preset: .fadeIn))")
            .expect("card animation");

    assert!(shadow > clip);
    assert!(animation > shadow);
    assert!(card_output.contains(
        ".rotationEffect(.degrees(doweResponsive(viewportWidth, xs: Double(-7)) ?? Double(0)))"
    ));
    assert!(card_output.contains(
        ".scaleEffect(CGFloat(doweResponsive(viewportWidth, xs: Double(1.05)) ?? Double(1)))"
    ));
    assert!(card_output.contains(".offset(x: CGFloat(doweResponsive(viewportWidth, xs: Double(-6)) ?? Double(0)), y: CGFloat(0))"));
    assert!(
        card_output.contains(".modifier(DoweGestureModifier(preset: .lift, transition: .spring))")
    );
    assert!(views.contains("@Environment(\\.accessibilityReduceMotion) private var reduceMotion"));
}

#[test]
fn generates_touch_driven_swiftui_gestures() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("@GestureState private var pressed = false"));
    assert!(views.contains("DragGesture(minimumDistance: 0)"));
    assert!(views.contains(".simultaneousGesture(pressGesture)"));
    assert!(views.contains("preset == .grow && (activeHover || activePress)"));
    assert!(views.contains("preset == .tilt && (activeHover || activePress)"));
    assert!(views.contains("return CGFloat(0.94)"));
    assert!(!views.contains(".onLongPressGesture(minimumDuration: 0"));
}

#[test]
fn applies_button_press_feedback_after_the_complete_swiftui_surface() {
    let mut button = VariantProps::default();
    button.style.motion_mut().gesture = Some(ViewGesture::Press);
    let mut button_route = route();
    button_route.layout_tree = ViewNode::Children;
    button_route.page_tree = ViewNode::Button {
        props: button,
        children: vec![text("Press")],
    };
    let output = generate_ios(
        &[button_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("button page");
    let background = page
        .content
        .find(".background(")
        .expect("button background");
    let button_style = page
        .content
        .find(".buttonStyle(.plain)")
        .expect("plain button style");
    let gesture = page
        .content
        .find(".modifier(DoweGestureModifier(preset: .press, transition: .smooth))")
        .expect("press gesture");

    assert!(background < button_style);
    assert!(button_style < gesture);
    assert_eq!(
        page.content
            .matches(".modifier(DoweGestureModifier(preset: .press, transition: .smooth))")
            .count(),
        1
    );
}

