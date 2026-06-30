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

    assert!(
        views
            .content
            .contains("DoweSideNavSubmenu(open = true, bordered = true")
    );
    assert!(
        views
            .content
            .contains(".padding(start = 16.dp)")
    );
    assert!(views.content.contains("DoweSideNavArrow(expanded = expanded)"));
    assert!(views.content.contains("doweSideNavArrowPaths"));
    assert!(views.content.contains("drawLine(DoweDesign.muted"));
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
    assert!(dev.content.contains("doweSideNavArrow"));
    assert!(dev.content.contains("doweSideNavSubmenuContent"));
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
            .contains("import androidx.compose.ui.platform.LocalConfiguration")
    );
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
    assert!(views.content.contains("DoweSideNav(items = listOf("));
    assert!(views.content.contains(
        "modifier = Modifier.doweWidth(doweResponsive(viewportWidth, xs = DoweSize.Fixed(384.dp)))"
    ));
    assert!(
        views
            .content
            .contains(".heightIn(max = LocalConfiguration.current.screenHeightDp.dp).background(DoweDesign.surface)")
    );
    assert!(
        views
            .content
            .contains("Modifier.fillMaxWidth().weight(1f).verticalScroll(rememberScrollState())")
    );
    assert!(views.content.contains("Text(\"Resource hub\""));
    assert!(views.content.contains("label = \"Side Home\""));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(
        dev.content
            .contains("doweResponsiveInt(viewportWidth, 384, null, null, null, null)")
    );
    assert!(
        dev.content.contains(
            "ShellHeight = Math.max(0, getResources().getDisplayMetrics().heightPixels - scrollView.getPaddingTop() - scrollView.getPaddingBottom());"
        )
    );
    assert!(
        dev.content.contains("ShellHeight));")
    );
    assert!(dev.content.contains("doweText(\"Resource hub\""));
    assert!(dev.content.contains("\"Side Home\""));
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
        views
            .content
            .contains("Modifier.fillMaxSize().safeDrawingPadding()")
    );
    assert!(views.content.contains("private val doweDrawerClosePaths = listOf("));
    assert!(views.content.contains("DoweSvg(viewBox = doweDrawerCloseViewBox"));
    assert!(views.content.contains("m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073"));
    assert!(
        views
            .content
            .contains("Modifier.fillMaxWidth().weight(1f).verticalScroll(rememberScrollState())")
    );
    assert!(views.content.contains("val doweDrawerNavigate = navigate"));
    assert!(views.content.contains("state.write(\"drawer01\", false)"));
    assert!(
        views
            .content
            .contains("doweDrawerNavigate(operation, target, fragment)")
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
    assert!(dev.content.contains("private Runnable doweDrawerNavigationClose = null;"));
    assert!(dev.content.contains("private void doweCloseDrawerForNavigation()"));
    assert!(dev.content.contains("doweDrawerNavigationClose = view"));
    assert!(dev.content.contains("new ScrollView(this);"));
    assert!(dev.content.contains("scrollView.getPaddingTop()"));
    assert!(dev.content.contains("new DoweSvgView(this, 0f, 0f, 24f, 24f, DOWE_ON_SOFT_MUTED"));
    assert!(dev.content.contains("setContentDescription(\"Close drawer\")"));
    assert!(dev.content.contains("m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073"));
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
    assert!(!dev.content.contains(".setText(\"x\")"));
    assert!(dev.content.contains(
        "new FrameLayout.LayoutParams(doweDp(28), doweDp(28), Gravity.TOP | Gravity.END)"
    ));
    assert!(
        dev.content
            .contains("doweAdd(parent, child, null, false);")
    );
    assert!(dev.content.contains("if (parent instanceof FrameLayout)"));
    assert!(dev.content.contains("doweFrameLayoutParams"));
    assert!(
        dev.content
            .contains("Params = doweFrameLayoutParams(view")
    );
    assert!(
        dev.content
            .contains("Params.width == ViewGroup.LayoutParams.WRAP_CONTENT")
    );
    assert!(!dev.content.contains("doweCard(DOWE_SURFACE, null)"));
}
