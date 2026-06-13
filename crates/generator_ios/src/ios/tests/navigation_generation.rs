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
    assert!(views.contains("Text(\"Brand\")"));
    assert!(views.contains(".background(DoweDesign.surface)"));
    assert!(views.contains(".foregroundStyle(DoweDesign.onSurface)"));
    assert!(views.contains(".clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))"));
    assert!(views.contains(
            ".overlay(RoundedRectangle(cornerRadius: DoweDesign.radiusBox).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))"
        ));
    assert!(
        !views.contains(".overlay(Rectangle().fill(DoweDesign.muted).frame(height: CGFloat(1))")
    );
    assert!(!views.contains(
            ".overlay(RoundedRectangle(cornerRadius: CGFloat(0)).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))"
        ));
    assert!(views.contains(".padding(.horizontal, CGFloat(16))"));
    assert!(views.contains(".frame(maxWidth: CGFloat(1152), alignment: .center)"));
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
    assert!(views.contains("DoweSideNavSubmenu(open: true)"));
    assert!(views.contains("withAnimation(.easeInOut(duration: 0.18))"));
    assert!(views.contains(".transition(.opacity.combined(with: .move(edge: .top)))"));
    assert!(views.contains("active: activePath == \"/bars\""));
    assert!(views.contains("Text(\"Workspace\")"));
    assert!(views.contains("Text(\"Blogs\")"));
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
    assert!(views.contains("DoweNavMenuItem(active: activePath == \"/\""));
    assert!(views.contains("DoweNavMenuItem(active: openIndex == 1"));
    assert!(views.contains("HStack(alignment: .top"));
    assert!(views.contains("Text(\"Resource hub\")"));
    assert!(views.contains("Text(\"Side Home\")"));
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
    assert!(views.contains("if activeTab == \"overview\""));
    assert!(views.contains("Text(\"Overview content\")"));
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
    assert!(views.contains("DoweDrawer(open: state.bool(\"drawer01\"), close: { state.write(\"drawer01\", value: false) }, position: \"end\""));
    assert!(views.contains("radius: CGFloat(0)"));
    assert!(views.contains("disableOverlayClose: true, hideCloseButton: false"));
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
