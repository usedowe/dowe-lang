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
        "Column(modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).zIndex(1f).padding(horizontal = 16.dp, vertical = 8.dp).clip(RoundedCornerShape(DoweDesign.radius)).background(DoweDesign.surface).border(1.dp, DoweDesign.muted, RoundedCornerShape(DoweDesign.radius)))"
    ));
    assert!(
        views.content.contains(
            "Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center)"
        )
    );
    assert!(views.content.contains(
        "Box(modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).background(DoweDesign.surface).border(1.dp, DoweDesign.muted, RoundedCornerShape(0.dp)), contentAlignment = Alignment.BottomCenter)"
    ));
    assert_eq!(
        views
            .content
            .matches("Modifier.widthIn(max = 1536.dp).fillMaxWidth()")
            .count(),
        3
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
    assert!(views.content.contains("Text(\"Directory\""));
    assert!(views.content.contains("Text(\"Copyright\""));
    let directory = views
        .content
        .find("Text(\"Directory\"")
        .expect("Footer top");
    let footer_start = views.content[..directory]
        .rfind("Column(modifier =")
        .expect("Footer column");
    assert!(
        views.content[footer_start..directory]
            .contains("CompositionLocalProvider(LocalContentColor provides DoweDesign.onSurface)")
    );
    assert!(views.content.contains("itemSize = 56.dp"));
    assert!(views.content.contains("featured = true"));
    assert!(
        views
            .content
            .contains("backgroundColor = DoweDesign.primary")
    );

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("doweBackground(DOWE_SURFACE, DOWE_RADIUS)")
    );
    assert!(!dev.content.contains("setElevation(doweDp(4))"));
    assert!(dev.content.contains("doweBackground(DOWE_PRIMARY, 999f)"));
    assert_eq!(
        dev.content
            .matches("doweBoxedContainer(true, 1536)")
            .count(),
        3
    );
    assert!(dev.content.contains("dowePinAppBar("));
    assert!(
        dev.content
            .contains("for (int index = 0; index < appBar.getChildCount(); index++)")
    );
    assert!(dev.content.contains(
        "child.measure(childWidthSpec, View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED))"
    ));
    assert!(
        dev.content
            .contains("appBarHeight = Math.max(appBarHeight, child.getMeasuredHeight())")
    );
    assert!(dev.content.contains(
        "new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, appBarHeight, Gravity.TOP | Gravity.START)"
    ));
    let pin_offset = dev
        .content
        .find("\n        dowePinAppBar(")
        .map(|index| index + 1)
        .expect("persistent AppBar pin");
    let boxed_content = dev.content[..pin_offset]
        .rfind("doweBoxedContainer(true, 1536)")
        .expect("boxed AppBar content");
    let boxed_content_line_start = dev.content[..boxed_content]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    let boxed_content_line = dev.content[boxed_content_line_start..]
        .lines()
        .next()
        .expect("boxed AppBar content line");
    let boxed_content_view = boxed_content_line
        .split_whitespace()
        .nth(1)
        .expect("boxed AppBar content view");
    let before_pin = &dev.content[boxed_content..pin_offset];
    assert!(before_pin.contains(&format!(", {boxed_content_view});")));
    let pin_line = dev.content[pin_offset..]
        .lines()
        .next()
        .expect("persistent AppBar pin line");
    assert!(!pin_line.contains(&format!(", {boxed_content_view})")));
    assert!(dev.content.contains("background.addView(appBar, params)"));
    assert!(
        dev.content
            .contains("safeArea.setBackgroundColor(DOWE_BACKGROUND)")
    );
    assert!(dev.content.contains("dowe-pinned-appbar-safe-area"));
    assert!(
        dev.content
            .contains("bottomSafeArea.setBackgroundColor(DOWE_BACKGROUND)")
    );
    assert!(dev.content.contains("dowe-pinned-appbar-bottom-safe-area"));
    assert!(dev.content.contains("doweRelayoutPinnedAppBar();"));
    assert!(
        dev.content
            .contains("scrollView.post(this::doweRelayoutPinnedAppBar)")
    );
    assert!(dev.content.contains("background.getRootWindowInsets()"));
    assert!(
        dev.content
            .contains("appBarParams.setMargins(leftInset, topInset, rightInset, 0)")
    );
    assert!(dev.content.contains("safeAreaParams.height = topInset"));
    assert!(
        dev.content
            .contains("bottomSafeAreaParams.height = bottomInset")
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
    assert!(dev.content.contains("doweText(\"Directory\""));
    assert!(dev.content.contains("doweText(\"Copyright\""));
}

#[test]
fn keeps_unbordered_persistent_appbar_visually_flat() {
    let output = generate_android(
        &[unbordered_persistent_appbar_route()],
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
        "Column(modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).zIndex(1f).background(DoweDesign.surface))"
    ));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("dowePinAppBar("));
    assert!(!dev.content.contains("setElevation(doweDp(4))"));
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
    assert!(views.content.contains(".padding(start = 16.dp)"));
    assert!(
        views
            .content
            .contains("DoweSideNavArrow(expanded = expanded)")
    );
    assert!(
        views
            .content
            .contains("modifier.then(if (wide) Modifier.fillMaxWidth() else Modifier)")
    );
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
    assert!(views.content.contains("gap = 10.dp"));
    assert!(views.content.contains("private fun DoweSideNavStatus"));
    assert!(views.content.contains("DoweSideNavStatus(text = \"2\""));
    assert!(
        views
            .content
            .contains("padding(horizontal = 8.dp, vertical = 2.dp)")
    );
    assert!(views.content.contains("color = DoweDesign.onSoftMuted"));
    assert!(
        views
            .content
            .contains("Row(horizontalArrangement = Arrangement.spacedBy(10.dp)")
    );
    assert!(views.content.contains("state.bool(\"wideEnabled\", false)"));
    assert!(views.content.contains(
        "DoweSideNavSubmenu(open = true, bordered = true, wide = state.bool(\"wideEnabled\", false)"
    ));
    assert!(
        views
            .content
            .contains("DoweSvg(viewBox = DoweSvgViewBox(0f, 0f, 24f, 24f)")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("setVisibility(View.VISIBLE)"));
    assert!(dev.content.contains("doweToggleSideNavSubmenu"));
    assert!(dev.content.contains("doweSideNavArrow"));
    assert!(dev.content.contains("doweSideNavSubmenuContent"));
    assert!(
        dev.content
            .contains("int rowContentColor = active ? activeContentColor : DOWE_ON_BACKGROUND;")
    );
    assert!(
        dev.content
            .contains("doweText(\"Blogs\", (false) ? DOWE_ON_SURFACE : DOWE_ON_BACKGROUND")
    );
    assert!(dev.content.contains(
        "wide ? ViewGroup.LayoutParams.MATCH_PARENT : ViewGroup.LayoutParams.WRAP_CONTENT"
    ));
    assert!(
        dev.content
            .contains("view.animate().alpha(0f).translationY(-doweDp(4)).setDuration(140)")
    );
    assert!(dev.content.contains("doweText(\"Blogs\""));
    assert!(
        dev.content
            .contains("new DoweSvgView(this, 0f, 0f, 24f, 24f")
    );
    assert!(dev.content.contains(", 10, true);\n        TextView"));
    assert!(dev.content.contains("private TextView doweSideNavStatus"));
    assert!(
        dev.content
            .contains("doweBackground(DOWE_SOFT_MUTED, 999f)")
    );
    assert!(dev.content.contains("Status, 10, true);"));
    assert!(dev.content.contains("Arrow, 10, true);"));
    assert!(dev.content.contains("if (doweBool(\"wideEnabled\", null))"));
    assert!(dev.content.contains("doweBool(\"wideEnabled\", null) ? ViewGroup.LayoutParams.MATCH_PARENT : ViewGroup.LayoutParams.WRAP_CONTENT"));
}

#[test]
fn generates_compose_and_dev_rail_nav() {
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
    let output = generate_android(
        &[rail_route],
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
            .contains("DoweRailNavItem(label = \"Home\", showLabel = true")
    );
    assert!(views.content.contains("active = activePath == \"/login\""));
    assert!(views.content.contains("Modifier.size(24.dp)"));
    assert!(views.content.contains(".width(64.dp)"));
    assert!(views.content.contains("contentDescription = label"));
    assert!(views.content.contains("textAlign = TextAlign.Center"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("setContentDescription(\"Home\")"));
    assert!(
        dev.content
            .contains("new LinearLayout.LayoutParams(doweDp(64)")
    );
    assert!(
        dev.content
            .contains("new LinearLayout.LayoutParams(doweDp(24), doweDp(24))")
    );
    assert!(dev.content.contains("setGravity(Gravity.CENTER)"));
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
            .contains("Popup(onDismissRequest = { openIndex = null }")
    );
    assert!(
        views
            .content
            .contains("DoweNavMenuPopoverSurface(onDismiss = { openIndex = null })")
    );
    assert!(views.content.contains("PointerEventPass.Final"));
    assert!(views.content.contains("PointerEventType.Release"));
    assert!(views.content.contains(
        "popoverBackgroundColor = DoweDesign.background, popoverContentColor = DoweDesign.onBackground"
    ));
    let resource = dowe_components::translation_resource_name("home.hero.title");
    assert!(
        views
            .content
            .contains(&format!("stringResource(R.string.{resource})"))
    );
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
            .contains("modifier = Modifier.size(14.dp).rotate(if (openIndex == 1) 180f else 0f)")
    );
    assert!(!views.content.contains("Text(text = \"⌄\""));
    assert!(views.content.contains(
        "Box(modifier = Modifier.fillMaxWidth().weight(1f), contentAlignment = Alignment.TopCenter)"
    ));
    assert!(
        views
            .content
            .contains("Row(modifier = Modifier.widthIn(max = 1536.dp).fillMaxSize())")
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

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("DoweDismissOnTouchLayout"));
    assert!(dev.content.contains("new PopupWindow("));
    assert!(dev.content.contains("showAsDropDown("));
    assert!(
        dev.content
            .contains("doweInputBackground(DOWE_BACKGROUND, null, DOWE_RADIUS)")
    );
    assert!(dev.content.contains("doweNavMenuArrow(DOWE_ON_BACKGROUND)"));
    assert!(dev.content.contains("setOnDismissListener"));
    assert!(
        dev.content
            .contains("doweNavigate(\"push\", \"/docs\", null); if (")
    );
    assert!(dev.content.contains("Label.setOnClickListener(v ->"));
    assert!(dev.content.contains(".performClick());"));
    assert!(dev.content.contains("post(dismissAction);"));
    assert!(!dev.content.contains("dismissAction.run();"));
    assert!(
        dev.content
            .contains(&format!("getString(R.string.{resource})"))
    );
    assert!(
        dev.content
            .contains("doweResponsiveInt(viewportWidth, 384, null, null, null, null)")
    );
    assert!(
        dev.content.contains(
            "ShellHeight = Math.max(0, getResources().getDisplayMetrics().heightPixels - scrollView.getPaddingTop() - scrollView.getPaddingBottom());"
        )
    );
    assert!(dev.content.contains("ShellHeight));"));
    assert!(dev.content.contains("doweText(\"Resource hub\""));
    assert!(
        !dev.content
            .contains("new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.MATCH_PARENT, 1f)")
    );
    assert!(dev.content.contains("\"Side Home\""));
    assert!(dev.content.contains("doweBoxedContainer(true, 1536)"));
    assert!(
        dev.content
            .contains("Params.gravity = Gravity.CENTER_HORIZONTAL")
    );
    assert!(dev.content.contains(
        "LinearLayout view1 = doweContainer(false);\n        doweWrapContentWidth(view1);\n        doweAdd(view0, view1);"
    ));
    assert!(dev.content.contains(
        "LinearLayout view2 = doweContainer(true);\n        doweWrapContentWidth(view2);\n        doweAdd(view1, view2);"
    ));
    assert!(dev.content.contains(
        "LinearLayout view3 = doweContainer(true);\n        view3.setGravity(Gravity.CENTER_VERTICAL);\n        view3.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        doweWrapContentWidth(view3);"
    ));
    assert!(dev.content.contains("doweAdd(view2, view3);"));
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
            .contains("doweText(\"Overview\", DOWE_ON_PRIMARY")
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
    assert!(dev.content.contains("DOWE_SOFT_MUTED"));
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

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("if (doweBool(\"drawer01\"))"));
    assert!(dev.content.contains("new PopupWindow("));
    assert!(dev.content.contains("doweWrite(\"drawer01\", false)"));
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
            .contains("new DoweSvgView(this, 0f, 0f, 24f, 24f, DOWE_ON_SOFT_MUTED")
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
    assert!(
        dev.content
            .contains("root.post(() -> { if (root.getWindowToken() != null) { view")
    );
    assert!(
        dev.content
            .contains(r#"doweDrawerBackground(DOWE_SURFACE, null, "end", 0f)"#)
    );
    assert!(!dev.content.contains(".setText(\"x\")"));
    assert!(dev.content.contains(
        "new FrameLayout.LayoutParams(doweDp(28), doweDp(28), Gravity.TOP | Gravity.END)"
    ));
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
