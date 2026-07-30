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
    assert!(views.contains(".background(DoweDesign.surface)"));
    assert!(views.contains(".foregroundStyle(DoweDesign.onSurface)"));
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
            .matches(
                ".frame(maxWidth: .infinity, minHeight: CGFloat(48), alignment: .center)"
            )
            .count(),
        3
    );
    assert!(views.contains("itemSize: CGFloat(56)"));
    assert!(views.contains("backgroundColor: DoweDesign.primary"));
    assert!(views.contains("featured: true"));
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
    assert!(views.contains("DoweSideNav(items: ["));
    assert!(views.contains("kind: \"submenu\""));
    assert!(views.contains("DoweSideNavSubmenu(open: item.open, bordered: item.bordered, wide: wide)"));
    assert!(views.contains("DoweSideNavRow(active: item.path == activePath"));
    assert!(views.contains(".frame(maxWidth: wide ? .infinity : nil, alignment: .leading)"));
    assert!(views.contains(".frame(maxWidth: wide ? .infinity : nil, alignment: .leading)\n        .contentShape(Rectangle())\n        .background(active ? backgroundColor : Color.clear)"));
    assert!(views.contains("label(expanded)\n                    .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)\n                    .contentShape(Rectangle())"));
    assert!(views.contains(".buttonStyle(.plain)\n            .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)\n            .contentShape(Rectangle())"));
    assert!(views.contains("struct DoweSideNavArrow: View"));
    assert!(views.contains("m19.704 12l-8.491-8.727a.75.75"));
    assert!(views.contains("withAnimation(.easeInOut(duration: 0.18))"));
    assert!(views.contains(".transition(.opacity)"));
    assert!(!views.contains(".transition(.opacity.combined(with: .move(edge: .top)))"));
    assert!(views.contains("VStack(alignment: .leading, spacing: CGFloat(0)) {\n                if expanded {"));
    assert!(views.contains("                    .transition(.opacity)\n                }\n            }\n            .clipped()"));
    assert!(views.contains(".frame(maxWidth: wide ? .infinity : nil, alignment: .leading)\n        .clipped()\n        .animation(.easeInOut(duration: 0.18), value: expanded)"));
    assert!(views.contains("VStack(alignment: .leading, spacing: CGFloat(2)) {\n                        content\n                    }\n                    .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)"));
    assert!(!views.contains("content\n                    .padding(.leading, bordered ? CGFloat(8) : CGFloat(0))"));
    assert!(views.contains("label: \"Workspace\""));
    assert!(views.contains("label: \"Blogs\""));
    assert!(views.contains("gap: CGFloat(10)"));
    assert!(views.contains("struct DoweSideNavStatus: View"));
    assert!(views.contains("status: \"2\""));
    assert!(views.contains("DoweSideNavStatus(text: status"));
    assert!(views.contains(".padding(.horizontal, CGFloat(8))"));
    assert!(views.contains(".background(DoweDesign.softMuted)"));
    assert!(views.contains(".foregroundStyle(DoweDesign.onSoftMuted)"));
    assert!(views.contains("icon: DoweSideNavIcon(viewBox: DoweSvgViewBox"));
    assert!(views.contains(
        "DoweSvgView(viewBox: icon.viewBox, color: icon.color, paths: icon.paths, animated: icon.animated)"
    ));
    assert!(views.contains("wide: state.bool(\"wideEnabled\", fallback: false)"));
}

#[test]
fn keeps_explicit_swiftui_side_nav_for_complex_icon_styles() {
    let mut route = side_nav_route();
    let ViewNode::SideNav { items, .. } = &mut route.page_tree else {
        panic!("expected side nav route");
    };
    let SideNavItem::Submenu { items, .. } = &mut items[1] else {
        panic!("expected side nav submenu");
    };
    let icon = items[0].icon.as_mut().expect("submenu icon");
    icon.props.style.rounded = Some(ResponsiveValue::scalar(RoundedSize::Md));

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("DoweSideNavSubmenu(open: true, bordered: true, wide: state.bool(\"wideEnabled\", fallback: false))"));
    assert!(views.contains("DoweSideNavRow(active: activePath == \"/bars\""));
    assert!(views.contains("DoweSvgView(viewBox: DoweSvgViewBox"));
    assert!(!views.contains("DoweSideNav(items: ["));
}

#[test]
fn generates_compact_swiftui_side_nav_with_static_icons() {
    let mut route = side_nav_route();
    route.page_tree = ViewNode::SideNav {
        props: SideNavProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Ghost),
                color: Some(ColorFamily::Muted),
                ..Default::default()
            },
            size: SideNavSize::Md,
            wide: true,
            reactive_wide: None,
        },
        items: (0..80)
            .map(|index| {
                SideNavItem::Item(SideNavItemProps {
                    label: format!("Item {index}"),
                    i18n: None,
                    description: None,
                    description_i18n: None,
                    status: None,
                    status_i18n: None,
                    icon: Some(side_nav_icon()),
                    on_click: None,
                    navigation: Some(NavigationAction::Internal {
                        path: format!("/item-{index}"),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                })
            })
            .collect(),
    };

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("DoweSideNav(items: ["));
    assert!(views.contains("icon: DoweSideNavIcon(viewBox: DoweSvgViewBox"));
    assert!(views.contains("paths: [DoweSvgPathData(data: \"M3 11l9-8 9 8v10H3z\""));
    assert!(views.contains("width: nil, maxWidth: nil, height: nil, maxHeight: nil, minWidth: nil, minHeight: nil"));
    assert!(!views.contains("DoweSideNavRow(active: activePath == \"/item-0\""));
}

#[test]
fn generates_swiftui_rail_nav() {
    let mut rail_route = route();
    rail_route.page_tree = ViewNode::RailNav {
        props: RailNavProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Soft),
                color: Some(ColorFamily::Primary),
                ..Default::default()
            },
            size: SideNavSize::Md,
            show_labels: true,
        },
        items: vec![
            RailNavItem::Item(RailNavItemProps {
                label: "Home".to_string(),
                i18n: None,
                icon: solar_control_icon("home").expect("icon"),
                on_click: None,
                navigation: Some(NavigationAction::Internal {
                    path: "/login".to_string(),
                    fragment: None,
                    operation: NavigationOperation::Push,
                }),
            }),
            RailNavItem::Divider,
        ],
    };
    let output = generate_ios(
        &[rail_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("VStack(alignment: .center, spacing: CGFloat(4))"));
    assert!(views.contains("struct DoweRailNavItem: View"));
    assert!(views.contains("DoweRailNavItem(label: \"Home\", showLabel: true"));
    assert!(views.contains("DoweRailNavIcon(viewBox: DoweSvgViewBox"));
    assert!(views.contains("active: activePath == \"/login\""));
    assert!(views.contains(".accessibilityLabel(label)"));
    assert!(views.contains(".frame(width: itemSize)\n            .frame(minHeight: itemSize)"));
    assert!(!views.contains(".frame(width: itemSize, minHeight: itemSize)"));
    assert!(views.contains(".frame(width: CGFloat(64), alignment: .top)"));
}

#[test]
fn generates_swiftui_navigation_shell_components() {
    let output = generate_ios(
        &[navigation_shell_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("DoweNavMenu(gap:"));
    assert!(views.contains("Text(String(localized: \"home.hero.title\"))"));
    assert!(views.contains("DoweNavMenuItem(active: activePath == \"/\""));
    assert!(views.contains("DoweNavMenuItem(active: openIndex == 1"));
    assert!(views.contains(".rotationEffect(openIndex == 1 ? .degrees(180) : .degrees(0))"));
    assert!(!views.contains("Text(\"⌄\")"));
    assert!(views.contains("HStack(alignment: .top"));
    assert!(views.contains(
        ".frame(maxWidth: CGFloat(1536), maxHeight: .infinity, alignment: .topLeading)"
    ));
    assert!(views.contains(
        ".frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)"
    ));
    assert!(views.contains("Text(verbatim: \"Resource hub\")"));
    assert!(views.contains("label: \"Side Home\""));
    assert!(views.contains(
        ".frame(width: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(384)))), alignment: .leading)"
    ));
    assert!(views.contains(
        ".frame(maxWidth: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(384)))), alignment: .leading)"
    ));
    assert!(views.contains(
        ".frame(maxHeight: UIScreen.main.bounds.height, alignment: .topLeading)"
    ));
    assert!(views.contains(".clipped()"));
    assert!(views.contains("ScrollView {"));
}

#[test]
fn generates_swiftui_tabs() {
    let output = generate_ios(
        &[tabs_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweTabs<Content: View>: View"));
    assert!(views.contains("DoweTabs(items: [DoweTabItem(id: \"overview\", label: \"Overview\"), DoweTabItem(id: \"details\", label: \"Details\")], initialId: \"overview\""));
    assert!(views.contains("position: \"start\", variant: \"line\""));
    assert!(views.contains("backgroundColor: Color.clear"));
    assert!(views.contains("accentColor: DoweDesign.primary"));
    assert!(views.contains("ViewThatFits(in: .horizontal)"));
    assert!(views.contains("ScrollView(.horizontal, showsIndicators: false)"));
    assert!(views.contains("Rectangle().fill(accentColor)"));
    assert!(!views.contains(
        "RoundedRectangle(cornerRadius: tabRadius).stroke(active && selectedLine"
    ));
    assert!(views.contains("if activeTab == \"overview\""));
    assert!(views.contains("Text(verbatim: \"Overview content\")"));
}

#[test]
fn generates_swiftui_drawer() {
    let output = generate_ios(
        &[drawer_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweDrawer<Content: View>: View"));
    assert!(views.contains("struct DoweDrawerPresenter<Content: View>: UIViewRepresentable"));
    assert!(views.contains("window.addSubview(controller.view)"));
    assert!(views.contains("controller.view.translatesAutoresizingMaskIntoConstraints = false"));
    assert!(views.contains("controller.view.leadingAnchor.constraint(equalTo: window.safeAreaLayoutGuide.leadingAnchor)"));
    assert!(views.contains("controller.view.bottomAnchor.constraint(equalTo: window.safeAreaLayoutGuide.bottomAnchor)"));
    assert!(views.contains("DoweDrawer(open: state.bool(\"drawer01\"), close: { state.write(\"drawer01\", value: false) }, position: \"end\""));
    assert!(views.contains("radius: CGFloat(0)"));
    assert!(views.contains("disableOverlayClose: true, hideCloseButton: false"));
    assert!(views.contains("let doweDrawerNavigate = navigate"));
    assert!(views.contains("state.write(\"drawer01\", value: false)"));
    assert!(views.contains("doweDrawerNavigate(operation, target, fragment)"));
    assert!(views.contains("ScrollView {"));
    assert!(views.contains(".frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)"));
    assert!(!views.contains("controller.safeAreaRegions = []"));
    assert!(!views.contains("safeAreaInsets: doweDrawerEdgeInsets(window.safeAreaInsets)"));
    assert!(views.contains(".frame(width: CGFloat(44), height: CGFloat(44))\n                                    .contentShape(Rectangle())"));
    assert!(views.contains("struct DoweDrawerCloseIcon: View"));
    assert!(views.contains("DoweSvgPathData(data: \"m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073"));
    assert!(views.contains("return CGSize(width: CGFloat(320), height: CGFloat(0))"));
    assert!(views.contains("private var panelShape: UnevenRoundedRectangle"));
    assert!(views.contains("return UnevenRoundedRectangle(topLeadingRadius: radius, bottomLeadingRadius: radius, bottomTrailingRadius: CGFloat(0), topTrailingRadius: CGFloat(0))"));
    let rounded_style = StyleProps {
        rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
        ..Default::default()
    };
    assert_eq!(
        super::swift_drawer_radius(&rounded_style),
        "doweResponsive(viewportWidth, xs: CGFloat(12)) ?? CGFloat(0)"
    );
}
