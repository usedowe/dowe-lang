#[test]
fn carousel_geometry_is_emitted_for_every_variant() {
    for variant in dowe_components::CarouselVariant::all() {
        let mut route = media_display_form_route();
        let ViewNode::Box { children, .. } = &mut route.page_tree else {
            panic!("fixture root");
        };
        let carousel = children
            .iter_mut()
            .find(|node| matches!(node, ViewNode::Carousel { .. }))
            .unwrap();
        let ViewNode::Carousel { props, .. } = carousel else {
            unreachable!()
        };
        props.variant = *variant;
        props.slide_width = None;
        let output = generate_android(
            &[route],
            &FontConfig::default(),
            &DesignConfig::default(),
            &[],
        );
        let source = output
            .files
            .iter()
            .map(|file| file.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            source.contains(&format!("DoweCarousel(variant = \"{}\"", variant.as_str())),
            "{variant:?}"
        );
        assert!(
            source.contains("geometry = DoweCarouselGeometry(contentGap = 12"),
            "{variant:?}"
        );
    }
}

#[test]
fn carousel_title_uses_shared_24_point_typography() {
    let output = generate_android(
        &[media_display_form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let source = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(source.contains("fontSize = 24.sp, lineHeight = 29.sp, fontWeight = FontWeight.Bold"));
}

#[test]
fn carousel_launcher_keeps_control_arrows_square_and_in_the_row() {
    let output = generate_android(
        &[media_display_form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);
    let lines: Vec<_> = dev.content.lines().collect();
    let mut inline_arrows = 0;
    for line in &lines {
        if !line.contains("= doweIconButton(") || !line.contains(" slide\"") {
            continue;
        }
        let button = line.split_whitespace().nth(1).unwrap();
        assert!(line.contains("doweDp(20)") || line.contains("doweDp(24)"));
        let square = format!("{button}.setLayoutParams(new LinearLayout.LayoutParams(");
        if let Some(index) = lines
            .iter()
            .position(|candidate| candidate.contains(&square))
        {
            let dimensions = lines[index]
                .split("new LinearLayout.LayoutParams(")
                .nth(1)
                .unwrap()
                .trim_end_matches("));");
            let (width, height) = dimensions.split_once(", ").unwrap();
            assert_eq!(width, height);
            assert!(lines[index + 1].contains(&format!(", {button}, ")));
            assert!(lines[index + 1].ends_with(", true);"));
            inline_arrows += 1;
        }
    }
    assert!(inline_arrows >= 2);
    assert!(dev
        .content
        .contains("new FrameLayout.LayoutParams(iconSize, iconSize, Gravity.CENTER)"));
}

#[test]
fn carousel_launcher_uses_shared_scheme_accent_for_controls() {
    let mut route = media_display_form_route();
    let ViewNode::Box { children, .. } = &mut route.page_tree else {
        panic!("fixture root");
    };
    let carousel = children
        .iter_mut()
        .find(|node| matches!(node, ViewNode::Carousel { .. }))
        .unwrap();
    let ViewNode::Carousel { props, .. } = carousel else {
        unreachable!()
    };
    props.style.color = Some(dowe_components::ColorFamily::Success);
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);
    assert!(dev.content.contains("DOWE_SUCCESS"));
    assert!(!dev.content.contains("DOWE_SUCCESS_TITLE, doweDp"));
}
