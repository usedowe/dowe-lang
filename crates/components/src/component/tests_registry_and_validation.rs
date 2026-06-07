use std::{fs, path::Path};

use super::{
    Breakpoint, BuiltinComponent, ButtonSize, COMPONENT_REGISTRY, CodeLanguage, CodeTokenKind,
    ColorFamily, ColorToken, ComponentError, ComponentProp, ComponentVariant, DividerOrientation,
    FontFamily, GapValue, GridAlignment, GridTracks, OverlayPaint, PropValue, ResponsivePropEntry,
    ScaleValue, SectionBackground, SizeValue, SvgPathFill, TableColumnAlign, TableSize,
    TabsPosition, TabsVariant, TextSize, TextSpacing, TextWeight, VideoAspect, ViewAnimation,
    ViewNode, VisibilityCondition,
    bar_component_node, box_node, candlestick_node, children_node, code_node, compose_tree,
    container_component_node, divider_node, first_text, font_catalog, input_node, select_node,
    select_option_component, svg_component_node, svg_path_component, table_column_component,
    table_node, tabs_component_node, tabs_tab_component, text_component_node, text_node,
    text_spacing_em, text_typography, text_weight_number, validate_view_tree, video_node,
};

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
        COMPONENT_REGISTRY.get("Candlestick"),
        Some(BuiltinComponent::Candlestick)
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
    assert_eq!(
        COMPONENT_REGISTRY.get("Tabs"),
        Some(BuiltinComponent::Tabs)
    );
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
fn validates_box_children() {
    assert!(box_node(Vec::new()).is_ok());
    assert!(box_node(vec![text_node("Hello").expect("text")]).is_ok());
}

#[test]
fn validates_text_content() {
    assert_eq!(
        text_node("   ").expect_err("text error"),
        ComponentError::text_requires_static_text(BuiltinComponent::Text)
    );

    assert_eq!(
        text_node("  Hello  ").expect("text"),
        ViewNode::Text {
            props: Default::default(),
            value: "Hello".to_string()
        }
    );
}

#[test]
fn validates_code_source_and_highlighting() {
    let node = code_node(
        vec![
            string_prop("language", "dowe"),
            string_prop("variant", "soft"),
            string_prop("scheme", "surface"),
        ],
        vec![
            "page loginPage".to_string(),
            "  Card scheme:\"primary\"".to_string(),
            "    Text".to_string(),
            "      Login".to_string(),
        ],
    )
    .expect("code");

    match node {
        ViewNode::Code { props } => {
            assert_eq!(props.language, CodeLanguage::Dowe);
            assert_eq!(
                props
                    .tokens
                    .iter()
                    .map(|token| token.text.as_str())
                    .collect::<String>(),
                props.source
            );
            assert!(
                props
                    .tokens
                    .iter()
                    .any(|token| token.kind == CodeTokenKind::Keyword && token.text == "page")
            );
            assert!(
                props
                    .tokens
                    .iter()
                    .any(|token| token.kind == CodeTokenKind::Type && token.text == "Card")
            );
            assert!(
                props
                    .tokens
                    .iter()
                    .any(|token| token.kind == CodeTokenKind::Attribute && token.text == "scheme")
            );
        }
        _ => panic!("code"),
    }

    assert_eq!(
        code_node(Vec::new(), Vec::new()).expect_err("lines"),
        ComponentError::invalid_prop("lines", "non-empty string array")
    );
    assert_eq!(
        code_node(
            vec![string_prop("language", "python")],
            vec!["print()".to_string()]
        )
        .expect_err("language"),
        ComponentError::invalid_prop("language", "dowe, typescript, go or rust")
    );
}

#[test]
fn validates_video_source_and_defaults() {
    let node = video_node(vec![
        string_prop("src", "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8"),
        string_prop("poster", "/images/video.jpg"),
    ])
    .expect("video");

    match node {
        ViewNode::Video { props } => {
            assert_eq!(
                props.src,
                "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8"
            );
            assert_eq!(props.poster.as_deref(), Some("/images/video.jpg"));
            assert!(!props.autoplay);
            assert_eq!(props.aspect, VideoAspect::Horizontal);
            assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
        }
        _ => panic!("video"),
    }

    assert_eq!(
        video_node(Vec::new()).expect_err("src"),
        ComponentError::invalid_prop("src", "https URL")
    );
    assert_eq!(
        video_node(vec![string_prop("src", "http://example.com/video.mp4")]).expect_err("https"),
        ComponentError::invalid_prop("src", "https URL")
    );
    assert_eq!(
        video_node(vec![
            string_prop("src", "https://example.com/video.mp4"),
            string_prop("aspect", "wide"),
        ])
        .expect_err("aspect"),
        ComponentError::invalid_prop("aspect", "horizontal, vertical or square")
    );
}

#[test]
fn validates_candlestick_props_and_defaults() {
    let node = candlestick_node(vec![
        string_prop("data", "candles"),
        string_prop("stream", "/api/market/candles"),
        string_prop("variant", "soft"),
        string_prop("scheme", "surface"),
        string_prop("upColor", "success"),
        string_prop("downColor", "danger"),
        string_prop("emptyLabel", "Waiting for candles"),
        number_prop("maxPoints", 120),
    ])
    .expect("candlestick");

    match node {
        ViewNode::Candlestick { props } => {
            assert_eq!(props.data, "candles");
            assert_eq!(props.stream.as_deref(), Some("/api/market/candles"));
            assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
            assert_eq!(props.up_color, ColorToken::Success);
            assert_eq!(props.down_color, ColorToken::Danger);
            assert_eq!(props.empty_label, "Waiting for candles");
            assert_eq!(props.max_points, 120);
            assert!(props.style.style.sizing.h.is_some());
        }
        _ => panic!("candlestick"),
    }

    let default_node =
        candlestick_node(vec![string_prop("data", "candles")]).expect("default candlestick");
    match default_node {
        ViewNode::Candlestick { props } => {
            assert_eq!(props.stream, None);
            assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
            assert_eq!(props.up_color, ColorToken::Success);
            assert_eq!(props.down_color, ColorToken::Danger);
            assert_eq!(props.empty_label, "No candle data");
            assert_eq!(props.max_points, 240);
        }
        _ => panic!("candlestick"),
    }
}

#[test]
fn rejects_invalid_candlestick_props() {
    assert_eq!(
        candlestick_node(Vec::new()).expect_err("data"),
        ComponentError::invalid_prop("data", "signal array path")
    );
    assert_eq!(
        candlestick_node(vec![
            string_prop("data", "candles"),
            string_prop("stream", "http://example.com/events")
        ])
        .expect_err("stream"),
        ComponentError::invalid_prop("stream", "absolute path or https URL")
    );
    assert_eq!(
        candlestick_node(vec![
            string_prop("data", "candles"),
            string_prop("upColor", "brand")
        ])
        .expect_err("upColor"),
        ComponentError::invalid_prop("upColor", "color token")
    );
    assert_eq!(
        candlestick_node(vec![
            string_prop("data", "candles"),
            number_prop("maxPoints", 0)
        ])
        .expect_err("maxPoints"),
        ComponentError::invalid_prop("maxPoints", "positive integer")
    );
}

#[test]
fn validates_table_props_columns_and_defaults() {
    let name = table_column_component(vec![
        string_prop("field", "name"),
        string_prop("label", "Name"),
        string_prop("width", "12rem"),
    ])
    .expect("name column");
    let role = table_column_component(vec![
        string_prop("field", "profile.role"),
        string_prop("label", "Role"),
        string_prop("align", "end"),
    ])
    .expect("role column");
    let node = table_node(
        vec![
            string_prop("data", "users"),
            string_prop("variant", "soft"),
            string_prop("scheme", "surface"),
            string_prop("size", "lg"),
            boolean_prop("striped", true),
            boolean_prop("bordered", true),
            boolean_prop("dividers", false),
            string_prop("emptyTitle", "No users"),
            string_prop("emptyDescription", "Invite a user first"),
        ],
        vec![name, role],
    )
    .expect("table");

    match node {
        ViewNode::Table { props } => {
            assert_eq!(props.data, "users");
            assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
            assert_eq!(props.size, TableSize::Lg);
            assert!(props.striped);
            assert!(props.bordered);
            assert!(!props.dividers);
            assert_eq!(props.empty_title, "No users");
            assert_eq!(props.empty_description, "Invite a user first");
            assert_eq!(props.columns.len(), 2);
            assert_eq!(props.columns[0].width.as_deref(), Some("12rem"));
            assert_eq!(props.columns[1].align, TableColumnAlign::End);
        }
        _ => panic!("table"),
    }

    let default_node = table_node(
        vec![string_prop("data", "users")],
        vec![
            table_column_component(vec![
                string_prop("field", "name"),
                string_prop("label", "Name"),
            ])
            .expect("column"),
        ],
    )
    .expect("default table");
    match default_node {
        ViewNode::Table { props } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
            assert_eq!(props.size, TableSize::Md);
            assert!(!props.striped);
            assert!(!props.bordered);
            assert!(props.dividers);
        }
        _ => panic!("table"),
    }
}

#[test]
fn rejects_invalid_table_props_and_columns() {
    assert_eq!(
        table_node(Vec::new(), Vec::new()).expect_err("columns"),
        ComponentError::invalid_prop_combination("Table requires at least one column")
    );
    assert_eq!(
        table_node(
            vec![
                string_prop("data", "users"),
                string_prop("color", "primary")
            ],
            vec![
                table_column_component(vec![
                    string_prop("field", "name"),
                    string_prop("label", "Name"),
                ])
                .expect("column"),
            ],
        )
        .expect_err("color"),
        ComponentError::new("unknown prop `color` on `Table`; use `scheme` for visual family")
    );
    assert_eq!(
        table_node(
            Vec::new(),
            vec![
                table_column_component(vec![
                    string_prop("field", "name"),
                    string_prop("label", "Name"),
                ])
                .expect("column"),
            ],
        )
        .expect_err("data"),
        ComponentError::invalid_prop("data", "signal array path")
    );
    assert_eq!(
        table_column_component(vec![string_prop("label", "Name")]).expect_err("field"),
        ComponentError::invalid_prop("field", "relative field path")
    );
    assert_eq!(
        table_column_component(vec![
            string_prop("field", ".name"),
            string_prop("label", "Name"),
        ])
        .expect_err("field"),
        ComponentError::invalid_prop("field", "relative field path")
    );
    assert_eq!(
        table_column_component(vec![
            string_prop("field", "name"),
            string_prop("label", "Name"),
            string_prop("align", "right"),
        ])
        .expect_err("align"),
        ComponentError::invalid_prop("align", "start, center or end")
    );
    assert_eq!(
        table_column_component(vec![
            string_prop("field", "name"),
            string_prop("label", "Name"),
            string_prop("width", "calc(100%)"),
        ])
        .expect_err("width"),
        ComponentError::invalid_prop("width", "portable table width")
    );
}

#[test]
fn validates_tabs_props_entries_and_defaults() {
    let overview = tabs_tab_component(
        vec![string_prop("id", "overview"), string_prop("label", "Overview")],
        vec![text_node("Overview content").expect("text")],
    )
    .expect("overview tab");
    let details = tabs_tab_component(
        vec![string_prop("id", "details"), string_prop("label", "Details")],
        vec![text_node("Details content").expect("text")],
    )
    .expect("details tab");
    let node = tabs_component_node(
        vec![
            string_prop("variant", "line"),
            string_prop("scheme", "primary"),
            string_prop("position", "end"),
        ],
        vec![overview, details],
    )
    .expect("tabs");

    match node {
        ViewNode::Tabs { props, tabs } => {
            assert_eq!(props.variant, TabsVariant::Line);
            assert_eq!(props.color, ColorFamily::Primary);
            assert_eq!(props.position, TabsPosition::End);
            assert_eq!(tabs.len(), 2);
            assert_eq!(tabs[0].id, "overview");
            assert_eq!(tabs[0].label, "Overview");
            assert_eq!(first_text(&tabs[1].children[0]), Some("Details content".to_string()));
        }
        _ => panic!("tabs"),
    }

    let default_node = tabs_component_node(
        Vec::new(),
        vec![
            tabs_tab_component(
                vec![string_prop("id", "one"), string_prop("label", "One")],
                vec![text_node("One").expect("text")],
            )
            .expect("tab"),
        ],
    )
    .expect("default tabs");
    match default_node {
        ViewNode::Tabs { props, .. } => {
            assert_eq!(props.variant, TabsVariant::Solid);
            assert_eq!(props.color, ColorFamily::Muted);
            assert_eq!(props.position, TabsPosition::Top);
        }
        _ => panic!("tabs"),
    }
}

#[test]
fn rejects_invalid_tabs_contracts() {
    assert_eq!(
        tabs_component_node(Vec::new(), Vec::new()).expect_err("empty tabs"),
        ComponentError::invalid_prop_combination("Tabs requires at least one tab")
    );
    assert_eq!(
        tabs_component_node(
            vec![string_prop("color", "primary")],
            vec![
                tabs_tab_component(
                    vec![string_prop("id", "one"), string_prop("label", "One")],
                    vec![text_node("One").expect("text")],
                )
                .expect("tab"),
            ],
        )
        .expect_err("color"),
        ComponentError::new("unknown prop `color` on `Tabs`; use `scheme` for visual family")
    );
    assert_eq!(
        tabs_component_node(
            Vec::new(),
            vec![
                tabs_tab_component(
                    vec![string_prop("id", "one"), string_prop("label", "One")],
                    vec![text_node("One").expect("text")],
                )
                .expect("tab"),
                tabs_tab_component(
                    vec![string_prop("id", "one"), string_prop("label", "Duplicate")],
                    vec![text_node("Duplicate").expect("text")],
                )
                .expect("duplicate tab"),
            ],
        )
        .expect_err("duplicate"),
        ComponentError::invalid_prop_combination("duplicate Tabs tab id `one`")
    );
    assert_eq!(
        tabs_tab_component(
            vec![string_prop("id", "one"), string_prop("label", "One")],
            Vec::new(),
        )
        .expect_err("children"),
        ComponentError::invalid_prop_combination("Tabs tab `one` requires at least one child")
    );
    assert_eq!(
        container_component_node(
            BuiltinComponent::Tab,
            vec![string_prop("id", "one"), string_prop("label", "One")],
            vec![text_node("One").expect("text")],
            false,
        )
        .expect_err("tab outside tabs"),
        ComponentError::invalid_prop_combination("tab can only be used inside Tabs")
    );
}

#[test]
fn validates_divider_props_and_defaults() {
    let default_node = divider_node(Vec::new()).expect("divider");
    match default_node {
        ViewNode::Divider { props } => {
            assert_eq!(props.orientation, DividerOrientation::Horizontal);
            assert_eq!(props.color, ColorFamily::Muted);
            assert!(props.style.element.id.is_none());
        }
        _ => panic!("divider"),
    }

    let node = divider_node(vec![
        string_prop("orientation", "vertical"),
        string_prop("scheme", "primary"),
        string_prop("id", "main-divider"),
        number_prop("h", 24),
    ])
    .expect("divider");

    match node {
        ViewNode::Divider { props } => {
            assert_eq!(props.orientation, DividerOrientation::Vertical);
            assert_eq!(props.color, ColorFamily::Primary);
            assert_eq!(props.style.element.id.as_deref(), Some("main-divider"));
            assert!(props.style.sizing.h.is_some());
        }
        _ => panic!("divider"),
    }

    assert_eq!(
        divider_node(vec![string_prop("orientation", "diagonal")]).expect_err("orientation"),
        ComponentError::invalid_prop("orientation", "horizontal or vertical")
    );
}

#[test]
fn validates_children_scope() {
    assert_eq!(
        children_node(false).expect_err("children error"),
        ComponentError::children_outside_layout()
    );

    assert_eq!(children_node(true).expect("children"), ViewNode::Children);
}

#[test]
fn validates_design_props() {
    let node = container_component_node(
        BuiltinComponent::Box,
        vec![
            string_prop("bg", "primary"),
            string_prop("font", "roboto"),
            number_string_prop("px", "0.5"),
            number_prop("p", 8),
            responsive_number_prop("h", &[("xs", 16), ("md", 24)]),
        ],
        vec![text_node("Hello").expect("text")],
        false,
    )
    .expect("box");

    match node {
        ViewNode::Box { props, .. } => {
            assert!(props.bg.is_some());
            assert_eq!(
                props.font.expect("font").entries[0].value,
                FontFamily::Roboto
            );
            assert_eq!(
                props.spacing.p.expect("p").entries[0].value,
                ScaleValue::from_half_steps(16)
            );
            assert_eq!(
                props.spacing.px.expect("px").entries[0].value,
                ScaleValue::from_half_steps(1)
            );
            assert_eq!(
                props.sizing.h.expect("h").entries[1].value,
                SizeValue::Scale(ScaleValue::from_half_steps(48))
            );
        }
        _ => panic!("box"),
    }
}

#[test]
fn validates_container_refactor_props() {
    let flex = container_component_node(
        BuiltinComponent::Flex,
        vec![
            string_prop("justify", "space-between"),
            string_prop("gap", "20px"),
        ],
        vec![text_node("Hello").expect("text")],
        false,
    )
    .expect("flex");

    match flex {
        ViewNode::Flex { props, .. } => {
            assert_eq!(
                props.justify.expect("justify").entries[0].value.as_str(),
                "between"
            );
            assert!(matches!(
                props.gap.expect("gap").entries[0].value,
                GapValue::Single(_)
            ));
        }
        _ => panic!("flex"),
    }

    let grid = container_component_node(
        BuiltinComponent::Grid,
        vec![
            number_prop("columns", 3),
            string_prop("rows", "100px auto"),
            string_prop("justify", "center"),
            string_prop("gap", "10px 20px"),
        ],
        vec![
            container_component_node(
                BuiltinComponent::Box,
                vec![number_prop("colSpan", 2)],
                vec![text_node("Wide").expect("text")],
                false,
            )
            .expect("box"),
            container_component_node(
                BuiltinComponent::Card,
                vec![
                    string_prop("scheme", "surface"),
                    string_prop("rounded", "full"),
                    string_prop("cover", "/images/card.jpg"),
                    boolean_prop("overlay", true),
                ],
                vec![text_node("Card").expect("text")],
                false,
            )
            .expect("card"),
        ],
        false,
    )
    .expect("grid");

    validate_view_tree(&grid).expect("valid grid tree");

    match grid {
        ViewNode::Grid { props, children } => {
            assert_eq!(
                props.columns.expect("columns").entries[0].value,
                GridTracks::Count(3)
            );
            assert_eq!(
                props.justify.expect("justify").entries[0].value,
                GridAlignment::Center
            );
            assert_eq!(children.len(), 2);
        }
        _ => panic!("grid"),
    }
}

#[test]
fn rejects_grid_spans_outside_direct_grid_children() {
    let tree = container_component_node(
        BuiltinComponent::Box,
        Vec::new(),
        vec![
            container_component_node(
                BuiltinComponent::Box,
                vec![number_prop("colSpan", 2)],
                vec![text_node("Wide").expect("text")],
                false,
            )
            .expect("box"),
        ],
        false,
    )
    .expect("tree");

    assert!(validate_view_tree(&tree).is_err());
}

#[test]
fn validates_section_background_props() {
    let node = container_component_node(
        BuiltinComponent::Section,
        vec![
            string_prop("background", "aurora"),
            string_prop("color", "onBackground"),
            string_prop("animation", "fadeIn"),
        ],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("section");

    match node {
        ViewNode::Section { props, .. } => {
            assert_eq!(
                props.background.expect("background").entries[0].value,
                SectionBackground::Aurora
            );
            assert!(props.text.is_some());
            assert_eq!(props.animation, Some(ViewAnimation::FadeIn));
        }
        _ => panic!("section"),
    }
}

#[test]
fn rejects_invalid_section_background_props() {
    let invalid_background = container_component_node(
        BuiltinComponent::Section,
        vec![string_prop("background", "custom")],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("background");
    assert_eq!(
        invalid_background,
        ComponentError::invalid_prop(
            "background",
            "soft, aurora, sunrise, ocean, meadow or slate"
        )
    );

    let combined_layers = container_component_node(
        BuiltinComponent::Section,
        vec![
            string_prop("background", "aurora"),
            string_prop("cover", "/hero.jpg"),
        ],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("layers");
    assert_eq!(
        combined_layers,
        ComponentError::invalid_prop_combination(
            "`cover` and `background` cannot be used together on `Section`"
        )
    );
}

#[test]
fn rejects_overlay_without_cover() {
    let error = container_component_node(
        BuiltinComponent::Box,
        vec![boolean_prop("overlay", true)],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("overlay error");

    assert_eq!(
        error,
        ComponentError::invalid_prop_combination("`overlay` requires `cover` on `Box`")
    );
}

#[test]
fn parses_overlay_forms() {
    let rgba = container_component_node(
        BuiltinComponent::Box,
        vec![
            string_prop("cover", "/images/hero.jpg"),
            string_prop("overlay", "rgba(0,0,0,0.5)"),
        ],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect("rgba");

    match rgba {
        ViewNode::Box { props, .. } => {
            assert!(matches!(
                props.overlay.expect("overlay").entries[0].value,
                OverlayPaint::Rgba(_)
            ));
        }
        _ => panic!("box"),
    }

    assert!(
        container_component_node(
            BuiltinComponent::Box,
            vec![
                string_prop("cover", "/images/hero.jpg"),
                string_prop("overlay", "blur(4px)"),
            ],
            vec![text_node("Hero").expect("text")],
            false,
        )
        .is_err()
    );
}

#[test]
fn validates_variant_props() {
    let node = input_node(vec![
        string_prop("variant", "soft"),
        string_prop("scheme", "danger"),
        string_prop("bind", "blog.title"),
        string_prop("label", "Title"),
        string_prop("placeholder", "Write a title"),
        boolean_prop("labelFloating", true),
    ])
    .expect("input");

    match node {
        ViewNode::Input { props } => {
            assert_eq!(props.variant, Some(ComponentVariant::Soft));
            assert_eq!(props.color, Some(ColorFamily::Danger));
            assert_eq!(props.element.bind.as_deref(), Some("blog.title"));
            assert_eq!(props.label.as_deref(), Some("Title"));
            assert_eq!(props.placeholder.as_deref(), Some("Write a title"));
            assert!(props.label_floating);
        }
        _ => panic!("input"),
    }
}

#[test]
fn validates_layout_bar_props_and_regions() {
    let node = bar_component_node(
        BuiltinComponent::AppBar,
        vec![
            string_prop("variant", "soft"),
            string_prop("scheme", "surface"),
            boolean_prop("bordered", true),
            boolean_prop("blurred", true),
            boolean_prop("boxed", true),
            boolean_prop("floating", true),
        ],
        vec![text_node("Menu").expect("text")],
        vec![text_node("Brand").expect("text")],
        vec![children_node(true).expect("children")],
        true,
    )
    .expect("appbar");

    match node {
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
            assert!(props.bordered);
            assert!(props.blurred);
            assert!(props.boxed);
            assert!(props.floating);
            assert_eq!(start.len(), 1);
            assert_eq!(center.len(), 1);
            assert_eq!(end, vec![ViewNode::Children]);
        }
        _ => panic!("appbar"),
    }

    let error = bar_component_node(
        BuiltinComponent::Footer,
        vec![boolean_prop("floating", true)],
        vec![text_node("Footer").expect("text")],
        Vec::new(),
        Vec::new(),
        false,
    )
    .expect_err("footer floating");

    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Footer, "floating")
    );
}

#[test]
fn validates_select_options() {
    let option = select_option_component(vec![
        string_prop("value", "admin"),
        string_prop("label", "Admin"),
        string_prop("description", "Full access"),
    ])
    .expect("option");
    assert_eq!(option.value, "admin");
    assert_eq!(option.label, "Admin");
    assert_eq!(option.description.as_deref(), Some("Full access"));

    let node = select_node(
        vec![
            string_prop("bind", "profile.role"),
            string_prop("label", "Role"),
            string_prop("placeholder", "Choose role"),
            boolean_prop("labelFloating", true),
            string_prop("variant", "outlined"),
            string_prop("scheme", "secondary"),
        ],
        vec![
            option,
            select_option_component(vec![
                string_prop("value", "viewer"),
                string_prop("label", "Viewer"),
            ])
            .expect("viewer"),
        ],
    )
    .expect("select");

    match node {
        ViewNode::Select { props, options } => {
            assert_eq!(props.element.bind.as_deref(), Some("profile.role"));
            assert_eq!(props.label.as_deref(), Some("Role"));
            assert_eq!(props.placeholder.as_deref(), Some("Choose role"));
            assert!(props.label_floating);
            assert_eq!(props.variant, Some(ComponentVariant::Outlined));
            assert_eq!(props.color, Some(ColorFamily::Secondary));
            assert_eq!(options.len(), 2);
        }
        _ => panic!("select"),
    }

    let duplicate = select_node(
        Vec::new(),
        vec![
            select_option_component(vec![
                string_prop("value", "admin"),
                string_prop("label", "Admin"),
            ])
            .expect("admin"),
            select_option_component(vec![
                string_prop("value", "admin"),
                string_prop("label", "Duplicate"),
            ])
            .expect("duplicate"),
        ],
    )
    .expect_err("duplicate");
    assert_eq!(
        duplicate,
        ComponentError::invalid_prop_combination("duplicate Select option value `admin`")
    );
}

#[test]
fn keeps_card_padding_author_controlled() {
    let default_card = container_component_node(
        BuiltinComponent::Card,
        Vec::new(),
        vec![text_node("Card").expect("text")],
        false,
    )
    .expect("card");

    match default_card {
        ViewNode::Card { props, .. } => {
            assert!(props.style.spacing.p.is_none());
            assert!(props.style.spacing.px.is_none());
            assert!(props.style.spacing.py.is_none());
        }
        _ => panic!("card"),
    }

    let padded_card = container_component_node(
        BuiltinComponent::Card,
        vec![number_prop("p", 4)],
        vec![text_node("Card").expect("text")],
        false,
    )
    .expect("padded card");

    match padded_card {
        ViewNode::Card { props, .. } => {
            assert_eq!(
                props.style.spacing.p.expect("p").entries[0].value,
                ScaleValue::from_half_steps(8)
            );
        }
        _ => panic!("card"),
    }
}

#[test]
fn validates_button_events_and_alert_props() {
    let button = container_component_node(
        BuiltinComponent::Button,
        vec![string_prop("onClick", "saveBlog")],
        vec![text_node("Save").expect("text")],
        false,
    )
    .expect("button");
    match button {
        ViewNode::Button { props, .. } => {
            assert_eq!(props.element.on_click.as_deref(), Some("saveBlog"));
        }
        _ => panic!("button"),
    }

    let alert = container_component_node(
        BuiltinComponent::Alert,
        vec![
            string_prop("type", "success"),
            string_prop("message", "alert.message"),
            string_prop("visible", "alert.visible"),
            string_prop("onClose", "closeAlert"),
        ],
        Vec::new(),
        false,
    )
    .expect("alert");
    match alert {
        ViewNode::Alert { props } => {
            assert_eq!(props.kind.as_str(), "success");
            assert_eq!(props.message, "alert.message");
            assert_eq!(props.visible.as_deref(), Some("alert.visible"));
            assert_eq!(props.on_close.as_deref(), Some("closeAlert"));
        }
        _ => panic!("alert"),
    }
}

#[test]
fn normalizes_button_visual_props() {
    let node = container_component_node(
        BuiltinComponent::Button,
        vec![string_prop("size", "lg"), number_prop("pl", 1)],
        vec![text_node("Save").expect("text")],
        false,
    )
    .expect("button");

    match node {
        ViewNode::Button { props, .. } => {
            assert_eq!(props.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.color, Some(ColorFamily::Primary));
            assert_eq!(props.size, Some(ButtonSize::Lg));
            assert_eq!(
                props.style.spacing.pl.expect("pl").entries[0].value,
                ScaleValue::from_half_steps(2)
            );
            assert_eq!(
                props.style.spacing.pr.expect("pr").entries[0].value,
                ScaleValue::from_half_steps(10)
            );
            assert_eq!(
                props.style.spacing.py.expect("py").entries[0].value,
                ScaleValue::from_half_steps(6)
            );
            assert_eq!(
                props.style.sizing.min_h.expect("minH").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(22))
            );
        }
        _ => panic!("button"),
    }
}
