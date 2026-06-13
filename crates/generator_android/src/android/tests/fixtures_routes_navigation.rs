fn index_route_with_signup_link() -> ViewRoute {
    ViewRoute {
        id: "index".to_string(),
        route_path: "/".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                element: ElementProps {
                    id: Some("hero".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![
                ViewNode::Button {
                    props: VariantProps {
                        navigation: Some(NavigationAction::Internal {
                            path: "/signup".to_string(),
                            fragment: Some("join".to_string()),
                            operation: NavigationOperation::Push,
                        }),
                        ..Default::default()
                    },
                    children: vec![text("Signup")],
                },
                ViewNode::Button {
                    props: VariantProps {
                        navigation: Some(NavigationAction::Section {
                            fragment: "hero".to_string(),
                            operation: NavigationOperation::Replace,
                        }),
                        ..Default::default()
                    },
                    children: vec![text("Hero")],
                },
                ViewNode::Button {
                    props: VariantProps {
                        navigation: Some(NavigationAction::Back),
                        ..Default::default()
                    },
                    children: vec![text("Back")],
                },
            ],
        },
        sections: vec![ViewSection {
            id: "hero".to_string(),
        }],
        navigation_actions: Vec::new(),
    }
}

fn signup_route() -> ViewRoute {
    ViewRoute {
        id: "signup".to_string(),
        route_path: "/signup".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Button {
            props: VariantProps {
                navigation: Some(NavigationAction::Internal {
                    path: "/".to_string(),
                    fragment: None,
                    operation: NavigationOperation::Push,
                }),
                ..Default::default()
            },
            children: vec![text("Signup")],
        },
        sections: vec![ViewSection {
            id: "join".to_string(),
        }],
        navigation_actions: Vec::new(),
    }
}

fn responsive_route() -> ViewRoute {
    ViewRoute {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        layout_tree: ViewNode::Box {
            props: StyleProps {
                spacing: dowe_components::SpacingProps {
                    p: Some(responsive_scale(&[
                        (Breakpoint::Xs, 4),
                        (Breakpoint::Md, 8),
                    ])),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Children],
        },
        page_tree: ViewNode::Box {
            props: StyleProps {
                spacing: dowe_components::SpacingProps {
                    p: Some(responsive_scale(&[(Breakpoint::Md, 8)])),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Text {
                props: TextProps {
                    size: Some(ResponsiveValue::ordered(vec![ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: TextSize::Lg,
                    }])),
                    weight: Some(ResponsiveValue::ordered(vec![
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Xs,
                            value: TextWeight::Thin,
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: TextWeight::Extralight,
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Lg,
                            value: TextWeight::Black,
                        },
                    ])),
                    ..Default::default()
                },
                value: "Login".to_string(),
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn show_route() -> ViewRoute {
    ViewRoute {
        id: "ready".to_string(),
        route_path: "/ready".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scope {
            signals: vec![
                ViewSignal {
                    id: "ready01".to_string(),
                    name: "isReady".to_string(),
                    initial: ViewSignalValue::Bool(false),
                    schema: None,
                },
                ViewSignal {
                    id: "rows01".to_string(),
                    name: "rows".to_string(),
                    initial: ViewSignalValue::Array(vec![ViewSignalValue::Object(vec![
                        ("id".to_string(), ViewSignalValue::String("1".to_string())),
                        ("ready".to_string(), ViewSignalValue::Bool(true)),
                    ])]),
                    schema: None,
                },
            ],
            actions: Vec::new(),
            children: vec![
                ViewNode::Box {
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
                },
                ViewNode::Each {
                    item: "row".to_string(),
                    collection: "rows".to_string(),
                    key: "row.id".to_string(),
                    children: vec![ViewNode::Text {
                        props: TextProps {
                            style: StyleProps {
                                element: ElementProps {
                                    show: Some(VisibilityCondition::Signal(
                                        "row.ready".to_string(),
                                    )),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "Row".to_string(),
                    }],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}
