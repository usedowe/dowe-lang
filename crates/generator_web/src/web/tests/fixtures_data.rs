fn svg_tree() -> ViewNode {
    ViewNode::Svg {
        props: SvgProps {
            style: StyleProps {
                text: Some(ResponsiveValue::scalar(
                    dowe_components::ColorToken::Tertiary,
                )),
                sizing: dowe_components::SizingProps {
                    w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        dowe_components::ScaleValue::from_half_steps(16),
                    ))),
                    h: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        dowe_components::ScaleValue::from_half_steps(16),
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
            data: None,
            motion: None,
        },
        paths: vec![
            SvgPath {
                data: "M0 0h24v24H0z".to_string(),
                fill: SvgPathFill::None,
                transform: None,
            },
            SvgPath {
                data: "M22 12c0-5.523-4.477-10-10-10".to_string(),
                fill: SvgPathFill::Fill {
                    color: None,
                    opacity: 255,
                    even_odd: true,
                },
                transform: Some(SvgTransform {
                    a: "2".to_string(),
                    b: "0".to_string(),
                    c: "0".to_string(),
                    d: "2".to_string(),
                    e: "4".to_string(),
                    f: "6".to_string(),
                }),
            },
        ],
    }
}

fn code_tree() -> ViewNode {
    dowe_components::code_node(
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
        "page docsPage\n  Text\n    Documentation".to_string(),
    )
    .expect("code")
}

fn video_tree() -> ViewNode {
    ViewNode::Video {
        props: VideoProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Solid),
                color: Some(ColorFamily::Surface),
                ..Default::default()
            },
            src: "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8".to_string(),
            poster: Some("/images/video.jpg".to_string()),
            autoplay: false,
            aspect: VideoAspect::Horizontal,
        },
    }
}

fn iframe_tree() -> ViewNode {
    dowe_components::iframe_node(vec![
        ComponentProp {
            name: "src".to_string(),
            value: PropValue::String("https://example.com/embed".to_string()),
        },
        ComponentProp {
            name: "title".to_string(),
            value: PropValue::String("Example embed".to_string()),
        },
        ComponentProp {
            name: "loading".to_string(),
            value: PropValue::String("eager".to_string()),
        },
        ComponentProp {
            name: "allow".to_string(),
            value: PropValue::String("fullscreen; autoplay".to_string()),
        },
        ComponentProp {
            name: "sandbox".to_string(),
            value: PropValue::String("scripts same-origin".to_string()),
        },
        ComponentProp {
            name: "allowFullscreen".to_string(),
            value: PropValue::Boolean(true),
        },
    ])
    .expect("iframe")
}

fn candlestick_tree() -> ViewNode {
    dowe_components::candlestick_node(vec![
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
    .expect("candlestick")
}

fn charts_tree() -> ViewNode {
    ViewNode::Box {
        props: Default::default(),
        children: vec![
            dowe_components::arc_chart_component_node(vec![
                ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("segments".to_string()),
                },
                ComponentProp {
                    name: "palette".to_string(),
                    value: PropValue::String("ocean".to_string()),
                },
                ComponentProp {
                    name: "centerText".to_string(),
                    value: PropValue::String("Share".to_string()),
                },
                ComponentProp {
                    name: "centerValue".to_string(),
                    value: PropValue::String("88%".to_string()),
                },
                ComponentProp {
                    name: "thickness".to_string(),
                    value: PropValue::Number("18".to_string()),
                },
                ComponentProp {
                    name: "gap".to_string(),
                    value: PropValue::Number("4".to_string()),
                },
                ComponentProp {
                    name: "startAngle".to_string(),
                    value: PropValue::Number("-90".to_string()),
                },
                ComponentProp {
                    name: "endAngle".to_string(),
                    value: PropValue::Number("270".to_string()),
                },
                ComponentProp {
                    name: "showInlineLabels".to_string(),
                    value: PropValue::Boolean(true),
                },
                ComponentProp {
                    name: "hideValues".to_string(),
                    value: PropValue::Boolean(true),
                },
                ComponentProp {
                    name: "showGlow".to_string(),
                    value: PropValue::Boolean(true),
                },
            ])
            .expect("arc chart"),
            dowe_components::area_chart_component_node(vec![
                ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("points".to_string()),
                },
                ComponentProp {
                    name: "curve".to_string(),
                    value: PropValue::String("smooth".to_string()),
                },
            ])
            .expect("area chart"),
            dowe_components::bar_chart_component_node(vec![ComponentProp {
                name: "data".to_string(),
                value: PropValue::String("segments".to_string()),
            }])
            .expect("bar chart"),
            dowe_components::line_chart_component_node(vec![
                ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("points".to_string()),
                },
                ComponentProp {
                    name: "palette".to_string(),
                    value: PropValue::String("forest".to_string()),
                },
            ])
            .expect("line chart"),
            dowe_components::pie_chart_component_node(vec![
                ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("segments".to_string()),
                },
                ComponentProp {
                    name: "donut".to_string(),
                    value: PropValue::Boolean(true),
                },
            ])
            .expect("pie chart"),
        ],
    }
}

fn table_tree() -> ViewNode {
    dowe_components::table_node(
        vec![
            ComponentProp {
                name: "data".to_string(),
                value: PropValue::String("users".to_string()),
            },
            ComponentProp {
                name: "variant".to_string(),
                value: PropValue::String("outlined".to_string()),
            },
            ComponentProp {
                name: "scheme".to_string(),
                value: PropValue::String("primary".to_string()),
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
    .expect("table")
}

fn translations() -> TranslationCatalog {
    TranslationCatalog {
        default_locale: Some("en".to_string()),
        locales: vec![
            TranslationLocale {
                locale: "en".to_string(),
                source_path: PathBuf::from("src/i18n/en.dowe"),
                values: vec![TranslationValue {
                    key: "home.hero.title".to_string(),
                    value: "Dowe builds systems.".to_string(),
                }],
            },
            TranslationLocale {
                locale: "es".to_string(),
                source_path: PathBuf::from("src/i18n/es.dowe"),
                values: vec![TranslationValue {
                    key: "home.hero.title".to_string(),
                    value: "Dowe construye sistemas.".to_string(),
                }],
            },
        ],
    }
}

fn divider_tree() -> ViewNode {
    ViewNode::Divider {
        props: DividerProps {
            style: StyleProps::default(),
            orientation: DividerOrientation::Vertical,
            color: ColorFamily::Primary,
        },
    }
}
