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
