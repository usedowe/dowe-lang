#[test]
fn generates_independent_native_safe_area_colors() {
    let mut route = navigation_shell_route();
    if let ViewNode::Scaffold { props, .. } = &mut route.page_tree {
        props.safe_area_top = Some(ColorToken::Surface);
        props.safe_area_bottom = Some(ColorToken::Primary);
    } else {
        panic!("scaffold");
    }
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("windowInsetsTopHeight(WindowInsets.safeDrawing).background(doweSafeAreaTopColor(currentEntry.path))"));
    assert!(views.content.contains("windowInsetsBottomHeight(WindowInsets.safeDrawing).background(doweSafeAreaBottomColor(currentEntry.path))"));
    assert!(views.content.contains("doweApplySystemBarIconAppearance(currentEntry.path)"));
    assert!(views.content.contains("WindowCompat.getInsetsController(currentActivity.window, currentActivity.window.decorView)"));
    assert!(views.content.contains("\"/\" -> DoweDesign.surface"));
    assert!(views.content.contains("\"/\" -> DoweDesign.primary"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("doweSafeAreaTopColor = DOWE_SURFACE;"));
    assert!(
        dev.content
            .contains("doweSafeAreaBottomColor = DOWE_PRIMARY;")
    );
    assert!(
        dev.content
            .contains("doweColorLuminance(doweSafeAreaTopColor)")
    );
    assert!(
        dev.content
            .contains("doweColorLuminance(doweSafeAreaBottomColor)")
    );
    assert!(dev.content.contains("doweApplySafeAreaColors();"));
    let main_activity = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("MainActivity.kt"))
        .expect("main activity");
    assert!(
        main_activity
            .content
            .contains("doweSafeAreaTopColor(incomingPath)")
    );
    assert!(
        main_activity
            .content
            .contains("doweSafeAreaBottomColor(incomingPath)")
    );
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
    assert!(views.content.contains("Modifier.drawBehind"));
    assert!(views.content.contains("drawLine(accentColor"));
    assert!(
        !views
            .content
            .contains("val border = if (active && selectedLine) BorderStroke")
    );
    assert!(
        views
            .content
            .contains("val listModifier = Modifier\n        .wrapContentWidth()")
    );
    assert!(views.content.contains("if (activeTab == \"overview\")"));
    assert!(views.content.contains("Text(\"Overview content\""));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("TextView[] view"));
    assert!(dev.content.contains("View[] view"));
    assert!(dev.content.contains("doweText(\"Overview\""));
    assert!(dev.content.contains("doweText(\"Details\""));
    assert!(dev.content.contains("doweText(\"Overview\", DOWE_PRIMARY"));
    assert!(
        !dev.content
            .contains("doweText(\"Overview\", DOWE_PRIMARY_TEXT")
    );
    assert!(
        dev.content
            .contains(".setGravity(Gravity.CENTER_VERTICAL);\n        doweWrapContentWidth(view")
    );
    assert!(dev.content.contains("doweTabLineBackground("));
    assert!(
        dev.content
            .contains("setVisibility(active ? View.VISIBLE : View.GONE)")
    );
}

#[test]
fn generates_compose_and_dev_stepper() {
    let mut route = tabs_route();
    let ViewNode::Tabs { props, .. } = &mut route.page_tree else {
        panic!("stepper");
    };
    props.variant = TabsVariant::Stepper;
    props.position = TabsPosition::Top;
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let dev = dev_java_source(&output);

    assert!(
        views
            .content
            .contains("position = \"top\", variant = \"stepper\"")
    );
    assert!(
        views
            .content
            .contains("items.forEachIndexed { index, item ->")
    );
    assert!(views.content.contains("CircleShape"));
    assert!(
        views
            .content
            .contains("horizontalScroll(rememberScrollState())")
    );
    assert!(dev.content.contains("\"1  \" +"));
    assert!(dev.content.contains("DOWE_MUTED"));
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
    assert!(views.content.contains("Modifier.fillMaxSize()"));
    assert!(
        views
            .content
            .contains("private val doweOverlayClosePaths = listOf(")
    );
    assert!(
        views
            .content
            .contains("DoweSvg(viewBox = doweOverlayCloseViewBox")
    );
    assert!(
        views
            .content
            .contains("m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073")
    );
    assert!(
        views
            .content
            .contains("Box(modifier = Modifier.fillMaxWidth().weight(1f).clipToBounds())")
    );
    assert!(views.content.contains("Column(modifier = Modifier.fillMaxWidth()"));
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

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("if (doweBool(\"drawer01\"))"));
    assert!(dev.content.contains("new PopupWindow("));
    assert!(dev.content.contains("doweWrite(\"drawer01\", false)"));
    assert!(dev.content.contains("setElevation(doweDp(8))"));
    assert!(
        dev.content
            .contains("private Runnable doweDrawerNavigationClose = null;")
    );
    assert!(
        dev.content
            .contains("private void doweCloseDrawerForNavigation()")
    );
    assert!(dev.content.contains("doweDrawerNavigationClose = view"));
    assert!(dev.content.contains("new ScrollView(this);"));
    assert!(
        dev.content
            .contains("new DoweSvgView(this, 0f, 0f, 24f, 24f, DOWE_MUTED_TEXT")
    );
    assert!(
        dev.content
            .contains("setContentDescription(\"Close drawer\")")
    );
    assert!(
        dev.content
            .contains("m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073")
    );
    assert!(
        dev.content
            .contains("private void renderCurrentRoute(boolean scrollToFragment)")
    );
    assert!(
        dev.content
            .contains("if (scrollToFragment) {\n            if (currentFragment == null) {\n                scrollView.scrollTo(0, 0);\n            } else {\n                doweScrollToFragment();\n            }\n        }")
    );
    assert!(dev.content.contains("renderCurrentRoute(false);"));
    assert!(dev.content.contains("addOnPreDrawListener"));
    assert!(
        dev.content
            .contains("target.getLocationInWindow(targetLocation);")
    );
    assert!(dev.content.contains(
        "visibleTop = Math.max(visibleTop, appBarLocation[1] + pinnedAppBar.getHeight());"
    ));
    assert!(
        dev.content
            .contains("scrollView.smoothScrollTo(0, destination);")
    );
    assert!(!dev.content.contains("doweTopRelativeToRoot"));
    assert!(dev.content.contains(
        "root.post(() -> { if (root.getWindowToken() != null) { doweActiveOverlay = view"
    ));
    assert!(dev.content.contains("doweActiveOverlay = null;"));
    assert!(
        dev.content
            .contains(r#"doweDrawerBackground(DOWE_SURFACE, null, "end", 0f)"#)
    );
    assert!(!dev.content.contains(".setText(\"x\")"));
    assert!(dev.content.contains(
        "new FrameLayout.LayoutParams(doweDp(28), doweDp(28), Gravity.TOP | Gravity.START)"
    ));
    assert!(dev
        .content
        .contains("Params.setMargins(doweDp(8), doweDp(8), 0, 0);"));
    assert!(dev.content.contains("doweAdd(parent, child, null, false);"));
    assert!(dev.content.contains("if (parent instanceof FrameLayout)"));
    assert!(dev.content.contains("doweFrameLayoutParams"));
    assert!(dev.content.contains("Params = doweFrameLayoutParams(view"));
    assert!(
        dev.content
            .contains("Params.width == ViewGroup.LayoutParams.WRAP_CONTENT")
    );
    assert!(!dev.content.contains("doweCard(DOWE_SURFACE, null)"));
}

