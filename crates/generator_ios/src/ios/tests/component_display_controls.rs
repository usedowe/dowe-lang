#[test]
fn generates_swiftui_display_chat_and_motion_components() {
    let output = generate_ios(
        &[display_chat_motion_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweAvatarGroup: View"));
    assert!(views.contains("return Array(items.prefix(max(1, maxCount)))"));
    assert!(!views.contains("maxCount - 1"));
    assert!(views.contains(
        "UIImage(named: source.trimmingCharacters(in: CharacterSet(charactersIn: \"/\")))"
    ));
    assert!(views.contains("DoweAvatarGroup(items: doweAvatarGroupItems(state.rows(\"people\")"));
    assert!(views.contains("DoweChatBox(state: state, messagesPath: \"messages\""));
    assert!(views.contains("DoweEmpty(kind: \"result\""));
    assert!(views.contains("iconViewBox: DoweSvgViewBox(minX: CGFloat(0), minY: CGFloat(0), width: CGFloat(24), height: CGFloat(24))"));
    assert!(views.contains("iconPaths: [DoweSvgPathData("));
    assert!(!views.contains("struct DoweEmptyIcon"));
    assert!(views.contains("DoweMarquee(speed: \"fast\""));
    assert!(views.contains(
        "withAnimation(.linear(duration: marqueeDuration).repeatForever(autoreverses: false))"
    ));
    assert!(!views.contains("Task.sleep(nanoseconds: 16_000_000)"));
    assert!(views.contains("DoweTypeWriter(texts: [\"Hello\", \"World\"]"));
}

#[test]
fn generates_swiftui_rich_control_map_components() {
    let output = generate_ios(
        &[rich_control_map_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweRichText: View"));
    assert!(views.contains("DoweRichTextLayout(gap: CGFloat(4))"));
    let rich_text_runtime = views
        .split("struct DoweRichText: View")
        .nth(1)
        .expect("rich text runtime")
        .split("private struct DoweRichTextRun: View")
        .next()
        .expect("rich text run after runtime");
    assert!(rich_text_runtime.contains(".frame(maxWidth: .infinity, alignment: .center)"));
    assert!(views.contains("private struct DoweRichTextLayout: Layout"));
    assert!(views.contains("let ideal = subview.sizeThatFits(.unspecified)"));
    assert!(views.contains("let constrainedWidth = min(ideal.width, width)"));
    assert!(
        views.contains(
            "subview.sizeThatFits(ProposedViewSize(width: constrainedWidth, height: nil))"
        )
    );
    assert!(views.contains(
        "let resolvedWidth = proposal.width.map { min(contentWidth, $0) } ?? contentWidth"
    ));
    assert!(views.contains("private struct DoweRichTextRun: View"));
    assert!(views.contains(".multilineTextAlignment(.center)"));
    assert!(views.contains(".fixedSize(horizontal: false, vertical: true)"));
    assert!(views.contains("DoweRichText(marks: [DoweRichTextMark(text: \"Launch\", style: \"grad\", scheme: \"primary\")"));
    assert!(views.contains("], font: .inter, fontSize:"));
    assert!(views.contains("contentColor: DoweDesign.backgroundText"));
    assert!(views.contains("RoundedRectangle(cornerRadius: CGFloat(2)).fill(accent)"));
    assert!(views.contains("doweButtonTextFamily(mark.scheme)"));
    assert!(views.contains("DoweRecord(name: \"voice\""));
    assert!(views.contains("DoweToggleGroup(value: state.binding(\"mode\""));
    assert!(views.contains("DowePagination(value: state.binding(\"page\""));
    assert!(views.contains(
        "pageCount: max(1, min(25, (max(0, Int(state.text(\"total\")) ?? 0) + 59) / 60))"
    ));
    assert!(views.contains("previousIcon: {"));
    assert!(views.contains("nextIcon: {"));
    assert!(views.contains("DoweCollapsible(label: \"Details\""));
    assert!(views.contains("arrowIcon: {"));
    let collapsible_runtime = views
        .split("struct DoweCollapsible<")
        .nth(1)
        .expect("collapsible runtime")
        .split("struct DoweCountdown")
        .next()
        .expect("countdown after collapsible");
    assert!(!collapsible_runtime.contains("Image(systemName: \"chevron.down\")"));
    assert!(views.contains("DoweCountdown(target: \"2030-01-01T00:00:00Z\""));
    assert!(views.contains("ScrollView(.horizontal)"));
    assert!(views.contains("ViewThatFits(in: .horizontal)"));
    assert!(views.contains(".frame(maxWidth: .infinity, alignment: .center)"));
    assert!(views.contains("countdownContent(displaySize: \"sm\")"));
    assert!(views.contains(
        ".frame(minWidth: metrics(for: displaySize).1, minHeight: metrics(for: displaySize).2)"
    ));
    assert!(views.contains("if targetDate <= value && !completed"));
    assert!(views.contains("ISO8601DateFormatter().date(from: target) ?? .distantPast"));
    assert!(views.contains("DoweMap(centerLat: \"4.7109\", centerLng: \"-74.0721\""));
    assert!(views.contains("DoweMapMarker(id: \"office\""));
}

#[test]
fn generates_full_scene_background_without_unsafe_content() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains(".background(DoweDesign.background.ignoresSafeArea())"));
    assert!(views.contains("struct DoweSafeAreaReporter: UIViewRepresentable"));
    assert!(views.contains("final class DoweSafeAreaReportingView: UIView"));
    assert!(views.contains("@State private var safeAreaInsets = EdgeInsets()"));
    assert!(views.contains("DoweSafeAreaReporter { insets in"));
    assert!(views.contains(
        "routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets), viewportHeight: doweSafeAreaHeight(geometry, safeAreaInsets))"
    ));
    assert!(views.contains(
        ".frame(width: doweSafeAreaWidth(geometry, safeAreaInsets), height: doweSafeAreaHeight(geometry, safeAreaInsets), alignment: .topLeading)"
    ));
    assert!(views.contains(
        ".frame(width: doweSafeAreaWidth(geometry, safeAreaInsets), height: doweSafeAreaHeight(geometry, safeAreaInsets), alignment: .topLeading)\n                .clipped()\n                .offset(x: safeAreaInsets.leading, y: safeAreaInsets.top)"
    ));
    assert!(views.contains(".offset(x: safeAreaInsets.leading, y: safeAreaInsets.top)"));
    assert!(views.contains("        .ignoresSafeArea()\n        .frame(maxWidth: .infinity"));
    assert!(views.contains(
        "func doweSafeAreaWidth(_ geometry: GeometryProxy, _ insets: EdgeInsets) -> CGFloat"
    ));
    assert!(views.contains(
        "func doweSafeAreaHeight(_ geometry: GeometryProxy, _ insets: EdgeInsets) -> CGFloat"
    ));
    assert!(views.contains("func doweInsetsEqual(_ lhs: EdgeInsets, _ rhs: EdgeInsets) -> Bool"));
    assert!(views.contains(
        "private func routeContent(_ entry: DoweRouteEntry, viewportWidth: CGFloat, viewportHeight: CGFloat) -> some View"
    ));
    assert!(
        views.contains("LoginView(viewportWidth: viewportWidth, viewportHeight: viewportHeight, activeFragment: entry.fragment")
    );
    assert!(
        views.contains(".frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)")
    );
    assert!(!views.contains(
        "LoginView(viewportWidth: viewportWidth, viewportHeight: viewportHeight, activeFragment: entry.fragment, safeAreaInsets:"
    ));
    assert!(!views.contains("let safeAreaInsets ="));
    assert!(!views.contains(".padding(.top, safeAreaInsets.top)"));
    assert!(!views.contains(".padding(.bottom, safeAreaInsets.bottom)"));
    assert!(views.contains(
            "        .background(DoweDesign.background)\n        .foregroundStyle(DoweDesign.backgroundText)"
        ));
    assert!(!views.contains(".ignoresSafeArea()\n        .foregroundStyle"));
}

#[test]
fn generates_portable_grid_controls_and_variant_colors() {
    let output = generate_ios(
        &[parity_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains(
            "DoweGridLayout(tracks: doweResponsive(viewportWidth, xs: [CGFloat(1)], md: [CGFloat(1), CGFloat(1)]) ?? [CGFloat(1)], rowGap: doweResponsive(viewportWidth, xs: CGFloat(16)), columnGap: doweResponsive(viewportWidth, xs: CGFloat(24)), justify: nil, align: nil, fillHeight: false) {"
        ));
    assert!(views.contains("struct DoweGridLayout: Layout"));
    assert!(
        views.contains("DoweInputField(value: nil, label: nil, placeholder: \"\", floating: false")
    );
    assert!(views.contains("minHeight: CGFloat(40), horizontalPadding: CGFloat(12)"));
    assert!(views.contains(
            "backgroundColor: Color.clear, contentColor: DoweDesign.secondary, borderColor: Optional(DoweDesign.muted)"
        ));
    assert!(views.contains(".foregroundStyle(DoweDesign.mutedText)"));
    assert!(views.contains(".background(DoweDesign.surface)"));
    assert!(views.contains(".foregroundStyle(DoweDesign.surfaceText)"));
    assert!(views.contains(".stroke(DoweDesign.surface, lineWidth: CGFloat(1))"));
    assert!(views.contains("HStack(spacing: 8)"));
    assert!(views.contains("DoweSvgView(viewBox: DoweSvgViewBox(minX: CGFloat(0), minY: CGFloat(0), width: CGFloat(24), height: CGFloat(24)), color: DoweDesign.primary"));
    assert!(!views.contains("doweButtonHorizontalPadding(\"md\")"));
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageParityView.swift"))
        .expect("parity page");
    let action = page
        .content
        .find("Text(verbatim: \"Action\")")
        .expect("button label");
    let button_tail = &page.content[action..];
    let frame = button_tail
        .find(".frame(maxWidth: .infinity, alignment: .center)")
        .expect("grid button frame");
    let background = button_tail
        .find(".background(Color.clear)")
        .expect("outlined button background");
    assert!(frame < background);
    assert!(button_tail.contains(".foregroundStyle(DoweDesign.primary)"));
}

#[test]
fn preserves_static_button_variants_with_reactive_scheme_on_ios() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "scheme01".to_string(),
            name: "schemeChoice".to_string(),
            storage_key: "schemeChoice".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::String("primary".to_string()),
            schema: None,
        }],
        actions: Vec::new(),
        children: [
            ComponentVariant::Solid,
            ComponentVariant::Soft,
            ComponentVariant::Outlined,
            ComponentVariant::Ghost,
        ]
        .into_iter()
        .map(|variant| ViewNode::Button {
            props: VariantProps {
                variant: Some(variant),
                reactive: ReactiveVariantProps {
                    scheme: Some("schemeChoice".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Action")],
        })
        .collect(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();

    assert!(generated.contains("doweButtonContainer(\"solid\", state.text("));
    assert!(generated.contains("doweButtonContainer(\"soft\", state.text("));
    assert!(generated.contains("doweButtonContainer(\"outlined\", state.text("));
    assert!(generated.contains("doweButtonContainer(\"ghost\", state.text("));
    assert!(generated.contains("lineWidth: \"outlined\" == \"outlined\" ? CGFloat(1)"));
}

#[test]
fn generates_swiftui_ghost_card_without_variant_border() {
    let output = generate_ios(
        &[ViewRoute {
            id: "cards".to_string(),
            route_path: "/cards".to_string(),
            layout_tree: ViewNode::Children,
            page_tree: ViewNode::Box {
                props: Default::default(),
                children: vec![
                    ViewNode::Card {
                        props: VariantProps {
                            variant: Some(ComponentVariant::Outlined),
                            color: Some(ColorFamily::Info),
                            ..Default::default()
                        },
                        children: vec![text("Outlined")],
                    },
                    ViewNode::Card {
                        props: VariantProps {
                            variant: Some(ComponentVariant::Ghost),
                            color: Some(ColorFamily::Warning),
                            ..Default::default()
                        },
                        children: vec![text("Ghost")],
                    },
                ],
            },
            sections: Vec::new(),
            navigation_actions: Vec::new(),
        }],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageCardsView.swift"))
        .expect("cards page");
    let outlined = page
        .content
        .find("Text(verbatim: \"Outlined\")")
        .expect("outlined card");
    let ghost = page
        .content
        .find("Text(verbatim: \"Ghost\")")
        .expect("ghost card");
    let outlined_card = &page.content[outlined..ghost];
    let ghost_card = &page.content[ghost..];

    assert!(outlined_card.contains(".stroke(DoweDesign.info, lineWidth: CGFloat(1))"));
    assert!(ghost_card.contains(".background(Color.clear)"));
    assert!(ghost_card.contains(".foregroundStyle(DoweDesign.warning)"));
    assert!(!ghost_card.contains(".stroke("));
}

#[test]
fn generates_labeled_input_and_select_fields() {
    let output = generate_ios(
        &[form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweInputField: View"));
    assert!(views.contains("let startIcon: DoweControlIcon?\n"));
    assert!(views.contains("let endIcon: DoweControlIcon?\n"));
    let input = views
        .lines()
        .find(|line| line.contains("DoweInputField(value: nil"))
        .expect("generated input field");
    assert!(input.contains("borderWidth: CGFloat(1)"), "{input}");
    assert!(input.contains("fontSize: doweTextSize(viewportWidth, min: CGFloat(12), preferredBase: CGFloat(11.2), preferredViewport: CGFloat(0.2), max: CGFloat(14))"), "{input}");
    assert!(views.contains(
        r#"DoweInputField(value: nil, label: "Name", placeholder: "Full name", floating: true"#
    ));
    assert!(views.contains(
        r#"DoweInputField(value: nil, label: "Name", placeholder: "Full name", floating: true, font:"#
    ));
    assert!(views.contains("minHeight: CGFloat(40), horizontalPadding: CGFloat(12)"));
    let select = views
        .lines()
        .find(|line| line.contains(r#"label: "Role", placeholder: "Choose role""#))
        .expect("generated large select");
    assert!(select.contains("fontSize: doweTextSize(viewportWidth, min: CGFloat(16), preferredBase: CGFloat(15.2), preferredViewport: CGFloat(0.3), max: CGFloat(18))"), "{select}");
    assert!(views.contains("let value: Binding<String>?"));
    assert!(views.contains("@State private var localText = \"\""));
    assert!(views.contains("private var visiblePlaceholder: String"));
    assert!(views.contains("TextField(visiblePlaceholder, text: textBinding)"));
    assert!(views.contains("struct DoweSelectField: View"));
    assert!(views.contains("struct DoweSelectPopover: View"));
    assert!(views.contains("struct DoweSelectAnchorPresenter: View"));
    assert!(
        views.contains("struct DoweAnchoredPopoverPresenter<Content: View>: UIViewRepresentable")
    );
    assert!(views.contains("UIHostingController<Content>"));
    assert!(views.contains("UIView.animate(withDuration: 0.16"));
    assert!(views.contains("UIView.animate(withDuration: 0.14"));
    assert!(views.contains("anchor.convert(anchor.bounds, to: window)"));
    assert!(views.contains("window.addSubview(container)"));
    assert!(views.contains("container.addSubview(scroller)"));
    assert!(views.contains("scroller.addSubview(controller.view)"));
    assert!(views.contains("private var preferredHeight: CGFloat"));
    assert!(views.contains("CGFloat(8) + options.reduce(CGFloat(0))"));
    assert!(views.contains("option.description == nil ? CGFloat(40) : CGFloat(58)"));
    assert!(views.contains("let measuredHeight = parent.preferredHeight ?? measuredSize.height"));
    assert!(views.contains("let contentHeight = max(CGFloat(44), measuredHeight)"));
    assert!(views.contains("let height = min(heightLimit, contentHeight)"));
    let select_popover_start = views
        .find("struct DoweSelectPopover: View")
        .expect("select popover runtime");
    let select_popover_end = views[select_popover_start..]
        .find("struct DoweSelectArrow: View")
        .map(|offset| select_popover_start + offset)
        .expect("select arrow after select popover");
    assert!(!views[select_popover_start..select_popover_end].contains(".shadow("));
    assert!(!views.contains("max(size.height, estimated)"));
    assert!(views.contains("@State private var expanded = false"));
    assert!(views.contains("DoweSelectAnchorPresenter("));
    assert!(views.contains(".contentShape(Rectangle())"));
    assert!(views.contains(".zIndex(expanded ? 1000 : 0)"));
    assert!(views.contains("doweControlHeight(size) + (floating ? CGFloat(8) : CGFloat(0))"));
    assert!(views.contains("ZStack(alignment: floating ? .topLeading : .leading)"));
    assert!(views.contains(".padding(.top, floating ? CGFloat(18) : CGFloat(0))"));
    assert!(!views.contains("Menu {"));
    assert!(!views.contains("Picker(selection:"));
    assert!(!views.contains("DoweSelectPortalOverlay"));
    assert!(views.contains(
        r#"DoweSelectField(value: nil, label: "Role", placeholder: "Choose role", floating: true"#
    ));
    let large_select = views
        .lines()
        .find(|line| line.contains(r#"DoweSelectField(value: nil, label: "Role""#))
        .expect("large floating select");
    assert!(
        large_select.contains("minHeight: CGFloat(56)"),
        "{large_select}"
    );
    assert!(views.contains(
        r#"DoweSelectOption(value: "admin", label: "Admin", description: "Manages users")"#
    ));
    assert!(views.contains("DoweSelectArrow(color: contentColor)"));
    assert!(views.contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4"));
    assert!(views.contains("if selectedOption != nil || !floating || expanded"));
    assert!(views.contains("Text(description).font(.caption)"));
}

