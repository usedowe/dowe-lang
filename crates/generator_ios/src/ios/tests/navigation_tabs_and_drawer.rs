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
    assert!(
        !views.contains("RoundedRectangle(cornerRadius: tabRadius).stroke(active && selectedLine")
    );
    assert!(views.contains("if activeTab == \"overview\""));
    assert!(views.contains("Text(verbatim: \"Overview content\")"));
}

#[test]
fn generates_swiftui_stepper() {
    let mut route = tabs_route();
    let ViewNode::Tabs { props, .. } = &mut route.page_tree else {
        panic!("stepper");
    };
    props.variant = TabsVariant::Stepper;
    props.position = TabsPosition::Top;
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("position: \"top\", variant: \"stepper\""));
    assert!(views.contains("Text(String(index + 1))"));
    assert!(views.contains("clipShape(Circle())"));
    assert!(views.contains("ScrollView(.horizontal, showsIndicators: false)"));
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
    assert!(views.contains(
        "controller.view.bottomAnchor.constraint(equalTo: window.safeAreaLayoutGuide.bottomAnchor)"
    ));
    assert!(views.contains("DoweDrawer(open: state.bool(\"drawer01\"), close: { state.write(\"drawer01\", value: false) }, position: \"end\""));
    assert!(views.contains("radius: CGFloat(0)"));
    assert!(views.contains("disableOverlayClose: true, hideCloseButton: false"));
    assert!(views.contains("let doweDrawerNavigate = navigate"));
    assert!(views.contains("let _ = navigate"));
    assert!(views.contains("let _ = goBack"));
    assert!(views.contains("let _ = openExternal"));
    assert!(views.contains("state.write(\"drawer01\", value: false)"));
    assert!(views.contains("doweDrawerNavigate(operation, target, fragment)"));
    assert!(views.contains("ScrollView {"));
    assert!(
        views.contains(".frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)")
    );
    assert!(!views.contains("controller.safeAreaRegions = []"));
    assert!(!views.contains("safeAreaInsets: doweDrawerEdgeInsets(window.safeAreaInsets)"));
    assert!(views
        .contains(".frame(width: CGFloat(44), height: CGFloat(44))\n                            .contentShape(Rectangle())"));
    assert!(views.contains(
        ".frame(maxWidth: .infinity, maxHeight: .infinity, alignment: closeButtonAlignment)"
    ));
    assert!(views.contains("private var closeButtonAlignment: Alignment"));
    assert!(views.contains("struct DoweOverlayCloseIcon: View"));
    assert!(
        views.contains("DoweSvgPathData(data: \"m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073")
    );
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
