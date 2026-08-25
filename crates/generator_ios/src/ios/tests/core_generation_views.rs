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
    let animation = card_output
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
                    alt: "Dowe".to_string(),
                    size: ButtonSize::Md,
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

