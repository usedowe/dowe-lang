#[test]
fn generates_compose_and_dev_display_overlay_components() {
    let output = generate_android(
        &[display_overlay_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweAvatar("));
    assert!(
        views
            .content
            .contains("DoweAvatar(source = null, name = \"Ada\"")
    );
    assert!(
        views
            .content
            .contains("DoweBadge(text = \"3\", position = \"bottom-right\"")
    );
    assert!(
        views
            .content
            .contains("DoweChip(text = \"Filter\", size = \"sm\"")
    );
    assert!(
        views
            .content
            .contains("DoweSkeleton(variant = \"rounded\", animation = \"pulse\"")
    );
    assert!(
        views
            .content
            .contains("DoweModal(open = state.bool(\"modal01\")")
    );
    assert!(
        views
            .content
            .contains("DoweAlertDialog(open = state.bool(\"modal01\")")
    );
    assert!(
        views
            .content
            .contains("DoweTooltip(label = \"More actions\", position = \"end\"")
    );
    assert!(
        views
            .content
            .contains("DoweToast(visible = true, title = \"Saved\"")
    );
    assert!(
        views
            .content
            .contains("DoweDropdown(backgroundColor = DoweDesign.surface")
    );
    assert!(
        views
            .content
            .contains("DoweCommand(open = state.bool(\"modal01\")")
    );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("doweText(\"Search\""));
    assert!(dev.content.contains("doweText(\"Profile\""));
    assert!(dev.content.contains("if (doweBool(\"modal01\"))"));
}

#[test]
fn generates_compose_and_dev_display_chat_and_motion_components() {
    let output = generate_android(
        &[display_chat_motion_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweAvatarGroup("));
    assert!(
        views
            .content
            .contains("DoweAvatarGroup(items = doweAvatarGroupItems(state.rows(\"people\")")
    );
    assert!(
        views
            .content
            .contains("DoweChatBox(state = state, messagesPath = \"messages\"")
    );
    assert!(views.content.contains("DoweEmpty(kind = \"result\""));
    assert!(views.content.contains("DoweMarquee(speed = \"fast\""));
    assert!(
        views
            .content
            .contains("DoweTypeWriter(texts = listOf(\"Hello\", \"World\")")
    );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("doweText(\"Chat\""));
    assert!(dev.content.contains("doweText(\"Nothing found\""));
    assert!(dev.content.contains("doweText(\"Hello World\""));
}

#[test]
fn generates_compose_and_dev_rich_control_map_components() {
    let output = generate_android(
        &[rich_control_map_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweRichText("));
    assert!(
        views
            .content
            .contains("DoweRichText(marks = listOf(DoweRichTextMark(text = \"Launch\"")
    );
    assert!(views.content.contains("DoweRecord(name = \"voice\""));
    assert!(
        views
            .content
            .contains("DoweToggleGroup(value = state.text(\"mode\")")
    );
    assert!(
        views
            .content
            .contains("DoweCollapsible(label = \"Details\"")
    );
    assert!(
        views
            .content
            .contains("DoweCountdown(target = \"2030-01-01T00:00:00Z\"")
    );
    assert!(
        views
            .content
            .contains("DoweMap(centerLat = \"4.7109\", centerLng = \"-74.0721\"")
    );
    assert!(views.content.contains("DoweMapMarker(id = \"office\""));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("doweText(\"Launch ready\""));
    assert!(dev.content.contains("doweText(\"voice\""));
    assert!(dev.content.contains("doweText(\"Details\""));
    assert!(dev.content.contains("doweText(\"Office\""));
}

#[test]
fn generates_portable_grid_controls_and_variant_colors() {
    let output = generate_android(
        &[parity_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("DoweGrid(modifier ="));
    assert!(
        views
            .content
            .contains("columns = doweResponsive(viewportWidth, xs = 1, md = 2) ?: 1")
    );
    assert!(
        views
            .content
            .contains("horizontalGap = doweResponsive(viewportWidth, xs = 16.dp) ?: 0.dp")
    );
    assert!(views.content.contains("DoweInput("));
    assert!(views.content.contains("modifier = Modifier.weight(1f)"));
    assert!(views.content.contains("minHeight = 40.dp"));
    assert!(views.content.contains("horizontalPadding = 12.dp"));
    assert!(
        views
            .content
            .contains("contentColor = DoweDesign.secondary")
    );
    assert!(views.content.contains("borderColor = DoweDesign.muted"));
    assert!(
        views
            .content
            .contains("contentColor = DoweDesign.onSoftMuted")
    );
    assert!(views.content.contains(
            "CardDefaults.cardColors(containerColor = DoweDesign.surface, contentColor = DoweDesign.onSurface), border = BorderStroke(1.dp, DoweDesign.surface)"
        ));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains("DoweGridLayout"));
    assert!(dev.content.contains(
            "doweGrid(doweResponsiveInt(viewportWidth, 1, null, 2, null, null), doweResponsiveInt(viewportWidth, 16, null, null, null, null), doweResponsiveInt(viewportWidth, 16, null, null, null, null))"
        ));
    assert!(dev.content.contains("setIncludeFontPadding(false)"));
    assert!(dev.content.contains("setMinHeight(doweDp(40))"));
    assert!(
        dev.content
            .contains("setPadding(doweDp(12), 0, doweDp(12), 0)")
    );
    assert!(
        dev.content
            .contains("background.setCornerRadius(doweDp(radius));")
    );
    assert!(dev.content.contains("private float doweDp(float value)"));
    assert!(dev.content.contains(
        "setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f))"
    ));
    assert!(dev.content.contains("doweCard(DOWE_SOFT_MUTED, null)"));
    assert!(dev.content.contains("doweCard(DOWE_SURFACE, DOWE_SURFACE)"));
    assert!(
        dev.content
            .contains("doweText(\"Surface\", DOWE_ON_SURFACE")
    );
}

#[test]
fn generates_labeled_input_and_select_fields() {
    let output = generate_android(
        &[form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweInput("));
    assert!(
        views
            .content
            .contains(r#"label = "Email", placeholder = "Email address", floating = false"#)
    );
    assert!(
        views
            .content
            .contains(r#"label = "Name", placeholder = "Full name", floating = true"#)
    );
    assert!(views.content.contains("private fun DoweSelect("));
    assert!(views.content.contains("private fun DoweSelectPopover("));
    assert!(views.content.contains("popupMounted"));
    assert!(
        views
            .content
            .contains("targetValue = if (visible) 1f else 0f")
    );
    assert!(views.content.contains("Popup("));
    assert!(!views.content.contains("DropdownMenu("));
    assert!(!views.content.contains("DropdownMenuItem("));
    assert!(
        views.content.contains(
            r#"label = "Department", placeholder = "Choose department", floating = false"#
        )
    );
    assert!(
        views
            .content
            .contains(r#"label = "Role", placeholder = "Choose role", floating = true"#)
    );
    assert!(views.content.contains(
        r#"DoweSelectOption(value = "admin", label = "Admin", description = "Manages users")"#
    ));
    assert!(views.content.contains("private val doweSelectArrowPaths"));
    assert!(
        views
            .content
            .contains("DoweSvg(viewBox = doweSelectArrowViewBox")
    );
    assert!(
        views
            .content
            .contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4")
    );
    assert!(
        views
            .content
            .contains("val active = expanded || selected != null")
    );
    assert!(
        views
            .content
            .contains("if (selected != null || !floating || expanded)")
    );
    assert!(views.content.contains("Text(text = option.description"));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(
        dev.content
            .contains(r#"doweControlLabel("Email", DOWE_PRIMARY"#)
    );
    assert!(dev.content.contains(r#".setHint("Email address")"#));
    assert!(dev.content.contains("doweFloatingInput("));
    assert!(dev.content.contains(r#""Name", "Full name", DOWE_PRIMARY"#));
    assert!(dev.content.contains("doweUpdateFloatingInputLabel"));
    assert!(
        dev.content
            .contains(r#"doweControlLabel("Department", DOWE_PRIMARY"#)
    );
    assert!(dev.content.contains("doweFloatingSelect("));
    assert!(dev.content.contains("doweUpdateFloatingSelectLabel"));
    assert!(dev.content.contains("expanded || hasSelection"));
    assert!(
        dev.content
            .contains("label.setTextSize(active ? 12f : baseSize);")
    );
    assert!(dev.content.contains("input.setPadding(input.getPaddingLeft(), active ? doweDp(10) : 0, input.getPaddingRight(), input.getPaddingBottom());"));
    assert!(dev.content.contains("doweSelectFrame("));
    assert!(dev.content.contains("doweSelectPopup("));
    assert!(dev.content.contains("PopupWindow popup = new PopupWindow"));
    assert!(dev.content.contains("doweSelectArrow("));
    assert!(
        dev.content
            .contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4")
    );
    assert!(!dev.content.contains("Spinner view"));
    assert!(!dev.content.contains("import android.widget.Spinner;"));
    assert!(dev.content.contains(r#"new String[]{"Admin"}"#));
    assert!(dev.content.contains(r#"new String[]{"Manages users"}"#));
    assert!(!dev.content.contains(r#".setPrompt("Role")"#));
}

#[test]
fn generates_compose_and_dev_media_display_form_components() {
    let output = generate_android(
        &[media_display_form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");

    assert!(views.content.contains("private fun DoweAudio("));
    assert!(views.content.contains("DoweAudio(source ="));
    assert!(views.content.contains("private fun DoweImage("));
    assert!(views.content.contains("DoweAccordion("));
    assert!(views.content.contains("DoweCarousel("));
    assert!(views.content.contains("DoweCheckbox("));
    assert!(views.content.contains("DoweColorField("));
    assert!(views.content.contains("DoweDateField("));
    assert!(views.content.contains("DoweDateRangeField("));
    assert!(views.content.contains("DoweRadioGroup("));
    assert!(views.content.contains("DoweToggle("));
    assert!(views.content.contains("CheckboxDefaults.colors"));
    assert!(views.content.contains("DoweInput(value = value"));
    assert!(
        views
            .content
            .contains("doweHexColor(value, backgroundColor)")
    );
    assert!(views.content.contains("BasicTextField("));
    assert!(views.content.contains("RadioButtonDefaults.colors"));
    assert!(views.content.contains("SwitchDefaults.colors"));
    assert!(dev.content.contains("android.widget.CheckBox"));
    assert!(dev.content.contains("Color.parseColor("));
    assert!(dev.content.contains("doweControlLabel(\"Theme\""));
    assert!(dev.content.contains("doweControlLabel(\"Ship date\""));
    assert!(dev.content.contains("android.widget.RadioGroup"));
    assert!(dev.content.contains("android.widget.Switch"));
    assert!(dev.content.contains("doweText(\"Off\""));
    assert!(dev.content.contains("doweText(\"On\""));
}

#[test]
fn generates_compose_advanced_form_components() {
    let output = generate_android(
        &[advanced_form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweComboBox("));
    assert!(views.content.contains("DoweComboBox(value = state.text(\"profile.role\")"));
    assert!(
        views
            .content
            .contains("DoweSelectOption(\"admin\", \"Admin\", \"Full access\")")
    );
    assert!(views.content.contains("private data class DoweCsvColumn"));
    assert!(views.content.contains("DoweCsvField(label = \"Import\""));
    assert!(views.content.contains("DoweCsvColumn(\"email\", \"Email\")"));
    assert!(views.content.contains("private data class DoweDragGroup"));
    assert!(views.content.contains("DoweDragDrop(label = \"Tasks\""));
    assert!(views.content.contains("DoweDragItem(\"draft\", \"Draft\", \"Prepare\", false)"));
    assert!(views.content.contains("DoweEditorField(value = state.text(\"profile.notes\")"));
    assert!(views.content.contains("DoweImageCropper(value = state.text(\"profile.avatar\")"));
    assert!(views.content.contains("DowePasswordField(value = state.text(\"profile.password\")"));
    assert!(views.content.contains("DowePhoneField(value = state.text(\"profile.phone\")"));
    assert!(views.content.contains("DowePinField(value = state.text(\"profile.pin\")"));
    assert!(views.content.contains("DoweTextarea(value = state.text(\"profile.bio\")"));
}

#[test]
fn generates_svg_compose_and_dev_views() {
    let output = generate_android(
        &[svg_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweSvg("));
    assert!(views.content.contains("DoweSvgViewBox(0f, 0f, 24f, 24f)"));
    assert!(views.content.contains("DoweSvgFill.CurrentColor"));
    assert!(views.content.contains(
        "doweResponsive(viewportWidth, xs = DoweDesign.tertiary) ?: LocalContentColor.current"
    ));
    assert!(
        views
            .content
            .contains("PathParser().parsePathString(entry.data).toPath()")
    );

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(
        dev.content
            .contains("private static final class DoweSvgView extends View")
    );
    assert!(
        dev.content
            .contains("private static final class DoweSvgPathParser")
    );
    assert!(
        dev.content
            .contains("Path path = DoweSvgPathParser.parse(entry.data)")
    );
    assert!(dev.content.contains(
        "Integer fill = entry.currentColor ? Integer.valueOf(currentColor) : entry.color;"
    ));
    assert!(!dev.content.contains("import android.graphics.PathParser;"));
    assert!(dev.content.contains("new DoweSvgView(this, 0f, 0f, 24f, 24f, doweColor(doweResponsiveInt(viewportWidth, DOWE_TERTIARY, null, null, null, null), DOWE_ON_BACKGROUND)"));
}

fn advanced_form_route() -> ViewRoute {
    ViewRoute {
        id: "advanced".to_string(),
        route_path: "/advanced".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: advanced_form_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn advanced_form_tree() -> ViewNode {
    ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::ComboBox {
                props: ComboBoxProps {
                    style: bound_style("profile.role", "Role", "Choose role"),
                    value: Some("editor".to_string()),
                    search_placeholder: "Search roles".to_string(),
                    empty_text: "No roles".to_string(),
                    loading_text: "Loading".to_string(),
                    loading_more_text: "Loading more".to_string(),
                    clearable: true,
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
                options: vec![ComboOption {
                    value: "admin".to_string(),
                    label: "Admin".to_string(),
                    description: Some("Full access".to_string()),
                    src: None,
                    icon: None,
                    disabled: false,
                }],
            },
            ViewNode::CsvField {
                props: CsvFieldProps {
                    style: advanced_style("Import", None, ComponentVariant::Outlined),
                    button_text: "Upload CSV".to_string(),
                    modal_title: "Review import".to_string(),
                    instructions: "Columns are checked".to_string(),
                    cancel_text: "Cancel".to_string(),
                    confirm_text: "Import".to_string(),
                    clear_text: "Clear".to_string(),
                    preview_title: "Preview".to_string(),
                    multiple: false,
                    show_preview: true,
                    preview_rows: 3,
                    preview_page_size: 10,
                    error_text: None,
                },
                columns: vec![CsvColumn {
                    name: "email".to_string(),
                    label: Some("Email".to_string()),
                }],
            },
            ViewNode::DragDrop {
                props: DragDropProps {
                    style: advanced_style("Tasks", None, ComponentVariant::Soft),
                    empty_text: "No tasks".to_string(),
                    direction: DragDropDirection::Horizontal,
                    allow_group_transfer: true,
                    disabled: false,
                    size: ButtonSize::Md,
                },
                items: Vec::new(),
                groups: vec![DragGroup {
                    id: "todo".to_string(),
                    title: Some("Todo".to_string()),
                    items: vec![DragItem {
                        id: "draft".to_string(),
                        label: Some("Draft".to_string()),
                        description: Some("Prepare".to_string()),
                        disabled: false,
                    }],
                }],
            },
            ViewNode::Editor {
                props: EditorProps {
                    style: bound_style("profile.notes", "Notes", "Write notes"),
                    value: None,
                    min_height: 180,
                    hide_toolbar: false,
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::ImageCropper {
                props: ImageCropperProps {
                    style: bound_style("profile.avatar", "Avatar", "Upload avatar"),
                    src: None,
                    alt: "Avatar".to_string(),
                    accept: "image/*".to_string(),
                    aspect_ratio: None,
                    min_width: 128,
                    min_height: 128,
                    max_width: None,
                    max_height: None,
                    shape: ImageCropperShape::Circle,
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::PasswordField {
                props: PasswordFieldProps {
                    style: bound_style("profile.password", "Password", "Create password"),
                    value: None,
                    hide_strength: false,
                    weak_label: "Weak".to_string(),
                    medium_label: "Medium".to_string(),
                    strong_label: "Strong".to_string(),
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::PhoneField {
                props: PhoneFieldProps {
                    style: bound_style("profile.phone", "Phone", "Phone number"),
                    value: None,
                    country: Some("US".to_string()),
                    dial_code_name: "dialCode".to_string(),
                    search_placeholder: "Search countries".to_string(),
                    empty_text: "No countries".to_string(),
                    loading_text: "Loading".to_string(),
                    priority_countries: vec!["US".to_string()],
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::PinField {
                props: PinFieldProps {
                    style: bound_style("profile.pin", "Code", ""),
                    value: None,
                    length: 6,
                    kind: PinFieldKind::Number,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::Textarea {
                props: TextareaProps {
                    style: bound_style("profile.bio", "Bio", "Short bio"),
                    value: None,
                    rows: 4,
                    cols: None,
                    max_length: Some(160),
                    resize: true,
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
        ],
    }
}

fn bound_style(bind: &str, label: &str, placeholder: &str) -> VariantProps {
    let mut style = advanced_style(label, Some(placeholder), ComponentVariant::Outlined);
    style.element.bind = Some(bind.to_string());
    style.label_floating = true;
    style
}

fn advanced_style(
    label: &str,
    placeholder: Option<&str>,
    variant: ComponentVariant,
) -> VariantProps {
    VariantProps {
        label: Some(label.to_string()),
        placeholder: placeholder.map(str::to_string),
        variant: Some(variant),
        color: Some(ColorFamily::Primary),
        ..Default::default()
    }
}

#[test]
fn generates_android_view_motion() {
    let output = generate_android(
        &[motion_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(
        views
            .content
            .contains("private enum class DoweAnimationPreset")
    );
    assert!(
        views
            .content
            .contains(".doweAnimation(DoweAnimationPreset.FadeIn)")
    );
    assert!(
        views
            .content
            .contains(".doweAnimation(DoweAnimationPreset.SlideUp)")
    );
    assert!(views.content.contains("animateFloatAsState("));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(dev.content.contains(r#"doweAnimate(view0, "fadeIn");"#));
    assert!(dev.content.contains(r#"doweAnimate(view1, "slideUp");"#));
}
