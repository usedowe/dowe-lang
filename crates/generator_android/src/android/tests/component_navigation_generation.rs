#[test]
fn generates_compose_and_dev_layout_bars() {
    let output = generate_android(
        &[bar_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains(
            "Box(modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).padding(horizontal = 16.dp, vertical = 8.dp).clip(RoundedCornerShape(DoweDesign.radiusBox)).background(DoweDesign.surface).border(1.dp, DoweDesign.muted, RoundedCornerShape(DoweDesign.radiusBox)), contentAlignment = Alignment.Center)"
        ));
    assert!(
        views
            .content
            .contains("Modifier.fillMaxWidth().widthIn(max = 1152.dp)")
    );
    assert!(
        views
            .content
            .contains("CompositionLocalProvider(LocalContentColor provides DoweDesign.onSurface)")
    );
    assert!(
        views
            .content
            .contains("horizontalArrangement = Arrangement.Center")
    );
    assert!(views.content.contains("Text(\"Brand\""));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(
        dev.content
            .contains("doweBackground(DOWE_SURFACE, DOWE_RADIUS_BOX)")
    );
    assert!(
        dev.content
            .contains("setGravity(Gravity.CENTER_VERTICAL | Gravity.START)")
    );
    assert!(
        dev.content
            .contains("setGravity(Gravity.CENTER_VERTICAL | Gravity.CENTER)")
    );
    assert!(
        dev.content
            .contains("setGravity(Gravity.CENTER_VERTICAL | Gravity.END)")
    );
    assert!(
        dev.content
            .contains("new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f)")
    );
    assert!(
        dev.content
            .contains("new LinearLayout.LayoutParams(0, 0, 1f)")
    );
    assert!(dev.content.contains("doweText(\"Brand\""));
    assert!(dev.content.contains("doweText(\"Footer\""));
}

#[test]
fn generates_compose_and_dev_side_nav() {
    let output = generate_android(
        &[side_nav_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("DoweSideNavSubmenu(open = true"));
    assert!(
        views
            .content
            .contains("Column(modifier = Modifier.padding(start = 16.dp))")
    );
    assert!(views.content.contains("AnimatedVisibility("));
    assert!(views.content.contains(
        "fadeIn(animationSpec = tween(160)) + expandVertically(animationSpec = tween(180))"
    ));
    assert!(views.content.contains(
        "fadeOut(animationSpec = tween(120)) + shrinkVertically(animationSpec = tween(180))"
    ));
    assert!(views.content.contains(r#"active = activePath == "/bars""#));
    assert!(views.content.contains(r#"Text(text = "Workspace""#));
    assert!(views.content.contains(r#"Text(text = "Blogs""#));
    assert!(
        views
            .content
            .contains("DoweSvg(viewBox = DoweSvgViewBox(0f, 0f, 24f, 24f)")
    );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("setVisibility(View.VISIBLE)"));
    assert!(dev.content.contains("doweToggleSideNavSubmenu"));
    assert!(
        dev.content
            .contains("view.animate().alpha(0f).translationY(-doweDp(4)).setDuration(140)")
    );
    assert!(dev.content.contains("doweText(\"Blogs\""));
    assert!(
        dev.content
            .contains("new DoweSvgView(this, 0f, 0f, 24f, 24f")
    );
}
#[test]
fn generates_compose_and_dev_navigation_shell_components() {
    let output = generate_android(
        &[navigation_shell_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("DoweNavMenu("));
    assert!(
        views
            .content
            .contains("DoweNavMenuItem(active = activePath == \"/\"")
    );
    assert!(
        views
            .content
            .contains("DoweNavMenuItem(active = openIndex == 1")
    );
    assert!(
        views
            .content
            .contains("Row(modifier = Modifier.fillMaxWidth().weight(1f))")
    );
    assert!(views.content.contains("Text(\"Resource hub\""));
    assert!(views.content.contains("Text(text = \"Side Home\""));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("doweText(\"Resource hub\""));
    assert!(dev.content.contains("doweText(\"Side Home\""));
}

#[test]
fn generates_compose_and_dev_tabs() {
    let output = generate_android(
        &[tabs_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(
        views
            .content
            .contains("private fun DoweTabs(items: List<DoweTabItem>")
    );
    assert!(views.content.contains("DoweTabs(items = listOf(DoweTabItem(id = \"overview\", label = \"Overview\"), DoweTabItem(id = \"details\", label = \"Details\")), initialId = \"overview\""));
    assert!(
        views
            .content
            .contains("position = \"start\", variant = \"line\"")
    );
    assert!(
        views
            .content
            .contains("backgroundColor = Color.Transparent")
    );
    assert!(views.content.contains("accentColor = DoweDesign.primary"));
    assert!(views.content.contains("if (activeTab == \"overview\")"));
    assert!(views.content.contains("Text(\"Overview content\""));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("TextView[] view"));
    assert!(dev.content.contains("View[] view"));
    assert!(dev.content.contains("doweText(\"Overview\""));
    assert!(dev.content.contains("doweText(\"Details\""));
    assert!(
        dev.content
            .contains("setVisibility(active ? View.VISIBLE : View.GONE)")
    );
}

#[test]
fn generates_compose_and_dev_drawer() {
    let output = generate_android(
        &[drawer_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(
        views
            .content
            .contains("private fun DoweDrawer(open: Boolean")
    );
    assert!(views.content.contains("DoweDrawer(open = state.bool(\"drawer01\"), onClose = { state.write(\"drawer01\", false) }, position = \"end\""));
    assert!(views.content.contains("radius = 0.dp"));
    assert!(
        views
            .content
            .contains("disableOverlayClose = true, hideCloseButton = false")
    );
    assert!(
        views
            .content
            .contains("Modifier.fillMaxHeight().widthIn(max = 320.dp)")
    );
    assert!(
        views.content.contains(
            "private fun doweDrawerShape(position: String, radius: Dp): RoundedCornerShape"
        )
    );
    assert!(views.content.contains(r#"RoundedCornerShape(topStart = radius, topEnd = 0.dp, bottomEnd = 0.dp, bottomStart = radius)"#));
    let rounded_style = StyleProps {
        rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
        ..Default::default()
    };
    assert_eq!(
        super::compose_drawer_radius(&rounded_style),
        "doweResponsive(viewportWidth, xs = 12.dp) ?: 0.dp"
    );
    assert_eq!(
        super::dev_drawer_radius(&rounded_style),
        "doweFloat(doweResponsiveFloat(viewportWidth, 12f, null, null, null, null), 0f)"
    );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("if (doweBool(\"drawer01\"))"));
    assert!(dev.content.contains("new PopupWindow("));
    assert!(dev.content.contains("doweWrite(\"drawer01\", false)"));
    assert!(
        dev.content
            .contains("private void renderCurrentRoute(boolean scrollToFragment)")
    );
    assert!(
        dev.content
            .contains("if (scrollToFragment) {\n            doweScrollToFragment();\n        }")
    );
    assert!(dev.content.contains("renderCurrentRoute(false);"));
    assert!(
        dev.content
            .contains("scrollView.scrollTo(0, doweTopRelativeToRoot(target));")
    );
    assert!(
        dev.content
            .contains("root.post(() -> { if (root.getWindowToken() != null) { view")
    );
    assert!(!dev.content.contains("smoothScrollTo"));
    assert!(
        dev.content
            .contains(r#"doweDrawerBackground(DOWE_SURFACE, null, "end", 0f)"#)
    );
    assert!(dev.content.contains("TextView"));
    assert!(dev.content.contains(
        "new FrameLayout.LayoutParams(doweDp(28), doweDp(28), Gravity.TOP | Gravity.END)"
    ));
    assert!(!dev.content.contains("doweCard(DOWE_SURFACE, null)"));
}
