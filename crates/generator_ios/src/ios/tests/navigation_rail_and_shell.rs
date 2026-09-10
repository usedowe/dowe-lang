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

    assert!(views.contains("open: true, bordered: true, wide: state.bool(\"wideEnabled\", fallback: false)"));
    assert!(views.contains("DoweSideNavRow(active: activePath == \"/bars\""));
    assert!(views.contains("DoweSvgView(viewBox: DoweSvgViewBox"));
    assert!(views.contains(
        "color: activePath == \"/bars\" ? DoweDesign.surfaceText : DoweDesign.backgroundText"
    ));
    assert!(views.contains(
        ".foregroundStyle(activePath == \"/bars\" ? DoweDesign.surfaceText : DoweDesign.backgroundText)"
    ));
    assert!(views.contains(
        ".foregroundStyle(false ? DoweDesign.surfaceText : DoweDesign.backgroundText)"
    ));
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
    assert!(views.contains(
        "width: nil, maxWidth: nil, height: nil, maxHeight: nil, minWidth: nil, minHeight: nil"
    ));
    assert!(!views.contains("DoweSideNavRow(active: activePath == \"/item-0\""));
}

#[test]
fn generates_swiftui_rail_nav() {
    let mut rail_route = route();
    rail_route.page_tree = ViewNode::RailNav {
        props: RailNavProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Solid),
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
    assert!(views.contains("openIndex = openIndex == index ? nil : index"));
    assert!(
        views.contains(
            "DoweAnchoredPopoverPresenter(\n                isPresented: openIndex != nil"
        )
    );
    assert!(views.contains("DoweNavMenuPopover("));
    assert!(!views.contains(".popover(\n            isPresented: Binding("));
    assert!(!views.contains(".presentationCompactAdaptation(.popover)"));
    assert!(!views.contains(".presentationBackground(popoverBackgroundColor)"));
    assert!(views.contains(".simultaneousGesture(TapGesture().onEnded"));
    assert!(!views.contains("if openIndex != nil {"));
    assert!(views.contains("Text(String(localized: \"home.hero.title\"))"));
    assert!(views.contains("DoweNavMenuItem(active: activePath == \"/\""));
    assert!(views.contains("DoweNavMenuItem(active: openIndex == 1"));
    assert!(views.contains(".rotationEffect(openIndex == 1 ? .degrees(180) : .degrees(0))"));
    assert!(!views.contains("Text(\"⌄\")"));
    assert!(views.contains("HStack(alignment: .top"));
    assert!(
        views.contains(
            ".frame(maxWidth: CGFloat(1536), maxHeight: .infinity, alignment: .topLeading)"
        )
    );
    assert!(views.contains(".frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)"));
    assert!(views.contains("Text(verbatim: \"Resource hub\")"));
    assert!(views.contains("label: \"Side Home\""));
    assert!(views.contains(
        ".frame(width: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(384)))), alignment: .leading)"
    ));
    assert!(views.contains(
        ".frame(maxWidth: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(384)))), alignment: .leading)"
    ));
    assert!(
        views.contains(".frame(maxHeight: UIScreen.main.bounds.height, alignment: .topLeading)")
    );
    assert!(views.contains(".clipped()"));
    assert!(views.contains("ScrollView {"));
}
