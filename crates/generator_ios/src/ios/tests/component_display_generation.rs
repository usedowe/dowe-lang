#[test]
fn generates_swiftui_display_overlay_components() {
    let output = generate_ios(
        &[display_overlay_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweAvatar<Icon: View>: View"));
    assert!(views.contains("DoweAvatar(source: nil, name: \"Ada\""));
    assert!(views.contains("DoweBadge(text: \"3\", position: \"bottom-right\""));
    assert!(views.contains("DoweChip(text: \"Filter\", size: \"sm\""));
    assert!(views.contains("DoweSkeleton(variant: \"rounded\", animation: \"pulse\")"));
    assert!(views.contains("private let pathBuilder: @Sendable (CGRect) -> Path"));
    assert!(views.contains("DoweModal(open: state.bool(\"modal01\")"));
    assert!(views.contains("DoweAlertDialog(open: state.bool(\"modal01\")"));
    assert!(views.contains("DoweTooltip(label: \"More actions\", position: \"end\""));
    assert!(views.contains("DoweToast(visible: true, title: \"Saved\""));
    assert!(views.contains("DoweDropdown(backgroundColor: DoweDesign.surface"));
    assert!(views.contains("DoweCommand(open: state.bool(\"modal01\")"));
}

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
    assert!(views.contains("DoweAvatarGroup(items: doweAvatarGroupItems(state.rows(\"people\")"));
    assert!(views.contains("DoweChatBox(state: state, messagesPath: \"messages\""));
    assert!(views.contains("DoweEmpty(kind: \"result\""));
    assert!(views.contains("DoweMarquee(speed: \"fast\""));
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
    assert!(views.contains("DoweRichText(marks: [DoweRichTextMark(text: \"Launch\""));
    assert!(views.contains("], font: .inter, fontSize:"));
    assert!(!views.contains("DoweRichText(marks: [DoweRichTextMark(text: \"Launch\", style: \"grad\", color: DoweDesign.primary), DoweRichTextMark(text: \"ready\", style: \"pill\", color: DoweDesign.success)], font: doweFont("));
    assert!(views.contains("DoweRecord(name: \"voice\""));
    assert!(views.contains("DoweToggleGroup(value: state.binding(\"mode\""));
    assert!(views.contains("DoweCollapsible(label: \"Details\""));
    assert!(views.contains("DoweCountdown(target: \"2030-01-01T00:00:00Z\""));
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
        "routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets))"
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
        "private func routeContent(_ entry: DoweRouteEntry, viewportWidth: CGFloat) -> some View"
    ));
    assert!(
        views.contains("LoginView(viewportWidth: viewportWidth, activeFragment: entry.fragment")
    );
    assert!(
        views.contains(".frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)")
    );
    assert!(!views.contains("safeAreaInsets:"));
    assert!(!views.contains("let safeAreaInsets"));
    assert!(!views.contains(".padding(.top, safeAreaInsets.top)"));
    assert!(!views.contains(".padding(.bottom, safeAreaInsets.bottom)"));
    assert!(views.contains(
            "        .background(DoweDesign.background)\n        .foregroundStyle(DoweDesign.onBackground)"
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
            "LazyVGrid(columns: doweGridColumns(doweResponsive(viewportWidth, xs: 1, md: 2) ?? 1, spacing: doweResponsive(viewportWidth, xs: CGFloat(16))),"
        ));
    assert!(
        views.contains("DoweInputField(value: nil, label: nil, placeholder: \"\", floating: false")
    );
    assert!(views.contains("minHeight: CGFloat(40), horizontalPadding: CGFloat(12)"));
    assert!(views.contains(
            "backgroundColor: Color.clear, contentColor: DoweDesign.secondary, borderColor: Optional(DoweDesign.muted)"
        ));
    assert!(views.contains(".foregroundStyle(DoweDesign.onSoftMuted)"));
    assert!(views.contains(".background(DoweDesign.surface)"));
    assert!(views.contains(".foregroundStyle(DoweDesign.onSurface)"));
    assert!(views.contains(".stroke(DoweDesign.surface, lineWidth: CGFloat(1))"));
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
    assert!(views.contains(
        r#"DoweInputField(value: nil, label: "Name", placeholder: "Full name", floating: true"#
    ));
    assert!(views.contains("let value: Binding<String>?"));
    assert!(views.contains("@State private var localText = \"\""));
    assert!(views.contains("private var visiblePlaceholder: String"));
    assert!(views.contains("TextField(visiblePlaceholder, text: textBinding)"));
    assert!(views.contains("struct DoweSelectField: View"));
    assert!(views.contains("struct DoweSelectPopover: View"));
    assert!(views.contains("struct DoweSelectAnchorPresenter: UIViewRepresentable"));
    assert!(views.contains("UIHostingController<DoweSelectPopover>"));
    assert!(views.contains("UIView.animate(withDuration: 0.16"));
    assert!(views.contains("UIView.animate(withDuration: 0.14"));
    assert!(views.contains("anchor.convert(anchor.bounds, to: window)"));
    assert!(views.contains("window.addSubview(controller.view)"));
    assert!(views.contains("let contentHeight = CGFloat(8) + parent.options.reduce(CGFloat(0))"));
    assert!(views.contains("option.description == nil ? CGFloat(40) : CGFloat(58)"));
    assert!(views.contains("let height = min(maxHeight, max(CGFloat(44), contentHeight))"));
    assert!(!views.contains("max(size.height, estimated)"));
    assert!(views.contains("@State private var expanded = false"));
    assert!(views.contains("DoweSelectAnchorPresenter("));
    assert!(views.contains(".contentShape(Rectangle())"));
    assert!(views.contains(".zIndex(expanded ? 1000 : 0)"));
    assert!(!views.contains("Menu {"));
    assert!(!views.contains("Picker("));
    assert!(!views.contains("DoweSelectPortalOverlay"));
    assert!(views.contains(
        r#"DoweSelectField(value: nil, label: "Role", placeholder: "Choose role", floating: true"#
    ));
    assert!(views.contains(
        r#"DoweSelectOption(value: "admin", label: "Admin", description: "Manages users")"#
    ));
    assert!(views.contains("DoweSelectArrow(color: contentColor)"));
    assert!(views.contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4"));
    assert!(views.contains("if selectedOption != nil || !floating || expanded"));
    assert!(views.contains("Text(description).font(.caption)"));
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
    assert!(views.contains("DoweAudioView(source:"));
    assert!(views.contains("struct DoweImageView: View"));
    assert!(views.contains("DoweAccordionView(multiple:"));
    assert!(views.contains("DoweCarouselView(autoplay:"));
    assert!(views.contains("DoweCheckboxView(checked:"));
    assert!(views.contains("DoweColorField(value:"));
    assert!(views.contains("DoweDateField(value:"));
    assert!(views.contains("DoweDateRangeField(startValue:"));
    assert!(views.contains("DoweRadioGroupView(value:"));
    assert!(views.contains("DoweToggleView(checked:"));
    assert!(views.contains("Image(systemName: \"checkmark\")"));
    assert!(views.contains("doweColorFromHex(value.wrappedValue"));
    assert!(views.contains("DoweInputField(value: value"));
    assert!(views.contains("TextField(\"Start\", text: startValue)"));
    assert!(views.contains("Circle()"));
    assert!(views.contains(".tint(accentColor)"));
    assert!(views.contains("func boolBinding(_ path: String) -> Binding<Bool>"));
    assert!(!views.contains("DoweSimpleField"));
}

#[test]
fn generates_fragment_aware_native_history_and_deep_links() {
    let output = generate_ios(
        &[index_route_with_navigation(), signup_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    let routing = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweRouting.swift"))
        .expect("routing");

    assert!(views.contains("struct DoweRouteEntry: Hashable"));
    assert!(views.contains("@State private var rootEntry = DoweRouteEntry"));
    assert!(views.contains("@State private var navigationPath: [DoweRouteEntry] = []"));
    assert!(views.contains(
        "routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets))"
    ));
    assert!(views.contains(".simultaneousGesture(backSwipeGesture)"));
    assert!(!views.contains("NavigationStack(path: $navigationPath)"));
    assert!(!views.contains(".navigationDestination(for: DoweRouteEntry.self)"));
    assert!(!views.contains(".toolbar(.hidden, for: .navigationBar)"));
    assert!(!views.contains(".navigationBarHidden(true)"));
    assert!(views.contains("private var backSwipeGesture: some Gesture"));
    assert!(views.contains("navigationPath.append(destination)"));
    assert!(views.contains("navigationPath.removeLast()"));
    assert!(views.contains(
        "private func navigate(_ operation: String, _ target: String, _ fragment: String?)"
    ));
    assert!(views.contains(r#"{ navigate("push", "/signup", "join") }"#));
    assert!(views.contains(r#"{ navigate("replace", "", "hero") }"#));
    assert!(views.contains("{ goBack() }"));
    assert!(views.contains(r#"navigate("replace", path, url.fragment)"#));
    assert!(views.contains("ScrollViewReader { proxy in"));
    assert!(views.contains("doweScroll(proxy, activeFragment)"));
    assert!(
        views.contains(".onChange(of: activeFragment) { _, value in doweScroll(proxy, value) }")
    );
    assert!(!views.contains(".onChange(of: activeFragment) { value in doweScroll(proxy, value) }"));
    assert!(views.contains(".id(\"hero\")"));
    assert!(
        routing
            .content
            .contains("static let sections: [String: [String]]")
    );
    assert!(routing.content.contains(r#""/signup": ["join"]"#));
}

#[test]
fn generates_swiftui_svg_views() {
    let output = generate_ios(
        &[svg_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweSvgView: View"));
    assert!(views.contains("DoweSvgViewBox(minX: CGFloat(0), minY: CGFloat(0), width: CGFloat(24), height: CGFloat(24))"));
    assert!(views.contains("DoweSvgFill.currentColor"));
    assert!(views.contains(
        "doweResponsive(viewportWidth, xs: DoweDesign.tertiary) ?? DoweDesign.onBackground"
    ));
    assert!(views.contains("DoweSvgPathParser(data)"));
    assert!(views.contains("c.253.847.1 1.895-.62 2.618a.75.75"));
    assert!(views.contains("if characters[index] == \"-\" || characters[index] == \"+\""));
    assert!(views.contains(
            ".frame(width: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(views.contains(
            ".frame(maxWidth: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(views.contains(
            ".frame(height: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(views.contains(
            ".frame(maxHeight: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(!views.contains(", maxWidth: doweMaxSize("));
    assert!(!views.contains(", maxHeight: doweMaxSize("));
}

#[test]
fn generates_swiftui_view_motion() {
    let output = generate_ios(
        &[motion_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("enum DoweAnimationPreset"));
    assert!(views.contains(".modifier(DoweAnimationModifier(preset: .fadeIn))"));
    assert!(views.contains(".modifier(DoweAnimationModifier(preset: .slideUp))"));
    assert!(views.contains(".animation(.easeOut(duration: 0.22), value: active)"));
}
