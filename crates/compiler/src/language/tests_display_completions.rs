#[test]
fn completions_include_stepper_props_and_orientation_values() {
    let root = Path::new("/project");
    let component_document = LanguageDocument {
        path: Path::new("/project/pages/onboarding.dowe").to_path_buf(),
        source: "page onboardingPage\n  Stepper \n    step \n".to_string(),
    };
    let stepper = complete_document(root, &component_document, 2, 11);
    assert!(stepper.iter().any(|item| item.label == "scheme"));
    assert!(stepper.iter().any(|item| item.label == "orientation"));
    let step = complete_document(root, &component_document, 3, 10);
    assert!(step.iter().any(|item| item.label == "id"));
    assert!(step.iter().any(|item| item.label == "label"));

    let value_document = LanguageDocument {
        path: Path::new("/project/pages/onboarding.dowe").to_path_buf(),
        source: "page onboardingPage\n  Stepper orientation:\n".to_string(),
    };
    let values = complete_document(root, &value_document, 2, 23);
    assert!(values.iter().any(|item| item.label == "\"horizontal\""));
    assert!(values.iter().any(|item| item.label == "\"vertical\""));
}

#[test]
fn completions_include_appbar_position_prop_and_values() {
    let props_document = LanguageDocument {
        path: Path::new("/project/layouts/main.dowe").to_path_buf(),
        source: "layout MainLayout\n  AppBar \n".to_string(),
    };
    let props = complete_document(Path::new("/project"), &props_document, 2, 10);
    assert!(props.iter().any(|item| item.label == "position"));

    let values_document = LanguageDocument {
        path: Path::new("/project/layouts/main.dowe").to_path_buf(),
        source: "layout MainLayout\n  AppBar position:\n".to_string(),
    };
    let values = complete_document(Path::new("/project"), &values_document, 2, 20);
    for value in ["\"static\"", "\"sticky\"", "\"fixed\""] {
        assert!(values.iter().any(|item| item.label == value));
    }
}

#[test]
fn completions_include_display_overlay_component_props_and_values() {
    let source = [
        "page overlayPage",
        "  Avatar ",
        "  Avatar status:",
        "  Avatar scheme:",
        "  Badge position:",
        "  Chip variant:",
        "  Chip startIcon:",
        "  Chip endIcon:",
        "  Skeleton variant:",
        "  Skeleton animation:",
        "  Modal scheme:",
        "  AlertDialog variant:",
        "  Tooltip position:",
        "  Toast type:",
        "  Toast variant:",
        "  Dropdown ",
        "  Command ",
        "  Command variant:",
        "  item ",
        "  group ",
    ]
    .join("\n");
    let document = LanguageDocument {
        path: Path::new("/project/pages/overlay.dowe").to_path_buf(),
        source,
    };
    let root = Path::new("/project");

    let base = complete_document(root, &document, 1, 1);
    for label in [
        "Avatar",
        "Badge",
        "Chip",
        "Skeleton",
        "Modal",
        "AlertDialog",
        "Tooltip",
        "Toast",
        "Dropdown",
        "Command",
    ] {
        assert!(base.iter().any(|item| item.label == label));
    }

    let avatar_props = complete_document(root, &document, 2, "  Avatar ".len() + 1);
    assert!(avatar_props.iter().any(|item| item.label == "scheme"));
    assert!(avatar_props.iter().any(|item| item.label == "status"));
    assert!(avatar_props.iter().any(|item| item.label == "onClick"));
    assert!(!avatar_props.iter().any(|item| item.label == "color"));

    let avatar_status = complete_document(root, &document, 3, "  Avatar status:".len() + 1);
    assert!(avatar_status.iter().any(|item| item.label == "\"online\""));
    assert!(avatar_status.iter().any(|item| item.label == "\"away\""));

    let avatar_scheme = complete_document(root, &document, 4, "  Avatar scheme:".len() + 1);
    assert!(avatar_scheme.iter().any(|item| item.label == "\"success\""));
    assert!(avatar_scheme.iter().any(|item| item.label == "\"surface\""));

    let badge_position = complete_document(root, &document, 5, "  Badge position:".len() + 1);
    assert!(
        badge_position
            .iter()
            .any(|item| item.label == "\"bottom-right\"")
    );

    let chip_variant = complete_document(root, &document, 6, "  Chip variant:".len() + 1);
    assert!(chip_variant.iter().any(|item| item.label == "\"outlined\""));
    assert!(chip_variant.iter().any(|item| item.label == "\"ghost\""));

    let chip_start_icon = complete_document(root, &document, 7, "  Chip startIcon:".len() + 1);
    assert!(
        chip_start_icon
            .iter()
            .any(|item| item.label == "\"settings\"")
    );

    let chip_end_icon = complete_document(root, &document, 8, "  Chip endIcon:".len() + 1);
    assert!(
        chip_end_icon
            .iter()
            .any(|item| item.label == "\"magnifier\"")
    );

    let skeleton_variant = complete_document(root, &document, 9, "  Skeleton variant:".len() + 1);
    assert!(
        skeleton_variant
            .iter()
            .any(|item| item.label == "\"circular\"")
    );

    let skeleton_animation =
        complete_document(root, &document, 10, "  Skeleton animation:".len() + 1);
    assert!(
        skeleton_animation
            .iter()
            .any(|item| item.label == "\"none\"")
    );

    let modal_scheme = complete_document(root, &document, 11, "  Modal scheme:".len() + 1);
    assert!(modal_scheme.iter().any(|item| item.label == "\"surface\""));

    let dialog_variant = complete_document(root, &document, 12, "  AlertDialog variant:".len() + 1);
    assert!(dialog_variant.iter().any(|item| item.label == "\"ghost\""));

    let tooltip_position = complete_document(root, &document, 13, "  Tooltip position:".len() + 1);
    assert!(tooltip_position.iter().any(|item| item.label == "\"end\""));

    let toast_type = complete_document(root, &document, 14, "  Toast type:".len() + 1);
    assert!(toast_type.iter().any(|item| item.label == "\"success\""));
    assert!(toast_type.iter().any(|item| item.label == "\"error\""));
    let toast_variant = complete_document(root, &document, 15, "  Toast variant:".len() + 1);
    assert!(
        toast_variant
            .iter()
            .any(|item| item.label == "\"outlined\"")
    );
    assert!(toast_variant.iter().any(|item| item.label == "\"ghost\""));

    let dropdown_props = complete_document(root, &document, 16, "  Dropdown ".len() + 1);
    assert!(dropdown_props.iter().any(|item| item.label == "scheme"));
    assert!(!dropdown_props.iter().any(|item| item.label == "variant"));

    let command_props = complete_document(root, &document, 17, "  Command ".len() + 1);
    assert!(command_props.iter().any(|item| item.label == "shortcut"));
    assert!(command_props.iter().any(|item| item.label == "scheme"));

    let command_variant = complete_document(root, &document, 18, "  Command variant:".len() + 1);
    assert!(command_variant.iter().any(|item| item.label == "\"ghost\""));

    let item_props = complete_document(root, &document, 19, "  item ".len() + 1);
    assert!(item_props.iter().any(|item| item.label == "history"));
    assert!(item_props.iter().any(|item| item.label == "onClick"));

    let group_props = complete_document(root, &document, 20, "  group ".len() + 1);
    assert!(group_props.iter().any(|item| item.label == "label"));
}

#[test]
fn completions_include_rich_control_map_component_props_and_values() {
    let source = [
        "page componentsPage",
        "  RichText ",
        "    mark ",
        "  RichText size:",
        "  RichText title:",
        "  Record ",
        "  Record variant:",
        "  ToggleGroup ",
        "    item ",
        "  ToggleGroup size:",
        "  Collapsible ",
        "  Countdown ",
        "  Countdown size:",
        "  Map ",
        "    marker ",
        "    waypoint ",
        "  Map scheme:",
    ]
    .join("\n");
    let document = LanguageDocument {
        path: Path::new("/project/pages/components.dowe").to_path_buf(),
        source,
    };
    let root = Path::new("/project");

    let base = complete_document(root, &document, 1, 1);
    for label in [
        "RichText",
        "Record",
        "ToggleGroup",
        "Collapsible",
        "Countdown",
        "Map",
    ] {
        assert!(base.iter().any(|item| item.label == label));
    }

    let rich_text_props = complete_document(root, &document, 2, "  RichText ".len() + 1);
    assert!(rich_text_props.iter().any(|item| item.label == "i18n"));
    assert!(rich_text_props.iter().any(|item| item.label == "weight"));
    assert!(rich_text_props.iter().any(|item| item.label == "title"));

    let mark_props = complete_document(root, &document, 3, "    mark ".len() + 1);
    assert!(mark_props.iter().any(|item| item.label == "text"));
    assert!(mark_props.iter().any(|item| item.label == "style"));
    assert!(mark_props.iter().any(|item| item.label == "scheme"));

    let rich_text_size = complete_document(root, &document, 4, "  RichText size:".len() + 1);
    assert!(rich_text_size.iter().any(|item| item.label == "\"xl\""));

    let rich_text_title = complete_document(root, &document, 5, "  RichText title:".len() + 1);
    assert!(rich_text_title.iter().any(|item| item.label == "true"));
    assert!(rich_text_title.iter().any(|item| item.label == "false"));

    let record_props = complete_document(root, &document, 6, "  Record ".len() + 1);
    assert!(record_props.iter().any(|item| item.label == "name"));
    assert!(record_props.iter().any(|item| item.label == "maxDuration"));
    assert!(record_props.iter().any(|item| item.label == "onConfirm"));
    assert!(!record_props.iter().any(|item| item.label == "color"));

    let record_variant = complete_document(root, &document, 7, "  Record variant:".len() + 1);
    assert!(record_variant.iter().any(|item| item.label == "\"solid\""));
    assert!(!record_variant.iter().any(|item| item.label == "\"ghost\""));

    let toggle_props = complete_document(root, &document, 8, "  ToggleGroup ".len() + 1);
    assert!(toggle_props.iter().any(|item| item.label == "bind"));
    assert!(toggle_props.iter().any(|item| item.label == "selected"));
    assert!(toggle_props.iter().any(|item| item.label == "onChange"));

    let item_props = complete_document(root, &document, 9, "    item ".len() + 1);
    assert!(item_props.iter().any(|item| item.label == "id"));
    assert!(item_props.iter().any(|item| item.label == "label"));
    assert!(item_props.iter().any(|item| item.label == "icon"));

    let toggle_size = complete_document(root, &document, 10, "  ToggleGroup size:".len() + 1);
    assert!(toggle_size.iter().any(|item| item.label == "\"sm\""));

    let collapsible_props = complete_document(root, &document, 11, "  Collapsible ".len() + 1);
    assert!(collapsible_props.iter().any(|item| item.label == "label"));
    assert!(
        collapsible_props
            .iter()
            .any(|item| item.label == "defaultOpen")
    );

    let countdown_props = complete_document(root, &document, 12, "  Countdown ".len() + 1);
    assert!(countdown_props.iter().any(|item| item.label == "target"));
    assert!(
        countdown_props
            .iter()
            .any(|item| item.label == "showSeconds")
    );
    assert!(
        countdown_props
            .iter()
            .any(|item| item.label == "onComplete")
    );

    let countdown_size = complete_document(root, &document, 13, "  Countdown size:".len() + 1);
    assert!(countdown_size.iter().any(|item| item.label == "\"xl\""));

    let map_props = complete_document(root, &document, 14, "  Map ".len() + 1);
    assert!(map_props.iter().any(|item| item.label == "centerLat"));
    assert!(
        map_props
            .iter()
            .any(|item| item.label == "showLocationControl")
    );
    assert!(map_props.iter().any(|item| item.label == "onRoute"));

    let marker_props = complete_document(root, &document, 15, "    marker ".len() + 1);
    assert!(marker_props.iter().any(|item| item.label == "lat"));
    assert!(marker_props.iter().any(|item| item.label == "popup"));
    assert!(marker_props.iter().any(|item| item.label == "onClick"));

    let waypoint_props = complete_document(root, &document, 16, "    waypoint ".len() + 1);
    assert!(waypoint_props.iter().any(|item| item.label == "lat"));
    assert!(waypoint_props.iter().any(|item| item.label == "lng"));

    let map_scheme = complete_document(root, &document, 17, "  Map scheme:".len() + 1);
    assert!(map_scheme.iter().any(|item| item.label == "\"primary\""));
    assert!(!map_scheme.iter().any(|item| item.label == "\"surface\""));
}

#[test]
fn completions_include_flex_direction_prop_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/layout.dowe").to_path_buf(),
        source: "page layoutPage\n  Flex \n  Flex direction:\n".to_string(),
    };

    let props = complete_document(Path::new("/project"), &document, 2, "  Flex ".len() + 1);
    assert!(props.iter().any(|item| item.label == "direction"));
    assert!(props.iter().any(|item| item.label == "wrap"));

    let values = complete_document(
        Path::new("/project"),
        &document,
        3,
        "  Flex direction:".len() + 1,
    );
    assert!(values.iter().any(|item| item.label == "\"row\""));
    assert!(values.iter().any(|item| item.label == "\"column\""));
}

#[test]
fn completions_include_flex_item_prop_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/layout.dowe").to_path_buf(),
        source: "page layoutPage\n  Box \n  Section \n  Flex \n  Grid \n  Card \n  Box flex:\n"
            .to_string(),
    };

    for (line, column) in [(2, 7), (3, 11), (4, 8), (5, 8), (6, 8)] {
        let props = complete_document(Path::new("/project"), &document, line, column);
        assert!(props.iter().any(|item| item.label == "flex"));
    }

    let values = complete_document(Path::new("/project"), &document, 7, "  Box flex:".len() + 1);
    assert!(values.iter().any(|item| item.label == "\"initial\""));
    assert!(values.iter().any(|item| item.label == "\"auto\""));
    assert!(values.iter().any(|item| item.label == "\"none\""));
    assert!(values.iter().any(|item| item.label == "1"));
}

