#[test]
fn completions_include_current_view_component_props() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Box \n  Section \n  Text \n  Card \n  Button \n  Input \n  Alert \n  Svg \n  Path \n  Select \n    Option \n  Video \n  Divider \n  Tabs \n  Drawer \n"
            .to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Text"));
    assert!(base.iter().any(|item| item.label == "Section"));
    assert!(base.iter().any(|item| item.label == "Svg"));
    assert!(base.iter().any(|item| item.label == "Select"));
    assert!(base.iter().any(|item| item.label == "AppBar"));
    assert!(base.iter().any(|item| item.label == "Footer"));
    assert!(base.iter().any(|item| item.label == "BottomBar"));
    assert!(base.iter().any(|item| item.label == "SideNav"));
    assert!(base.iter().any(|item| item.label == "RailNav"));
    assert!(base.iter().any(|item| item.label == "Tabs"));
    assert!(base.iter().any(|item| item.label == "Video"));
    assert!(base.iter().any(|item| item.label == "Divider"));
    assert!(base.iter().any(|item| item.label == "Drawer"));
    assert!(base.iter().any(|item| item.label == "ToggleTheme"));
    assert!(base.iter().any(|item| item.label == "SelectTheme"));
    assert!(base.iter().any(|item| item.label == "Fab"));
    assert!(base.iter().any(|item| item.label == "fabAction"));
    assert!(base.iter().any(|item| item.label == "Slider"));
    assert!(base.iter().any(|item| item.label == "Dropzone"));
    assert!(!base.iter().any(|item| item.label == "Body"));

    let box_props = complete_document(Path::new("/project"), &document, 2, 7);
    assert!(box_props.iter().any(|item| item.label == "color"));
    assert!(box_props.iter().any(|item| item.label == "animation"));
    assert!(box_props.iter().any(|item| item.label == "position"));
    assert!(box_props.iter().any(|item| item.label == "top"));
    assert!(box_props.iter().any(|item| item.label == "right"));
    assert!(box_props.iter().any(|item| item.label == "bottom"));
    assert!(box_props.iter().any(|item| item.label == "left"));
    assert!(box_props.iter().any(|item| item.label == "maxW"));
    assert!(box_props.iter().any(|item| item.label == "maxH"));
    assert!(box_props.iter().any(|item| item.label == "flex"));
    assert!(!box_props.iter().any(|item| item.label == "text"));

    let section_props = complete_document(Path::new("/project"), &document, 3, 11);
    assert!(section_props.iter().any(|item| item.label == "background"));
    assert!(section_props.iter().any(|item| item.label == "centerX"));
    assert!(section_props.iter().any(|item| item.label == "gap"));
    assert!(section_props.iter().any(|item| item.label == "flex"));
    assert!(section_props.iter().any(|item| item.label == "boxed"));
    assert!(section_props.iter().any(|item| item.label == "cover"));
    assert!(section_props.iter().any(|item| item.label == "color"));

    let text_props = complete_document(Path::new("/project"), &document, 4, 8);
    assert!(text_props.iter().any(|item| item.label == "color"));
    assert!(text_props.iter().any(|item| item.label == "i18n"));
    assert!(!text_props.iter().any(|item| item.label == "text"));

    let card_props = complete_document(Path::new("/project"), &document, 5, 8);
    assert!(card_props.iter().any(|item| item.label == "scheme"));
    assert!(card_props.iter().any(|item| item.label == "color"));
    assert!(card_props.iter().any(|item| item.label == "border"));
    assert!(!card_props.iter().any(|item| item.label == "text"));

    for (line, column) in [(6, 10), (7, 9), (8, 9)] {
        let props = complete_document(Path::new("/project"), &document, line, column);
        assert!(props.iter().any(|item| item.label == "scheme"));
        assert!(!props.iter().any(|item| item.label == "color"));
        assert!(!props.iter().any(|item| item.label == "text"));
    }
    let button_props = complete_document(Path::new("/project"), &document, 6, 10);
    assert!(button_props.iter().any(|item| item.label == "iconStart"));
    assert!(button_props.iter().any(|item| item.label == "iconEnd"));
    assert!(button_props.iter().any(|item| item.label == "loading"));
    assert!(
        !button_props
            .iter()
            .any(|item| item.label == "showIconStart")
    );
    for (line, column) in [(5, 8), (8, 9)] {
        let props = complete_document(Path::new("/project"), &document, line, column);
        assert!(!props.iter().any(|item| item.label == "size"));
    }
    let input_props = complete_document(Path::new("/project"), &document, 7, 9);
    assert!(input_props.iter().any(|item| item.label == "size"));
    assert!(card_props.iter().any(|item| item.label == "animation"));

    let svg_props = complete_document(Path::new("/project"), &document, 9, 7);
    assert!(svg_props.iter().any(|item| item.label == "viewBox"));
    assert!(svg_props.iter().any(|item| item.label == "color"));

    let path_props = complete_document(Path::new("/project"), &document, 10, 8);
    assert!(path_props.iter().any(|item| item.label == "d"));
    assert!(path_props.iter().any(|item| item.label == "fill"));

    let select_props = complete_document(Path::new("/project"), &document, 11, 10);
    assert!(select_props.iter().any(|item| item.label == "label"));
    assert!(select_props.iter().any(|item| item.label == "placeholder"));
    assert!(select_props.iter().any(|item| item.label == "size"));

    let option_props = complete_document(Path::new("/project"), &document, 12, 12);
    assert!(option_props.iter().any(|item| item.label == "value"));
    assert!(option_props.iter().any(|item| item.label == "description"));

    let video_props = complete_document(Path::new("/project"), &document, 13, 10);
    assert!(video_props.iter().any(|item| item.label == "src"));
    assert!(video_props.iter().any(|item| item.label == "poster"));
    assert!(video_props.iter().any(|item| item.label == "aspect"));
    assert!(video_props.iter().any(|item| item.label == "scheme"));

    let divider_props = complete_document(Path::new("/project"), &document, 14, 11);
    assert!(divider_props.iter().any(|item| item.label == "orientation"));
    assert!(divider_props.iter().any(|item| item.label == "scheme"));
    assert!(!divider_props.iter().any(|item| item.label == "variant"));

    let tabs_props = complete_document(Path::new("/project"), &document, 15, 8);
    assert!(tabs_props.iter().any(|item| item.label == "variant"));
    assert!(tabs_props.iter().any(|item| item.label == "scheme"));
    assert!(tabs_props.iter().any(|item| item.label == "position"));
    assert!(!tabs_props.iter().any(|item| item.label == "color"));

    let drawer_props = complete_document(Path::new("/project"), &document, 16, 10);
    assert!(drawer_props.iter().any(|item| item.label == "bind"));
    assert!(drawer_props.iter().any(|item| item.label == "position"));
    assert!(drawer_props.iter().any(|item| item.label == "scheme"));
    assert!(drawer_props.iter().any(|item| item.label == "show"));

    let bar_document = LanguageDocument {
        path: Path::new("/project/pages/bars.dowe").to_path_buf(),
        source: "page barsPage\n  AppBar \n  Footer \n  BottomBar \n  SideNav \n  Sidebar \n  RailNav \n"
            .to_string(),
    };
    let appbar_props = complete_document(Path::new("/project"), &bar_document, 2, 11);
    assert!(appbar_props.iter().any(|item| item.label == "floating"));
    assert!(appbar_props.iter().any(|item| item.label == "dockOnScroll"));
    assert!(appbar_props.iter().any(|item| item.label == "bordered"));

    let footer_props = complete_document(Path::new("/project"), &bar_document, 3, 10);
    assert!(footer_props.iter().any(|item| item.label == "boxed"));
    assert!(!footer_props.iter().any(|item| item.label == "floating"));

    let bottombar_props = complete_document(Path::new("/project"), &bar_document, 4, 14);
    assert!(bottombar_props.iter().any(|item| item.label == "floating"));

    let side_nav_props = complete_document(Path::new("/project"), &bar_document, 5, 12);
    assert!(side_nav_props.iter().any(|item| item.label == "scheme"));
    assert!(side_nav_props.iter().any(|item| item.label == "size"));
    assert!(side_nav_props.iter().any(|item| item.label == "wide"));

    let sidebar_props = complete_document(Path::new("/project"), &bar_document, 6, 12);
    assert!(sidebar_props.iter().any(|item| item.label == "scheme"));
    assert!(sidebar_props.iter().any(|item| item.label == "variant"));
    assert!(!sidebar_props.iter().any(|item| item.label == "size"));
    assert!(!sidebar_props.iter().any(|item| item.label == "wide"));

    let rail_nav_props = complete_document(Path::new("/project"), &bar_document, 7, 12);
    assert!(rail_nav_props.iter().any(|item| item.label == "scheme"));
    assert!(rail_nav_props.iter().any(|item| item.label == "size"));
    assert!(rail_nav_props.iter().any(|item| item.label == "showLabels"));

    let control_document = LanguageDocument {
        path: Path::new("/project/pages/controls.dowe").to_path_buf(),
        source:
            "page controlsPage\n  ToggleTheme \n  Fab \n    fabAction \n  Slider \n  Dropzone \n"
                .to_string(),
    };
    let theme_props = complete_document(Path::new("/project"), &control_document, 2, 15);
    assert!(theme_props.iter().any(|item| item.label == "lightLabel"));
    assert!(theme_props.iter().any(|item| item.label == "darkLabel"));
    assert!(theme_props.iter().any(|item| item.label == "scheme"));

    let fab_props = complete_document(Path::new("/project"), &control_document, 3, 7);
    assert!(fab_props.iter().any(|item| item.label == "position"));
    assert!(fab_props.iter().any(|item| item.label == "offsetX"));
    assert!(fab_props.iter().any(|item| item.label == "icon"));

    let fab_action_props = complete_document(Path::new("/project"), &control_document, 4, 15);
    assert!(fab_action_props.iter().any(|item| item.label == "href"));
    assert!(fab_action_props.iter().any(|item| item.label == "onClick"));
    assert!(
        fab_action_props
            .iter()
            .any(|item| item.label == "externalMode")
    );

    let slider_props = complete_document(Path::new("/project"), &control_document, 5, 10);
    assert!(slider_props.iter().any(|item| item.label == "bind"));
    assert!(slider_props.iter().any(|item| item.label == "hideLabel"));
    assert!(slider_props.iter().any(|item| item.label == "step"));

    let dropzone_props = complete_document(Path::new("/project"), &control_document, 6, 12);
    assert!(dropzone_props.iter().any(|item| item.label == "accept"));
    assert!(dropzone_props.iter().any(|item| item.label == "maxSize"));
    assert!(dropzone_props.iter().any(|item| item.label == "errorText"));
}

#[test]
fn completions_include_container_width_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/widths.dowe").to_path_buf(),
        source: "page widthsPage\n  Box w:\n".to_string(),
    };
    let completions = complete_document(Path::new("/project"), &document, 2, 9);
    for value in [
        "full", "sm", "md", "lg", "xl", "2xl", "3xl", "4xl", "5xl", "6xl", "7xl", "10%", "20%",
        "30%", "40%", "50%", "60%", "70%", "80%", "90%", "100%",
    ] {
        assert!(
            completions
                .iter()
                .any(|item| item.label == format!("\"{value}\"")),
            "missing width value {value}"
        );
    }

    let max_width_document = LanguageDocument {
        path: Path::new("/project/pages/widths.dowe").to_path_buf(),
        source: "page widthsPage\n  Box maxW:\n".to_string(),
    };
    let max_width_completions =
        complete_document(Path::new("/project"), &max_width_document, 2, 12);
    assert!(
        max_width_completions
            .iter()
            .all(|item| item.label != "\"50%\"")
    );
}

#[test]
fn completions_and_diagnostics_support_box_positioning() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("pages");
    let document = LanguageDocument {
        path: root.path().join("pages/positioning.dowe"),
        source: "page positioningPage\n  Box position:\"relative\"\n    Box position:\n"
            .to_string(),
    };
    let values = complete_document(root.path(), &document, 3, "    Box position:".len() + 1);
    for value in ["\"static\"", "\"relative\"", "\"absolute\"", "\"fixed\""] {
        assert!(values.iter().any(|item| item.label == value));
    }

    let valid = LanguageDocument {
        path: root.path().join("pages/positioning-valid.dowe"),
        source: "page positioningPage\n  Box position:\"relative\" minH:64 maxW:{ xs:\"full\" md:64 } maxH:\"vh-16\"\n    Box position:\"absolute\" top:4 right:{ xs:4 md:6 }\n      Text\n        \"Proof\"\n"
            .to_string(),
    };
    assert!(
        analyze_document(root.path(), &valid).is_empty(),
        "valid positioned Box should have no diagnostics"
    );
}

#[test]
fn completions_include_section_boxed_boolean_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/landing.dowe").to_path_buf(),
        source: "page landingPage\n  Section boxed:\n".to_string(),
    };

    let values = complete_document(
        Path::new("/project"),
        &document,
        2,
        "  Section boxed:".len() + 1,
    );
    assert!(values.iter().any(|item| item.label == "true"));
    assert!(values.iter().any(|item| item.label == "false"));
}

#[test]
fn completions_include_section_center_boolean_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/landing.dowe").to_path_buf(),
        source: "page landingPage\n  Section centerX:\n".to_string(),
    };

    let values = complete_document(
        Path::new("/project"),
        &document,
        2,
        "  Section centerX:".len() + 1,
    );
    assert!(values.iter().any(|item| item.label == "true"));
    assert!(values.iter().any(|item| item.label == "false"));
}

#[test]
fn completions_include_section_gap_prop() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/landing.dowe").to_path_buf(),
        source: "page landingPage\n  Section \n".to_string(),
    };

    let values = complete_document(Path::new("/project"), &document, 2, "  Section ".len() + 1);
    assert!(values.iter().any(|item| item.label == "gap"));
}

#[test]
fn completions_and_hover_use_page_for_view_routes() {
    let document = LanguageDocument {
        path: Path::new("/project/routes/view.dowe").to_path_buf(),
        source: "views viewRoutes\n  group path:\"/\" layout:RootLayout\n    route \n".to_string(),
    };

    let route_props = complete_document(Path::new("/project"), &document, 3, 11);

    assert!(route_props.iter().any(|item| item.label == "page"));
    assert!(!route_props.iter().any(|item| item.label == "component"));
    assert!(
        hover_at(
            Path::new("/project"),
            &LanguageDocument {
                path: document.path,
                source: "views viewRoutes\n  route path:\"/\" page:homePage\n".to_string(),
            },
            2,
            19,
        )
        .expect("page hover")
        .contains("route.page")
    );
}

#[test]
fn completions_follow_multiline_property_suite_owner() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/canvas.dowe").to_path_buf(),
        source: "page canvasPage\n  Canvas:\n    scene:gameScene\n    \n    fit:\n".to_string(),
    };

    let props = complete_document(Path::new("/project"), &document, 4, 5);
    assert!(props.iter().any(|item| item.label == "viewWidth"));
    assert!(props.iter().any(|item| item.label == "label"));

    let values = complete_document(Path::new("/project"), &document, 5, 9);
    assert!(values.iter().any(|item| item.label == "\"contain\""));
    assert!(values.iter().any(|item| item.label == "\"cover\""));
}

#[test]
fn completions_include_translation_keys() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("i18n")).expect("i18n");
    fs::write(
        root.path().join("i18n/en.dowe"),
        "translations default:true\n  home\n    hero\n      title \"Dowe builds systems.\"\n",
    )
    .expect("english");
    fs::write(
        root.path().join("i18n/es.dowe"),
        "translations\n  home\n    hero\n      title \"Dowe construye sistemas.\"\n",
    )
    .expect("spanish");
    let document = LanguageDocument {
        path: root.path().join("pages/home.dowe"),
        source: "page homePage\n  Title i18n:\n    Dowe builds systems.\n".to_string(),
    };

    let completions = complete_document(root.path(), &document, 2, 14);

    assert!(
        completions
            .iter()
            .any(|item| item.label == "\"home.hero.title\"")
    );
}

