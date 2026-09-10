#[test]
fn completions_include_quoted_static_component_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Input scheme:\n  Input variant:\n  Path fill:\n  Button navigate:\n  Alert type:\n  AppBar scheme:\n  Text weight:\n  SideNav size:\n  SideNav scheme:\n  Divider orientation:\n  Divider scheme:\n  Drawer position:\n  Drawer scheme:\n  Tabs variant:\n  Tabs position:\n  Tabs scheme:\n  Carousel variant:\n"
            .to_string(),
    };

    let scheme = complete_document(Path::new("/project"), &document, 2, 16);
    assert!(scheme.iter().any(|item| item.label == "\"primary\""));
    assert!(scheme.iter().any(|item| item.label == "\"secondary\""));
    assert!(!scheme.iter().any(|item| item.label == "primary"));
    assert!(!scheme.iter().any(|item| item.label == "\"surface\""));
    assert!(
        scheme
            .iter()
            .all(|item| item.kind == LanguageCompletionKind::Value)
    );

    let variant = complete_document(Path::new("/project"), &document, 3, 17);
    assert!(variant.iter().any(|item| item.label == "\"outlined\""));
    assert!(variant.iter().any(|item| item.label == "\"ghost\""));

    let fill = complete_document(Path::new("/project"), &document, 4, 13);
    assert!(fill.iter().any(|item| item.label == "\"none\""));
    assert!(fill.iter().any(|item| item.label == "\"currentColor\""));
    assert!(fill.iter().any(|item| item.label == "\"accent\""));

    let navigate = complete_document(Path::new("/project"), &document, 5, 19);
    assert!(navigate.iter().any(|item| item.label == "\"push\""));
    assert!(navigate.iter().any(|item| item.label == "\"replace\""));

    let alert_type = complete_document(Path::new("/project"), &document, 6, 14);
    assert!(alert_type.iter().any(|item| item.label == "\"warning\""));

    let appbar_scheme = complete_document(Path::new("/project"), &document, 7, 18);
    assert!(appbar_scheme.iter().any(|item| item.label == "\"surface\""));
    assert!(
        appbar_scheme
            .iter()
            .any(|item| item.label == "\"background\"")
    );

    let text_weight = complete_document(Path::new("/project"), &document, 8, 15);
    assert!(text_weight.iter().any(|item| item.label == "\"thin\""));
    assert!(
        text_weight
            .iter()
            .any(|item| item.label == "\"extralight\"")
    );
    assert!(text_weight.iter().any(|item| item.label == "\"black\""));

    let side_nav_size = complete_document(Path::new("/project"), &document, 9, 16);
    assert!(side_nav_size.iter().any(|item| item.label == "\"sm\""));
    assert!(side_nav_size.iter().any(|item| item.label == "\"md\""));
    assert!(side_nav_size.iter().any(|item| item.label == "\"lg\""));

    let side_nav_scheme = complete_document(Path::new("/project"), &document, 10, 18);
    assert!(side_nav_scheme.iter().any(|item| item.label == "\"muted\""));
    assert!(
        !side_nav_scheme
            .iter()
            .any(|item| item.label == "\"surface\"" || item.label == "\"background\"")
    );

    let divider_orientation = complete_document(Path::new("/project"), &document, 11, 23);
    assert!(
        divider_orientation
            .iter()
            .any(|item| item.label == "\"horizontal\"")
    );
    assert!(
        divider_orientation
            .iter()
            .any(|item| item.label == "\"vertical\"")
    );

    let divider_scheme = complete_document(Path::new("/project"), &document, 12, 18);
    assert!(
        divider_scheme
            .iter()
            .any(|item| item.label == "\"surface\"")
    );

    let drawer_position = complete_document(Path::new("/project"), &document, 13, 19);
    assert!(drawer_position.iter().any(|item| item.label == "\"start\""));
    assert!(
        drawer_position
            .iter()
            .any(|item| item.label == "\"bottom\"")
    );

    let drawer_scheme = complete_document(Path::new("/project"), &document, 14, 17);
    assert!(drawer_scheme.iter().any(|item| item.label == "\"surface\""));

    let tabs_variant = complete_document(Path::new("/project"), &document, 15, 17);
    assert!(tabs_variant.iter().any(|item| item.label == "\"line\""));
    assert!(tabs_variant.iter().any(|item| item.label == "\"pills\""));

    let tabs_position = complete_document(Path::new("/project"), &document, 16, 18);
    assert!(tabs_position.iter().any(|item| item.label == "\"start\""));
    assert!(tabs_position.iter().any(|item| item.label == "\"bottom\""));

    let tabs_scheme = complete_document(Path::new("/project"), &document, 17, 17);
    assert!(tabs_scheme.iter().any(|item| item.label == "\"surface\""));

    let carousel_variant = complete_document(
        Path::new("/project"),
        &document,
        18,
        "  Carousel variant:".len() + 1,
    );
    for value in [
        "simple",
        "snapping",
        "masonry",
        "rtl",
        "sticky",
        "controls",
        "dots",
        "thumbnails",
        "coverFlow",
        "slideshow",
        "stories",
        "smartStack",
        "cardStack",
        "flipbook",
    ] {
        assert!(
            carousel_variant
                .iter()
                .any(|item| item.label == format!("\"{value}\""))
        );
    }

    let control_document = LanguageDocument {
        path: Path::new("/project/pages/controls.dowe").to_path_buf(),
        source: "page controlsPage\n  Fab position:\n  Fab icon:\n  fabAction icon:\n  Slider size:\n  Dropzone scheme:\n  Dropzone variant:\n"
            .to_string(),
    };
    let fab_position = complete_document(Path::new("/project"), &control_document, 2, 16);
    assert!(fab_position.iter().any(|item| item.label == "\"top-left\""));
    assert!(
        fab_position
            .iter()
            .any(|item| item.label == "\"bottom-right\"")
    );

    let fab_icon = complete_document(Path::new("/project"), &control_document, 3, 12);
    assert!(fab_icon.iter().any(|item| item.label == "\"settings\""));
    assert!(fab_icon.iter().any(|item| item.label == "\"moon\""));

    let action_icon = complete_document(Path::new("/project"), &control_document, 4, 18);
    assert!(action_icon.iter().any(|item| item.label == "\"link\""));
    assert!(action_icon.iter().any(|item| item.label == "\"upload\""));

    let slider_size = complete_document(Path::new("/project"), &control_document, 5, 15);
    assert!(slider_size.iter().any(|item| item.label == "\"sm\""));
    assert!(slider_size.iter().any(|item| item.label == "\"lg\""));
    assert!(!slider_size.iter().any(|item| item.label == "\"xl\""));

    let dropzone_scheme = complete_document(Path::new("/project"), &control_document, 6, 19);
    assert!(
        dropzone_scheme
            .iter()
            .any(|item| item.label == "\"surface\"")
    );
    assert!(
        dropzone_scheme
            .iter()
            .any(|item| item.label == "\"background\"")
    );

    let dropzone_variant = complete_document(Path::new("/project"), &control_document, 7, 20);
    assert!(
        dropzone_variant
            .iter()
            .any(|item| item.label == "\"ghost\"")
    );
    assert!(
        dropzone_variant
            .iter()
            .any(|item| item.label == "\"outlined\"")
    );

    let radio_document = LanguageDocument {
        path: Path::new("/project/pages/radio.dowe").to_path_buf(),
        source: "page radioPage\n  RadioGroup \n  RadioGroup orientation:\n".to_string(),
    };
    let radio_props = complete_document(Path::new("/project"), &radio_document, 2, 14);
    assert!(radio_props.iter().any(|item| item.label == "orientation"));
    assert!(radio_props.iter().any(|item| item.label == "scheme"));
    let radio_orientation = complete_document(Path::new("/project"), &radio_document, 3, 26);
    assert!(
        radio_orientation
            .iter()
            .any(|item| item.label == "\"vertical\"")
    );
    assert!(
        radio_orientation
            .iter()
            .any(|item| item.label == "\"horizontal\"")
    );
    let radio_card_document = LanguageDocument {
        path: Path::new("/project/pages/radio-card.dowe").to_path_buf(),
        source: "page radioCardPage\n  RadioCard \n    item \n".to_string(),
    };
    let radio_card_props = complete_document(Path::new("/project"), &radio_card_document, 2, 13);
    assert!(
        radio_card_props
            .iter()
            .any(|item| item.label == "orientation")
    );
    assert!(radio_card_props.iter().any(|item| item.label == "bind"));
    let radio_card_item = complete_document(Path::new("/project"), &radio_card_document, 3, 10);
    assert!(radio_card_item.iter().any(|item| item.label == "title"));
}

