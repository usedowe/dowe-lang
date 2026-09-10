#[test]
fn emits_persistent_view_store_metadata() {
    let tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "session01".to_string(),
            name: "session".to_string(),
            storage_key: "views/store/session:session".to_string(),
            scope: dowe_components::ViewSignalScope::Global,
            storage: dowe_components::ViewSignalStorage::Local,
            initial: ViewSignalValue::Object(vec![(
                "token".to_string(),
                ViewSignalValue::String(String::new()),
            )]),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![text("Session")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/auth.dowe"),
        "page authPage",
        &tree,
    );

    assert!(
        page.content
            .contains(r#""storageKey":"views/store/session:session""#)
    );
    assert!(
        page.content
            .contains(r#""scope":"global","storage":"local""#)
    );
}

#[test]
fn inherits_container_foreground_and_preserves_text_overrides() {
    let tree = container_foreground_tree();
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/views/pages/colors.dowe"),
        "page ColorsPage",
        &tree,
    );
    let html = render_page_body(&ViewNode::Children, &tree);

    assert!(html.contains(
        "<div class=\"box color-primaryText\"><p class=\"dowe-text text-md\">Box inherited</p><p class=\"dowe-text text-md color-danger\">Box override</p></div>"
    ));
    assert!(html.contains("<article class=\"card"));
    assert!(html.contains("is-solid is-muted"));
    assert!(html.contains("<p class=\"dowe-text text-md\">Card inherited</p>"));
    assert!(html.contains("<h2 class=\"dowe-title title-md\">Card title inherited</h2>"));
    assert!(html.contains("<h2 class=\"dowe-title title-md color-warning\">Card override</h2>"));
    assert!(
        page.css_content
            .contains(".color-primaryText{--dowe-content-text:var(--dowe-primaryText);--dowe-content-title:var(--dowe-primaryText);color:var(--dowe-primaryText);}")
    );
    assert!(page.css_content.contains(
        ".card.is-solid.is-muted{--dowe-content-text:var(--dowe-mutedText);--dowe-content-title:var(--dowe-mutedTitle);background-color:var(--dowe-muted);color:var(--dowe-mutedText);border-color:var(--dowe-muted);}"
    ));
}

#[test]
fn preserves_explicit_border_width_and_color_over_variant_defaults() {
    let tree = ViewNode::Card {
        props: VariantProps {
            style: StyleProps {
                border: Some(ResponsiveValue::scalar(dowe_components::BorderWidth(3))),
                border_color: Some(ColorFamily::Danger),
                ..Default::default()
            },
            variant: Some(ComponentVariant::Solid),
            color: Some(ColorFamily::Primary),
            ..Default::default()
        },
        children: vec![text("Card")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/borders.dowe"),
        "page BordersPage",
        &tree,
    );

    assert!(
        page.css_content
            .contains(".border-3{border-width:3px !important;border-style:solid !important;}")
    );
    assert!(
        page.css_content
            .contains(".border-color-danger{border-color:var(--dowe-danger) !important;}")
    );
    assert!(page.css_content.contains(".card.is-solid.is-primary"));
}

#[test]
fn defaults_border_color_to_primary_when_border_color_is_omitted() {
    let tree = ViewNode::Box {
        props: StyleProps {
            border: Some(ResponsiveValue::scalar(BorderWidth(1))),
            ..Default::default()
        },
        children: vec![text("Box")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/borders.dowe"),
        "page BordersPage",
        &tree,
    );

    assert!(page.content.contains("border-1 border-color-primary"));
    assert!(
        page.css_content
            .contains(".border-color-primary{border-color:var(--dowe-primary) !important;}")
    );
}

#[test]
fn rejects_incompatible_persisted_view_store_shapes() {
    let web = super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    };
    let router = super::router_js(&web);

    assert!(router.contains("Object.keys(initial).every"));
    assert!(router.contains(
        "stored===undefined||!compatibleSignalValue(stored,signal.initial)?signal.initial:stored"
    ));
}

#[test]
fn fills_request_path_placeholders_from_signal_names() {
    let web = super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    };
    let router = super::router_js(&web);

    assert!(router.contains("activeView?.signalNames?.[name]||name"));
    assert!(router.contains("readPath(state,binding,scope)"));
}

#[test]
fn emits_constants_outside_web_signal_state() {
    let tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "plans01".to_string(),
            name: "plans".to_string(),
            value: ViewSignalValue::Array(vec![ViewSignalValue::String("Starter".to_string())]),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![text("Plans")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/plans.dowe"),
        "page plans",
        &tree,
    );
    assert!(page.content.contains(r#""constants":[{"id":"plans01""#));
    assert!(page.content.contains(r#""signals":[]"#));
}

#[test]
fn renders_constant_each_rows_in_initial_html_and_route_render() {
    let tree = ViewNode::Scope {
        constants: vec![ViewConstant {
            id: "items01".to_string(),
            name: "items".to_string(),
            value: ViewSignalValue::Array(vec![
                ViewSignalValue::Object(vec![(
                    "label".to_string(),
                    ViewSignalValue::String("First".to_string()),
                )]),
                ViewSignalValue::Object(vec![(
                    "label".to_string(),
                    ViewSignalValue::String("Second".to_string()),
                )]),
            ]),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![ViewNode::Each {
            item: "item".to_string(),
            collection: "items".to_string(),
            key: "item.label".to_string(),
            children: vec![text("{item.label}")],
        }],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/items.dowe"),
        "page items",
        &tree,
    );
    let html = render_page_body(&ViewNode::Children, &tree);

    assert_eq!(html.matches("data-dowe-each-row").count(), 2);
    assert!(html.contains(">First</p>"));
    assert!(html.contains(">Second</p>"));
    assert_eq!(page.content.matches("data-dowe-each-row").count(), 2);
    assert!(page.content.contains("data-dowe-template"));
    assert!(page.content.contains("First"));
}

#[test]
fn emits_select_options_from_constant_each() {
    let tree = ViewNode::Scope {
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
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/options.dowe"),
        "page options",
        &tree,
    );
    assert!(page.content.contains("data-dowe-each=\\\"options01\\\""));
    assert!(
        page.content
            .contains("data-dowe-option-value-path=\\\"option.value\\\"")
    );
}

#[test]
fn emits_init_and_reactive_splash_boundary() {
    let tree = ViewNode::Scope {
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
            content: vec![text("Users")],
            children: vec![text("Loading users")],
        }],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/views/pages/users.dowe"),
        "page UsersPage",
        &tree,
    );
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    });

    assert!(page.content.contains("data-dowe-splash=\\\"loading01\\\""));
    assert!(page.content.contains("data-dowe-splash-main hidden"));
    assert!(page.content.contains("data-dowe-splash-content"));
    assert!(page.content.contains("\"autoload\":true"));
    assert!(page.content.contains("\"init\":true"));
    assert!(router.contains("renderSplashes"));
    assert!(router.contains("data-dowe-splash-main"));
}

#[test]
fn emits_terminal_replace_redirect_steps() {
    let tree = ViewNode::Scope {
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
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/views/pages/home.dowe"),
        "page HomePage",
        &tree,
    );
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    });

    assert!(
        page.content
            .contains(r#"{"kind":"redirect","path":"/login"}"#)
    );
    assert!(router.contains(
        r#"if(step.kind==="redirect"){await navigate(step.path,{replace:true});return true;}"#
    ));
    assert!(router.contains("if(step.kind===\"if\"&&await runSteps"));
}

