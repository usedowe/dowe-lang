#[test]
fn generates_global_toasts_for_sequential_request_functions() {
    let mut sequential = route();
    sequential.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "session01".to_string(),
            name: "session".to_string(),
            storage_key: "session".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::Object(Vec::new()),
            schema: None,
        }],
        actions: vec![ViewAction {
            id: "login01".to_string(),
            name: "login".to_string(),
            params: Vec::new(),
            return_type: None,
            kind: ViewActionKind::Sequence(vec![
                ViewFunctionStatement::Request {
                    result: "res".to_string(),
                    action: ViewRequestAction {
                        method: ViewRequestMethod::Post,
                        path: "/api/auth/login".to_string(),
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
                    result: "res".to_string(),
                    success: vec![
                        ViewFunctionStatement::Assign(ViewAssignAction {
                            target: "session".to_string(),
                            source: "$dowe:literal".to_string(),
                            literal: Some(ViewSignalValue::Object(Vec::new())),
                            call: None,
                        }),
                        ViewFunctionStatement::Toast(ViewToastAction {
                            kind: "success".to_string(),
                            title: "Success".to_string(),
                            message: "Signed in".to_string(),
                            duration: Some(3000),
                            scheme: Some("success".to_string()),
                            variant: Some("solid".to_string()),
                            position: Some("top-right".to_string()),
                        }),
                    ],
                    error: vec![ViewFunctionStatement::Toast(ViewToastAction {
                        kind: "error".to_string(),
                        title: "Error".to_string(),
                        message: "Login failed".to_string(),
                        duration: None,
                        scheme: None,
                        variant: None,
                        position: None,
                    })],
                },
            ]),
        }],
        children: vec![text("Login")],
    };
    let output = generate_android(
        &[sequential],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);

    assert!(generated.contains("DoweAction.Sequence(listOf(DoweStep.Request"));
    assert!(generated.contains("DoweStep.Toast(\"success\", \"Success\", \"Signed in\", 3000"));
    assert!(generated.contains("DoweAction.sequence(new DoweStep[] {DoweStep.request"));
    assert!(generated.contains(
        "return new DoweStep(\"assign\", null, null, null, null, target, source, literal, hasLiteral, call, null, null, null, null, null, null);"
    ));

    assert!(generated.contains("DoweGlobalToast(toast = state.toast, close = state::closeToast, viewportWidth = viewportWidth)"));
    assert!(generated.contains("doweCardContainer(toast.variant, toast.scheme)"));
    assert!(generated.contains("doweShowToast(step);"));
    assert!(generated.contains("setContentDescription(\"Close toast\")"));
    assert!(generated.contains("contentDescription = \"Close toast\""));
    assert!(!generated.contains("android.widget.Toast.makeText"));
    assert!(generated.contains("signals.entries.lastOrNull { it.value.name == name }"));
    assert!(generated.contains("doweRequestPath(action.path, body, item)"));
}

#[test]
fn generates_terminal_replace_redirect_for_compose_and_dev_shell() {
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
    let output = generate_android(
        &[redirect_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);

    assert!(generated.contains("DoweStep.Redirect(\"/login\")"));
    assert!(generated.contains("redirectPath = step.path"));
    assert!(generated.contains("navigate(\"replace\", path, null)"));
    assert!(generated.contains("DoweStep.redirect(\"/login\")"));
    assert!(generated.contains("doweNavigate(\"replace\", step.target, null);"));
}

#[test]
fn generates_init_and_reactive_splash_for_compose_and_dev_shell() {
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
            vec![ViewFunctionStatement::Assign(ViewAssignAction {
                target: "isLoading".to_string(),
                source: "$dowe:bool:false".to_string(),
                literal: None,
                call: None,
            })],
        )],
        children: vec![ViewNode::Splash {
            binding: "isLoading".to_string(),
            initial: true,
            content: vec![text("Users"), fixed_fab_page()],
            children: vec![text("Loading users")],
        }],
    };
    let generated = all_android_source(&generate_android(
        &[splash_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(generated.contains("LaunchedEffect(Unit) { state.load(listOf(\"init01\")) }"));
    assert!(generated.contains("if (state.bool(\"loading01\"))"));
    assert!(generated.contains("if (!state.bool(\"loading01\"))"));
    assert!(generated.contains("if (doweBool(\"loading01\"))"));
    assert!(generated.contains("doweRunStartup(new String[] {\"init01\"})"));
}

#[test]
fn generates_immutable_compose_constants() {
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
    let output = generate_android(
        &[constant_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);
    assert!(
        generated
            .contains("constants = mapOf<String, Any?>(\"plans01\" to listOf<Any?>(\"Starter\"))")
    );
    assert!(generated.contains("private val constants: Map<String, Any?>"));
}

#[test]
fn generates_dynamic_image_source_for_compose_and_dev_shell() {
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
    let output = generate_android(
        &[image_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);
    assert!(generated.contains("DoweImage(source = state.text(\"item.cover\", row.value)"));
    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("doweImage(doweTextValue(\"item.cover\", row0)")
    );
}

#[test]
fn generates_compose_select_options_from_constant_each() {
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
    let output = generate_android(
        &[constant_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);
    assert!(generated.contains("state.rows(\"options01\").map { row -> DoweSelectOption"));
    assert!(generated.contains("state.text(\"item.value\", row.value)"));
    assert!(generated.contains("state.text(\"item.label\", row.value)"));
    assert!(generated.contains("doweRowTextValues(\"options01\", \"item.label\")"));
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
                (
                    "rounded".to_string(),
                    ViewSignalValue::String("full".to_string()),
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
                        rounded: Some("button.rounded".to_string()),
                        ..Default::default()
                    },
                    style: StyleProps {
                        shadow: Some(ResponsiveValue::scalar(ShadowSize::Sm)),
                        shadow_color: Some(ColorFamily::Secondary),
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("{button.label}")],
            }],
        }],
    };
    let output = generate_android(
        &[scoped_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);
    assert!(generated.contains("state.text(\"item.variant\", row.value)"));
    assert!(generated.contains("state.text(\"item.scheme\", row.value)"));
    assert!(
        generated.contains("doweButtonHorizontalPadding(state.text(\"item.size\", row.value))")
    );
    assert!(generated.contains("doweButtonMinHeight(state.text(\"item.size\", row.value))"));
    assert!(generated.contains("(\"outlined\".equals(doweTextValue(\"item.variant\", row"));
    assert!(generated.contains("state.text(\"item.label\", row.value)"));
    assert!(generated.contains("doweTextValue(\"item.scheme\", row"));
    assert!(generated.contains("setText(doweTextValue(\"item.label\", row"));
    assert!(generated.contains(
        "shape = RoundedCornerShape(doweButtonRadius(state.text(\"item.rounded\", row.value)))"
    ));
    assert!(
        generated.contains("DOWE_SECONDARY, doweButtonRadius(doweTextValue(\"item.rounded\", row")
    );
    assert!(!generated.contains(".doweRounded(doweResponsive(viewportWidth, xs = 8.dp))"));
    assert!(!generated.contains("doweResponsiveFloat(viewportWidth, 8f, null, null, null, null)"));
}

