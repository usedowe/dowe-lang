#[test]
fn generates_android_svg_logo_paths_for_compose_and_dev() {
    let logo = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("svg-logos:github-icon".to_string()),
    }])
    .expect("SVG logo");
    let output = generate_android(
        &[ViewRoute {
            id: "logo".to_string(),
            route_path: "/logo".to_string(),
            layout_tree: ViewNode::Children,
            page_tree: logo,
            sections: Vec::new(),
            navigation_actions: Vec::new(),
        }],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();

    assert!(generated.contains("DoweSvgFill.Fill(Color(0xFF"));
    assert!(generated.contains("Color.rgb("));
}

#[test]
fn generates_dynamic_icon_lookup_for_android_dev_launcher() {
    let dynamic = icon_component_node(vec![
        ComponentProp {
            name: "name".to_string(),
            value: PropValue::String("@icon-binding:platform.icon".to_string()),
        },
        ComponentProp {
            name: "fill".to_string(),
            value: PropValue::String("muted".to_string()),
        },
    ])
    .expect("dynamic icon");
    let output = generate_android(
        &[ViewRoute {
            id: "dynamic-icon".to_string(),
            route_path: "/dynamic-icon".to_string(),
            layout_tree: ViewNode::Children,
            page_tree: dynamic,
            sections: Vec::new(),
            navigation_actions: Vec::new(),
        }],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev_route = output
        .files
        .iter()
        .find(|file| {
            file.relative_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("DoweDevRoute") && name.ends_with(".java"))
        })
        .expect("dynamic route shard");
    let dev_catalog = output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("DoweDevDynamicIcons") && name.ends_with(".java")
                })
        })
        .map(|file| file.content.as_str())
        .collect::<String>();
    assert!(dev_route.content.contains("runtime.doweDynamicIconPayload"));
    assert!(dev_route.content.contains("DOWE_MUTED"));
    assert!(dev_catalog.contains("route-bold-duotone"));
    assert!(dev_catalog.contains("svg-logos:android-icon"));
    assert!(dev_catalog.contains("values.put(\"svg-logos:apache-flink\", joinPayload("));
}

#[test]
fn generates_compose_and_dev_display_chat_and_motion_components() {
    let output = generate_android(
        &[display_chat_motion_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweAvatarGroup("));
    assert!(
        views.content.contains(
            "val visibleItems = maxCount?.let { items.take(it.coerceAtLeast(1)) } ?: items"
        )
    );
    assert!(!views.content.contains("imageLoadFinished &&"));
    assert!(
        !views
            .content
            .contains("items.take((visibleLimit - 1).coerceAtLeast(0))")
    );
    assert!(
        views
            .content
            .contains("DoweAvatarGroup(items = doweAvatarGroupItems(state.rows(\"people\")")
    );
    assert!(
        views
            .content
            .contains("DoweChatBox(state = state, messagesPath = \"messages\"")
    );
    assert!(views.content.contains("actionLabel = \"Continue\""));
    assert!(views.content.contains("DoweChatQuestion"));
    assert!(views.content.contains("private fun doweChatIsSpanish"));
    assert!(views.content.contains("Supuestos de trabajo"));
    assert!(views.content.contains("Choose an option above to continue."));
    assert!(views.content.contains("question.options.forEach"));
    assert!(
        views
            .content
            .contains("actionVisible = state.bool(\"actionVisible\")")
    );
    assert!(
        views
            .content
            .contains("if (actionVisible && onAction != null && !sending && !pendingChoice)")
    );
    assert!(views.content.contains("DoweEmpty(kind = \"result\""));
    assert!(
        views
            .content
            .contains("iconViewBox = DoweSvgViewBox(0f, 0f, 24f, 24f)")
    );
    assert!(views.content.contains("iconPaths = DoweSvgPathShard"));
    assert!(!views.content.contains("private fun DoweEmptyIcon("));
    assert!(views.content.contains("DoweMarquee(speed = \"fast\""));
    assert!(
        views
            .content
            .contains("DoweTypeWriter(texts = listOf(\"Hello\", \"World\")")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("doweAvatarGroup("));
    assert!(dev.content.contains("doweRows(dataPath)"));
    assert!(
        dev.content
            .contains("source = doweTextValue(\"item.src\", row);")
    );
    assert!(
        dev.content
            .contains("name = doweTextValue(\"item.name\", row);")
    );
    assert!(
        dev.content
            .contains("alt = doweTextValue(\"item.alt\", row);")
    );
    assert!(
        dev.content
            .contains("if (assetPath.startsWith(\"assets/\")) assetPath = assetPath.substring(7);")
    );
    assert!(dev.content.contains("doweText(\"Chat\""));
    assert!(dev.content.contains("doweText(\"Nothing found\""));
    assert!(dev.content.contains("doweText(\"Hello World\""));
}

#[test]
fn generates_compose_and_dev_rich_control_map_components() {
    let output = generate_android(
        &[rich_control_map_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweRichText("));
    assert!(
        views
            .content
            .contains("Layout(modifier = modifier, content = {")
    );
    assert!(
        views
            .content
            .contains("val childConstraints = constraints.copy(minWidth = 0, minHeight = 0)")
    );
    assert!(
        views
            .content
            .contains("val contentWidth = lines.maxOfOrNull { it.width } ?: 0")
    );
    assert!(
        views
            .content
            .contains("val layoutWidth = constraints.constrainWidth(contentWidth)")
    );
    assert!(
        views
            .content
            .contains("var lineLeft = ((layoutWidth - line.width) / 2).coerceAtLeast(0)")
    );
    assert!(views.content.contains("private fun DoweRichTextRun("));
    assert!(views.content.contains("textAlign = TextAlign.Center"));
    assert!(
        views
            .content
            .contains("var measuredTextWidth by remember(mark.text, fontFamily, fontSize)")
    );
    assert!(views.content.contains("onTextLayout = { layout ->"));
    assert!(
        views
            .content
            .contains("layout.getLineRight(index) - layout.getLineLeft(index)")
    );
    assert!(
        views
            .content
            .contains("decoration.then(measuredTextWidth?.let { Modifier.width(it) } ?: Modifier)")
    );
    assert!(
        views
            .content
            .contains("background(accent).padding(horizontal = 8.dp, vertical = 2.dp)")
    );
    assert!(views.content.contains("doweButtonTextFamily(mark.scheme)"));
    assert!(views.content.contains("doweButtonFamily(mark.scheme)"));
    assert!(views.content.contains(
        "if (contentColor == Color.Unspecified) DoweDesign.backgroundText else contentColor"
    ));
    assert!(
        views
            .content
            .contains("DoweRichText(marks = listOf(DoweRichTextMark(text = \"Launch\", style = \"grad\", scheme = \"primary\")")
    );
    assert!(views.content.contains("DoweRecord(name = \"voice\""));
    assert!(
        views
            .content
            .contains("DoweToggleGroup(value = state.text(\"mode\")")
    );
    assert!(views.content.contains("ButtonDefaults"));
    assert!(
        views
            .content
            .contains("DowePagination(value = state.text(\"page\")")
    );
    assert!(views.content.contains(
        "pageCount = maxOf(1, minOf(25, ((state.text(\"total\").toIntOrNull() ?: 0).coerceAtLeast(0) + 59) / 60))"
    ));
    assert!(views.content.contains("previousIcon = {"));
    assert!(views.content.contains("nextIcon = {"));
    assert!(
        views
            .content
            .contains("DoweCollapsible(label = \"Details\"")
    );
    assert!(views.content.contains("arrowIcon = {"));
    assert!(
        !views
            .content
            .contains("Text(text = if (open) \"⌃\" else \"⌄\"")
    );
    assert!(
        views
            .content
            .contains("DoweCountdown(target = \"2030-01-01T00:00:00Z\"")
    );
    assert!(
        views
            .content
            .contains("fillMaxWidth().horizontalScroll(rememberScrollState())")
    );
    assert!(views.content.contains("Modifier.widthIn(min = width)"));
    assert!(
        views
            .content
            .contains("BoxWithConstraints(modifier = modifier.fillMaxWidth())")
    );
    assert!(
        views.content.contains(
            "val displaySize = if (maxWidth < 480.dp && size != \"sm\") \"sm\" else size"
        )
    );
    assert!(views.content.contains("while (!completed)"));
    assert!(
        views
            .content
            .contains("DoweMap(centerLat = \"4.7109\", centerLng = \"-74.0721\"")
    );
    assert!(views.content.contains("DoweMapMarker(id = \"office\""));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("doweRichTextMark("));
    assert!(dev.content.contains("DoweFlexLayout"));
    assert!(
        dev.content
            .contains("DOWE_JUSTIFY_CENTER, DOWE_ALIGN_CENTER, 4")
    );
    assert!(dev.content.contains("view.setGravity(Gravity.CENTER);"));
    assert!(
        dev.content
            .contains("view.setBreakStrategy(android.text.Layout.BREAK_STRATEGY_SIMPLE);")
    );
    assert!(
        dev.content.contains(
            "view.setHyphenationFrequency(android.text.Layout.HYPHENATION_FREQUENCY_NONE);"
        )
    );
    assert!(
        dev.content
            .contains("private static final class DoweRichTextView extends TextView")
    );
    assert!(
        dev.content
            .contains("protected void onMeasure(int widthSpec, int heightSpec)")
    );
    assert!(
        dev.content
            .contains("if (MeasureSpec.getMode(widthSpec) == MeasureSpec.EXACTLY)")
    );
    assert!(dev.content.contains("layout.getLineWidth(index)"));
    assert!(dev.content.contains(
        "super.onMeasure(MeasureSpec.makeMeasureSpec(resolvedWidth, MeasureSpec.EXACTLY), heightSpec);"
    ));
    assert!(dev.content.contains("doweRichTextView(\"Launch\""));
    assert!(dev.content.contains("doweRichTextView(\"ready\""));
    assert!(dev.content.contains("doweText(\"voice\""));
    assert!(dev.content.contains("doweTextValue(\"mode\", null)"));
    assert!(dev.content.contains("doweTextValue(\"mode\", null)"));
    assert!(
        dev.content
            .contains("doweWrite(\"mode\", \"list\"); renderCurrentRoute(false);")
    );
    assert!(
        dev.content
            .contains("doweWrite(\"mode\", \"map\"); renderCurrentRoute(false);")
    );
    assert!(dev.content.contains(".equals(view"));
    assert!(dev.content.contains("Active) { doweWrite"));
    assert!(dev.content.contains("doweText(\"Details\""));
    assert!(
        dev.content
            .contains("doweCountdown(\"2030-01-01T00:00:00Z\"")
    );
    assert!(
        dev.content
            .contains("private HorizontalScrollView doweCountdown(")
    );
    assert!(dev.content.contains(
        "String displaySize = viewportWidth < 480 && !\"sm\".equals(size) ? \"sm\" : size;"
    ));
    assert!(dev.content.contains("doweWrapContentWidth(column);"));
    assert!(
        dev.content
            .contains("java.time.Instant.parse(target).toEpochMilli()")
    );
    assert!(dev.content.contains("if (deadline <= current)"));
    assert!(
        dev.content
            .contains("if (onComplete != null) onComplete.run();")
    );
    assert!(dev.content.contains("update[0].run();"));
    assert!(dev.content.contains("doweText(\"Office\""));
    assert!(
        dev.content
            .contains("setContentDescription(\"Previous page\")")
    );
    assert!(dev.content.contains("setContentDescription(\"Next page\")"));
}

