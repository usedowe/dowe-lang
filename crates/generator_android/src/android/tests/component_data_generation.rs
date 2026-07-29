#[test]
fn keeps_implicit_box_and_theme_select_visible_inside_dev_flex() {
    let output = generate_android(
        &[flex_box_theme_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);
    assert_eq!(dev.content.matches("doweWrapContentWidth(").count(), 2);
    assert!(dev.content.contains(
        "setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f))"
    ));
    assert!(dev.content.contains("doweSelectTrigger"));
    assert!(dev.content.contains("doweBindSelect"));
    assert!(dev.content.contains("value -> doweSetTheme(value)"));
    assert!(dev.content.contains("doweApplyTheme(name);"));
    assert!(dev.content.contains("renderCurrentRoute(false);"));
    assert!(!dev.content.contains("Spinner"));

    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("DoweThemeSelect("));
}

#[test]
fn expands_theme_select_to_remaining_android_flex_width() {
    let output = generate_android(
        &[flex_theme_button_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);
    assert!(dev.content.contains(
        "setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f))"
    ));

    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views
        .content
        .contains("DoweThemeSelect(modifier = Modifier.weight(1f)"));
    assert!(views
        .content
        .contains("Row(modifier = Modifier.fillMaxWidth()"));
}

#[test]
fn generates_android_table_for_compose_and_dev_runtime() {
    let output = generate_android(
        &[table_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweTable("));
    assert!(views
        .content
        .contains("DoweTable(state = state, dataPath = \"users\""));
    assert!(views.content.contains("DoweTableColumn(field = \"status\", label = \"Status\", align = DoweTableColumnAlign.End, width = \"8rem\")"));
    assert!(views.content.contains("size = DoweTableSize.Lg"));
    assert!(views
        .content
        .contains("striped = true, bordered = true, dividers = true"));
    assert!(views.content.contains("emptyTitle = \"No users\""));
    assert!(views
        .content
        .contains("backgroundColor = Color.Transparent"));
    assert!(views
        .content
        .contains("contentColor = DoweDesign.primary"));
    assert!(views.content.contains("borderColor = DoweDesign.primary"));
    assert!(views
        .content
        .contains("background(DoweDesign.softMuted)"));
    assert!(views
        .content
        .contains("DoweDesign.onSurface.copy(alpha = 0.12f)"));
    assert!(views
        .content
        .contains("DoweDesign.onSurface.copy(alpha = 0.28f)"));
    assert!(views.content.contains("state.rows(dataPath)"));
    assert!(views.content.contains(
        "columns.fold(0.dp) { total, column -> total + doweTableColumnWidth(column.width) }"
    ));
    assert!(views
        .content
        .contains("val tableWidth = maxOf(maxWidth, minimumWidth)"));
    assert!(views.content.contains(
        "val columnExpansion = (tableWidth - minimumWidth) / columns.size.coerceAtLeast(1).toFloat()"
    ));
    assert!(views.content.contains(
        "Modifier.width(doweTableColumnWidth(column.width) + columnExpansion)"
    ));
    assert!(!views.content.contains(
        "doweTableColumnWidth(column.width) + metrics.horizontalPadding * 2"
    ));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("private LinearLayout doweTable("));
    assert!(dev
        .content
        .contains("LinearLayout view0 = doweTable(\"users\""));
    assert!(dev.content.contains("new String[]{\"name\", \"status\"}"));
    assert!(dev
        .content
        .contains("new int[]{Gravity.START, Gravity.END}"));
    assert!(dev
        .content
        .contains("doweTableValue(rows.get(rowIndex), fields[columnIndex])"));
    assert!(dev
        .content
        .contains("Color.TRANSPARENT, DOWE_PRIMARY, DOWE_PRIMARY"));
    assert!(dev.content.contains("header.setBackgroundColor(DOWE_SOFT_MUTED)"));
    assert!(dev
        .content
        .contains("doweAlpha(DOWE_ON_SURFACE, 0.12f)"));
    assert!(dev
        .content
        .contains("doweAlpha(DOWE_ON_SURFACE, 0.28f)"));
    assert!(dev.content.contains(
        "bordered\n                ? doweInputBackground(backgroundColor, doweAlpha(DOWE_ON_SURFACE, 0.28f), DOWE_RADIUS)"
    ));
    assert!(dev.content.contains("scroll.setFillViewport(true);"));
    assert!(dev.content.contains(
        "new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT)"
    ));
    assert!(dev.content.contains(
        "new LinearLayout.LayoutParams(doweTableColumnWidth(width) - (reserveSeparator ? doweDp(1) : 0), ViewGroup.LayoutParams.WRAP_CONTENT, 1f)"
    ));
    assert!(dev.content.contains("private View doweTableSeparator()"));
    assert!(!dev
        .content
        .contains("cell.setBackground(doweInputBackground"));
    assert!(dev.content.contains("value += doweTableColumnWidth(width);"));
    assert!(!dev
        .content
        .contains("doweTableColumnWidth(width) + doweDp(horizontal * 2)"));
}

#[test]
fn generates_android_divider_with_native_view() {
    let output = generate_android(
        &[divider_route()],
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
        "Box(modifier = Modifier.width(1.dp).fillMaxHeight().background(DoweDesign.primary))"
    ));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("new View(this)"));
    assert!(dev.content.contains("setBackgroundColor(DOWE_PRIMARY)"));
    assert!(dev
        .content
        .contains("new LinearLayout.LayoutParams(doweDp(1), ViewGroup.LayoutParams.MATCH_PARENT)"));
}

#[test]
fn generates_compose_responsive_runtime_values() {
    let output = generate_android(
        &[responsive_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("BoxWithConstraints"));
    assert!(views.content.contains("val viewportWidth = maxWidth"));
    assert!(views.content.contains(
        "fun LoginScreen(viewportWidth: Dp, scrollState: ScrollState, sectionRegistry: DoweSectionRegistry, navigate:"
    ));
    assert!(views
        .content
        .contains("doweResponsive(viewportWidth, xs = 16.dp, md = 32.dp)"));
    assert!(views
        .content
        .contains("doweResponsive(viewportWidth, md = 32.dp)"));
    assert!(
            views
                .content
                .contains("doweResponsive(viewportWidth, md = doweTextSize(viewportWidth, min = 16f, preferredBase = 15.2f, preferredViewport = 0.3f, max = 18f)) ?: doweTextSize(viewportWidth, min = 14f, preferredBase = 13.12f, preferredViewport = 0.25f, max = 16f)")
        );
    assert!(
            views
                .content
                .contains("fontWeight = doweResponsive(viewportWidth, xs = FontWeight.Thin, md = FontWeight.ExtraLight, lg = FontWeight.Black) ?: FontWeight.Normal")
        );

    let dev = dev_java_source(&output);

    assert!(dev
        .content
        .contains("viewportWidth = getResources().getConfiguration().screenWidthDp;"));
    assert!(dev
        .content
        .contains("int viewportWidth = this.viewportWidth;"));
    assert!(dev
        .content
        .contains("doweResponsiveInt(viewportWidth, 16, null, 32, null, null)"));
    assert!(dev
        .content
        .contains("doweResponsiveInt(viewportWidth, null, null, 32, null, null)"));
    assert!(dev.content.contains(
        "doweTextWeight(doweResponsiveInt(viewportWidth, 100, null, 200, 900, null), 400)"
    ));
}
#[test]
fn generates_show_visibility_conditions() {
    let output = generate_android(
        &[show_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views
        .content
        .contains("if (doweResponsive(viewportWidth, xs = false, md = true) ?: true) {"));
    assert!(views.content.contains("if (state.bool(\"ready01\")) {"));
    assert!(views
        .content
        .contains("if (state.bool(\"item.ready\", row.value)) {"));

    let dev = dev_java_source(&output);

    assert!(dev.content.contains(
        "if (doweShow(doweResponsiveBool(viewportWidth, false, null, true, null, null))) {"
    ));
    assert!(dev.content.contains("if (doweBool(\"ready01\", null)) {"));
    assert!(dev.content.contains("if (doweBool(\"item.ready\", row"));
}

#[test]
fn generates_dev_flex_justify_and_align_gravity() {
    let output = generate_android(
        &[flex_alignment_route()],
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
            "horizontalArrangement = doweHorizontalArrangement(doweResponsive(viewportWidth, xs = DoweJustify.End), doweResponsive(viewportWidth, xs = 12.dp))"
        ));
    assert!(views.content.contains(
            "itemVerticalAlignment = doweVerticalAlignment(doweResponsive(viewportWidth, xs = DoweAlign.Center))"
        ));
    assert!(views.content.contains(
        "doweResponsive(viewportWidth, xs = DoweFlexDirection.Column, md = DoweFlexDirection.Row)"
    ));
    assert!(views.content.contains("Column(modifier ="));
    assert!(views.content.contains("FlowRow(modifier ="));
    assert!(views.content.contains(
        "verticalArrangement = doweVerticalArrangement(doweResponsive(viewportWidth, xs = DoweJustify.End), doweResponsive(viewportWidth, xs = 12.dp))"
    ));

    let dev = dev_java_source(&output);

    assert!(dev
        .content
        .contains("private static final class DoweFlexLayout extends ViewGroup"));
    assert!(dev.content.contains("DoweFlexLayout view0 = doweFlex("));
    assert!(dev.content.contains(
        "doweResponsiveInt(viewportWidth, DOWE_DIRECTION_COLUMN, null, DOWE_DIRECTION_ROW, null, null)"
    ));
    assert!(dev.content.contains("if (direction == DOWE_DIRECTION_COLUMN)"));
    assert!(dev.content.contains("doweFlex(doweResponsiveInt(viewportWidth"));
    assert!(dev.content.contains(", true,"));
    assert!(dev.content.contains("if (wrap)"));
    assert!(dev.content.contains("doweWrapContentWidth(view1)"));
}

#[test]
fn generates_fragment_aware_native_history_and_deep_links() {
    let output = generate_android(
        &[index_route_with_signup_link(), signup_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert!(dev.content.contains("private String currentPath = \"/\";"));
    assert!(dev
        .content
        .contains("private String currentFragment = null;"));
    assert!(dev
        .content
        .contains("private static final class DoweRouteEntry"));
    assert_eq!(dev.content.matches("final class DoweDevRoute").count(), 2);
    assert_eq!(
        dev.content
            .matches("static void render(DoweDevActivity this, LinearLayout root)")
            .count(),
        2
    );
    assert!(dev
        .content
        .contains("setOnClickListener(v -> doweNavigate(\"push\", \"/signup\", \"join\"))"));
    assert!(dev
        .content
        .contains("setOnClickListener(v -> doweNavigate(\"replace\", currentPath, \"hero\"))"));
    assert!(dev.content.contains("setOnClickListener(v -> doweBack())"));
    assert!(dev.content.contains(
            "setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT))"
        ));
    assert!(dev.content.contains("setAllCaps(false)"));
    assert!(dev
        .content
        .contains("backStack.add(new DoweRouteEntry(currentPath, currentFragment));"));
    assert!(dev.content.contains("private boolean doweCanSection"));
    assert!(dev.content.contains("data.getFragment()"));
    assert!(dev
        .content
        .contains("public void handleBack() {\n        doweBack();"));
    assert!(dev
        .content
        .contains("\"/\".equals(path) || \"/signup\".equals(path)"));
    assert!(dev.content.contains("doweApplyIntentRoute();"));
    assert!(dev.content.contains("doweScrollToFragment();"));
    assert!(dev.content.contains(
        "if (currentFragment == null) {\n                scrollView.scrollTo(0, 0);\n            } else {\n                doweScrollToFragment();\n            }"
    ));
    assert!(dev
        .content
        .contains("scrollView.scrollTo(0, doweTopRelativeToRoot(target));"));
    assert!(dev.content.contains(r#"doweRegisterSection("hero", "#));

    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private data class DoweRouteEntry"));
    assert!(views
        .content
        .contains("fun navigate(operation: String, target: String, fragment: String?)"));
    assert!(views
        .content
        .contains(r#"{ navigate("push", "/signup", "join") }"#));
    assert!(views
        .content
        .contains(r#"{ navigate("replace", "", "hero") }"#));
    assert!(views.content.contains("BackHandler(enabled = true)"));
    assert!(views.content.contains("class DoweSectionRegistry"));
    assert!(views
        .content
        .contains("LaunchedEffect(currentEntry.path) {\n        scrollState.scrollTo(0)\n    }"));
    assert!(views.content.contains(
        "if (currentEntry.fragment == null) {\n            scrollState.scrollTo(0)\n        } else if (targetSection != null)"
    ));
    assert!(views
        .content
        .contains("scrollState.animateScrollTo(targetSection)"));
    assert!(views
        .content
        .contains("viewportWidth: Dp, scrollState: ScrollState"));
    assert!(views
        .content
        .contains(r#".doweSection(sectionRegistry, "hero")"#));

    let main_activity = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("MainActivity.kt"))
        .expect("main activity");
    assert!(main_activity
        .content
        .contains("override fun onNewIntent(intent: Intent)"));
    assert!(main_activity
        .content
        .contains("intent?.data?.fragment?.takeIf"));
    assert!(main_activity.content.contains("incomingRequest += 1"));
    assert!(views.content.contains("LaunchedEffect(navigationRequest)"));
}
