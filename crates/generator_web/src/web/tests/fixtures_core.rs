fn strip_web_for_test(path: &Path) -> String {
    path.strip_prefix("web")
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn layout_tree() -> ViewNode {
    ViewNode::Box {
        props: Default::default(),
        children: vec![text("Layout"), ViewNode::Children],
    }
}

fn page_tree() -> ViewNode {
    ViewNode::Box {
        props: Default::default(),
        children: vec![text("Login")],
    }
}

fn show_tree() -> ViewNode {
    ViewNode::Scope {
        signals: vec![ViewSignal {
            id: "ready01".to_string(),
            name: "isReady".to_string(),
            initial: ViewSignalValue::Bool(false),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![ViewNode::Box {
            props: StyleProps {
                element: ElementProps {
                    show: Some(VisibilityCondition::Static(responsive_bool(&[
                        (Breakpoint::Xs, false),
                        (Breakpoint::Md, true),
                    ]))),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Text {
                props: TextProps {
                    style: StyleProps {
                        element: ElementProps {
                            show: Some(VisibilityCondition::Signal("isReady".to_string())),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                value: "Ready".to_string(),
            }],
        }],
    }
}

fn reactive_tree(signal_id: &str, action_id: &str, children: bool) -> ViewNode {
    let mut input = VariantProps::default();
    input.element.bind = Some("alert.message".to_string());
    let mut button = VariantProps::default();
    button.element.on_click = Some("close".to_string());
    let mut nodes = vec![
        ViewNode::Input { props: input },
        ViewNode::Button {
            props: button,
            children: vec![text("Close")],
        },
    ];
    if children {
        nodes.push(ViewNode::Children);
    }
    ViewNode::Scope {
        signals: vec![ViewSignal {
            id: signal_id.to_string(),
            name: "alert".to_string(),
            initial: ViewSignalValue::Object(vec![(
                "message".to_string(),
                ViewSignalValue::String(String::new()),
            )]),
            schema: None,
        }],
        actions: vec![ViewAction {
            id: action_id.to_string(),
            name: "close".to_string(),
            kind: ViewActionKind::Reset(ViewResetAction {
                target: "alert".to_string(),
            }),
        }],
        children: nodes,
    }
}

fn text(value: &str) -> ViewNode {
    ViewNode::Text {
        props: Default::default(),
        value: value.to_string(),
    }
}

fn show_design_css() -> String {
    let web = super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    };
    web_artifacts(&web, &FontConfig::default(), &DesignConfig::default())
        .into_iter()
        .find(|artifact| artifact.relative_path == Path::new("web/design.css"))
        .expect("design css")
        .content
}

fn responsive_bool(entries: &[(Breakpoint, bool)]) -> ResponsiveValue<bool> {
    ResponsiveValue::ordered(
        entries
            .iter()
            .map(|(breakpoint, value)| ResponsiveEntry {
                breakpoint: *breakpoint,
                value: *value,
            })
            .collect(),
    )
}
