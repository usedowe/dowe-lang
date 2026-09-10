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
        COMPONENT_REGISTRY.get("Tree"),
        Some(BuiltinComponent::Tree)
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

