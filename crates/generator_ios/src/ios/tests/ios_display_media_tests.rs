#[test]
fn emits_large_theme_catalog_once_outside_route_view_expressions() {
    let mut design = DesignConfig::default();
    let base_theme = design.themes[0].clone();
    design.themes = (1..=18)
        .map(|index| {
            let mut theme = base_theme.clone();
            theme.name = if index == 1 {
                "light".to_string()
            } else {
                format!("palette-{index}")
            };
            theme
        })
        .collect();
    let themes = design
        .themes
        .iter()
        .map(|theme| theme.name.clone())
        .collect::<Vec<_>>();
    let select = ViewNode::SelectTheme {
        props: ThemeSelectProps {
            style: Default::default(),
            label: "Theme".to_string(),
            placeholder: "Choose a theme".to_string(),
            themes,
            default_theme: "light".to_string(),
        },
    };
    let mut themed_route = route();
    themed_route.page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![ViewNode::Flex {
            props: Default::default(),
            children: vec![select.clone(), select],
        }],
    };

    let output = generate_ios(&[themed_route], &FontConfig::default(), &design, &[]);
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("generated route page");
    let theme = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweTheme.swift"))
        .expect("generated theme module");

    assert_eq!(
        page.content
            .matches("DoweThemeModule.selectOptions")
            .count(),
        2
    );
    assert!(!page.content.contains("DoweSelectOption(value:"));
    assert_eq!(
        page.content
            .matches("helpText: nil, errorText: nil, validationRules: []")
            .count(),
        2
    );
    assert!(
        page.content
            .contains("private func routeBranch0() -> some View")
    );
    assert!(
        page.content
            .contains("private func routeBranch2() -> some View")
    );
    assert!(
        theme
            .content
            .contains("static let selectOptions: [DoweSelectOption] = [")
    );
    assert!(theme.content.contains(
        "DoweSelectOption(value: \"palette-18\", label: \"Palette 18\", description: nil)"
    ));
}

#[test]
fn keeps_inherited_font_children_inside_their_swiftui_expression() {
    let mut inherited_route = route();
    inherited_route.page_tree = ViewNode::Box {
        props: StyleProps {
            font: Some(ResponsiveValue::scalar(dowe_components::FontFamily::Inter)),
            ..Default::default()
        },
        children: vec![text("One"), text("Two"), text("Three"), text("Four")],
    };

    let output = generate_ios(
        &[inherited_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("generated route page");

    assert!(!page.content.contains("private func routeBranch"));
    assert!(page.content.contains("xs: .inter"));
}

#[test]
fn generates_floating_input_icons_with_active_visibility() {
    let output = generate_ios(
        &[form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    let input = views
        .lines()
        .find(|line| {
            line.contains(
                r#"DoweInputField(value: nil, label: "Name", placeholder: "Full name", floating: true"#,
            )
        })
        .expect("floating input");

    assert!(input.contains("startIcon: DoweControlIcon("));
    assert!(input.contains("endIcon: DoweControlIcon("));
    assert!(input.contains(r#"data: "M4 12h16""#));
    assert!(input.contains(r#"data: "M12 4v16""#));
    assert!(views.contains("private var iconsVisible: Bool {\n        !floating || active\n    }"));
    assert!(views.contains("if let startIcon, iconsVisible {"));
    assert!(views.contains("if let endIcon, iconsVisible {"));
    assert!(
        views.contains(".padding(.leading, active && startIcon != nil ? CGFloat(32) : CGFloat(0))")
    );
}

#[test]
fn generates_swiftui_media_display_form_components() {
    let output = generate_ios(
        &[media_display_form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweAudioView: View"));

    let mut card_route = media_display_form_route();
    card_route.page_tree = ViewNode::RadioGroup {
        props: RadioGroupProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Outlined),
                color: Some(ColorFamily::Primary),
                element: ElementProps {
                    bind: Some("workspace".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            size: ButtonSize::Md,
            orientation: RadioGroupOrientation::Horizontal,
            presentation: RadioGroupPresentation::Card,
            name: Some("workspace".to_string()),
            info: None,
            error: None,
        },
        options: vec![RadioOption {
            value: "local".to_string(),
            label: "Local".to_string(),
            description: Some("Edit files on your computer".to_string()),
            icon: Some(solar_control_icon("laptop").expect("laptop icon")),
            disabled: false,
        }],
    };
    let card_output = generate_ios(
        &[card_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let card_views = swift_content(&card_output);
    assert!(card_views.contains("DoweRadioCardView("));
    assert!(card_views.contains("DoweRadioCardOption("));
    assert!(card_views.contains("Edit files on your computer"));
    assert!(card_views.contains("accessibilityAddTraits(selected ? .isSelected"));
    assert!(views.contains("DoweAudioView(source:"));
    assert!(views.contains("@State private var player: AVPlayer"));
    assert!(views.contains("ForEach(0..<50"));
    assert!(views.contains("DragGesture(minimumDistance: 0)"));
    assert!(views.contains("private func doweAudioTime"));
    assert!(views.contains("playIcon: DoweVideoIcon"));
    assert!(views.contains("private let doweAudioWaveform: [CGFloat]"));
    assert!(views.contains("private struct DoweAudioControlButton: View"));
    assert!(views.contains(".animation(.easeInOut(duration: 0.3), value: currentTime)"));
    assert!(views.contains("struct DoweCoverImage: View"));
    let cover_runtime = views
        .split("struct DoweCoverImage: View")
        .nth(1)
        .expect("cover runtime")
        .split("private func doweImageURL")
        .next()
        .expect("cover runtime boundary");
    assert!(cover_runtime.contains("GeometryReader { proxy in"));
    assert!(cover_runtime.contains(".frame(width: proxy.size.width, height: proxy.size.height)"));
    assert!(views.contains("struct DoweImageView: View"));
    let image_runtime = views
        .split("struct DoweImageView: View")
        .nth(1)
        .expect("image runtime")
        .split("private func doweImageAspect")
        .next()
        .expect("image aspect helper");
    assert!(image_runtime.contains("DoweImageAspectLayout(ratio: doweImageAspect(aspect))"));
    assert!(image_runtime.contains("struct DoweImageAspectLayout: Layout"));
    assert!(
        image_runtime
            .contains("return CGSize(width: resolvedWidth, height: resolvedWidth / resolvedRatio)")
    );
    assert!(image_runtime.contains("proposal: ProposedViewSize(bounds.size)"));
    assert!(image_runtime.contains(".clipped()"));
    assert!(image_runtime.contains(".accessibilityAddTraits(.isImage)"));
    assert!(image_runtime.contains(".accessibilityLabel(Text(alt))"));
    assert!(image_runtime.contains(".accessibilityHidden(alt.isEmpty)"));
    assert!(!image_runtime.contains("if !hideControls"));
    assert!(!image_runtime.contains(".background(backgroundColor.opacity(0.72))"));
    assert!(!image_runtime.contains(".aspectRatio(doweImageAspect(aspect)"));
    assert!(views.contains("DoweAccordionView(multiple:"));
    assert!(views.contains("defaultOpenIds: [\"intro\"]"));
    assert!(views.contains("{ openIds, toggleItem in"));
    assert!(views.contains("open: openIds.contains(\"intro\")"));
    assert!(views.contains(
        "@ViewBuilder let content: (Set<String>, @escaping (String) -> Void) -> Content"
    ));
    assert!(views.contains("arrowIcon: {"));
    assert!(views.matches("m19.704 12l-8.491-8.727a.75.75").count() >= 2);
    assert!(!views.contains("__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__"));
    assert!(views.contains(".rotationEffect(open ? .degrees(90) : .degrees(0))"));
    assert!(views.contains("Text(label)\n                        .font(.system(size: CGFloat(15), weight: .bold))\n                        .foregroundStyle(contentColor)"));
    assert!(
        views.contains("variant == \"ghost\" || variant == \"line\" ? CGFloat(0) : CGFloat(8)")
    );
    assert!(
        views.contains("variant == \"ghost\" || variant == \"line\" ? CGFloat(0) : CGFloat(4)")
    );
    assert!(views.contains(".frame(maxWidth: .infinity, alignment: .leading)"));
    assert!(views.contains("borderStyle: \"separator\""));
    assert!(views.contains("if borderStyle == \"separator\""));
    assert!(views.contains("radius: CGFloat(0), action: { toggleItem(\"intro\") }"));
    assert!(!views.contains("radius: if variant == \"ghost\""));
    assert!(!views.contains("Button(playing ? \"Pause\" : \"Play\")"));
    assert!(!views.contains("Text(open ? \"^\" : \"v\")"));
    assert!(views.contains("DoweCarouselView(variant: \"snapping\""));
    assert!(views.contains("ScrollView(.horizontal"));
    assert!(views.contains("showsIndicators: false"));
    assert!(views.contains("if showNavigation"));
    assert!(views.contains(".disabled(disableLoop && currentIndex == 0)"));
    assert!(views.contains("containerRelativeFrame(.horizontal"));
    assert!(views.contains("carouselHorizontalOffset"));
    assert!(views.contains(".scrollPosition(id: $scrollId)"));
    assert!(views.contains(".onChange(of: scrollId) { _, value in"));
    assert!(!views.contains(".onChange(of: scrollId) { value in"));
    assert!(views.contains(".scrollTransition(.interactive, axis: .horizontal)"));
    assert!(views.contains("rotation3DEffect"));
    assert!(views.contains("nonisolated private func carouselRotation(_ phase: Double) -> Double"));
    assert!(views.contains("nonisolated private func carouselScale(_ phase: Double) -> CGFloat"));
    assert!(views.contains("nonisolated private func carouselTilt(_ phase: Double) -> Double"));
    assert!(views.contains("nonisolated private func carouselOffset(_ phase: Double) -> CGFloat"));
    assert!(views.contains("nonisolated private func carouselOpacity(_ phase: Double) -> Double"));
    let carousel_runtime = views
        .split("struct DoweCarouselView<Content: View>: View")
        .nth(1)
        .expect("carousel runtime")
        .split("struct DoweCarouselSlideView<Content: View>: View")
        .next()
        .expect("carousel body");
    assert!(!carousel_runtime.contains("ScrollViewReader"));
    assert!(!carousel_runtime.contains("proxy.scrollTo"));
    assert!(carousel_runtime.contains("withAnimation { scrollId = slideIds[next] }"));
    for variant in [
        "coverFlow",
        "stories",
        "smartStack",
        "cardStack",
        "flipbook",
        "masonry",
        "rtl",
        "controls",
        "dots",
        "thumbnails",
    ] {
        assert!(views.contains(variant));
    }
    assert!(views.contains("DoweCheckboxView(checked:"));
    assert!(views.contains("DoweColorField(value:"));
    assert!(views.contains("DoweDateField(value:"));
    assert!(views.contains("DoweDateRangeField(startValue:"));
    assert!(views.contains("DoweColorField(value:") && views.contains("fontSize: doweTextSize(viewportWidth, min: CGFloat(12), preferredBase: CGFloat(11.2), preferredViewport: CGFloat(0.2), max: CGFloat(14))"));
    assert!(views.contains("DoweDateField(value:") && views.contains("fontSize: doweTextSize(viewportWidth, min: CGFloat(16), preferredBase: CGFloat(15.2), preferredViewport: CGFloat(0.3), max: CGFloat(18))"));
    assert!(views.contains("let fontSize: CGFloat"));
    assert!(views.contains("case \"sm\":\n        return CGFloat(32)"));
    assert!(
        views.contains("minHeight: doweControlHeight(size) + (floating ? CGFloat(8) : CGFloat(0))")
    );
    assert!(views.contains("DoweDateCalendar("));
    assert!(views.contains("DoweAnchoredPopoverPresenter("));
    assert!(views.contains("DoweRadioGroupView(value:"));
    assert!(views.contains("orientation: \"horizontal\""));
    assert!(views.contains("DoweToggleView(checked:"));
    assert!(views.contains("struct DoweSliderView: View"));
    assert!(views.contains("Image(systemName: \"checkmark\")"));
    assert!(views.contains("private struct DoweColorPickerPanel: View"));
    assert!(views.contains("doweColorFromHsv(hue, saturation, brightness)"));
    assert!(views.contains("doweColorCmykText(doweColorRgb(value))"));
    assert!(views.contains("doweColorOklchText(doweColorRgb(value))"));
    assert!(
        views
            .contains("DoweAnchoredPopoverPresenter(isPresented: expanded, minWidth: CGFloat(300)")
    );
    assert!(views.contains("trigger\n                    .allowsHitTesting(false)"));
    assert!(views.contains(".padding(.leading, doweControlSwatchSize(size) + CGFloat(10))"));
    assert!(
        views.contains("Button(action: { expanded.toggle() }) {\n                    Color.clear")
    );
    assert!(views.contains(".zIndex(expanded ? 1000 : 0)"));
    assert!(!views.contains("TextField(\"Start\", text: startValue)"));
    assert!(views.contains("DoweRadioOptionView(value:"));
    assert!(views.contains(".tint(accentColor)"));
    assert!(views.contains("func boolBinding(_ path: String) -> Binding<Bool>"));
    assert!(!views.contains("DoweSimpleField"));
}

