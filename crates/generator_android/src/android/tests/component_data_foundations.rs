#[test]
fn keeps_implicit_box_and_theme_select_visible_inside_dev_flex() {
    let output = generate_android(
        &[flex_box_theme_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);
    assert_eq!(dev.content.matches("doweWrapContentWidth(").count(), 3);
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
fn generates_android_diagram_with_viewport_and_connection_runtime() {
    let output = generate_android(
        &[diagram_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweDiagram("));
    assert!(views.content.contains(
        "DoweDiagram(state = state, nodesPath = \"flowNodes\", edgesPath = \"flowEdges\", fitView = true"
    ));
    assert!(views.content.contains("controls = true"));
    assert!(views.content.contains("minimap = true"));
    assert!(views.content.contains("fun fitViewport()"));
    assert!(views.content.contains("fun zoomAtCenter(factor: Float)"));
    assert!(views.content.contains("fun borderPoint(node: DoweDiagramNode, towardX: Float, towardY: Float): Offset"));
    assert!(views.content.contains("fun persistConnection(source: String, target: String)"));
    assert!(views.content.contains("fun updateNode(node: DoweDiagramNode, x: Float, y: Float)"));
    assert!(views.content.contains("PathEffect.dashPathEffect(floatArrayOf(6f, 4f))"));
    assert!(views.content.contains("private fun DoweDiagramControlButton("));
    assert!(views.content.contains("detectTransformGestures"));
    assert!(views.content.contains("actionScope.launch { state.run(onNodeClick, item) }"));
    assert!(views.content.contains("state.run(onConnect, item)"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("private DoweDiagramView doweDiagram("));
    assert!(dev.content.contains("boolean controls, boolean minimap, boolean showGrid"));
    assert!(dev.content.contains("private final class DoweDiagramView extends View"));
    assert!(dev.content.contains("android.view.ScaleGestureDetector"));
    assert!(dev.content.contains("DashPathEffect"));
    assert!(dev.content.contains("void applyFitView()"));
    assert!(dev.content.contains("void drawMinimap(android.graphics.Canvas canvas)"));
    assert!(dev.content.contains("void drawControls(android.graphics.Canvas canvas)"));
    assert!(dev.content.contains("void moveViewportToMinimap(float x, float y)"));
    assert!(dev.content.contains("doweDiagram(\"flowNodes\", \"flowEdges\", true, true, true, true, true, true"));
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
        .contains("background(DoweDesign.muted)"));
    assert!(views
        .content
        .contains("DoweDesign.surfaceText.copy(alpha = 0.12f)"));
    assert!(views
        .content
        .contains("DoweDesign.surfaceText.copy(alpha = 0.28f)"));
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
    assert!(dev.content.contains("header.setBackgroundColor(DOWE_MUTED)"));
    assert!(dev
        .content
        .contains("doweAlpha(DOWE_SURFACE_TEXT, 0.12f)"));
    assert!(dev
        .content
        .contains("doweAlpha(DOWE_SURFACE_TEXT, 0.28f)"));
    assert!(dev.content.contains(
        "bordered\n                ? doweInputBackground(backgroundColor, doweAlpha(DOWE_SURFACE_TEXT, 0.28f), DOWE_RADIUS)"
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

