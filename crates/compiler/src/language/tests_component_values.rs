#[test]
fn diagnostics_accept_flex_item_prop_on_grid() {
    let root = tempdir().expect("root");
    fs::create_dir_all(root.path().join("pages")).expect("pages");
    let document = LanguageDocument {
        path: root.path().join("pages/layout.dowe"),
        source: "page layoutPage\n  Grid flex:1\n    Text\n      \"Grid\"\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn completions_include_view_animation_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Box animation:\n  Section animation:\n  Section background:\n  Card animation:\n  Flex animation:\n  Chip transition:\n  Chip gesture:\n  Chip \n"
            .to_string(),
    };

    let box_animation = complete_document(Path::new("/project"), &document, 2, 17);
    assert!(box_animation.iter().any(|item| item.label == "\"fadeIn\""));
    assert!(
        box_animation
            .iter()
            .any(|item| item.label == "\"slideRight\"")
    );

    let section_animation = complete_document(Path::new("/project"), &document, 3, 21);
    assert!(
        section_animation
            .iter()
            .any(|item| item.label == "\"fadeIn\"")
    );

    let section_background = complete_document(Path::new("/project"), &document, 4, 22);
    assert!(
        section_background
            .iter()
            .any(|item| item.label == "\"aurora\"")
    );
    assert!(
        section_background
            .iter()
            .any(|item| item.label == "\"slate\"")
    );

    let card_animation = complete_document(Path::new("/project"), &document, 5, 18);
    assert!(
        card_animation
            .iter()
            .any(|item| item.label == "\"scaleIn\"")
    );

    let flex_animation = complete_document(Path::new("/project"), &document, 6, 18);
    assert!(flex_animation.iter().any(|item| item.label == "\"fadeIn\""));

    let chip_transition = complete_document(Path::new("/project"), &document, 7, 19);
    assert!(
        chip_transition
            .iter()
            .any(|item| item.label == "\"spring\"")
    );

    let chip_gesture = complete_document(Path::new("/project"), &document, 8, 16);
    assert!(chip_gesture.iter().any(|item| item.label == "\"tilt\""));

    let chip_props = complete_document(Path::new("/project"), &document, 9, 8);
    for prop in [
        "animation",
        "rotate",
        "scale",
        "translateX",
        "translateY",
        "transition",
        "gesture",
        "onClick",
    ] {
        assert!(chip_props.iter().any(|item| item.label == prop));
    }
}

#[test]
fn completions_include_code_component_props_and_languages() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Code \n  Code language:\n  Code scheme:\n  Editor \n  Editor language:\n".to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Code"));

    let props = complete_document(Path::new("/project"), &document, 2, 8);
    assert!(props.iter().any(|item| item.label == "content"));
    assert!(props.iter().any(|item| item.label == "language"));
    assert!(props.iter().any(|item| item.label == "copyLabel"));
    assert!(props.iter().any(|item| item.label == "copiedLabel"));

    let languages = complete_document(Path::new("/project"), &document, 3, 17);
    assert!(languages.iter().any(|item| item.label == "\"dowe\""));
    assert!(languages.iter().any(|item| item.label == "\"typescript\""));
    assert!(languages.iter().any(|item| item.label == "\"javascript\""));
    assert!(languages.iter().any(|item| item.label == "\"go\""));
    assert!(languages.iter().any(|item| item.label == "\"rust\""));
    assert!(languages.iter().any(|item| item.label == "\"python\""));

    let editor_props = complete_document(Path::new("/project"), &document, 5, 10);
    assert!(editor_props.iter().any(|item| item.label == "language"));
    assert!(editor_props.iter().any(|item| item.label == "onSave"));

    let editor_languages = complete_document(Path::new("/project"), &document, 6, 19);
    assert!(editor_languages.iter().any(|item| item.label == "\"dowe\""));

    let schemes = complete_document(Path::new("/project"), &document, 4, 15);
    assert!(schemes.iter().any(|item| item.label == "\"surface\""));
    assert!(schemes.iter().any(|item| item.label == "\"danger\""));
}

#[test]
fn completions_include_video_component_props_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Video \n  Video aspect:\n  Video scheme:\n".to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Video"));

    let props = complete_document(Path::new("/project"), &document, 2, 9);
    assert!(props.iter().any(|item| item.label == "src"));
    assert!(props.iter().any(|item| item.label == "poster"));
    assert!(props.iter().any(|item| item.label == "autoplay"));
    assert!(props.iter().any(|item| item.label == "aspect"));
    assert!(props.iter().any(|item| item.label == "scheme"));

    let aspects = complete_document(Path::new("/project"), &document, 3, 16);
    assert!(aspects.iter().any(|item| item.label == "\"horizontal\""));
    assert!(aspects.iter().any(|item| item.label == "\"vertical\""));
    assert!(aspects.iter().any(|item| item.label == "\"square\""));

    let schemes = complete_document(Path::new("/project"), &document, 4, 16);
    assert!(schemes.iter().any(|item| item.label == "\"surface\""));
    assert!(schemes.iter().any(|item| item.label == "\"accent\""));
}

#[test]
fn completions_include_canvas_component_props_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/canvas.dowe").to_path_buf(),
        source: "page canvasPage\n  signal scene value:[]\n  fn capture\n    set scene value:item\n  Canvas \n  Canvas scene:\n  Canvas fit:\n  Canvas background:\n  Canvas onPointer:\n".to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Canvas"));
    let props = complete_document(Path::new("/project"), &document, 5, 10);
    for prop in [
        "scene",
        "viewWidth",
        "viewHeight",
        "fit",
        "fps",
        "autoplay",
        "background",
        "pixelated",
        "label",
        "onPointer",
        "onKey",
        "onMotion",
        "motionRate",
    ] {
        assert!(props.iter().any(|item| item.label == prop), "{prop}");
    }
    let scene = complete_document(Path::new("/project"), &document, 6, 16);
    assert!(scene.iter().any(|item| item.label == "scene"));
    let fits = complete_document(Path::new("/project"), &document, 7, 14);
    assert!(fits.iter().any(|item| item.label == "\"contain\""));
    assert!(fits.iter().any(|item| item.label == "\"cover\""));
    assert!(fits.iter().any(|item| item.label == "\"stretch\""));
    let backgrounds = complete_document(Path::new("/project"), &document, 8, 21);
    assert!(backgrounds.iter().any(|item| item.label == "\"surface\""));
    assert!(
        backgrounds
            .iter()
            .any(|item| item.label == "\"transparent\"")
    );
    let actions = complete_document(Path::new("/project"), &document, 9, 21);
    assert!(actions.iter().any(|item| item.label == "capture"));
}

#[test]
fn completions_include_candlestick_component_props_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/market.dowe").to_path_buf(),
        source: "page marketPage\n  signal candles value:[]\n  Candlestick \n  Candlestick data:\n  Candlestick scheme:\n  Candlestick upColor:\n"
            .to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Candlestick"));

    let props = complete_document(Path::new("/project"), &document, 3, 15);
    assert!(props.iter().any(|item| item.label == "data"));
    assert!(props.iter().any(|item| item.label == "stream"));
    assert!(props.iter().any(|item| item.label == "upColor"));
    assert!(props.iter().any(|item| item.label == "downColor"));
    assert!(props.iter().any(|item| item.label == "emptyLabel"));
    assert!(props.iter().any(|item| item.label == "maxPoints"));

    let data = complete_document(Path::new("/project"), &document, 4, 20);
    assert!(data.iter().any(|item| item.label == "candles"));

    let schemes = complete_document(Path::new("/project"), &document, 5, 22);
    assert!(schemes.iter().any(|item| item.label == "\"surface\""));
    assert!(schemes.iter().any(|item| item.label == "\"accent\""));

    let colors = complete_document(Path::new("/project"), &document, 6, 23);
    assert!(colors.iter().any(|item| item.label == "\"success\""));
    assert!(colors.iter().any(|item| item.label == "\"danger\""));
}

#[test]
fn completions_include_chart_component_props_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/charts.dowe").to_path_buf(),
        source: "page chartPage\n  signal points value:[]\n  LineChart \n  LineChart data:\n  LineChart palette:\n  LineChart legendPosition:\n  LineChart curve:\n  PieChart \n"
            .to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    for label in ["ArcChart", "AreaChart", "BarChart", "LineChart", "PieChart"] {
        assert!(base.iter().any(|item| item.label == label));
    }

    let line_props = complete_document(Path::new("/project"), &document, 3, 13);
    assert!(line_props.iter().any(|item| item.label == "data"));
    assert!(line_props.iter().any(|item| item.label == "series"));
    assert!(line_props.iter().any(|item| item.label == "curve"));
    assert!(line_props.iter().any(|item| item.label == "palette"));

    let data = complete_document(Path::new("/project"), &document, 4, 18);
    assert!(data.iter().any(|item| item.label == "points"));

    let palettes = complete_document(Path::new("/project"), &document, 5, 21);
    assert!(palettes.iter().any(|item| item.label == "\"ocean\""));
    assert!(palettes.iter().any(|item| item.label == "\"forest\""));

    let legends = complete_document(Path::new("/project"), &document, 6, 28);
    assert!(legends.iter().any(|item| item.label == "\"bottom\""));
    assert!(legends.iter().any(|item| item.label == "\"right\""));

    let curves = complete_document(Path::new("/project"), &document, 7, 20);
    assert!(curves.iter().any(|item| item.label == "\"smooth\""));

    let pie_props = complete_document(Path::new("/project"), &document, 8, 12);
    assert!(pie_props.iter().any(|item| item.label == "donut"));
    assert!(pie_props.iter().any(|item| item.label == "donutWidth"));
}

#[test]
fn completions_include_table_component_and_column_props() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/users.dowe").to_path_buf(),
        source: "page usersPage\n  signal users value:[]\n  Table \n    column \n  Table data:\n  Table size:\n  Table scheme:\n    column align:\n"
            .to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Table"));

    let props = complete_document(Path::new("/project"), &document, 3, 9);
    assert!(props.iter().any(|item| item.label == "data"));
    assert!(props.iter().any(|item| item.label == "scheme"));
    assert!(props.iter().any(|item| item.label == "emptyTitle"));
    assert!(props.iter().any(|item| item.label == "dividers"));

    let column_props = complete_document(Path::new("/project"), &document, 4, 12);
    assert!(column_props.iter().any(|item| item.label == "field"));
    assert!(column_props.iter().any(|item| item.label == "label"));
    assert!(column_props.iter().any(|item| item.label == "align"));
    assert!(column_props.iter().any(|item| item.label == "width"));

    let data = complete_document(Path::new("/project"), &document, 5, 14);
    assert!(data.iter().any(|item| item.label == "users"));

    let sizes = complete_document(Path::new("/project"), &document, 6, 14);
    assert!(sizes.iter().any(|item| item.label == "\"lg\""));

    let schemes = complete_document(Path::new("/project"), &document, 7, 16);
    assert!(schemes.iter().any(|item| item.label == "\"surface\""));

    let align = complete_document(Path::new("/project"), &document, 8, 19);
    assert!(align.iter().any(|item| item.label == "\"end\""));
}

#[test]
fn completions_include_tree_component_props_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/tree.dowe").to_path_buf(),
        source: "page treePage\n  signal files value:[]\n  Tree data:\n  Tree defaultOpen:\n  Tree variant:\n  Tree scheme:\n".to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Tree"));

    let props = complete_document(Path::new("/project"), &document, 3, 9);
    assert!(props.iter().any(|item| item.label == "data"));
    assert!(props.iter().any(|item| item.label == "bind"));
    assert!(props.iter().any(|item| item.label == "defaultOpen"));
    assert!(props.iter().any(|item| item.label == "onSelect"));

    let data = complete_document(Path::new("/project"), &document, 3, 14);
    assert!(data.iter().any(|item| item.label == "files"));

    let defaults = complete_document(Path::new("/project"), &document, 4, 20);
    assert!(defaults.iter().any(|item| item.label == "true"));
    assert!(defaults.iter().any(|item| item.label == "false"));

    let variants = complete_document(Path::new("/project"), &document, 5, 17);
    assert!(variants.iter().any(|item| item.label == "\"ghost\""));

    let schemes = complete_document(Path::new("/project"), &document, 6, 16);
    assert!(schemes.iter().any(|item| item.label == "\"surface\""));
}

