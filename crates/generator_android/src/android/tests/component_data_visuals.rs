#[test]
fn generates_android_tree_for_compose_and_dev_runtime() {
    let output = generate_android(
        &[tree_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweTree("));
    assert!(views.content.contains("DoweTree(state = state, dataPath = \"fileTree\", bindPath = \"selectedFile\""));
    assert!(views.content.contains("state.treeNodes(dataPath)"));
    assert!(views.content.contains("openIds[node.id] = !open"));
    assert!(views.content.contains("state.run(action, node.value)"));
    assert!(views.content.contains("folder-with-files"));
    assert!(views.content.contains("file-text"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("private LinearLayout doweTree("));
    assert!(dev.content.contains("LinearLayout view0 = doweTree(\"fileTree\", \"selectedFile\""));
    assert!(dev.content.contains("private static final class DoweTreeNode"));
    assert!(dev.content.contains("tree.open.put(node.id, nextOpen)"));
    assert!(dev.content.contains("doweRunAction(tree.onSelect, node.value)"));
    assert!(dev.content.contains("renderCurrentRoute(false);"));
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

