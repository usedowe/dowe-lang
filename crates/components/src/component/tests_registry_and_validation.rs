use std::{fs, path::Path};

use super::{
    AvatarGroupProps, BrandProps, Breakpoint, BuiltinComponent, ButtonSize, COMPONENT_REGISTRY, CanvasBackground, CanvasFit,
    CarouselVariant, ChartCurve,
    ChartLegendPosition, ChartPalette, ChartSize, CodeLanguage, CodeTemplateSegment, CodeTokenKind, ColorFamily,
    ColorToken, ComponentError, ComponentProp, ComponentVariant, DividerOrientation, FlexDirection, FontFamily,
    BoxPosition, FabProps, GapValue, GridAlignment, GridTracks, OverlayCornerPosition, OverlayPaint, PropValue, RadioGroupOrientation,
    ResponsivePropEntry, ScaleValue, SectionBackground, SizeValue, SpacingProps, SvgLineCap, SvgLineJoin, SvgPathFill, SvgTransform, TableColumnAlign,
    BarPosition, DeviceProfile, IframeLoading, NativeExternalMode, NavigationAction, TableSize, TabsPosition, TabsVariant, TextSize, TextSpacing,
    TextWeight, VideoAspect, WebTarget,
    ViewAnimation, ViewIcon, ViewNode, VisibilityCondition,
    arc_chart_component_node, area_chart_component_node, bar_chart_component_node,
    bar_component_node, box_node, canvas_component_node, candlestick_node, children_node, code_node, compose_tree, fixed_box_nodes, fixed_fab_nodes,
    carousel_component_node, carousel_slide_component, container_component_node, device_node,
    divider_node, first_text, font_catalog, icon_component_node, iframe_node, input_node,
    line_chart_component_node, pie_chart_component_node, radio_group_component_node,
    radio_option_component, select_node, select_option_component, svg_component_node,
    svg_path_component, table_column_component, table_node,
    tabs_component_node, tabs_tab_component, text_binding_path, text_component_node, text_node,
    text_spacing_em,
    all_icon_names, country_flag_icon, phone_countries, text_typography, text_weight_number,
    section_content_spacing, validate_solar_icon_catalog, validate_svg_logo_catalog,
    validate_svg_spinner_catalog, validate_view_tree, video_node, COUNTRY_FLAGS, SVG_LOGOS,
    SVG_SPINNERS,
};

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
    assert_eq!(COMPONENT_REGISTRY.get("Canvas"), Some(BuiltinComponent::Canvas));
    assert_eq!(COMPONENT_REGISTRY.get("Iframe"), Some(BuiltinComponent::Iframe));
    assert_eq!(COMPONENT_REGISTRY.get("Device"), Some(BuiltinComponent::Device));
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
        assert_eq!(BuiltinComponent::from_name(component.as_str()), Some(*component));
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
        "page loginPage\n  Card scheme:\"primary\"\n    Text\n      Login".to_string(),
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
        code_node(Vec::new(), String::new()).expect_err("content"),
        ComponentError::invalid_prop("content", "non-empty multiline string")
    );
    assert_eq!(
        code_node(vec![string_prop("language", "ruby")], "puts()".to_string())
        .expect_err("language"),
        ComponentError::invalid_prop(
            "language",
            "dowe, typescript, javascript, go, rust or python"
        )
    );
}

#[test]
fn highlights_javascript_python_and_reactive_code_segments() {
    let javascript = code_node(
        vec![string_prop("language", "javascript")],
        "const value = new Promise(() => true);".to_string(),
    )
    .expect("javascript");
    let python = code_node(
        vec![string_prop("language", "python")],
        "def greet(name: str):\n  # welcome\n  return f\"Hello {name}\"".to_string(),
    )
    .expect("python");
    let template = code_node(
        vec![
            string_prop("language", "dowe"),
            ComponentProp {
                name: "template".to_string(),
                value: PropValue::Boolean(true),
            },
        ],
        "Button scheme:\"{scheme}\"\n  \"Preview\"".to_string(),
    )
    .expect("template");

    for node in [javascript, python] {
        let ViewNode::Code { props } = node else {
            panic!("code")
        };
        assert_eq!(
            props
                .tokens
                .iter()
                .map(|token| token.text.as_str())
                .collect::<String>(),
            props.source
        );
        assert!(props.tokens.iter().any(|token| token.kind == CodeTokenKind::Keyword));
        assert!(props.tokens.iter().any(|token| token.kind == CodeTokenKind::Type));
    }

    let ViewNode::Code { props } = template else {
        panic!("code")
    };
    assert!(props.template_segments.iter().any(|segment| matches!(
        segment,
        CodeTemplateSegment::Static { tokens, .. }
            if tokens.iter().any(|token| token.kind == CodeTokenKind::Type)
    )));
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
fn validates_iframe_source_policy_and_defaults() {
    let node = iframe_node(vec![
        string_prop("src", "https://example.com/embed"),
        string_prop("title", "Example embed"),
        string_prop("allow", "fullscreen; autoplay"),
        string_prop("sandbox", "scripts same-origin"),
    ]).expect("iframe");
    let ViewNode::Iframe { props } = node else { panic!("iframe") };
    assert_eq!(props.loading, IframeLoading::Lazy);
    assert_eq!(props.allow, vec!["fullscreen", "autoplay"]);
    assert_eq!(props.sandbox, Some(vec!["scripts".to_string(), "same-origin".to_string()]));
    assert!(!props.allow_fullscreen);
    let internal = iframe_node(vec![
        string_prop("src", "/examples/appbar-one"),
        string_prop("title", "Local example"),
    ]).expect("internal iframe");
    let ViewNode::Iframe { props } = internal else { panic!("iframe") };
    assert_eq!(props.src, "/examples/appbar-one");
    assert_eq!(
        iframe_node(vec![string_prop("src", "http://example.com"), string_prop("title", "Example")]).expect_err("https"),
        ComponentError::invalid_prop("src", "https URL or internal route")
    );
    assert_eq!(
        iframe_node(vec![string_prop("src", "//example.com"), string_prop("title", "Example")]).expect_err("scheme relative"),
        ComponentError::invalid_prop("src", "https URL or internal route")
    );
    assert_eq!(
        iframe_node(vec![string_prop("src", "/examples/../admin"), string_prop("title", "Example")]).expect_err("traversal"),
        ComponentError::invalid_prop("src", "https URL or internal route")
    );
    assert_eq!(
        iframe_node(vec![string_prop("src", "https://example.com")]).expect_err("title"),
        ComponentError::invalid_prop("title", "non-empty string")
    );
}

#[test]
fn validates_device_profile_and_iframe_child() {
    let iframe = iframe_node(vec![
        string_prop("src", "/preview"),
        string_prop("title", "Preview"),
    ])
    .expect("iframe");
    let node = device_node(vec![string_prop("device", "laptop")], vec![iframe.clone()])
        .expect("device");
    let ViewNode::Device { props, iframe: nested } = node else {
        panic!("device")
    };
    assert_eq!(props.device, DeviceProfile::Laptop);
    assert_eq!(props.device.dimensions(), (1440, 900));
    assert_eq!(nested.title, "Preview");
    assert_eq!(
        device_node(vec![string_prop("device", "watch")], vec![iframe.clone()])
            .expect_err("device"),
        ComponentError::invalid_prop("device", "mobile, tablet, laptop or monitor")
    );
    assert!(device_node(Vec::new(), Vec::new()).is_err());
    assert!(device_node(Vec::new(), vec![iframe.clone(), iframe]).is_err());
}

#[test]
fn validates_canvas_props_and_defaults() {
    let node = canvas_component_node(vec![
        string_prop("scene", "gameScene"),
        string_prop("label", "Game scene"),
        string_prop("fit", "cover"),
        number_prop("viewWidth", 640),
        number_prop("viewHeight", 360),
        number_prop("fps", 30),
        boolean_prop("autoplay", false),
        string_prop("background", "surface"),
        boolean_prop("pixelated", true),
        string_prop("onPointer", "capturePointer"),
        string_prop("onKey", "captureKey"),
        string_prop("onMotion", "captureMotion"),
        number_prop("motionRate", 45),
    ])
    .expect("canvas");
    match node {
        ViewNode::Canvas { props } => {
            assert_eq!(props.scene, "gameScene");
            assert_eq!(props.view_width, 640);
            assert_eq!(props.view_height, 360);
            assert_eq!(props.fit, CanvasFit::Cover);
            assert_eq!(props.fps, 30);
            assert!(!props.autoplay);
            assert_eq!(props.background, CanvasBackground::Color(ColorToken::Surface));
            assert!(props.pixelated);
            assert_eq!(props.label, "Game scene");
            assert_eq!(props.on_pointer.as_deref(), Some("capturePointer"));
            assert_eq!(props.on_key.as_deref(), Some("captureKey"));
            assert_eq!(props.on_motion.as_deref(), Some("captureMotion"));
            assert_eq!(props.motion_rate, 45);
        }
        _ => panic!("canvas"),
    }

    assert!(canvas_component_node(vec![string_prop("label", "Missing scene")]).is_err());
    assert!(canvas_component_node(vec![string_prop("scene", "scene"), string_prop("label", "")]).is_err());
    assert!(canvas_component_node(vec![string_prop("scene", "scene"), string_prop("label", "Scene"), number_prop("fps", 121)]).is_err());
    assert!(canvas_component_node(vec![string_prop("scene", "scene"), string_prop("label", "Scene"), string_prop("fit", "center")]).is_err());
    assert!(canvas_component_node(vec![string_prop("scene", "scene"), string_prop("label", "Scene"), number_prop("motionRate", 61)]).is_err());
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
fn validates_chart_component_props_and_defaults() {
    let arc = arc_chart_component_node(vec![
        string_prop("data", "segments"),
        string_prop("palette", "ocean"),
        string_prop("legendPosition", "left"),
        number_prop("thickness", 18),
        boolean_prop("showInlineLabels", true),
    ])
    .expect("arc chart");
    match arc {
        ViewNode::ArcChart { props } => {
            assert_eq!(props.common.data.as_deref(), Some("segments"));
            assert_eq!(props.common.palette, ChartPalette::Ocean);
            assert_eq!(props.common.legend_position, ChartLegendPosition::Left);
            assert_eq!(props.common.size, ChartSize::Md);
            assert_eq!(props.thickness, 18);
            assert!(props.show_inline_labels);
        }
        _ => panic!("arc chart"),
    }

    let area = area_chart_component_node(vec![
        string_prop("series", "traffic"),
        string_prop("curve", "smooth"),
        number_string_prop("fillOpacity", "0.42"),
        boolean_prop("showPoints", true),
    ])
    .expect("area chart");
    match area {
        ViewNode::AreaChart { props } => {
            assert_eq!(props.common.series.as_deref(), Some("traffic"));
            assert_eq!(props.common.legend_position, ChartLegendPosition::Bottom);
            assert_eq!(props.curve, ChartCurve::Smooth);
            assert_eq!(props.fill_opacity, 42);
            assert!(props.show_points);
        }
        _ => panic!("area chart"),
    }

    let bar = bar_chart_component_node(vec![
        string_prop("data", "sales"),
        string_prop("size", "lg"),
        string_prop("scheme", "surface"),
        boolean_prop("grouped", true),
    ])
    .expect("bar chart");
    match bar {
        ViewNode::BarChart { props } => {
            assert_eq!(props.common.data.as_deref(), Some("sales"));
            assert_eq!(props.common.size, ChartSize::Lg);
            assert_eq!(props.common.style.color, Some(ColorFamily::Surface));
            assert!(props.grouped);
        }
        _ => panic!("bar chart"),
    }

    let line = line_chart_component_node(vec![
        string_prop("data", "trend"),
        string_prop("palette", "forest"),
        string_prop("curve", "smooth"),
        boolean_prop("showGradientFill", true),
    ])
    .expect("line chart");
    match line {
        ViewNode::LineChart { props } => {
            assert_eq!(props.common.data.as_deref(), Some("trend"));
            assert_eq!(props.common.palette, ChartPalette::Forest);
            assert_eq!(props.curve, ChartCurve::Smooth);
            assert!(props.show_gradient_fill);
        }
        _ => panic!("line chart"),
    }

    let pie = pie_chart_component_node(vec![
        string_prop("data", "segments"),
        boolean_prop("donut", true),
        number_prop("donutWidth", 72),
        string_prop("centerLabel", "Total"),
    ])
    .expect("pie chart");
    match pie {
        ViewNode::PieChart { props } => {
            assert_eq!(props.common.data.as_deref(), Some("segments"));
            assert_eq!(props.common.legend_position, ChartLegendPosition::Right);
            assert!(props.donut);
            assert_eq!(props.donut_width, 72);
            assert_eq!(props.center_label.as_deref(), Some("Total"));
        }
        _ => panic!("pie chart"),
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
fn validates_radio_group_orientation_props_and_defaults() {
    let option = radio_option_component(vec![
        string_prop("value", "basic"),
        string_prop("label", "Basic"),
    ])
    .expect("option");
    let default_node =
        radio_group_component_node(Vec::new(), vec![option.clone()]).expect("radio group");
    match default_node {
        ViewNode::RadioGroup { props, .. } => {
            assert_eq!(props.orientation, RadioGroupOrientation::Vertical);
            assert_eq!(props.size, ButtonSize::Md);
        }
        _ => panic!("radio group"),
    }

    let horizontal_node = radio_group_component_node(
        vec![string_prop("orientation", "horizontal")],
        vec![option.clone()],
    )
    .expect("horizontal radio group");
    match horizontal_node {
        ViewNode::RadioGroup { props, .. } => {
            assert_eq!(props.orientation, RadioGroupOrientation::Horizontal);
        }
        _ => panic!("radio group"),
    }

    assert_eq!(
        radio_group_component_node(vec![string_prop("orientation", "grid")], vec![option])
            .expect_err("orientation"),
        ComponentError::invalid_prop("orientation", "vertical or horizontal")
    );
}

#[test]
fn validates_carousel_variants_and_defaults() {
    let slide = carousel_slide_component(
        vec![string_prop("id", "one")],
        vec![text_node("Slide").expect("text")],
    )
    .expect("slide");
    let default_node =
        carousel_component_node(Vec::new(), vec![slide.clone()]).expect("carousel");
    match default_node {
        ViewNode::Carousel { props, .. } => assert_eq!(props.variant, CarouselVariant::Simple),
        _ => panic!("carousel"),
    }

    for variant in CarouselVariant::all() {
        let node = carousel_component_node(
            vec![string_prop("variant", variant.as_str())],
            vec![slide.clone()],
        )
        .expect("carousel variant");
        match node {
            ViewNode::Carousel { props, .. } => assert_eq!(props.variant, *variant),
            _ => panic!("carousel"),
        }
    }

    assert!(
        carousel_component_node(
            vec![string_prop("variant", "wheel")],
            vec![slide],
        )
        .is_err()
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
            string_prop("minH", "vh-16"),
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
            assert_eq!(
                props.sizing.min_h.expect("minH").entries[0].value,
                SizeValue::ViewportMinus(ScaleValue::from_half_steps(32))
            );
        }
        _ => panic!("box"),
    }

    assert_eq!(
        container_component_node(
            BuiltinComponent::Box,
            vec![string_prop("h", "vh-nope")],
            vec![text_node("Hello").expect("text")],
            false,
        )
        .expect_err("invalid viewport height"),
        ComponentError::invalid_prop("h", "Dowe scale value, full or vh-<scale>")
    );

    assert_eq!(
        container_component_node(
            BuiltinComponent::Box,
            vec![string_prop("w", "vh-16")],
            vec![text_node("Hello").expect("text")],
            false,
        )
        .expect_err("viewport height as width"),
        ComponentError::invalid_prop("w", "Dowe scale value or full")
    );
}

#[test]
fn validates_container_refactor_props() {
    let flex = container_component_node(
        BuiltinComponent::Flex,
        vec![
            responsive_string_prop("direction", &[("xs", "column"), ("md", "row")]),
            boolean_prop("wrap", true),
            string_prop("justify", "space-between"),
            string_prop("gap", "20px"),
        ],
        vec![text_node("Hello").expect("text")],
        false,
    )
    .expect("flex");

    match flex {
        ViewNode::Flex { props, .. } => {
            assert_eq!(props.direction.entries[0].value, FlexDirection::Column);
            assert_eq!(props.direction.entries[1].breakpoint, Breakpoint::Md);
            assert_eq!(props.direction.entries[1].value, FlexDirection::Row);
            assert!(props.wrap);
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

    let default_flex = container_component_node(
        BuiltinComponent::Flex,
        Vec::new(),
        vec![text_node("Default").expect("text")],
        false,
    )
    .expect("default flex");
    match default_flex {
        ViewNode::Flex { props, .. } => {
            assert_eq!(props.direction.entries[0].breakpoint, Breakpoint::Xs);
            assert_eq!(props.direction.entries[0].value, FlexDirection::Row);
            assert!(!props.wrap);
        }
        _ => panic!("flex"),
    }

    assert_eq!(
        container_component_node(
            BuiltinComponent::Flex,
            vec![string_prop("direction", "row-reverse")],
            Vec::new(),
            false,
        )
        .expect_err("invalid flex direction"),
        ComponentError::invalid_prop("direction", "row or column")
    );

    assert_eq!(
        container_component_node(
            BuiltinComponent::Flex,
            vec![string_prop("wrap", "true")],
            Vec::new(),
            false,
        )
        .expect_err("invalid flex wrap"),
        ComponentError::invalid_prop("wrap", "boolean")
    );

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
fn validates_relative_absolute_and_fixed_box_positioning() {
    let tree = container_component_node(
        BuiltinComponent::Box,
        vec![string_prop("position", "relative")],
        vec![container_component_node(
            BuiltinComponent::Box,
            vec![
                string_prop("position", "absolute"),
                number_prop("top", 4),
                number_prop("right", 6),
            ],
            vec![text_node("Proof").expect("text")],
            false,
        )
        .expect("absolute box")],
        false,
    )
    .expect("relative box");

    validate_view_tree(&tree).expect("valid positioned tree");
    let ViewNode::Box { props, children } = &tree else {
        panic!("box");
    };
    assert_eq!(props.position().mode, BoxPosition::Relative);
    let ViewNode::Box { props, .. } = &children[0] else {
        panic!("absolute box");
    };
    assert_eq!(props.position().mode, BoxPosition::Absolute);
    assert_eq!(
        props.position().top.as_ref().expect("top").entries[0]
            .value
            .native_units(),
        16
    );
    assert_eq!(
        props.position().right.as_ref().expect("right").entries[0]
            .value
            .native_units(),
        24
    );

    let fixed = container_component_node(
        BuiltinComponent::Box,
        vec![
            string_prop("position", "fixed"),
            number_prop("bottom", 4),
            number_prop("right", 4),
        ],
        vec![text_node("Persistent").expect("text")],
        false,
    )
    .expect("fixed box");
    validate_view_tree(&fixed).expect("valid fixed box");
    assert_eq!(fixed_box_nodes(&fixed).len(), 1);
}

#[test]
fn rejects_invalid_box_positioning_contracts() {
    let static_offset_error = container_component_node(
            BuiltinComponent::Box,
            vec![number_prop("top", 4)],
            Vec::new(),
            false,
        )
        .expect_err("static offset");
    assert!(
        static_offset_error
            .to_string()
            .contains("require `position:\"absolute\"` or `position:\"fixed\"`"),
        "{static_offset_error}"
    );
    assert!(
        container_component_node(
            BuiltinComponent::Box,
            vec![
                string_prop("position", "absolute"),
                number_prop("left", 2),
                number_prop("right", 2),
            ],
            Vec::new(),
            false,
        )
        .expect_err("ambiguous horizontal axis")
        .to_string()
        .contains("`left` and `right`")
    );

    let orphan = container_component_node(
        BuiltinComponent::Box,
        vec![string_prop("position", "absolute")],
        Vec::new(),
        false,
    )
    .expect("absolute box");
    assert!(
        validate_view_tree(&orphan)
            .expect_err("orphan absolute box")
            .to_string()
            .contains("direct child of `Box position:\"relative\"`")
    );

    let fixed_in_each = ViewNode::Each {
        item: "item".to_string(),
        collection: "items".to_string(),
        key: "item.id".to_string(),
        children: vec![container_component_node(
            BuiltinComponent::Box,
            vec![string_prop("position", "fixed")],
            Vec::new(),
            false,
        )
        .expect("fixed box")],
    };
    assert!(
        validate_view_tree(&fixed_in_each)
            .expect_err("fixed inside each")
            .to_string()
            .contains("cannot be nested inside `each` or `Splash`")
    );
}

#[test]
fn validates_section_background_props() {
    let node = container_component_node(
        BuiltinComponent::Section,
        vec![
            string_prop("background", "aurora"),
            string_prop("color", "onBackground"),
            string_prop("animation", "fadeIn"),
            boolean_prop("boxed", true),
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
            assert!(props.boxed);
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
fn rejects_non_boolean_section_boxed_prop() {
    let error = container_component_node(
        BuiltinComponent::Section,
        vec![string_prop("boxed", "true")],
        vec![text_node("Hero").expect("text")],
        false,
    )
    .expect_err("boxed");

    assert_eq!(error, ComponentError::invalid_prop("boxed", "boolean"));
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
            string_prop("position", "sticky"),
        ],
        Vec::new(),
        vec![text_node("Menu").expect("text")],
        vec![text_node("Brand").expect("text")],
        vec![children_node(true).expect("children")],
        Vec::new(),
        true,
    )
    .expect("appbar");

    match node {
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
            top,
            bottom,
        } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
            assert!(props.bordered);
            assert!(props.blurred);
            assert!(props.boxed);
            assert!(props.floating);
            assert_eq!(props.position, BarPosition::Sticky);
            assert_eq!(start.len(), 1);
            assert_eq!(center.len(), 1);
            assert_eq!(end, vec![ViewNode::Children]);
            assert!(top.is_empty());
            assert!(bottom.is_empty());
        }
        _ => panic!("appbar"),
    }

    let footer = bar_component_node(
        BuiltinComponent::Footer,
        vec![boolean_prop("boxed", true)],
        vec![text_node("Directory").expect("text")],
        Vec::new(),
        vec![text_node("Navigation").expect("text")],
        Vec::new(),
        vec![text_node("Legal").expect("text")],
        false,
    )
    .expect("footer");

    let ViewNode::Footer {
        props,
        top,
        center,
        bottom,
        ..
    } = footer
    else {
        panic!("footer");
    };
    assert!(props.boxed);
    assert_eq!(top.len(), 1);
    assert_eq!(center.len(), 1);
    assert_eq!(bottom.len(), 1);

    let error = bar_component_node(
        BuiltinComponent::Footer,
        vec![boolean_prop("floating", true)],
        Vec::new(),
        vec![text_node("Footer").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    )
    .expect_err("footer floating");

    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Footer, "floating")
    );

    let error = bar_component_node(
        BuiltinComponent::AppBar,
        vec![string_prop("position", "absolute")],
        Vec::new(),
        vec![text_node("Menu").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    )
    .expect_err("appbar position");
    assert_eq!(
        error,
        ComponentError::invalid_prop("position", "static, sticky or fixed")
    );

    let error = bar_component_node(
        BuiltinComponent::BottomBar,
        vec![string_prop("position", "fixed")],
        Vec::new(),
        vec![text_node("Menu").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    )
    .expect_err("bottom bar position");
    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::BottomBar, "position")
    );
}

#[test]
fn applies_footer_horizontal_padding_defaults_and_preserves_overrides() {
    let default_footer = bar_component_node(
        BuiltinComponent::Footer,
        Vec::new(),
        vec![text_node("Directory").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    )
    .expect("default footer");

    let ViewNode::Footer { props, .. } = default_footer else {
        panic!("footer");
    };
    let horizontal = props.style.style.spacing.px.expect("default px");
    assert_eq!(horizontal.entries.len(), 2);
    assert_eq!(horizontal.entries[0].breakpoint, Breakpoint::Xs);
    assert_eq!(horizontal.entries[0].value, ScaleValue::from_half_steps(8));
    assert_eq!(horizontal.entries[1].breakpoint, Breakpoint::Md);
    assert_eq!(horizontal.entries[1].value, ScaleValue::from_half_steps(12));

    let authored_footer = bar_component_node(
        BuiltinComponent::Footer,
        vec![number_prop("px", 2)],
        vec![text_node("Directory").expect("text")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    )
    .expect("authored footer");

    let ViewNode::Footer { props, .. } = authored_footer else {
        panic!("footer");
    };
    let horizontal = props.style.style.spacing.px.expect("authored px");
    assert_eq!(horizontal.entries.len(), 1);
    assert_eq!(horizontal.entries[0].value, ScaleValue::from_half_steps(4));
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
        ViewNode::Select { props, options, .. } => {
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
fn normalizes_card_padding_default_and_author_override() {
    let default_card = container_component_node(
        BuiltinComponent::Card,
        Vec::new(),
        vec![text_node("Card").expect("text")],
        false,
    )
    .expect("card");

    match default_card {
        ViewNode::Card { props, .. } => {
            let padding = props.style.spacing.p.expect("default padding");
            assert_eq!(padding.entries.len(), 2);
            assert_eq!(padding.entries[0].breakpoint, Breakpoint::Xs);
            assert_eq!(padding.entries[0].value, ScaleValue::from_half_steps(8));
            assert_eq!(padding.entries[1].breakpoint, Breakpoint::Lg);
            assert_eq!(padding.entries[1].value, ScaleValue::from_half_steps(10));
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

    let vertical_card = container_component_node(
        BuiltinComponent::Card,
        vec![number_prop("py", 6)],
        vec![text_node("Card").expect("text")],
        false,
    )
    .expect("vertical card");

    match vertical_card {
        ViewNode::Card { props, .. } => {
            assert!(props.style.spacing.p.is_none());
            assert_eq!(
                props.style.spacing.py.expect("py").entries[0].value,
                ScaleValue::from_half_steps(12)
            );
            let horizontal = props.style.spacing.px.expect("default px");
            assert_eq!(horizontal.entries[0].value, ScaleValue::from_half_steps(8));
            assert_eq!(horizontal.entries[1].value, ScaleValue::from_half_steps(10));
        }
        _ => panic!("card"),
    }
}

#[test]
fn derives_section_axis_padding_defaults_and_preserves_overrides() {
    let default_spacing = section_content_spacing(&SpacingProps::default());
    let horizontal = default_spacing.px.expect("default horizontal padding");
    let vertical = default_spacing.py.expect("default vertical padding");
    assert_eq!(horizontal.entries[0].value, ScaleValue::from_half_steps(8));
    assert_eq!(horizontal.entries[1].value, ScaleValue::from_half_steps(12));
    assert_eq!(vertical.entries[0].value, ScaleValue::from_half_steps(20));
    assert_eq!(vertical.entries[1].value, ScaleValue::from_half_steps(32));

    let authored = SpacingProps {
        py: Some(super::ResponsiveValue::scalar(ScaleValue::from_half_steps(12))),
        ..Default::default()
    };
    let effective = section_content_spacing(&authored);
    assert_eq!(
        effective.py.expect("authored vertical padding").entries[0].value,
        ScaleValue::from_half_steps(12)
    );
    assert_eq!(
        effective.px.expect("default horizontal padding").entries[1].value,
        ScaleValue::from_half_steps(12)
    );
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
fn resolves_icon_button_and_control_icon_regions() {
    let icon_button = container_component_node(
        BuiltinComponent::IconButton,
        vec![string_prop("icon", "settings"), string_prop("label", "Open settings")],
        Vec::new(),
        false,
    )
    .expect("icon button");
    match icon_button {
        ViewNode::Button { props, children } => {
            assert!(props.icon_only);
            assert!(props.icon_start.is_some());
            assert_eq!(props.label.as_deref(), Some("Open settings"));
            assert!(children.is_empty());
            assert_eq!(
                props.style.sizing.w.expect("width").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(20))
            );
            assert_eq!(
                props.style.sizing.h.expect("height").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(20))
            );
            let icon = props.icon_start.expect("icon");
            assert_eq!(
                icon.props.style.sizing.w.expect("icon width").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(12))
            );
            assert_eq!(
                icon.props.style.sizing.h.expect("icon height").entries[0].value,
                SizeValue::Scale(ScaleValue::from_half_steps(12))
            );
        }
        _ => panic!("icon button"),
    }

    let input = input_node(vec![
        string_prop("iconStart", "magnifier"),
        string_prop("iconEnd", "close-circle"),
    ])
    .expect("input icons");
    match input {
        ViewNode::Input { props } => {
            assert!(props.icon_start.is_some());
            assert!(props.icon_end.is_some());
        }
        _ => panic!("input"),
    }

    assert!(container_component_node(
        BuiltinComponent::IconButton,
        vec![string_prop("icon", "settings")],
        Vec::new(),
        false,
    )
    .is_err());
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
            assert_eq!(props.variant, None);
            assert_eq!(props.color, None);
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

#[test]
fn validates_country_flag_catalog_and_icon_names() {
    assert_eq!(COUNTRY_FLAGS.len(), 245);
    for country in phone_countries() {
        assert!(country_flag_icon(country.code).is_some(), "missing flag {}", country.code);
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
