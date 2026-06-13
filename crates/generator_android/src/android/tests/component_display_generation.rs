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
