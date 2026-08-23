#[test]
fn generates_ios_app_metadata() {
    let output = generate_ios_with_app_and_translations(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &TranslationCatalog::default(),
        "Clinic Desk",
        "com.example.clinic",
    );
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");

    assert!(plist.content.contains("<string>Clinic Desk</string>"));
    assert!(
        plist
            .content
            .contains("<string>com.example.clinic</string>")
    );
    assert!(plist.content.contains("<key>CFBundleName</key>"));
}

#[test]
fn generates_ios_app_icon_metadata_for_phone_and_tablet() {
    let output = generate_ios_with_app_translations_and_icons(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &TranslationCatalog::default(),
        "Clinic Desk",
        "com.example.clinic",
        true,
    );
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");

    assert!(plist.content.contains("<key>CFBundleIcons</key>"));
    assert!(plist.content.contains("<key>CFBundleIcons~ipad</key>"));
    assert!(plist.content.contains("<string>AppIcon60x60</string>"));
    assert!(plist.content.contains("<string>AppIcon76x76</string>"));
    assert!(plist.content.contains("<key>CFBundleIconName</key>"));
    assert!(plist.content.contains("<string>AppIcon</string>"));
}

#[test]
fn generates_swiftui_section_backgrounds() {
    let output = generate_ios(
        &[section_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("enum DoweSectionBackground"));
    assert!(views.contains("DoweSectionBackgroundView(background: background)"));
    assert!(views.contains(".padding(EdgeInsets(top: doweResponsive(viewportWidth, xs: CGFloat(40), md: CGFloat(64)) ?? CGFloat(0), leading: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(24)) ?? CGFloat(0), bottom: doweResponsive(viewportWidth, xs: CGFloat(40), md: CGFloat(64)) ?? CGFloat(0), trailing: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(24)) ?? CGFloat(0)))"));
    let section = &views[views
        .find("doweResponsive(viewportWidth, xs: DoweSectionBackground.aurora")
        .expect("section")..];
    let padding = section
        .find(".padding(EdgeInsets")
        .expect("section padding");
    let max_width = section
        .find(".frame(maxWidth: CGFloat(1536), alignment: .leading)")
        .expect("boxed max width");
    let centered = section
        .find(".frame(maxWidth: .infinity, alignment: .center)")
        .expect("boxed centering");
    assert!(padding < max_width);
    assert!(max_width < centered);
    assert!(views.contains("doweResponsive(viewportWidth, xs: DoweSectionBackground.aurora, md: DoweSectionBackground.ocean)"));
    assert!(views.contains("LinearGradient(colors: [DoweDesign.primary, DoweDesign.secondary, DoweDesign.tertiary]"));
    assert!(views.contains("DoweCoverImage(source:"));
    assert!(views.contains("https://example.com/hero.jpg"));
    assert!(views.contains("DoweOverlay.color(Color.black.opacity(0.35))"));
    assert!(views.contains("DoweOverlayView(overlay: overlay)"));
}

#[test]
fn generates_responsive_section_centering_for_swiftui() {
    let mut route = section_route();
    let ViewNode::Box { children, .. } = &mut route.page_tree else {
        panic!("section route root");
    };
    let ViewNode::Section { props, .. } = &mut children[0] else {
        panic!("section route child");
    };
    props.center_x = Some(ResponsiveValue::ordered(vec![
        ResponsiveEntry {
            breakpoint: Breakpoint::Xs,
            value: false,
        },
        ResponsiveEntry {
            breakpoint: Breakpoint::Md,
            value: true,
        },
    ]));

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains(
        "VStack(alignment: (doweResponsive(viewportWidth, xs: false, md: true) ?? false) ? .center : .leading, spacing: 0)"
    ));
    assert!(views.contains(".frame(maxWidth: .infinity, alignment: .leading)"));
}

#[test]
fn fills_height_bounded_section_body_for_swiftui() {
    let mut route = section_route();
    let ViewNode::Box { children, .. } = &mut route.page_tree else {
        panic!("section route root");
    };
    let ViewNode::Section {
        props,
        children: section_children,
    } = &mut children[0]
    else {
        panic!("section route child");
    };
    props.sizing.min_h = Some(ResponsiveValue::scalar(SizeValue::ViewportMinus(
        ScaleValue::from_half_steps(0),
    )));
    *section_children = vec![ViewNode::Grid {
        props: GridProps {
            columns: Some(ResponsiveValue::scalar(GridTracks::Count(1))),
            style: StyleProps {
                sizing: SizingProps {
                    min_h: Some(ResponsiveValue::scalar(SizeValue::Full)),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Grid")],
    }];

    let views = swift_content(&generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));
    assert!(views.contains(".frame(maxHeight: .infinity)"));
    assert!(views.contains(
        ".frame(maxHeight: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.full)))"
    ));
}

#[test]
fn generates_responsive_auto_and_full_height_constraints_for_containers() {
    let sizing = SizingProps {
        h: Some(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: SizeValue::Auto,
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: SizeValue::Full,
            },
        ])),
        min_h: Some(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: SizeValue::Auto,
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: SizeValue::Full,
            },
        ])),
        max_h: Some(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: SizeValue::Auto,
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: SizeValue::Full,
            },
        ])),
        ..Default::default()
    };
    let mut height_route = route();
    height_route.layout_tree = ViewNode::Children;
    height_route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![
            ViewNode::Box {
                props: StyleProps {
                    sizing: sizing.clone(),
                    ..Default::default()
                },
                children: Vec::new(),
            },
            ViewNode::Section {
                props: StyleProps {
                    sizing: sizing.clone(),
                    ..Default::default()
                },
                children: Vec::new(),
            },
            ViewNode::Grid {
                props: GridProps {
                    style: StyleProps {
                        sizing,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: Vec::new(),
            },
        ],
    };
    let generated = swift_content(&generate_ios(
        &[height_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(
        generated.contains("doweResponsive(viewportWidth, xs: DoweSize.auto, md: DoweSize.full)")
    );
    assert!(generated.contains("private struct DoweParentHeightCapLayout: Layout"));
    assert!(generated.contains(
        ".doweMaxHeight(doweResponsive(viewportWidth, xs: DoweSize.auto, md: DoweSize.full))"
    ));
}

#[test]
fn generates_responsive_section_gap_for_swiftui() {
    let mut route = section_route();
    let ViewNode::Box { children, .. } = &mut route.page_tree else {
        panic!("section route root");
    };
    let ViewNode::Section { props, .. } = &mut children[0] else {
        panic!("section route child");
    };
    props.gap = Some(ResponsiveValue::ordered(vec![
        ResponsiveEntry {
            breakpoint: Breakpoint::Xs,
            value: GapValue::Single(GapSize::Scale(ScaleValue(4))),
        },
        ResponsiveEntry {
            breakpoint: Breakpoint::Md,
            value: GapValue::Single(GapSize::Scale(ScaleValue(8))),
        },
    ]));

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains(
        "VStack(alignment: .leading, spacing: doweResponsive(viewportWidth, xs: CGFloat(8), md: CGFloat(16)) ?? CGFloat(0))"
    ));
}

#[test]
fn generates_native_ios_translation_resources() {
    let mut localized_route = route();
    localized_route.page_tree = ViewNode::Title {
        props: TextProps {
            i18n: Some("home.hero.title".to_string()),
            ..Default::default()
        },
        value: "Dowe builds systems.".to_string(),
    };
    let output = generate_ios_with_translations(
        &[localized_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &translations(),
    );
    let views = swift_content(&output);
    assert!(views.contains(r#"String(localized: "home.hero.title")"#));
    let english = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("en.lproj/Localizable.strings"))
        .expect("english");
    assert!(english.content.contains("Dowe builds systems."));
    let spanish = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("es.lproj/Localizable.strings"))
        .expect("spanish");
    assert!(spanish.content.contains("Dowe construye sistemas."));
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");
    assert!(plist.content.contains("CFBundleDevelopmentRegion"));
    assert!(plist.content.contains("<string>en</string>"));
}

