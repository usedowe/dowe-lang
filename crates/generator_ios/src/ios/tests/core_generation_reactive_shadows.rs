#[test]
fn keeps_button_shadow_after_reactive_effective_radius() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Button {
        props: VariantProps {
            style: StyleProps {
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
                shadow: Some(ResponsiveValue::scalar(ShadowSize::Sm)),
                shadow_color: Some(ColorFamily::Secondary),
                extras: Some(Box::new(StyleExtras {
                    motion: ViewMotionStyle {
                        animation: Some(ViewAnimation::SlideUp),
                        ..Default::default()
                    },
                    ..Default::default()
                })),
                ..Default::default()
            },
            reactive: ReactiveVariantProps {
                rounded: Some("radius".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Reactive")],
    };

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("page");
    let radius = "doweButtonRadius(state.text(\"radius\", fallback: \"md\"))";
    let clip = page
        .content
        .find(&format!(
            ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
        ))
        .expect("effective button clip");
    let shadow = page
        .content
        .find(&format!(
            "DoweShadowSpec(color: DoweDesign.secondary.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(12)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(4)) ?? CGFloat(0)), cornerRadius: {radius}"
        ))
        .expect("effective button shadow");
    let animation = page
        .content
        .find(".modifier(DoweAnimationModifier(preset: .slideUp))")
        .expect("button animation");
    assert!(clip < shadow);
    assert!(shadow < animation);
}

#[test]
fn generates_progressive_neutral_swiftui_shadow_strength() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: StyleProps {
            shadow: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: ShadowSize::Xs,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Sm,
                    value: ShadowSize::Sm,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: ShadowSize::Md,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Lg,
                    value: ShadowSize::Lg,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xl,
                    value: ShadowSize::Xl,
                },
            ])),
            ..Default::default()
        },
        children: vec![text("Neutral")],
    };

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains(".shadow(color: Color.black.opacity(doweResponsive(viewportWidth, xs: Double(0.12), sm: Double(0.14), md: Double(0.16), lg: Double(0.18), xl: Double(0.22)) ?? Double(0)), radius: doweResponsive(viewportWidth, xs: CGFloat(2), sm: CGFloat(12), md: CGFloat(24), lg: CGFloat(44), xl: CGFloat(70)) ?? CGFloat(0), x: CGFloat(0), y: doweResponsive(viewportWidth, xs: CGFloat(1), sm: CGFloat(4), md: CGFloat(10), lg: CGFloat(18), xl: CGFloat(28)) ?? CGFloat(0))"));
}

