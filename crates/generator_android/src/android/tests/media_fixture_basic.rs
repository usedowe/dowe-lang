fn svg_route() -> ViewRoute {
    ViewRoute {
        id: "svg".to_string(),
        route_path: "/svg".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: svg_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn runtime_svg_route() -> ViewRoute {
    ViewRoute {
        id: "runtime-svg".to_string(),
        route_path: "/runtime-svg".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scope {
            constants: Vec::new(),
            signals: vec![ViewSignal {
                id: "iconData01".to_string(),
                name: "iconData".to_string(),
                storage_key: "runtime-svg:iconData".to_string(),
                scope: dowe_components::ViewSignalScope::Page,
                storage: dowe_components::ViewSignalStorage::None,
                initial: ViewSignalValue::String(
                    r#"{"viewBox":"0 0 24 24","paths":[{"d":"M2 2h20v20H2z","paint":"fill","color":"currentColor"}]}"#.to_string(),
                ),
                schema: None,
            }],
            actions: Vec::new(),
            children: vec![ViewNode::Svg {
                props: SvgProps {
                    style: StyleProps::default(),
                    view_box: SvgViewBox {
                        min_x: "0".to_string(),
                        min_y: "0".to_string(),
                        width: "24".to_string(),
                        height: "24".to_string(),
                    },
                    data: Some("iconData".to_string()),
                    icon_name: None,
                    icon_fallback: None,
                    icon_fill: None,
            icon_fill_binding: None,
                    icon_stroke: None,
            icon_stroke_binding: None,
                    motion: None,
                },
                paths: Vec::new(),
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn motion_route() -> ViewRoute {
    ViewRoute {
        id: "motion".to_string(),
        route_path: "/motion".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                extras: Some(Box::new(StyleExtras {
                    motion: ViewMotionStyle {
                        animation: Some(ViewAnimation::FadeIn),
                        ..Default::default()
                    },
                    ..Default::default()
                })),
                ..Default::default()
            },
            children: vec![ViewNode::Card {
                props: VariantProps {
                    style: StyleProps {
                        extras: Some(Box::new(StyleExtras {
                            motion: ViewMotionStyle {
                                animation: Some(ViewAnimation::SlideUp),
                                rotate: Some(ResponsiveValue::scalar(ViewRotation(-7))),
                                scale: Some(ResponsiveValue::scalar(ViewScale(105))),
                                translate_x: Some(ResponsiveValue::scalar(ViewTranslation(-3))),
                                transition: Some(ViewTransition::Spring),
                                gesture: Some(ViewGesture::Lift),
                                ..Default::default()
                            },
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Motion")],
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn code_route() -> ViewRoute {
    ViewRoute {
        id: "code".to_string(),
        route_path: "/code".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::code_node(
            vec![
                ComponentProp {
                    name: "language".to_string(),
                    value: PropValue::String("dowe".to_string()),
                },
                ComponentProp {
                    name: "scheme".to_string(),
                    value: PropValue::String("surface".to_string()),
                },
            ],
            "page docsPage\n  Card variant:\"solid\" p:4 show:true\n    Text\n      Documentation".to_string(),
        )
        .expect("code"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn video_route() -> ViewRoute {
    ViewRoute {
        id: "video".to_string(),
        route_path: "/video".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::video_node(vec![
            ComponentProp {
                name: "src".to_string(),
                value: PropValue::String(
                    "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8".to_string(),
                ),
            },
            ComponentProp {
                name: "poster".to_string(),
                value: PropValue::String("/images/video.jpg".to_string()),
            },
            ComponentProp {
                name: "aspect".to_string(),
                value: PropValue::String("vertical".to_string()),
            },
            ComponentProp {
                name: "scheme".to_string(),
                value: PropValue::String("surface".to_string()),
            },
        ])
        .expect("video"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn iframe_route() -> ViewRoute {
    ViewRoute {
        id: "iframe".to_string(),
        route_path: "/iframe".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::iframe_node(vec![
            ComponentProp { name: "src".to_string(), value: PropValue::String("https://example.com/embed".to_string()) },
            ComponentProp { name: "title".to_string(), value: PropValue::String("Example embed".to_string()) },
            ComponentProp { name: "allow".to_string(), value: PropValue::String("autoplay".to_string()) },
            ComponentProp { name: "sandbox".to_string(), value: PropValue::String("scripts same-origin".to_string()) },
        ]).expect("iframe"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn canvas_route() -> ViewRoute {
    ViewRoute {
        id: "canvas".to_string(),
        route_path: "/canvas".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::canvas_component_node(vec![
            ComponentProp {
                name: "scene".to_string(),
                value: PropValue::String("scene".to_string()),
            },
            ComponentProp {
                name: "viewWidth".to_string(),
                value: PropValue::Number("640".to_string()),
            },
            ComponentProp {
                name: "viewHeight".to_string(),
                value: PropValue::Number("360".to_string()),
            },
            ComponentProp {
                name: "fit".to_string(),
                value: PropValue::String("cover".to_string()),
            },
            ComponentProp {
                name: "fps".to_string(),
                value: PropValue::Number("30".to_string()),
            },
            ComponentProp {
                name: "autoplay".to_string(),
                value: PropValue::Boolean(false),
            },
            ComponentProp {
                name: "background".to_string(),
                value: PropValue::String("background".to_string()),
            },
            ComponentProp {
                name: "pixelated".to_string(),
                value: PropValue::Boolean(true),
            },
            ComponentProp {
                name: "rounded".to_string(),
                value: PropValue::String("md".to_string()),
            },
            ComponentProp {
                name: "border".to_string(),
                value: PropValue::Number("1".to_string()),
            },
            ComponentProp {
                name: "borderColor".to_string(),
                value: PropValue::String("primary".to_string()),
            },
            ComponentProp {
                name: "label".to_string(),
                value: PropValue::String("Animated scene".to_string()),
            },
        ])
        .expect("canvas"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn candlestick_route() -> ViewRoute {
    ViewRoute {
        id: "market".to_string(),
        route_path: "/market".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::candlestick_node(vec![
            ComponentProp {
                name: "data".to_string(),
                value: PropValue::String("candles".to_string()),
            },
            ComponentProp {
                name: "stream".to_string(),
                value: PropValue::String("/api/candles".to_string()),
            },
            ComponentProp {
                name: "variant".to_string(),
                value: PropValue::String("solid".to_string()),
            },
            ComponentProp {
                name: "scheme".to_string(),
                value: PropValue::String("surface".to_string()),
            },
            ComponentProp {
                name: "emptyLabel".to_string(),
                value: PropValue::String("Market closed".to_string()),
            },
        ])
        .expect("candlestick"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

