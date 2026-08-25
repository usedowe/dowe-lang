fn text(value: &str) -> ViewNode {
    ViewNode::Text {
        props: Default::default(),
        value: value.to_string(),
    }
}

fn container_foreground_tree() -> ViewNode {
    ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![
            ViewNode::Box {
                props: StyleProps {
                    text: Some(ResponsiveValue::scalar(ColorToken::PrimaryText)),
                    ..Default::default()
                },
                children: vec![
                    text("Box inherited"),
                    ViewNode::Text {
                        props: TextProps {
                            style: StyleProps {
                                text: Some(ResponsiveValue::scalar(ColorToken::Danger)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "Box override".to_string(),
                    },
                ],
            },
            ViewNode::Card {
                props: VariantProps {
                    variant: Some(ComponentVariant::Solid),
                    color: Some(ColorFamily::Muted),
                    ..Default::default()
                },
                children: vec![
                    text("Card inherited"),
                    ViewNode::Title {
                        props: Default::default(),
                        value: "Card title inherited".to_string(),
                    },
                    ViewNode::Title {
                        props: TextProps {
                            style: StyleProps {
                                text: Some(ResponsiveValue::scalar(ColorToken::Warning)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "Card override".to_string(),
                    },
                ],
            },
            ViewNode::Section {
                props: StyleProps {
                    text: Some(ResponsiveValue::scalar(ColorToken::SecondaryText)),
                    ..Default::default()
                },
                children: vec![text("Section inherited")],
            },
            ViewNode::Flex {
                props: LayoutProps {
                    style: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::AccentText)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Flex inherited")],
            },
            ViewNode::Grid {
                props: GridProps {
                    style: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::MutedText)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Grid inherited")],
            },
            ViewNode::Brand {
                props: BrandProps {
                    style: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::SurfaceText)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Brand inherited")],
            },
            ViewNode::Banner {
                props: BannerProps {
                    style: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::InfoText)),
                        ..Default::default()
                    },
                    navigation: NavigationAction::External {
                        url: "https://dowe.dev".to_string(),
                        web_target: dowe_components::WebTarget::Blank,
                        native_external_mode: dowe_components::NativeExternalMode::System,
                    },
                    label: None,
                },
                children: vec![text("Banner inherited")],
            },
            ViewNode::Marquee {
                props: MarqueeProps {
                    style: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::WarningText)),
                        ..Default::default()
                    },
                    speed: MarqueeSpeed::Normal,
                    pause_on_hover: false,
                    reverse: false,
                    orientation: MarqueeOrientation::Horizontal,
                    fade: false,
                    fade_color: ColorToken::Background,
                    gap: ScaleValue::from_half_steps(8),
                },
                children: vec![text("Marquee inherited")],
            },
            ViewNode::Scaffold {
                props: ScaffoldProps {
                    style: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::DangerText)),
                        ..Default::default()
                    },
                    boxed: false,
                },
                app_bar: Vec::new(),
                start: Vec::new(),
                main: vec![text("Scaffold inherited")],
                end: Vec::new(),
                bottom_bar: Vec::new(),
                overlays: Vec::new(),
            },
            ViewNode::Collapsible {
                props: CollapsibleProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Solid),
                        color: Some(ColorFamily::Primary),
                        ..Default::default()
                    },
                    label: "Details".to_string(),
                    default_open: true,
                    disabled: false,
                },
                children: vec![text("Collapsible inherited")],
            },
            ViewNode::TypeWriter {
                props: TypeWriterProps {
                    style: StyleProps::default(),
                    type_speed: 10,
                    delete_speed: 5,
                    after_typed: 20,
                    after_deleted: 10,
                    repeat: false,
                },
                items: vec![TypeWriterItem {
                    text: "TypeWriter inherited".to_string(),
                }],
            },
        ],
    }
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

fn bar_props(floating: bool) -> BarProps {
    BarProps {
        style: VariantProps {
            variant: Some(ComponentVariant::Solid),
            color: Some(ColorFamily::Surface),
            ..Default::default()
        },
        bordered: true,
        blurred: true,
        boxed: true,
        floating,
        position: BarPosition::Static,
        hide_on_scroll: false,
        dock_on_scroll: false,
    }
}

fn responsive_scale(entries: &[(Breakpoint, u16)]) -> ResponsiveValue<ScaleValue> {
    ResponsiveValue::ordered(
        entries
            .iter()
            .map(|(breakpoint, value)| ResponsiveEntry {
                breakpoint: *breakpoint,
                value: ScaleValue::from_half_steps(value * 2),
            })
            .collect(),
    )
}
