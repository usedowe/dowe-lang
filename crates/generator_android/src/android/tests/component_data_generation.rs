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
    assert!(
        views
            .content
            .contains("DoweTable(state = state, dataPath = \"users\"")
    );
    assert!(views.content.contains("DoweTableColumn(field = \"status\", label = \"Status\", align = DoweTableColumnAlign.End, width = \"8rem\")"));
    assert!(views.content.contains("size = DoweTableSize.Lg"));
    assert!(
        views
            .content
            .contains("striped = true, bordered = true, dividers = true")
    );
    assert!(views.content.contains("emptyTitle = \"No users\""));
    assert!(
        views
            .content
            .contains("backgroundColor = DoweDesign.surface")
    );
    assert!(
        views
            .content
            .contains("contentColor = DoweDesign.onSurface")
    );
    assert!(views.content.contains("state.rows(dataPath)"));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
    assert!(dev.content.contains("private LinearLayout doweTable("));
    assert!(
        dev.content
            .contains("LinearLayout view0 = doweTable(\"users\"")
    );
    assert!(dev.content.contains("new String[]{\"name\", \"status\"}"));
    assert!(
        dev.content
            .contains("new int[]{Gravity.START, Gravity.END}")
    );
    assert!(
        dev.content
            .contains("doweTableValue(rows.get(rowIndex), fields[columnIndex])")
    );
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

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
    assert!(dev.content.contains("new View(this)"));
    assert!(dev.content.contains("setBackgroundColor(DOWE_PRIMARY)"));
    assert!(
        dev.content.contains(
            "new LinearLayout.LayoutParams(doweDp(1), ViewGroup.LayoutParams.MATCH_PARENT)"
        )
    );
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
    assert!(views.content.contains(
        "fun LoginScreen(viewportWidth: Dp, sectionRegistry: DoweSectionRegistry, navigate:"
    ));
    assert!(
        views
            .content
            .contains("doweResponsive(viewportWidth, xs = 16.dp, md = 32.dp)")
    );
    assert!(
        views
            .content
            .contains("doweResponsive(viewportWidth, md = 32.dp)")
    );
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

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert!(
        dev.content
            .contains("viewportWidth = getResources().getConfiguration().screenWidthDp;")
    );
    assert!(
        dev.content
            .contains("int viewportWidth = this.viewportWidth;")
    );
    assert!(
        dev.content
            .contains("doweResponsiveInt(viewportWidth, 16, null, 32, null, null)")
    );
    assert!(
        dev.content
            .contains("doweResponsiveInt(viewportWidth, null, null, 32, null, null)")
    );
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

    assert!(
        views
            .content
            .contains("if (doweResponsive(viewportWidth, xs = false, md = true) ?: true) {")
    );
    assert!(views.content.contains("if (state.bool(\"ready01\")) {"));
    assert!(
        views
            .content
            .contains("if (state.bool(\"item.ready\", row.value)) {")
    );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

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
            "verticalAlignment = doweVerticalAlignment(doweResponsive(viewportWidth, xs = DoweAlign.Center))"
        ));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert!(
        dev.content
            .contains("private static final class DoweFlexLayout extends ViewGroup")
    );
    assert!(dev.content.contains(
            "DoweFlexLayout view0 = doweFlex(doweResponsiveInt(viewportWidth, DOWE_JUSTIFY_END, null, null, null, null), doweResponsiveInt(viewportWidth, DOWE_ALIGN_CENTER, null, null, null, null), doweResponsiveInt(viewportWidth, 12, null, null, null, null))"
        ));
}

#[test]
fn generates_fragment_aware_native_history_and_deep_links() {
    let output = generate_android(
        &[index_route_with_signup_link(), signup_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");

    assert!(dev.content.contains("private String currentPath = \"/\";"));
    assert!(
        dev.content
            .contains("private String currentFragment = null;")
    );
    assert!(
        dev.content
            .contains("private static final class DoweRouteEntry")
    );
    assert!(
        dev.content
            .contains("private void renderIndex(LinearLayout root)")
    );
    assert!(
        dev.content
            .contains("private void renderSignup(LinearLayout root)")
    );
    assert!(
        dev.content
            .contains("setOnClickListener(v -> doweNavigate(\"push\", \"/signup\", \"join\"))")
    );
    assert!(
        dev.content
            .contains("setOnClickListener(v -> doweNavigate(\"replace\", currentPath, \"hero\"))")
    );
    assert!(dev.content.contains("setOnClickListener(v -> doweBack())"));
    assert!(dev.content.contains(
            "setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT))"
        ));
    assert!(dev.content.contains("setAllCaps(false)"));
    assert!(
        dev.content
            .contains("backStack.add(new DoweRouteEntry(currentPath, currentFragment));")
    );
    assert!(dev.content.contains("private boolean doweCanSection"));
    assert!(dev.content.contains("data.getFragment()"));
    assert!(dev.content.contains("doweRegisterBackHandler();"));
    assert!(
        dev.content
            .contains("getOnBackInvokedDispatcher().registerOnBackInvokedCallback(")
    );
    assert!(dev.content.contains("this::doweBack"));
    assert!(
        dev.content
            .contains("public void onBackPressed() {\n        doweBack();")
    );
    assert!(
        dev.content
            .contains("\"/\".equals(path) || \"/signup\".equals(path)")
    );
    assert!(dev.content.contains("doweApplyIntentRoute();"));
    assert!(dev.content.contains("doweScrollToFragment();"));
    assert!(
        dev.content
            .contains("scrollView.scrollTo(0, doweTopRelativeToRoot(target));")
    );
    assert!(dev.content.contains(r#"doweRegisterSection("hero", "#));

    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private data class DoweRouteEntry"));
    assert!(
        views
            .content
            .contains("fun navigate(operation: String, target: String, fragment: String?)")
    );
    assert!(
        views
            .content
            .contains(r#"{ navigate("push", "/signup", "join") }"#)
    );
    assert!(
        views
            .content
            .contains(r#"{ navigate("replace", "", "hero") }"#)
    );
    assert!(views.content.contains("BackHandler(enabled = true)"));
    assert!(views.content.contains("class DoweSectionRegistry"));
    assert!(
        views
            .content
            .contains("scrollState.animateScrollTo(targetSection)")
    );
    assert!(
        views
            .content
            .contains(r#".doweSection(sectionRegistry, "hero")"#)
    );

    let main_activity = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("MainActivity.kt"))
        .expect("main activity");
    assert!(
        main_activity
            .content
            .contains("override fun onNewIntent(intent: Intent)")
    );
    assert!(
        main_activity
            .content
            .contains("intent?.data?.fragment?.takeIf")
    );
    assert!(main_activity.content.contains("incomingRequest += 1"));
    assert!(views.content.contains("LaunchedEffect(navigationRequest)"));
}
