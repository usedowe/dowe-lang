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

fn motion_route() -> ViewRoute {
    ViewRoute {
        id: "motion".to_string(),
        route_path: "/motion".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                animation: Some(ViewAnimation::FadeIn),
                ..Default::default()
            },
            children: vec![ViewNode::Card {
                props: VariantProps {
                    style: StyleProps {
                        animation: Some(ViewAnimation::SlideUp),
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
            vec![
                "page docsPage".to_string(),
                "  Card variant:\"soft\" p:4 show:true".to_string(),
                "    Text".to_string(),
                "      Documentation".to_string(),
            ],
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
                value: PropValue::String("soft".to_string()),
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

fn charts_route() -> ViewRoute {
    ViewRoute {
        id: "charts".to_string(),
        route_path: "/charts".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                dowe_components::arc_chart_component_node(vec![ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("segments".to_string()),
                }])
                .expect("arc chart"),
                dowe_components::area_chart_component_node(vec![ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("points".to_string()),
                }])
                .expect("area chart"),
                dowe_components::bar_chart_component_node(vec![ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("segments".to_string()),
                }])
                .expect("bar chart"),
                dowe_components::line_chart_component_node(vec![ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("points".to_string()),
                }])
                .expect("line chart"),
                dowe_components::pie_chart_component_node(vec![ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("segments".to_string()),
                }])
                .expect("pie chart"),
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn table_route() -> ViewRoute {
    ViewRoute {
        id: "users".to_string(),
        route_path: "/users".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::table_node(
            vec![
                ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("users".to_string()),
                },
                ComponentProp {
                    name: "variant".to_string(),
                    value: PropValue::String("soft".to_string()),
                },
                ComponentProp {
                    name: "scheme".to_string(),
                    value: PropValue::String("surface".to_string()),
                },
                ComponentProp {
                    name: "size".to_string(),
                    value: PropValue::String("lg".to_string()),
                },
                ComponentProp {
                    name: "striped".to_string(),
                    value: PropValue::Boolean(true),
                },
                ComponentProp {
                    name: "bordered".to_string(),
                    value: PropValue::Boolean(true),
                },
                ComponentProp {
                    name: "emptyTitle".to_string(),
                    value: PropValue::String("No users".to_string()),
                },
            ],
            vec![
                dowe_components::table_column_component(vec![
                    ComponentProp {
                        name: "field".to_string(),
                        value: PropValue::String("name".to_string()),
                    },
                    ComponentProp {
                        name: "label".to_string(),
                        value: PropValue::String("Name".to_string()),
                    },
                ])
                .expect("name column"),
                dowe_components::table_column_component(vec![
                    ComponentProp {
                        name: "field".to_string(),
                        value: PropValue::String("status".to_string()),
                    },
                    ComponentProp {
                        name: "label".to_string(),
                        value: PropValue::String("Status".to_string()),
                    },
                    ComponentProp {
                        name: "align".to_string(),
                        value: PropValue::String("end".to_string()),
                    },
                    ComponentProp {
                        name: "width".to_string(),
                        value: PropValue::String("8rem".to_string()),
                    },
                ])
                .expect("status column"),
            ],
        )
        .expect("table"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn divider_route() -> ViewRoute {
    ViewRoute {
        id: "divider".to_string(),
        route_path: "/divider".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Divider {
            props: DividerProps {
                style: StyleProps::default(),
                orientation: DividerOrientation::Vertical,
                color: ColorFamily::Primary,
            },
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn svg_tree() -> ViewNode {
    ViewNode::Svg {
        props: SvgProps {
            style: StyleProps {
                text: Some(ResponsiveValue::scalar(
                    dowe_components::ColorToken::Tertiary,
                )),
                sizing: dowe_components::SizingProps {
                    w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    h: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            view_box: SvgViewBox {
                min_x: "0".to_string(),
                min_y: "0".to_string(),
                width: "24".to_string(),
                height: "24".to_string(),
            },
        },
        paths: vec![
            SvgPath {
                data: "M0 0h24v24H0z".to_string(),
                fill: SvgPathFill::None,
            },
            SvgPath {
                data: "M7.5 8.744c.253.847.1 1.895-.62 2.618a.75.75 0 0 1 0 1.5".to_string(),
                fill: SvgPathFill::CurrentColor,
            },
        ],
    }
}
