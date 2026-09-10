#[test]
fn generates_swiftui_layout_bars() {
    let output = generate_ios(
        &[bar_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("ZStack {"));
    assert!(views.contains("HStack(alignment: .center, spacing: 0)"));
    assert!(views.contains("Text(verbatim: \"Brand\")"));
    assert!(views.contains("Text(verbatim: \"Directory\")"));
    assert!(views.contains("Text(verbatim: \"Copyright\")"));
    let directory = views
        .find("Text(verbatim: \"Directory\")")
        .expect("Footer top");
    let copyright = views[directory..]
        .find("Text(verbatim: \"Copyright\")")
        .map(|index| directory + index)
        .expect("Footer bottom");
    let footer_boxed = views[copyright..]
        .find(".frame(maxWidth: CGFloat(1536), alignment: .center)")
        .map(|index| copyright + index)
        .expect("Footer boxed regions");
    assert!(directory < copyright);
    assert!(copyright < footer_boxed);
    assert!(views.contains(".background(DoweDesign.surface)"));
    assert!(views.contains(".foregroundStyle(DoweDesign.surfaceText)"));
    assert!(views.contains(".zIndex(1)"));
    assert!(views.contains(".clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))"));
    assert!(views.contains(
            ".overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))"
        ));
    assert!(
        !views.contains(".overlay(Rectangle().fill(DoweDesign.muted).frame(height: CGFloat(1))")
    );
    assert!(!views.contains(
            ".overlay(RoundedRectangle(cornerRadius: CGFloat(0)).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))"
        ));
    assert!(views.contains(".padding(.horizontal, CGFloat(16))"));
    assert_eq!(
        views
            .matches(".frame(maxWidth: CGFloat(1536), alignment: .center)")
            .count(),
        3
    );
    assert_eq!(
        views
            .matches(".frame(minWidth: CGFloat(0), maxWidth: .infinity, minHeight: CGFloat(48), alignment: .center)")
            .count(),
        3
    );
    assert!(views.contains("itemSize: CGFloat(56)"));
    assert!(views.contains("backgroundColor: DoweDesign.primary"));
    assert!(views.contains("featured: true"));
}

#[test]
fn generates_scroll_docking_appbar_for_swiftui() {
    let output = generate_ios(
        &[docking_appbar_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("DoweDockingScaffold"));
    assert!(views.contains("@StateObject private var state = DoweDockingState()"));
    assert!(views.contains(".environment(\\.doweDockingState, state)"));
    assert!(views.contains("state.scrollOffset > CGFloat(100)"));
    assert!(views.contains("struct DoweDockingScrollObserver: UIViewRepresentable"));
    assert!(
        views.contains("fileprivate func connect(from view: UIView, state: DoweDockingState?)")
    );
    assert!(views.contains("scrollView.observe(\\.contentOffset"));
    assert!(views.contains("scrollView.contentOffset.y + scrollView.adjustedContentInset.top"));
    assert!(views.contains(".background(DoweDockingScrollObserver())"));
    assert!(!views.contains("DoweDockingScrollOffsetKey"));
    assert!(!views.contains("DoweDockingScrollOffsetReader"));
    assert!(!views.contains("doweDockingScroll"));
    assert!(views.matches(".layoutPriority(1)").count() >= 2);
    assert_eq!(
        views
            .matches(".fixedSize(horizontal: true, vertical: false)")
            .count(),
        3
    );
    assert!(views.contains("DoweDockingAppBarModifier"));
    assert!(views.contains(".timingCurve(0.4, 0, 0.2, 1, duration: 0.3)"));
    assert!(views.contains("reduceMotion ? nil"));
    assert!(views.contains("docked ? CGFloat(0) : CGFloat(16)"));
    assert!(views.contains("docked ? CGFloat(0) : CGFloat(8)"));
}

#[test]
fn generates_swiftui_nonfloating_bar_without_divider() {
    let output = generate_ios(
        &[appbar_divider_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(
        !views.contains(".overlay(Rectangle().fill(DoweDesign.muted).frame(height: CGFloat(1))")
    );
    assert!(!views.contains(
            ".overlay(RoundedRectangle(cornerRadius: CGFloat(0)).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))"
        ));
}

#[test]
fn generates_swiftui_side_nav() {
    let output = generate_ios(
        &[side_nav_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweSideNavRow<Content: View>: View"));
    assert!(views.contains("struct DoweSideNavEntry: Identifiable"));
    assert!(views.contains("struct DoweSideNavIcon"));
    assert!(views.contains("let color: Color?"));
    assert!(views.contains("DoweSideNav(items: ["));
    assert!(views.contains("titleColor: DoweDesign.surface"));
    assert!(views.contains(
        ".foregroundStyle(header ? titleColor : (item.path == activePath ? contentColor : DoweDesign.backgroundText))"
    ));
    assert!(views.contains("row(item, header: false, action: nil, expanded: expanded)"));
    assert!(views.contains("kind: \"submenu\""));
    assert!(
        views.contains("DoweSideNavSubmenu(stateKey: stateKey + \":\" + item.id, open: item.open, bordered: item.bordered, wide: wide)")
    );
    assert!(views.contains("DoweSideNavRow(active: item.path == activePath"));
    assert!(views.contains(".frame(maxWidth: wide ? .infinity : nil, alignment: .leading)"));
    assert!(views.contains(".frame(maxWidth: wide ? .infinity : nil, alignment: .leading)\n        .contentShape(Rectangle())\n        .background(active ? backgroundColor : Color.clear)"));
    assert!(views.contains("label(expanded)\n                    .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)\n                    .contentShape(Rectangle())"));
    assert!(views.contains(".buttonStyle(.plain)\n            .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)\n            .contentShape(Rectangle())"));
    assert!(views.contains("struct DoweSideNavArrow: View"));
    assert!(views.contains("final class DoweSideNavMemory"));
    assert!(views.contains("DoweSideNavMemory.shared.value(for: stateKey, initial: open)"));
    assert!(views.contains("DoweSideNavMemory.shared.set(next, for: stateKey)"));
    assert!(views.contains("m19.704 12l-8.491-8.727a.75.75"));
    assert!(views.contains("withAnimation(.easeInOut(duration: 0.18))"));
    assert!(views.contains(".transition(.opacity)"));
    assert!(!views.contains(".transition(.opacity.combined(with: .move(edge: .top)))"));
    assert!(views.contains(
        "VStack(alignment: .leading, spacing: CGFloat(0)) {\n                if expanded {"
    ));
    assert!(views.contains("                    .transition(.opacity)\n                }\n            }\n            .clipped()"));
    assert!(views.contains(".frame(maxWidth: wide ? .infinity : nil, alignment: .leading)\n        .clipped()\n        .animation(.easeInOut(duration: 0.18), value: expanded)"));
    assert!(views.contains("VStack(alignment: .leading, spacing: CGFloat(2)) {\n                        content\n                    }\n                    .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)"));
    assert!(!views.contains(
        "content\n                    .padding(.leading, bordered ? CGFloat(8) : CGFloat(0))"
    ));
    assert!(views.contains("label: \"Workspace\""));
    assert!(views.contains("label: \"Blogs\""));
    assert!(views.contains("gap: CGFloat(10)"));
    assert!(views.contains("struct DoweSideNavStatus: View"));
    assert!(views.contains("status: \"2\""));
    assert!(views.contains("DoweSideNavStatus(text: status"));
    assert!(views.contains(".padding(.horizontal, CGFloat(8))"));
    assert!(views.contains(".background(DoweDesign.muted)"));
    assert!(views.contains(".foregroundStyle(DoweDesign.mutedText)"));
    assert!(views.contains("icon: DoweSideNavIcon(viewBox: DoweSvgViewBox"));
    assert!(views.contains("color: nil"));
    assert!(views.contains("color: icon.color ?? (header ? titleColor : (item.path == activePath ? activeContentColor : DoweDesign.backgroundText))"));
    assert!(views.contains(
        "DoweSvgView(viewBox: icon.viewBox, color: icon.color, paths: icon.paths, animated: icon.animated)"
    ));
    assert!(views.contains("wide: state.bool(\"wideEnabled\", fallback: false)"));
}

#[test]
fn emits_generic_style_binding_on_ios() {
    let mut route = side_nav_route();
    if let ViewNode::SideNav { props, .. } = &mut route.page_tree {
        props.style.style.bg_binding = Some(dowe_components::PropBinding::string("item.fill"));
    }
    let views = swift_content(&generate_ios(&[route], &FontConfig::default(), &DesignConfig::default(), &[]));
    assert!(views.contains("state.text(\"item.fill\")"));
}

#[test]
fn emits_generic_variant_bindings_on_ios() {
    let mut route = side_nav_route();
    let ViewNode::SideNav { props, .. } = &mut route.page_tree else {
        panic!("expected side nav route");
    };
    props.style.reactive.variant = Some("item.variant".to_string());
    props.style.reactive.scheme = Some("theme.scheme".to_string());
    props.style.reactive.size = Some("item.size".to_string());
    let views = swift_content(&generate_ios(&[route], &FontConfig::default(), &DesignConfig::default(), &[]));
    assert!(views.contains("state.text(\"item.variant\", fallback: \"solid\")"));
    assert!(views.contains("state.text(\"theme.scheme\", fallback: \"primary\")"));
    assert!(views.contains("state.text(\"item.size\", fallback: \"md\")"));
}

#[test]
fn generates_reactive_swiftui_side_nav_header_color_from_scheme() {
    let mut route = side_nav_route();
    let ViewNode::SideNav { props, .. } = &mut route.page_tree else {
        panic!("expected side nav route");
    };
    props.style.reactive.scheme = Some("schemeChoice".to_string());

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains(
        "titleColor: doweSideNavHeaderColor(state.text(\"schemeChoice\", fallback: \"muted\"))"
    ));
}

