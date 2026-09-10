pub(super) fn props_for_component(component: &str) -> Vec<&'static str> {
    let mut props = match component {
        "Box" => BOX_PROPS.to_vec(),
        "Section" => SECTION_PROPS.to_vec(),
        "Flex" => LAYOUT_PROPS.to_vec(),
        "Grid" => GRID_PROPS.to_vec(),
        "Card" => combined_props(&["flex", "color", "border"], VARIANT_PROPS),
        "AppBar" => APP_BAR_PROPS.to_vec(),
        "BottomBar" => FLOATING_BAR_PROPS.to_vec(),
        "Footer" => BAR_PROPS.to_vec(),
        "SideNav" => SIDE_NAV_PROPS.to_vec(),
        "RailNav" => RAIL_NAV_PROPS.to_vec(),
        "Sidebar" => SIDEBAR_PROPS.to_vec(),
        "NavMenu" => NAV_MENU_PROPS.to_vec(),
        "Scaffold" => SCAFFOLD_PROPS.to_vec(),
        "Splash" => vec!["bind"],
        "Tabs" => TABS_PROPS.to_vec(),
        "tab" => TAB_PROPS.to_vec(),
        "Stepper" => STEPPER_PROPS.to_vec(),
        "step" => STEP_PROPS.to_vec(),
        "Drawer" => DRAWER_PROPS.to_vec(),
        "Avatar" => AVATAR_PROPS.to_vec(),
        "Badge" => BADGE_PROPS.to_vec(),
        "Chip" => CHIP_PROPS.to_vec(),
        "Skeleton" => SKELETON_PROPS.to_vec(),
        "Modal" => MODAL_PROPS.to_vec(),
        "AlertDialog" => ALERT_DIALOG_PROPS.to_vec(),
        "Tooltip" => TOOLTIP_PROPS.to_vec(),
        "Toast" => TOAST_PROPS.to_vec(),
        "Dropdown" => DROPDOWN_PROPS.to_vec(),
        "Command" => COMMAND_PROPS.to_vec(),
        "AvatarGroup" => AVATAR_GROUP_PROPS.to_vec(),
        "ChatBox" => CHAT_BOX_PROPS.to_vec(),
        "Empty" => EMPTY_PROPS.to_vec(),
        "Marquee" => MARQUEE_PROPS.to_vec(),
        "TypeWriter" => TYPE_WRITER_PROPS.to_vec(),
        "RichText" => RICH_TEXT_PROPS.to_vec(),
        "mark" => RICH_TEXT_MARK_PROPS.to_vec(),
        "Record" => RECORD_PROPS.to_vec(),
        "ToggleGroup" => TOGGLE_GROUP_PROPS.to_vec(),
        "Collapsible" => COLLAPSIBLE_PROPS.to_vec(),
        "Countdown" => COUNTDOWN_PROPS.to_vec(),
        "Map" => MAP_PROPS.to_vec(),
        "marker" => MAP_MARKER_PROPS.to_vec(),
        "waypoint" => MAP_WAYPOINT_PROPS.to_vec(),
        "RadioGroup" => RADIO_GROUP_PROPS.to_vec(),
        "RadioCard" => RADIO_GROUP_PROPS.to_vec(),
        "item" => ITEM_PROPS.to_vec(),
        "submenu" | "megamenu" => NAV_MENU_ENTRY_PROPS.to_vec(),
        "group" => COMMAND_GROUP_PROPS.to_vec(),
        "ToggleTheme" => TOGGLE_THEME_PROPS.to_vec(),
        "SelectTheme" => SELECT_THEME_PROPS.to_vec(),
        "Fab" => FAB_PROPS.to_vec(),
        "fabAction" => FAB_ACTION_PROPS.to_vec(),
        "Slider" => SLIDER_PROPS.to_vec(),
        "Dropzone" => DROPZONE_PROPS.to_vec(),
        "ComboBox" => COMBO_BOX_PROPS.to_vec(),
        "comboOption" => COMBO_OPTION_PROPS.to_vec(),
        "CsvField" => CSV_FIELD_PROPS.to_vec(),
        "csvColumn" => CSV_COLUMN_PROPS.to_vec(),
        "DragDrop" => DRAG_DROP_PROPS.to_vec(),
        "dragGroup" => DRAG_GROUP_PROPS.to_vec(),
        "dragItem" => DRAG_ITEM_PROPS.to_vec(),
        "Editor" => EDITOR_PROPS.to_vec(),
        "ImageCropper" => IMAGE_CROPPER_PROPS.to_vec(),
        "Password" => PASSWORD_PROPS.to_vec(),
        "Phone" => PHONE_PROPS.to_vec(),
        "Pin" => PIN_PROPS.to_vec(),
        "Textarea" => TEXTAREA_PROPS.to_vec(),
        "Input" => INPUT_PROPS.to_vec(),
        "Select" => SELECT_PROPS.to_vec(),
        "Option" => OPTION_PROPS.to_vec(),
        "Code" => CODE_PROPS.to_vec(),
        "Video" => VIDEO_PROPS.to_vec(),
        "Iframe" => IFRAME_PROPS.to_vec(),
        "Device" => DEVICE_PROPS.to_vec(),
        "Canvas" => CANVAS_PROPS.to_vec(),
        "Draw" => DRAW_PROPS.to_vec(),
        "Candlestick" => CANDLESTICK_PROPS.to_vec(),
        "Diagram" => DIAGRAM_PROPS.to_vec(),
        "ArcChart" => ARC_CHART_PROPS.to_vec(),
        "AreaChart" => AREA_CHART_PROPS.to_vec(),
        "BarChart" => BAR_CHART_PROPS.to_vec(),
        "LineChart" => LINE_CHART_PROPS.to_vec(),
        "PieChart" => PIE_CHART_PROPS.to_vec(),
        "Table" => TABLE_PROPS.to_vec(),
        "Tree" => TREE_PROPS.to_vec(),
        "column" => COLUMN_PROPS.to_vec(),
        "Divider" => DIVIDER_PROPS.to_vec(),
        "Button" => BUTTON_PROPS.to_vec(),
        "Brand" => BRAND_PROPS.to_vec(),
        "Banner" => BANNER_PROPS.to_vec(),
        "IconButton" => ICON_BUTTON_PROPS.to_vec(),
        "Swap" => SWAP_PROPS.to_vec(),
        "Alert" => ALERT_PROPS.to_vec(),
        "Icon" => ICON_PROPS.to_vec(),
        "Svg" => SVG_PROPS.to_vec(),
        "Path" => PATH_PROPS.to_vec(),
        "Title" => [&TEXT_PROPS[..], &["as"][..]].concat(),
        "Text" => TEXT_PROPS.to_vec(),
        "Audio" => combined_props(&["src", "subtitle", "avatarSrc"], VARIANT_PROPS),
        "Camera" => CAMERA_PROPS.to_vec(),
        "Microphone" => MICROPHONE_PROPS.to_vec(),
        "Image" => combined_props(
            &[
                "src",
                "alt",
                "aspect",
                "objectFit",
                "loading",
                "hideControls",
            ],
            VARIANT_PROPS,
        ),
        "Accordion" => combined_props(&["multiple"], VARIANT_PROPS),
        "Carousel" => combined_props(
            &[
                "autoplay",
                "autoplayInterval",
                "disableLoop",
                "hideControls",
                "hideIndicators",
                "showNavigation",
                "showCounter",
                "orientation",
                "size",
                "indicatorType",
                "title",
                "slideWidth",
                "slideHeight",
                "slidesPerView",
                "gap",
            ],
            VARIANT_PROPS,
        ),
        "Checkbox" => combined_props(
            &["checked", "disabled", "name", "helpText", "errorText"],
            VARIANT_PROPS,
        ),
        "validate" => vec!["rule", "message"],
        "Color" => combined_props(
            &[
                "value",
                "size",
                "name",
                "helpText",
                "errorText",
                "showHex",
                "showRgb",
                "showCmyk",
                "showOklch",
            ],
            VARIANT_PROPS,
        ),
        "Date" => combined_props(
            &[
                "value",
                "size",
                "name",
                "helpText",
                "errorText",
                "min",
                "max",
            ],
            VARIANT_PROPS,
        ),
        "DateRange" => combined_props(
            &[
                "start",
                "end",
                "startValue",
                "endValue",
                "size",
                "name",
                "helpText",
                "errorText",
                "min",
                "max",
            ],
            VARIANT_PROPS,
        ),
        "Toggle" => combined_props(
            &["checked", "disabled", "name", "labelLeft", "labelRight"],
            VARIANT_PROPS,
        ),
        _ => Vec::new(),
    };
    if BuiltinComponent::from_name(component).is_some_and(|component| {
        !matches!(
            component,
            BuiltinComponent::Option
                | BuiltinComponent::FabAction
                | BuiltinComponent::ComboOption
                | BuiltinComponent::CsvColumn
                | BuiltinComponent::DragGroup
                | BuiltinComponent::DragItem
                | BuiltinComponent::Svg
                | BuiltinComponent::Path
        )
    }) {
        for &prop in INTERACTIVE_STYLE_PROPS {
            if !props.contains(&prop) {
                props.push(prop);
            }
        }
    }
    props
}

fn combined_props(
    specific: &'static [&'static str],
    common: &'static [&'static str],
) -> Vec<&'static str> {
    specific.iter().chain(common).copied().collect()
}

fn import_completions(root: &Path, from: &Path) -> Vec<LanguageCompletion> {
    let source_root = normalize_path(root.to_path_buf());
    let from = normalize_path(from.to_path_buf());
    let files = dowe_files(&source_root);
    files
        .into_iter()
        .filter(|path| normalize_path(path.to_path_buf()) != from)
        .filter(|path| importable_project_path(&source_root, path))
        .filter_map(|path| project_root_import_path(&source_root, &path))
        .map(|label| completion(&label, LanguageCompletionKind::File, "Dowe source"))
        .collect()
}

pub(super) fn dowe_files(path: &Path) -> Vec<PathBuf> {
    let mut output = Vec::new();
    let Ok(entries) = fs::read_dir(path) else {
        return output;
    };
    for entry in entries.flatten() {
        if entry
            .file_type()
            .is_ok_and(|file_type| file_type.is_symlink())
        {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if name.starts_with('.') || name == "target" {
                continue;
            }
            output.extend(dowe_files(&path));
        } else if path.extension().and_then(|value| value.to_str()) == Some("dowe") {
            output.push(path);
        }
    }
    output.sort();
    output
}

pub(super) fn importable_project_path(project_root: &Path, target: &Path) -> bool {
    let normalized = normalize_path(target.to_path_buf());
    let Ok(relative) = normalized.strip_prefix(project_root) else {
        return false;
    };
    !matches!(
        relative.to_string_lossy().as_ref(),
        "config.dowe" | "theme.dowe" | "main.dowe" | "views.dowe"
    )
}

pub(super) fn project_root_import_path(project_root: &Path, target: &Path) -> Option<String> {
    let normalized = normalize_path(target.to_path_buf());
    let mut path = normalized.strip_prefix(project_root).ok()?.to_path_buf();
    path.set_extension("");
    let value = path.to_string_lossy().replace('\\', "/");
    Some(format!("@/{value}"))
}

fn completion(label: &str, kind: LanguageCompletionKind, detail: &str) -> LanguageCompletion {
    LanguageCompletion {
        label: label.to_string(),
        kind,
        detail: Some(detail.to_string()),
        documentation: None,
    }
}

fn documented_completion(
    label: &str,
    kind: LanguageCompletionKind,
    detail: &str,
    documentation: Option<String>,
) -> LanguageCompletion {
    LanguageCompletion {
        label: label.to_string(),
        kind,
        detail: Some(detail.to_string()),
        documentation,
    }
}
