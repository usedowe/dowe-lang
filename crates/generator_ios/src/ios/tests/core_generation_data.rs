#[test]
fn generates_immutable_swift_constants() {
    let mut constant_route = route();
    constant_route.page_tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "plans01".to_string(),
            name: "plans".to_string(),
            value: ViewSignalValue::Array(vec![ViewSignalValue::String("Starter".to_string())]),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![text("Plans")],
    };
    let output = generate_ios(
        &[constant_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = swift_content(&output);
    assert!(generated.contains("constants: [\"plans01\": [\"Starter\"]]"));
    assert!(generated.contains("private let constants: [String: Any]"));
}

#[test]
fn generates_dynamic_image_source_for_swiftui() {
    let mut image_route = route();
    image_route.page_tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "features01".to_string(),
            name: "features".to_string(),
            value: ViewSignalValue::Array(vec![ViewSignalValue::Object(vec![
                (
                    "id".to_string(),
                    ViewSignalValue::String("feature".to_string()),
                ),
                (
                    "cover".to_string(),
                    ViewSignalValue::String("/assets/feature.webp".to_string()),
                ),
            ])]),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![ViewNode::Each {
            item: "feature".to_string(),
            collection: "features".to_string(),
            key: "feature.id".to_string(),
            children: vec![ViewNode::Image {
                props: ImageProps {
                    style: VariantProps::default(),
                    src: String::new(),
                    reactive_src: Some("feature.cover".to_string()),
                    alt: "Feature".to_string(),
                    aspect: ImageAspect::Auto,
                    object_fit: ImageObjectFit::Cover,
                    loading: ImageLoading::Lazy,
                    hide_controls: true,
                },
            }],
        }],
    };
    let generated = swift_content(&generate_ios(
        &[image_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));
    assert!(
        generated.contains("DoweImageView(source: state.text(\"item.cover\", item: row.value)")
    );
    assert!(generated.contains("private func doweImageURL(_ source: String) -> URL?"));
    assert!(generated.contains("directory == \".\" ? \"assets\""));
}

#[test]
fn generates_init_sequence_and_reactive_splash_for_swiftui() {
    let mut splash_route = route();
    splash_route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "loading01".to_string(),
            name: "isLoading".to_string(),
            storage_key: "isLoading".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::Bool(true),
            schema: None,
        }],
        actions: vec![ViewAction::init(
            "init01".to_string(),
            vec![
                ViewFunctionStatement::Request {
                    result: "result".to_string(),
                    action: ViewRequestAction {
                        method: ViewRequestMethod::Get,
                        path: "/api/users".to_string(),
                        base_env: None,
                        headers: Vec::new(),
                        body: None,
                        update: None,
                        reset: None,
                        success_alert: None,
                        success_message: None,
                        error_alert: None,
                        error_message: None,
                        autoload: false,
                    },
                },
                ViewFunctionStatement::If {
                    result: "result".to_string(),
                    success: vec![
                        ViewFunctionStatement::Assign(ViewAssignAction {
                            target: "isLoading".to_string(),
                            source: "$dowe:bool:false".to_string(),
                            literal: None,
                            call: None,
                        }),
                        ViewFunctionStatement::Toast(ViewToastAction {
                            kind: "success".to_string(),
                            title: "Loaded".to_string(),
                            message: "Users loaded".to_string(),
                            duration: Some(1500),
                            scheme: Some("success".to_string()),
                            variant: Some("soft".to_string()),
                            position: Some("top-right".to_string()),
                        }),
                    ],
                    error: vec![
                        ViewFunctionStatement::Reset(dowe_components::ViewResetAction {
                            target: "isLoading".to_string(),
                        }),
                        ViewFunctionStatement::Toast(ViewToastAction {
                            kind: "error".to_string(),
                            title: "Error".to_string(),
                            message: "Users failed".to_string(),
                            duration: None,
                            scheme: None,
                            variant: None,
                            position: None,
                        }),
                    ],
                },
            ],
        )],
        children: vec![ViewNode::Splash {
            binding: "isLoading".to_string(),
            initial: true,
            content: vec![text("Users"), fixed_fab_page()],
            children: vec![text("Loading users")],
        }],
    };
    let generated = swift_content(&generate_ios(
        &[splash_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(generated.contains(".task { state.load([\"init01\"]) }"));
    assert!(generated.contains(".sequence([.request(\"result\""));
    assert!(generated.contains(".branch(\"result\""));
    assert!(generated.contains(".assign(\"loading01\""));
    assert!(generated.contains(".reset(\"loading01\")"));
    assert!(generated.contains(".toast(\"success\", \"Loaded\", \"Users loaded\", 1500"));
    assert!(generated.contains("if state.bool(\"loading01\")"));
    assert!(generated.contains("if !state.bool(\"loading01\")"));
}

#[test]
fn generates_terminal_replace_redirect_for_swiftui() {
    let mut redirect_route = route();
    redirect_route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: vec![ViewAction::init(
            "init01".to_string(),
            vec![ViewFunctionStatement::Redirect {
                path: "/login".to_string(),
            }],
        )],
        children: vec![text("Home")],
    };
    let generated = swift_content(&generate_ios(
        &[redirect_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(generated.contains(".redirect(\"/login\")"));
    assert!(generated.contains("redirectPath = path"));
    assert!(generated.contains("navigate(\"replace\", path, nil)"));
    assert!(generated.contains("if await runSteps"));
}

#[test]
fn resolves_button_values_from_each_item_scope() {
    let mut scoped_route = route();
    scoped_route.page_tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "buttons01".to_string(),
            name: "buttons".to_string(),
            value: ViewSignalValue::Array(vec![ViewSignalValue::Object(vec![
                (
                    "id".to_string(),
                    ViewSignalValue::String("success".to_string()),
                ),
                (
                    "label".to_string(),
                    ViewSignalValue::String("Success".to_string()),
                ),
                (
                    "variant".to_string(),
                    ViewSignalValue::String("solid".to_string()),
                ),
                (
                    "scheme".to_string(),
                    ViewSignalValue::String("success".to_string()),
                ),
                (
                    "size".to_string(),
                    ViewSignalValue::String("lg".to_string()),
                ),
            ])]),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![ViewNode::Each {
            item: "button".to_string(),
            collection: "buttons".to_string(),
            key: "button.id".to_string(),
            children: vec![ViewNode::Button {
                props: VariantProps {
                    reactive: ReactiveVariantProps {
                        variant: Some("button.variant".to_string()),
                        scheme: Some("button.scheme".to_string()),
                        size: Some("button.size".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("{button.label}")],
            }],
        }],
    };
    let output = generate_ios(
        &[scoped_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();
    assert!(generated.contains("state.text(\"item.variant\", item: row.value)"));
    assert!(generated.contains("state.text(\"item.scheme\", item: row.value)"));
    assert!(
        generated
            .contains("doweButtonHorizontalPadding(state.text(\"item.size\", item: row.value))")
    );
    assert!(generated.contains("doweButtonMinHeight(state.text(\"item.size\", item: row.value))"));
    assert!(generated.contains("state.text(\"item.label\", item: row.value)"));
}

#[test]
fn generates_swift_select_options_from_constant_each() {
    let mut constant_route = route();
    constant_route.page_tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "options01".to_string(),
            name: "options".to_string(),
            value: ViewSignalValue::Array(Vec::new()),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![ViewNode::Select {
            props: Default::default(),
            options: Vec::new(),
            option_each: Some(SelectOptionEach {
                item: "option".to_string(),
                collection: "options".to_string(),
                key: "option.id".to_string(),
                value: "option.value".to_string(),
                label: "option.label".to_string(),
                description: None,
            }),
        }],
    };
    let output = generate_ios(
        &[constant_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = swift_content(&output);
    assert!(generated.contains("state.rows(\"options01\").map { row in DoweSelectOption"));
    assert!(generated.contains("state.text(\"item.value\", item: row.value)"));
    assert!(generated.contains("state.text(\"item.label\", item: row.value)"));
}

fn swift_content(output: &IosOutput) -> String {
    output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .extension()
                .and_then(|value| value.to_str())
                == Some("swift")
        })
        .map(|file| file.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

