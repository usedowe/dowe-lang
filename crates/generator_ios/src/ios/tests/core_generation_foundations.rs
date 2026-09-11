#[test]
fn generates_swiftui_box_and_text() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    assert!(!output.files.iter().any(|file| {
        file.relative_path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.starts_with("DowePhoneCatalog"))
    }));
    let views = swift_content(&output);

    assert!(views.contains("VStack(alignment: .leading, spacing: 0)"));
    assert!(views.contains("AnyView("));
    assert!(!views.contains("() -> AnyView in"));
    assert!(views.contains("routeSection0()"));
    assert!(views.contains("private func routeSection0() -> some View"));
    assert!(views.contains("private let activePath = \"/login\""));
    assert!(!views.contains("        let activePath ="));
    assert!(!views.contains("VStack(alignment: .leading) {"));
    assert!(views.contains(".frame(maxWidth: .infinity, alignment: .leading)"));
    assert!(views.contains(".background(DoweDesign.primary)"));
    assert!(views.contains("Text(verbatim: \"Layout\")"));
    assert!(views.contains("Text(verbatim: \"Login\")"));

    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");
    assert!(plist.content.contains("CFBundleExecutable"));
    assert!(plist.content.contains("DoweIosApp"));
    assert!(plist.content.contains("CFBundleURLSchemes"));
    assert!(plist.content.contains("dowe-dev"));
    assert!(plist.content.contains("UILaunchScreen"));
    assert!(plist.content.contains("NSAllowsLocalNetworking"));
    assert!(plist.content.contains("UIAppFonts"));
    assert!(plist.content.contains("Fonts/inter-regular.ttf"));

    let host = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweIosDevHost.swift"))
        .expect("dev host");
    let module = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweIosViewModule.swift"))
        .expect("dev module");
    assert!(host.content.contains("dlopen(file.path"));
    assert!(host.content.contains("/_dowe/dev/modules/manifest.json?dowe_hmr="));
    assert!(host.content.contains("request.cachePolicy = .reloadIgnoringLocalCacheData"));
    assert!(host.content.contains("request.setValue(\"no-cache\", forHTTPHeaderField: \"Cache-Control\")"));
    assert!(host.content.contains("moduleEndpoint = resolveEndpoint()"));
    assert!(host.content.contains("showWaitingState(in: controller)"));
    assert!(host.content.contains("Preparing Dowe app"));
    assert!(
        host.content
            .contains("The first iOS build can take a few minutes.")
    );
    assert!(host.content.contains("waitingView?.removeFromSuperview()"));
    assert!(
        host.content
            .contains("UserDefaults.standard.set(value, forKey: endpointKey)")
    );
    assert!(
        host.content
            .contains("UserDefaults.standard.string(forKey: endpointKey)")
    );
    assert!(host.content.contains(
        "moduleEndpoint = resolveEndpoint()\n        restoreCachedModule()\n        poll()"
    ));
    assert!(host.content.contains("applicationSupportDirectory"));
    assert!(!host.content.contains("temporaryDirectory"));
    assert!(
        host.content
            .contains("UserDefaults.standard.string(forKey: activeVersionKey)")
    );
    assert!(
        host.content
            .contains("UserDefaults.standard.set(version, forKey: activeVersionKey)")
    );
    assert!(host.content.contains("private var activeRoute = \"/\""));
    assert!(!host.content.contains("dowe.hmr.route"));
    assert!(
        !host
            .content
            .contains("UserDefaults.standard.string(forKey: activeRouteKey)")
    );
    assert!(
        !host
            .content
            .contains("UserDefaults.standard.set(path, forKey: activeRouteKey)")
    );
    assert!(host.content.contains("persistCurrentPath()"));
    assert!(
        module
            .content
            .contains("@_cdecl(\"dowe_create_root_view_controller\")")
    );
    assert!(
        module
            .content
            .contains("@objc(DoweIosDevModuleController___DOWE_IOS_SOURCE_REVISION__)")
    );
    let explicit_objc_names = output
        .files
        .iter()
        .flat_map(|file| file.content.lines())
        .filter(|line| line.trim_start().starts_with("@objc("))
        .collect::<Vec<_>>();
    assert!(!explicit_objc_names.is_empty());
    assert!(
        explicit_objc_names
            .iter()
            .all(|line| line.contains("__DOWE_IOS_SOURCE_REVISION__"))
    );
    let pages = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.swift"))
        .expect("pages");
    assert!(!pages.content.contains("__DOWE_IOS_SOURCE_REVISION__"));
    assert!(views.contains("init(initialPath: String = DoweRoutes.initialPath"));
    assert!(views.contains("routeChanged(path)"));
}

#[test]
fn emits_swiftui_and_responsive_headers_in_pages_artifact() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let pages = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.swift"))
        .expect("pages artifact");

    assert!(pages.content.starts_with("import SwiftUI\n"));
    assert!(
        pages
            .content
            .contains("func doweResponsive<T>(_ viewportWidth: CGFloat")
    );
}

#[test]
fn generates_swiftui_text_alignment() {
    let mut aligned = route();
    aligned.page_tree = ViewNode::Title {
        props: TextProps {
            align: Some(ResponsiveValue::scalar(TextAlign::End)),
            ..Default::default()
        },
        value: "Aligned".to_string(),
    };
    let output = generate_ios(
        &[aligned],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains("doweText(\"Aligned\", alignment: doweResponsive"));
    assert!(views.contains(".frame(maxWidth: .infinity, alignment: doweResponsive"));
    assert!(views.contains("Alignment.trailing"));
    assert!(views.contains("DoweTextAlignment.end"));
}

#[test]
fn generates_swiftui_justified_text() {
    let mut aligned = route();
    aligned.page_tree = ViewNode::Text {
        props: TextProps {
            align: Some(ResponsiveValue::scalar(TextAlign::Justify)),
            ..Default::default()
        },
        value: "Justified".to_string(),
    };
    let output = generate_ios(
        &[aligned],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains("doweText(\"Justified\", alignment: doweResponsive"));
    assert!(views.contains("DoweTextAlignment.justify"));
    assert!(views.contains("doweJustifiedAttributedText"));
}

#[test]
fn generates_static_text_as_verbatim_swiftui_content() {
    let mut literal_route = route();
    literal_route.page_tree = text("info@dowe.dev");
    let views = swift_content(&generate_ios(
        &[literal_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(views.contains("Text(verbatim: \"info@dowe.dev\")"));
    assert!(!views.contains("Text(\"info@dowe.dev\")"));
}

#[test]
fn preserves_multiline_text_in_swiftui_content() {
    let mut multiline = route();
    multiline.page_tree = ViewNode::Title {
        props: TextProps::default(),
        value: "Full-stack development,\nfrom one codebase".to_string(),
    };
    let views = swift_content(&generate_ios(
        &[multiline],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(views.contains("Text(verbatim: \"Full-stack development,\\nfrom one codebase\")"));
}

#[test]
fn inherits_container_foreground_and_preserves_text_overrides() {
    let mut color_route = route();
    color_route.layout_tree = ViewNode::Children;
    color_route.page_tree = container_foreground_tree();
    let views = swift_content(&generate_ios(
        &[color_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    let box_inherited = views
        .find("Text(verbatim: \"Box inherited\")")
        .expect("Box text");
    let box_override = views
        .find("Text(verbatim: \"Box override\")")
        .expect("Box override");
    assert!(!views[box_inherited..box_override].contains(".foregroundStyle("));
    assert!(views[box_override..].contains(
        ".foregroundStyle(doweResponsive(viewportWidth, xs: DoweDesign.danger) ?? DoweDesign.backgroundText)"
    ));
    assert!(views[box_override..].contains(
        ".foregroundStyle(doweResponsive(viewportWidth, xs: DoweDesign.primaryText) ?? DoweDesign.backgroundText)"
    ));

    let card_inherited = views
        .find("Text(verbatim: \"Card inherited\")")
        .expect("Card text");
    assert!(views.contains("Text(verbatim: \"Card title inherited\")"));
    assert!(views.contains(".modifier(DoweTitleColorModifier(explicitColor: nil))"));
    assert!(views.contains("static let defaultValue: Color? = nil"));
    assert!(!views.contains("static let defaultValue: Color = DoweDesign.backgroundTitle"));
    assert!(views.contains(
        "content.foregroundStyle(explicitColor ?? inheritedColor ?? DoweDesign.backgroundTitle)"
    ));
    assert!(views.contains(".environment(\\.doweTitleColor, DoweDesign.mutedTitle)"));
    let mut colored_card_route = route();
    colored_card_route.page_tree = ViewNode::Card {
        props: VariantProps {
            style: StyleProps {
                text: Some(ResponsiveValue::scalar(ColorToken::White)),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![ViewNode::Title {
            props: TextProps::default(),
            value: "Colored card title".to_string(),
        }],
    };
    let colored_card_views = swift_content(&generate_ios(
        &[colored_card_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));
    assert!(colored_card_views.contains(
        ".environment(\\.doweTitleColor, doweResponsive(viewportWidth, xs: Color.white) ?? Color.clear)"
    ));
    let card_override = views
        .find("Text(verbatim: \"Card override\")")
        .expect("Card override");
    assert!(!views[card_inherited..card_override].contains(".foregroundStyle("));
    let card_tail = &views[card_override..];
    let override_color = card_tail
        .find(".modifier(DoweTitleColorModifier(explicitColor: doweResponsive(viewportWidth, xs: DoweDesign.warning)))")
        .expect("Card override color");
    let inherited_color = card_tail
        .find(".foregroundStyle(DoweDesign.mutedText)")
        .expect("Card content color");
    assert!(override_color < inherited_color);
}

#[test]
fn keeps_fixed_width_box_content_leading_aligned() {
    let mut fixed_width = route();
    fixed_width.layout_tree = ViewNode::Children;
    fixed_width.page_tree = ViewNode::Box {
        props: StyleProps {
            bg: Some(ResponsiveValue::scalar(
                ColorToken::from_name("primary").expect("color token"),
            )),
            text: Some(ResponsiveValue::scalar(
                ColorToken::from_name("primaryText").expect("color token"),
            )),
            spacing: dowe_components::SpacingProps {
                p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(6))),
                ..Default::default()
            },
            sizing: dowe_components::SizingProps {
                w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                    ScaleValue::from_half_steps(24),
                ))),
                ..Default::default()
            },
            rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
            ..Default::default()
        },
        children: vec![text("H")],
    };

    let output = generate_ios(
        &[fixed_width],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("VStack(alignment: .leading, spacing: 0)"));
    assert!(views.contains(
        ".frame(width: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(48)))), alignment: .leading)"
    ));
    assert!(views.contains(
        ".frame(maxWidth: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(48)))), alignment: .leading)"
    ));
    assert!(!views.contains(
        ".frame(width: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(48)))))"
    ));
}

#[test]
fn generates_space_between_flex_with_adaptive_spacers() {
    let mut flex_route = route();
    flex_route.layout_tree = ViewNode::Children;
    flex_route.page_tree = ViewNode::Flex {
        props: dowe_components::LayoutProps {
            justify: Some(ResponsiveValue::scalar(dowe_components::Justify::Between)),
            align: Some(ResponsiveValue::scalar(dowe_components::Align::Center)),
            gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                ScaleValue::from_half_steps(6),
            )))),
            ..Default::default()
        },
        children: vec![text("Palette"), text("Live")],
    };

    let output = generate_ios(
        &[flex_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains(
        "HStack(alignment: doweVerticalAlignment(doweResponsive(viewportWidth, xs: DoweAlign.center)), spacing: doweFlexStackSpacing(doweResponsive(viewportWidth, xs: DoweJustify.between), gap: doweResponsive(viewportWidth, xs: CGFloat(12))))"
    ));
    assert!(views.contains(
        "if let spacerGap = doweFlexBetweenSpacer(doweResponsive(viewportWidth, xs: DoweJustify.between), gap: doweResponsive(viewportWidth, xs: CGFloat(12)))"
    ));
    assert!(views.contains("Spacer(minLength: spacerGap)"));
    assert!(views.contains("enum DoweJustify: Equatable"));
    assert!(views.contains("justify == .between ? CGFloat(0) : gap ?? CGFloat(0)"));
}

#[test]
fn preserves_flex_end_inside_ios_cover_card() {
    let mut covered = route();
    covered.layout_tree = ViewNode::Children;
    covered.page_tree = ViewNode::Card {
        props: VariantProps {
            style: StyleProps {
                cover: Some(ResponsiveValue::scalar(CoverSource(
                    "https://images.example/card.jpg".to_string(),
                ))),
                overlay: Some(ResponsiveValue::scalar(OverlayPaint::BlackOpacity(
                    "0.62".to_string(),
                ))),
                text: Some(ResponsiveValue::scalar(ColorToken::White)),
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
                spacing: SpacingProps {
                    p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                    ..Default::default()
                },
                sizing: SizingProps {
                    min_h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                        ScaleValue::from_half_steps(144),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![ViewNode::Flex {
            props: dowe_components::LayoutProps {
                direction: ResponsiveValue::scalar(dowe_components::FlexDirection::Column),
                justify: Some(ResponsiveValue::scalar(dowe_components::Justify::End)),
                gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                    ScaleValue::from_half_steps(4),
                )))),
                style: StyleProps {
                    sizing: SizingProps {
                        min_h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                            ScaleValue::from_half_steps(120),
                        ))),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Foreground")],
        }],
    };

    let views = swift_content(&generate_ios(
        &[covered],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));
    let card = views
        .find("DoweCoverImage(source:")
        .and_then(|start| views.get(start..))
        .expect("cover card");
    assert!(card.contains("DoweJustify.end"));
    assert!(card.contains("doweFlexLeadingSpacer("));
    assert!(card.contains("DoweSize.fixed(CGFloat(240))"));
    assert!(views.contains("if justify == .end || justify == .endSafe || justify == .center || justify == .centerSafe"));
}
