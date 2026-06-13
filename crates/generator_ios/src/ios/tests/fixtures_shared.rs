fn text(value: &str) -> ViewNode {
    ViewNode::Text {
        props: Default::default(),
        value: value.to_string(),
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
