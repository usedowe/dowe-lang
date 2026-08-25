#[test]
fn avatar_group_max_counts_visible_items() {
    let props = AvatarGroupProps {
        style: Default::default(),
        items: None,
        size: ButtonSize::Md,
        max: Some(3),
        auto_fit: false,
        inline: false,
        bordered: false,
    };
    assert_eq!(props.visible_item_count(4), 3);
    assert_eq!(props.overflow_count(4), 1);
    assert_eq!(props.visible_item_count(2), 2);
    assert_eq!(props.overflow_count(2), 0);
}

#[test]
fn registry_finds_builtin_components() {
    assert_eq!(COMPONENT_REGISTRY.get("Box"), Some(BuiltinComponent::Box));
    assert_eq!(
        COMPONENT_REGISTRY.get("Section"),
        Some(BuiltinComponent::Section)
    );
    assert_eq!(COMPONENT_REGISTRY.get("Text"), Some(BuiltinComponent::Text));
    assert_eq!(COMPONENT_REGISTRY.get("Flex"), Some(BuiltinComponent::Flex));
    assert_eq!(COMPONENT_REGISTRY.get("Grid"), Some(BuiltinComponent::Grid));
    assert_eq!(
        COMPONENT_REGISTRY.get("Input"),
        Some(BuiltinComponent::Input)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Select"),
        Some(BuiltinComponent::Select)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Option"),
        Some(BuiltinComponent::Option)
    );
    assert_eq!(COMPONENT_REGISTRY.get("Code"), Some(BuiltinComponent::Code));
    assert_eq!(
        COMPONENT_REGISTRY.get("Video"),
        Some(BuiltinComponent::Video)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Canvas"),
        Some(BuiltinComponent::Canvas)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Iframe"),
        Some(BuiltinComponent::Iframe)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Device"),
        Some(BuiltinComponent::Device)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Candlestick"),
        Some(BuiltinComponent::Candlestick)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("ArcChart"),
        Some(BuiltinComponent::ArcChart)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("AreaChart"),
        Some(BuiltinComponent::AreaChart)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("BarChart"),
        Some(BuiltinComponent::BarChart)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("LineChart"),
        Some(BuiltinComponent::LineChart)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("PieChart"),
        Some(BuiltinComponent::PieChart)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Table"),
        Some(BuiltinComponent::Table)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Divider"),
        Some(BuiltinComponent::Divider)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Button"),
        Some(BuiltinComponent::Button)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Brand"),
        Some(BuiltinComponent::Brand)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Banner"),
        Some(BuiltinComponent::Banner)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Alert"),
        Some(BuiltinComponent::Alert)
    );
    assert_eq!(COMPONENT_REGISTRY.get("Svg"), Some(BuiltinComponent::Svg));
    assert_eq!(COMPONENT_REGISTRY.get("Path"), Some(BuiltinComponent::Path));
    assert_eq!(
        COMPONENT_REGISTRY.get("AppBar"),
        Some(BuiltinComponent::AppBar)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Footer"),
        Some(BuiltinComponent::Footer)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("BottomBar"),
        Some(BuiltinComponent::BottomBar)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("SideNav"),
        Some(BuiltinComponent::SideNav)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("RailNav"),
        Some(BuiltinComponent::RailNav)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Sidebar"),
        Some(BuiltinComponent::Sidebar)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("NavMenu"),
        Some(BuiltinComponent::NavMenu)
    );
    assert_eq!(
        COMPONENT_REGISTRY.get("Scaffold"),
        Some(BuiltinComponent::Scaffold)
    );
    assert_eq!(COMPONENT_REGISTRY.get("Tabs"), Some(BuiltinComponent::Tabs));
    assert_eq!(COMPONENT_REGISTRY.get("tab"), Some(BuiltinComponent::Tab));
    assert_eq!(
        COMPONENT_REGISTRY.get("Drawer"),
        Some(BuiltinComponent::Drawer)
    );
    assert_eq!(COMPONENT_REGISTRY.get("Body"), None);
    assert_eq!(COMPONENT_REGISTRY.get("Card"), Some(BuiltinComponent::Card));
    assert_eq!(
        COMPONENT_REGISTRY.get("Title"),
        Some(BuiltinComponent::Title)
    );
    assert_eq!(COMPONENT_REGISTRY.get("Stack"), None);
    assert_eq!(
        COMPONENT_REGISTRY.get("Password"),
        Some(BuiltinComponent::Password)
    );
    assert_eq!(COMPONENT_REGISTRY.get("PasswordField"), None);
    assert_eq!(
        COMPONENT_REGISTRY.get("Phone"),
        Some(BuiltinComponent::Phone)
    );
    assert_eq!(COMPONENT_REGISTRY.get("PhoneField"), None);
    assert_eq!(COMPONENT_REGISTRY.get("Pin"), Some(BuiltinComponent::Pin));
    assert_eq!(COMPONENT_REGISTRY.get("PinField"), None);
}

#[test]
fn builtin_component_catalog_is_complete_and_unique() {
    let names = BuiltinComponent::ALL
        .iter()
        .map(|component| component.as_str())
        .collect::<Vec<_>>();
    let unique = names
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(names.len(), unique.len());
    for component in BuiltinComponent::ALL {
        assert_eq!(
            BuiltinComponent::from_name(component.as_str()),
            Some(*component)
        );
    }
}

#[test]
fn owns_cross_target_typography_metrics() {
    let body = text_typography(false, TextSize::NineXl);
    assert_eq!(body.font_size.min, "40");
    assert_eq!(body.font_size.preferred_base, "30.4");
    assert_eq!(body.font_size.preferred_viewport, "2.8");
    assert_eq!(body.font_size.max, "60");
    assert_eq!(body.line_height, "1.2");
    assert_eq!(text_weight_number(body.weight), "400");
    assert_eq!(body.letter_spacing_em, "0");
    assert_eq!(text_weight_number(TextWeight::Thin), "100");
    assert_eq!(text_weight_number(TextWeight::Extralight), "200");
    assert_eq!(text_weight_number(TextWeight::Black), "900");

    let title = text_typography(true, TextSize::NineXl);
    assert_eq!(title.font_size.min, "72");
    assert_eq!(title.font_size.preferred_base, "48");
    assert_eq!(title.font_size.preferred_viewport, "7");
    assert_eq!(title.font_size.max, "128");
    assert_eq!(title.line_height, "1");
    assert_eq!(text_weight_number(title.weight), "800");
    assert_eq!(title.letter_spacing_em, "-0.06");
    assert_eq!(text_spacing_em(TextSpacing::Tight), "-0.02");
}

#[test]
fn font_catalog_exposes_platform_asset_metadata() {
    let catalog = font_catalog();
    assert_eq!(catalog.len(), FontFamily::all().len());

    let system = FontFamily::System.catalog_entry();
    assert_eq!(system.display_name, "system-ui");
    assert_eq!(system.ios_family_name, ".system");
    assert_eq!(system.android_family_name, "sans-serif");
    assert!(!system.package_assets);
    assert!(system.weights.is_empty());

    let inter = FontFamily::Inter.catalog_entry();
    assert_eq!(inter.display_name, "Inter");
    assert!(inter.web_stack.contains("\"Dowe Inter\""));
    assert_eq!(inter.ios_family_name, "Inter");
    assert_eq!(inter.android_family_name, "Inter");
    assert!(inter.package_assets);
    assert!(inter.weights.iter().any(|weight| {
        weight.weight == TextWeight::Thin
            && weight.numeric_weight == 100
            && weight.asset_stem == "inter-light"
    }));
    assert!(inter.weights.iter().any(|weight| {
        weight.weight == TextWeight::Light
            && weight.numeric_weight == 300
            && weight.asset_stem == "inter-light"
    }));

    let poppins = FontFamily::Poppins.catalog_entry();
    assert_eq!(poppins.display_name, "Poppins");
    assert!(poppins.package_assets);
    assert!(poppins.weights.iter().any(|weight| {
        weight.weight == TextWeight::Black
            && weight.numeric_weight == 900
            && weight.asset_stem == "poppins-extrabold"
    }));
    assert!(poppins.weights.iter().any(|weight| {
        weight.weight == TextWeight::Extrabold
            && weight.numeric_weight == 800
            && weight.asset_stem == "poppins-extrabold"
    }));
}

#[test]
fn font_catalog_packaged_assets_exist() {
    let fonts_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");

    for entry in font_catalog().iter().filter(|entry| entry.package_assets) {
        let family_dir = fonts_root.join(entry.token.as_str());
        assert!(
            family_dir.is_dir(),
            "missing font family directory: {}",
            family_dir.display()
        );

        let license = family_dir.join("LICENSE.txt");
        assert!(
            license.is_file(),
            "missing font license: {}",
            license.display()
        );

        for weight in entry.weights {
            let asset = family_dir.join(format!("{}.ttf", weight.asset_stem));
            assert!(asset.is_file(), "missing font asset: {}", asset.display());
            assert!(
                fs::metadata(&asset).expect("font asset metadata").len() > 0,
                "empty font asset: {}",
                asset.display()
            );
        }
    }
}

#[test]
fn validates_country_flag_catalog_and_icon_names() {
    assert_eq!(COUNTRY_FLAGS.len(), 245);
    for country in phone_countries() {
        assert!(
            country_flag_icon(country.code).is_some(),
            "missing flag {}",
            country.code
        );
    }
    let colombia = country_flag_icon("CO").expect("Colombia flag");
    assert!(!colombia.paths.is_empty());
    let node = icon_component_node(vec![string_prop("name", "country-flags:CO")])
        .expect("country flag Icon");
    match node {
        ViewNode::Svg { paths, .. } => assert!(!paths.is_empty()),
        _ => panic!("country flag Icon lowers to SVG"),
    }
    assert!(all_icon_names().contains(&"country-flags:CO".to_string()));
}

#[test]
fn exposes_solar_variant_names_in_the_icon_catalog() {
    let names = all_icon_names();
    assert!(names.contains(&"alt-arrow-right".to_string()));
    assert!(names.contains(&"alt-arrow-right-broken".to_string()));
    assert!(names.contains(&"alt-arrow-right-outline".to_string()));
    assert!(names.contains(&"alt-arrow-right-bold".to_string()));
    assert!(names.contains(&"alt-arrow-right-line-duotone".to_string()));
    assert!(names.contains(&"alt-arrow-right-bold-duotone".to_string()));
    assert!(!names.contains(&"alt-arrow-right-linear".to_string()));
}

#[test]
fn maps_empty_kinds_to_bold_duotone_icons() {
    let mappings = [
        (EmptyKind::Playlist, "playlist-bold-duotone"),
        (EmptyKind::Result, "magnifier-bold-duotone"),
        (EmptyKind::Data, "database-bold-duotone"),
        (EmptyKind::Template, "widget-add-bold-duotone"),
    ];
    let names = all_icon_names();
    for (kind, name) in mappings {
        assert!(names.contains(&name.to_string()));
        assert!(!empty_icon(kind).expect("Empty icon").paths.is_empty());
    }
}

#[test]
fn validates_svg_spinner_catalog_and_icon_names() {
    assert_eq!(SVG_SPINNERS.len(), 46);
    assert_eq!(validate_svg_spinner_catalog().expect("catalog"), 46);
    assert!(all_icon_names().contains(&"svg-spinners:3-dots-bounce".to_string()));
    assert!(all_icon_names().contains(&"svg-spinners:ring-resize".to_string()));
}

#[test]
fn validates_svg_logo_catalog_and_icon_names() {
    assert_eq!(SVG_LOGOS.len(), 1863);
    assert_eq!(validate_svg_logo_catalog().expect("catalog"), 1863);
    assert!(all_icon_names().contains(&"svg-logos:github-icon".to_string()));
    assert!(all_icon_names().contains(&"svg-logos:daisyui-icon".to_string()));
    assert!(all_icon_names().contains(&"svg-logos:macos".to_string()));
}

#[test]
fn exposes_runtime_payloads_for_every_icon_name() {
    let names = all_icon_names();
    let catalog = runtime_icon_catalog().expect("runtime icon catalog");
    assert_eq!(catalog.len(), names.len());
    assert!(catalog.iter().any(|(name, payload)| {
        name == "route-bold-duotone" && payload.contains("\"viewBox\"")
    }));
    assert!(catalog
        .iter()
        .any(|(name, payload)| { name == "country-flags:CO" && payload.contains("\"paths\"") }));
    assert!(catalog.iter().any(|(name, payload)| {
        name == "svg-logos:github-icon" && payload.contains("\"paths\"")
    }));
}

#[test]
fn shares_the_runtime_icon_catalog_across_generators() {
    let first = runtime_icon_catalog_shared().expect("runtime icon catalog");
    let second = runtime_icon_catalog_shared().expect("runtime icon catalog");
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert_eq!(first.len(), all_icon_names().len());
}

#[test]
fn exposes_only_requested_runtime_icon_payloads() {
    let catalog = runtime_icon_catalog_for_names([
        "route-bold-duotone",
        "global-bold-duotone",
        "laptop-bold-duotone",
        "svg-logos:android-icon",
        "svg-logos:apple",
    ])
    .expect("selected runtime icon catalog");

    assert_eq!(catalog.len(), 5);
    assert!(catalog.iter().all(|(name, payload)| {
        payload.contains("\"viewBox\"")
            && matches!(
                name.as_str(),
                "route-bold-duotone"
                    | "global-bold-duotone"
                    | "laptop-bold-duotone"
                    | "svg-logos:android-icon"
                    | "svg-logos:apple"
            )
    }));
}

#[test]
fn exposes_text_and_title_roles_for_every_theme_color_family() {
    assert_eq!(ColorToken::all().len(), 30);
    assert_eq!(
        ColorToken::from_name("primaryText"),
        Some(ColorToken::PrimaryText)
    );
    assert_eq!(
        ColorToken::from_name("primaryTitle"),
        Some(ColorToken::PrimaryTitle)
    );
    assert_eq!(ColorToken::from_name("onPrimary"), None);
    assert_eq!(ColorToken::from_name("onSuccess"), None);
    assert_eq!(ColorToken::from_name("onSoftPrimary"), None);
    assert_eq!(ColorFamily::Primary.text_token(), ColorToken::PrimaryText);
    assert_eq!(ColorFamily::Primary.title_token(), ColorToken::PrimaryTitle);
    assert_eq!(
        ColorFamily::Background.title_token(),
        ColorToken::BackgroundTitle
    );
    assert_eq!(
        ColorFamily::from_theme_name("primary"),
        Some((ColorFamily::Primary, false))
    );
    assert_eq!(ColorFamily::from_theme_name("softPrimary"), None);
    assert_eq!(ColorFamily::from_theme_name("softBackground"), None);
    assert_eq!(
        ColorFamily::Primary.theme_tokens(),
        Some([
            ColorToken::Primary,
            ColorToken::PrimaryText,
            ColorToken::PrimaryTitle,
        ])
    );
    assert_eq!(
        ColorFamily::Background.theme_tokens(),
        Some([
            ColorToken::Background,
            ColorToken::BackgroundText,
            ColorToken::BackgroundTitle,
        ])
    );

    let expected_light = [
        (ColorToken::Primary, "#1F3A5F"),
        (ColorToken::PrimaryText, "#EBF2FA"),
        (ColorToken::PrimaryTitle, "#FFFFFF"),
        (ColorToken::Secondary, "#6BC670"),
        (ColorToken::SecondaryText, "#0F291E"),
        (ColorToken::SecondaryTitle, "#040D05"),
        (ColorToken::Accent, "#3F7A8A"),
        (ColorToken::AccentText, "#F0F7F9"),
        (ColorToken::AccentTitle, "#FFFFFF"),
        (ColorToken::Muted, "#E2E8F0"),
        (ColorToken::MutedText, "#334155"),
        (ColorToken::MutedTitle, "#1F3A5F"),
        (ColorToken::Background, "#F3F1EE"),
        (ColorToken::BackgroundText, "#334155"),
        (ColorToken::BackgroundTitle, "#1F3A5F"),
        (ColorToken::Surface, "#FFFFFF"),
        (ColorToken::SurfaceText, "#334155"),
        (ColorToken::SurfaceTitle, "#1F3A5F"),
        (ColorToken::Success, "#16A34A"),
        (ColorToken::SuccessText, "#E8F5E9"),
        (ColorToken::SuccessTitle, "#FFFFFF"),
        (ColorToken::Info, "#0084D1"),
        (ColorToken::InfoText, "#E1F5FE"),
        (ColorToken::InfoTitle, "#FFFFFF"),
        (ColorToken::Warning, "#D08700"),
        (ColorToken::WarningText, "#1F1400"),
        (ColorToken::WarningTitle, "#0D0900"),
        (ColorToken::Danger, "#E7000B"),
        (ColorToken::DangerText, "#FFEBEE"),
        (ColorToken::DangerTitle, "#FFFFFF"),
    ];
    let light = integrated_design_theme("light").expect("light theme");
    for (token, value) in expected_light {
        assert_eq!(light.color_value(token), value);
    }

    let dark = integrated_design_theme("dark").expect("dark theme");
    assert_eq!(dark.color_value(ColorToken::Primary), "#F3F1EE");
    assert_eq!(dark.color_value(ColorToken::Muted), "#334155");
    assert_eq!(dark.color_value(ColorToken::Background), "#111827");
    assert_eq!(dark.color_value(ColorToken::Surface), "#1F2937");
}

#[test]
fn represents_custom_theme_color_families_and_roles() {
    let happy = ColorFamily::from_name("happy").expect("custom family");
    let brand_accent = ColorFamily::from_name("brandAccent").expect("custom camel family");

    assert!(std::mem::size_of::<ColorFamily>() <= 2);
    assert!(std::mem::size_of::<ColorToken>() <= 2);
    assert_eq!(happy.as_str(), "happy");
    assert_eq!(happy.color_token().as_str(), "happy");
    assert_eq!(happy.text_token().as_str(), "happyText");
    assert_eq!(happy.title_token().as_str(), "happyTitle");
    assert_eq!(brand_accent.as_str(), "brandAccent");
    assert_eq!(ColorFamily::from_theme_name("softHappy"), None);
    assert_eq!(ColorFamily::from_name("softHappy"), None);
    assert_eq!(ColorFamily::from_name("happyText"), None);
    assert_eq!(ColorFamily::from_name("onHappy"), None);
    assert_eq!(ColorFamily::from_name("colors"), None);
    assert_eq!(ColorFamily::from_name("text"), None);
    assert_eq!(ColorFamily::from_name("Happy"), None);
    assert_eq!(ColorFamily::from_name("happy-day"), None);
    assert_eq!(ColorToken::from_name("happy"), Some(happy.color_token()));
    assert_eq!(ColorToken::from_name("happyText"), Some(happy.text_token()));
    assert_eq!(
        ColorToken::from_name("happyTitle"),
        Some(happy.title_token())
    );
}
