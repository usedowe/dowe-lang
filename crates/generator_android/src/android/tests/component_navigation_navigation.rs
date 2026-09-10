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
            .contains("DoweSideNavSubmenu(stateKey = \"structure:")
    );
    assert!(views.content.contains("private val doweSideNavExpandedMemory = mutableStateMapOf<String, Boolean>()"));
    assert!(views.content.contains("doweSideNavExpandedMemory.getOrPut(stateKey) { open }"));
    assert!(views.content.contains("doweSideNavExpandedMemory[stateKey] = !expanded"));
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
    assert!(views.content.contains("color = DoweDesign.surface"));
    let submenu_label = views
        .content
        .find(r#"Text(text = "Content""#)
        .expect("submenu label");
    let submenu_label = &views.content[submenu_label..];
    let submenu_label = &submenu_label[..submenu_label.find('\n').expect("submenu label line")];
    assert!(submenu_label.contains("fontWeight = FontWeight.Normal"));
    assert!(submenu_label.contains("color = LocalContentColor.current"));
    assert!(views.content.contains(
        "DoweSideNavEntryRow(item = item, header = false, activePath = activePath"
    ));
    assert!(views.content.contains(
        "color = if (header) titleColor else LocalContentColor.current"
    ));
    assert!(views.content.contains(r#"Text(text = "Blogs""#));
    assert!(views.content.contains("gap = 10.dp"));
    assert!(views.content.contains("private fun DoweSideNavStatus"));
    assert!(views.content.contains("DoweSideNavStatus(text = \"2\""));
    assert!(
        views
            .content
            .contains("padding(horizontal = 8.dp, vertical = 2.dp)")
    );
    assert!(views.content.contains("color = DoweDesign.mutedText"));
    assert!(
        views
            .content
            .contains("Row(horizontalArrangement = Arrangement.spacedBy(10.dp)")
    );
    assert!(views.content.contains("state.bool(\"wideEnabled\", false)"));
    assert!(views.content.contains(
        "open = true, bordered = true, wide = state.bool(\"wideEnabled\", false)"
    ));
    assert!(
        views
            .content
            .contains("DoweSvg(viewBox = DoweSvgViewBox(0f, 0f, 24f, 24f)")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("setVisibility(View.VISIBLE)"));
    assert!(dev.content.contains("doweToggleSideNavSubmenu"));
    assert!(dev.content.contains("private final HashMap<String, Boolean> doweSideNavMemory"));
    assert!(dev.content.contains("doweSideNavExpanded("));
    assert!(dev.content.contains("doweSideNavMemory.put(key, false)"));
    assert!(dev.content.contains("doweSideNavMemory.put(key, true)"));
    assert!(dev.content.contains("doweSideNavArrow"));
    assert!(dev.content.contains("doweSideNavSubmenuContent"));
    assert!(
        dev.content
            .contains("int rowContentColor = active ? activeContentColor : DOWE_BACKGROUND_TEXT;")
    );
    assert!(
        dev.content
            .contains("doweText(\"Blogs\", (false) ? DOWE_SURFACE_TEXT : DOWE_BACKGROUND_TEXT")
    );
    assert!(dev.content.contains(
        "wide ? ViewGroup.LayoutParams.MATCH_PARENT : ViewGroup.LayoutParams.WRAP_CONTENT"
    ));
    assert!(
        dev.content
            .contains("view.animate().alpha(0f).translationY(-doweDp(4)).setDuration(140)")
    );
    assert!(dev.content.contains("doweText(\"Blogs\""));
    assert!(dev.content.contains("DOWE_SURFACE"));
    assert!(dev.content.contains(
        "doweText(\"Content\", (false) ? DOWE_SURFACE_TEXT : DOWE_BACKGROUND_TEXT, 14f, 400"
    ));
    assert!(dev.content.contains(
        "LinearLayout trigger = doweSideNavRow(entry, false, wide"
    ));
    assert!(dev
        .content
        .contains("header ? titleColor : rowContentColor"));
    assert!(
        dev.content
            .contains("new DoweSvgView(this, 0f, 0f, 24f, 24f")
    );
    assert!(dev.content.contains(", 10, true);\n        TextView"));
    assert!(dev.content.contains("private TextView doweSideNavStatus"));
    assert!(
        dev.content
            .contains("doweBackground(DOWE_MUTED, 999f)")
    );
    assert!(dev.content.contains("Status, 10, true);"));
    assert!(dev.content.contains("Arrow, 10, true);"));
    assert!(dev.content.contains("if (doweBool(\"wideEnabled\", null))"));
    assert!(dev.content.contains("doweBool(\"wideEnabled\", null) ? ViewGroup.LayoutParams.MATCH_PARENT : ViewGroup.LayoutParams.WRAP_CONTENT"));
}

#[test]
fn colors_android_side_nav_header_icons_from_scheme() {
    let mut route = side_nav_route();
    let ViewNode::SideNav { items, .. } = &mut route.layout_tree else {
        panic!("expected side nav layout");
    };
    let SideNavItem::Header(header) = &mut items[0] else {
        panic!("expected side nav header");
    };
    header.icon = side_nav_item("Home", None).icon;

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
    assert!(views.content.contains(
        "DoweSvg(viewBox = DoweSvgViewBox(0f, 0f, 24f, 24f), modifier = Modifier, color = DoweDesign.surface, paths ="
    ));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains(
        "DoweSvgView(this, 0f, 0f, 24f, 24f, DOWE_SURFACE,"
    ));
}

#[test]
fn generates_compose_and_dev_rail_nav() {
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
    assert!(views
        .content
        .contains("openIndex = if (openIndex == index) null else index"));
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
        "popoverBackgroundColor = DoweDesign.background, popoverContentColor = DoweDesign.backgroundText"
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
            .contains("Box(modifier = Modifier.fillMaxWidth().weight(1f).clipToBounds())")
    );
    assert!(views.content.contains("Column(modifier = Modifier.fillMaxWidth()"));
    assert!(views.content.contains("Text(\"Resource hub\""));
    assert!(views.content.contains("label = \"Side Home\""));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("DoweDismissOnTouchLayout"));
    assert!(dev.content.contains("new PopupWindow("));
    assert!(dev.content.contains("setElevation(doweDp(8))"));
    assert!(dev.content.contains("setBackgroundColor(DOWE_SURFACE)"));
    assert!(dev.content.contains("showAsDropDown("));
    assert!(
        dev.content
            .contains("doweInputBackground(DOWE_BACKGROUND, null, DOWE_RADIUS)")
    );
    assert!(dev.content.contains("doweNavMenuArrow(DOWE_BACKGROUND_TEXT)"));
    assert!(dev.content.contains("setOnDismissListener"));
    assert!(dev.content.contains(".isShowing()) {"));
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


