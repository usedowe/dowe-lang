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
        },
        paths: vec![
            SvgPath {
                data: "M0 0h24v24H0z".to_string(),
                fill: SvgPathFill::None,
            },
            SvgPath {
                data: "M22 12c0-5.523-4.477-10-10-10".to_string(),
                fill: SvgPathFill::CurrentColor,
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
        vec![
            "page docsPage".to_string(),
            "  Text".to_string(),
            "    Documentation".to_string(),
        ],
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

fn table_tree() -> ViewNode {
    dowe_components::table_node(
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
