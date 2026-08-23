#[test]
fn generates_shared_swiftui_layout_once_for_multiple_routes() {
    let mut first = route();
    first.layout_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Box {
                props: Default::default(),
                children: vec![text("Layout")],
            },
            ViewNode::Children,
        ],
    };
    let mut second = first.clone();
    second.route_path = "/signup".to_string();
    second.page_tree = ViewNode::Text {
        props: Default::default(),
        value: "Signup".to_string(),
    };

    let output = generate_ios(
        &[first, second],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts_index = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayouts.swift"))
        .expect("layouts index");
    let layout = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayout0.swift"))
        .expect("layout");
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");
    let signup = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageSignupView.swift"))
        .expect("signup");

    assert_eq!(
        layout
            .content
            .matches("struct DoweLayout0<Content: View>")
            .count(),
        1
    );
    assert!(!layouts_index.content.contains("struct DoweLayout"));
    assert_eq!(
        layout.content.matches("Text(verbatim: \"Layout\")").count(),
        1
    );
    assert!(layout.content.contains("layoutSection0()"));
    assert!(
        layout
            .content
            .contains("private func layoutSection0() -> some View")
    );
    assert!(login.content.contains("DoweLayout0("));
    assert!(signup.content.contains("DoweLayout0("));
    assert!(!login.content.contains("Text(verbatim: \"Layout\")"));
    assert!(!signup.content.contains("Text(verbatim: \"Layout\")"));
    assert!(login.content.contains("Text(verbatim: \"Login\")"));
    assert!(signup.content.contains("Text(verbatim: \"Signup\")"));
}

#[test]
fn generates_reusable_swiftui_layouts_as_independent_files() {
    let mut first = route();
    first.layout_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![text("First layout"), ViewNode::Children],
    };
    let mut second = route();
    second.route_path = "/signup".to_string();
    second.layout_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![text("Second layout"), ViewNode::Children],
    };

    let output = generate_ios(
        &[first, second],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let first_layout = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayout0.swift"))
        .expect("first layout");
    let second_layout = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayout1.swift"))
        .expect("second layout");

    assert!(
        first_layout
            .content
            .contains("Text(verbatim: \"First layout\")")
    );
    assert!(
        !first_layout
            .content
            .contains("Text(verbatim: \"Second layout\")")
    );
    assert!(
        second_layout
            .content
            .contains("Text(verbatim: \"Second layout\")")
    );
    assert!(
        !second_layout
            .content
            .contains("Text(verbatim: \"First layout\")")
    );
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

    let output = generate_ios(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts = swift_content(&output);
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");

    assert!(!layouts.contains("struct DoweLayout0<"));
    assert!(!login.content.contains("DoweLayout0("));
    assert!(login.content.contains("state.text(\"layout.message\")"));
}

#[test]
fn reuses_stateful_swiftui_layout_when_page_does_not_read_layout_state() {
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

    let output = generate_ios(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts = swift_content(&output);
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");

    assert!(layouts.contains("struct DoweLayout0<"));
    assert!(login.content.contains("DoweLayout0("));
    assert!(login.content.contains("\"layout.open\": false"));
}

#[test]
fn reuses_stateful_scaffold_drawer_layout_when_page_mentions_binding_literals() {
    let contextual = stateful_scaffold_drawer_layout_route(false);

    let output = generate_ios(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts = swift_content(&output);
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");

    assert!(layouts.contains("struct DoweLayout0<"));
    assert!(layouts.contains("DoweDrawer(open: state.bool(\"layout.drawer.open\")"));
    assert_eq!(layouts.matches("private func layoutSection").count(), 5);
    assert!(layouts.contains("layoutSection0()"));
    assert!(layouts.contains("layoutSection1()"));
    assert!(layouts.contains("layoutSection2()"));
    assert!(login.content.contains("DoweLayout0("));
    assert!(login.content.contains("\"layout.drawer.open\": false"));
    assert!(login.content.contains("\"layout.drawer.visible\": true"));
    assert!(login.content.contains(
        "\"layout.drawer.open.action\": .assign(\"layout.drawer.open\", \"layout.drawer.visible\", nil, DoweActionMetadata(params: [:], returnType: nil))"
    ));
    assert!(
        !login
            .content
            .contains("DoweDrawer(open: state.bool(\"layout.drawer.open\")")
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();
    assert!(
        !login
            .content
            .contains("ScrollView {\n                DoweLayout0(")
    );
    assert!(generated.contains("ScrollView {"));
    assert!(layouts.contains(
        "content\n                        }\n                        .frame(maxWidth: .infinity, alignment: .topLeading)"
    ));
    assert!(!layouts.contains(
        "content\n                        }\n                        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)"
    ));
    assert!(layouts.contains(
        "                    }\n                    .frame(maxWidth: .infinity, alignment: .topLeading)\n                    }\n                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)"
    ));

    let boxed = stateful_scaffold_drawer_layout_route(true);
    let boxed_output = generate_ios(
        &[boxed],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let boxed_layouts = swift_content(&boxed_output);
    assert!(boxed_layouts.contains(".frame(maxWidth: CGFloat(1536), alignment: .topLeading)"));
    assert!(boxed_layouts.contains(".frame(maxWidth: .infinity, alignment: .top)"));
}

