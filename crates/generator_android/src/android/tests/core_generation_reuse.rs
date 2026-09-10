#[test]
fn reuses_identical_dev_layout_methods_across_routes() {
    let first = route();
    let mut second = route();
    second.id = "signup".to_string();
    second.route_path = "/signup".to_string();
    second.page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![text("Signup")],
    };

    let output = generate_android(
        &[first, second],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert_eq!(dev.content.matches("final class DoweDevLayout0").count(), 1);
    assert_eq!(
        dev.content
            .matches("DoweDevLayout0.render(this, root, pageRoot -> renderPage(this, pageRoot));")
            .count(),
        2
    );
}

#[test]
fn omits_absent_dev_padding_branches() {
    let props = StyleProps {
        spacing: SpacingProps {
            px: Some(responsive_scale(&[(Breakpoint::Xs, 8)])),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut output = String::new();

    super::apply_dev_android_style(&props, "view0", true, &mut output);

    assert!(output.contains("Integer view0PaddingX = doweResponsiveInt"));
    assert!(!output.contains("Integer view0Padding ="));
    assert!(!output.contains("Integer view0PaddingY ="));
    assert!(!output.contains("Integer view0PaddingLeft ="));
    assert!(!output.contains("Integer view0PaddingRight ="));
    assert!(!output.contains("Integer view0PaddingTop ="));
    assert!(!output.contains("Integer view0PaddingBottom ="));
}

#[test]
fn keeps_layouts_composed_when_page_reads_layout_state() {
    let mut contextual = route();
    contextual.layout_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "layout.message".to_string(),
            name: "message".to_string(),
            storage_key: "message".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::String("Layout message".to_string()),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Children],
        }],
    };
    contextual.page_tree = ViewNode::Text {
        props: Default::default(),
        value: "{message}".to_string(),
    };

    let output = generate_android(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert!(
        dev.content
            .contains("ViewGroup doweCreatePageContainer(ViewGroup parent)")
    );
    assert!(
        dev.content
            .contains(".render(this, doweCreatePageContainer(root));")
    );
    assert!(!dev.content.contains("final class DoweDevLayout0"));
    assert!(!dev.content.contains("private static void renderPage("));
    assert!(
        dev.content
            .contains("doweTextValue(\"layout.message\", null)")
    );
}

#[test]
fn reuses_stateful_dev_layout_when_page_does_not_read_layout_state() {
    let mut contextual = route();
    contextual.layout_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "layout.open".to_string(),
            name: "open".to_string(),
            storage_key: "open".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::Bool(false),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Children],
        }],
    };
    contextual.page_tree = ViewNode::Text {
        props: Default::default(),
        value: "Login".to_string(),
    };

    let output = generate_android(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert!(dev.content.contains("final class DoweDevLayout0"));
    assert!(dev.content.contains("private static void renderPage("));
    assert!(
        dev.content
            .contains("DoweDevLayout0.render(this, root, pageRoot -> renderPage(this, pageRoot));")
    );
    assert!(
        dev.content
            .contains("dowePutInitial(\"layout.open\", false);")
    );
}

#[test]
fn reuses_stateful_scaffold_drawer_layout_when_page_mentions_binding_literals() {
    let contextual = stateful_scaffold_drawer_layout_route(false);

    let output = generate_android(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert!(dev.content.contains("final class DoweDevLayout0"));
    assert!(dev.content.contains("private static void renderPage("));
    assert!(
        dev.content
            .contains("DoweDevLayout0.render(this, root, pageRoot -> renderPage(this, pageRoot));")
    );
    assert!(
        dev.content
            .contains("dowePutInitial(\"layout.drawer.open\", false);")
    );
    assert!(
        dev.content
            .contains("dowePutInitial(\"layout.drawer.visible\", true);")
    );
    assert!(
        dev.content
            .contains("doweActions.put(\"layout.drawer.open.action\", DoweAction.assign(\"layout.drawer.open\", \"layout.drawer.visible\"));")
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();
    assert!(generated.contains("Box(modifier = Modifier.fillMaxSize())"));
    assert!(generated.contains(
        "Row(modifier = Modifier.fillMaxWidth().weight(1f).verticalScroll(scrollState))"
    ));

    let boxed = stateful_scaffold_drawer_layout_route(true);
    let boxed_output = generate_android(
        &[boxed],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let boxed_generated = boxed_output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();
    assert!(boxed_generated.contains(
        "Box(modifier = Modifier.fillMaxWidth().weight(1f), contentAlignment = Alignment.TopCenter)"
    ));
    assert!(boxed_generated.contains(
        "Row(modifier = Modifier.widthIn(max = 1536.dp).fillMaxSize().verticalScroll(scrollState))"
    ));
}

#[test]
fn generates_compose_and_dev_section_backgrounds() {
    let output = generate_android(
        &[section_route()],
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
            .contains("private enum class DoweSectionBackground")
    );
    assert!(views.content.contains("DoweSectionBackgroundBox("));
    assert!(views.content.contains(
        "Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.TopCenter)"
    ));
    assert!(
        views.content.contains(
            "Column(modifier = Modifier.widthIn(max = 1536.dp).fillMaxWidth().dowePadding"
        )
    );
    assert!(views.content.contains("Column(modifier = Modifier.dowePadding(all = null, horizontal = doweResponsive(viewportWidth, xs = 16.dp, md = 24.dp), vertical = doweResponsive(viewportWidth, xs = 40.dp, md = 64.dp)"));
    assert!(views.content.contains("background = doweResponsive(viewportWidth, xs = DoweSectionBackground.Aurora, md = DoweSectionBackground.Ocean)"));
    assert!(views.content.contains("Brush.linearGradient(listOf(DoweDesign.primary, DoweDesign.secondary, DoweDesign.accent))"));
    assert!(views.content.contains("DoweCoverBox("));
    assert!(views.content.contains("https://example.com/hero.jpg"));
    assert!(
        views
            .content
            .contains("DoweOverlay.Solid(Color.Black.copy(alpha = 0.35f))")
    );

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("private GradientDrawable doweSectionBackground(String value)")
    );
    assert!(
        dev.content
            .contains("String view1SectionBackground = doweResponsiveString(viewportWidth, \"aurora\", null, \"ocean\", null, null)")
    );
    assert!(
        dev.content
            .contains("view1.setBackground(doweSectionBackground(view1SectionBackground));")
    );
    assert!(dev.content.contains("doweBoxedContainer(1536)"));
    assert!(
        dev.content
            .contains("PaddingX = doweResponsiveInt(viewportWidth, 16, null, 24, null, null)")
    );
    assert!(
        dev.content
            .contains("PaddingY = doweResponsiveInt(viewportWidth, 40, null, 64, null, null)")
    );
}

