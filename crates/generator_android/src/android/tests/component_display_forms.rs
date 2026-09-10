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
            .contains("import androidx.compose.runtime.mutableIntStateOf")
    );
    assert!(
        views
            .content
            .contains("import androidx.compose.runtime.key")
    );
    assert!(
        views
            .content
            .contains("private fun doweGridVerticalAlignment")
    );
    assert!(views.content.contains("DoweSize.Full -> fillMaxWidth()"));
    assert!(views.content.contains("DoweSize.Auto -> this"));
    assert!(views.content.contains(
        "tracks = doweResponsive(viewportWidth, xs = listOf(1f), md = listOf(1f, 1f)) ?: listOf(1f)"
    ));
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
            .contains("contentColor = DoweDesign.mutedText")
    );
    assert!(views.content.contains(
            "CardDefaults.cardColors(containerColor = Color.Transparent, contentColor = DoweDesign.surface), border = BorderStroke(1.dp, DoweDesign.surface)"
        ));
    assert!(
        views
            .content
            .contains("ButtonDefaults.buttonColors(containerColor = Color.Transparent, contentColor = DoweDesign.primary), border = BorderStroke(1.dp, DoweDesign.primary)")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("DoweGridLayout"));
    assert!(dev.content.contains(
            "doweGrid(doweResponsiveTracks(viewportWidth, new float[]{1f}, null, new float[]{1f, 1f}, null, null), doweResponsiveInt(viewportWidth, 16, null, null, null, null), doweResponsiveInt(viewportWidth, 16, null, null, null, null))"
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
    assert!(
        dev.content
            .contains("doweCard(DOWE_MUTED, (\"outlined\".equals(\"solid\") ? null : null))")
    );
    assert!(dev.content.contains(
        "doweCard(Color.TRANSPARENT, (\"outlined\".equals(\"solid\") ? DOWE_SURFACE : null))"
    ));
    assert!(dev.content.contains(
        "setBackground(doweInputBackground(Color.TRANSPARENT, DOWE_PRIMARY, DOWE_RADIUS))"
    ));
    assert!(dev.content.contains("setBackgroundTintList(null)"));
    assert!(dev.content.contains("doweText(\"Surface\", DOWE_SURFACE"));
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
    let small_input = views
        .content
        .lines()
        .find(|line| line.contains(r#"label = "Name", placeholder = "Full name""#))
        .expect("small floating input");
    assert!(small_input.contains("minHeight = 40.dp"), "{small_input}");
    assert!(small_input.contains("fontSize = doweTextSize(viewportWidth, min = 12f, preferredBase = 11.2f, preferredViewport = 0.2f, max = 14f)"), "{small_input}");
    assert!(views.content.contains("startIcon = { DoweSvg("));
    assert!(views.content.contains("endIcon = { DoweSvg("));
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
    let large_select = views
        .content
        .lines()
        .find(|line| line.contains(r#"label = "Role", placeholder = "Choose role""#))
        .expect("large floating select");
    assert!(large_select.contains("minHeight = 56.dp"), "{large_select}");
    assert!(large_select.contains("fontSize = doweTextSize(viewportWidth, min = 16f, preferredBase = 15.2f, preferredViewport = 0.3f, max = 18f)"), "{large_select}");
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
    assert!(views.content.contains(".heightIn(max = 260.dp)"));
    assert!(
        views
            .content
            .contains(".verticalScroll(rememberScrollState())")
    );

    let dev = dev_java_source(&output);
    if !dev.content.contains(r#"doweControlLabel("Email""#) {
        let i = dev.content.find("Email").unwrap_or(0);
        panic!("LBL: {}", &dev.content[i.saturating_sub(200)..i + 200]);
    }
    assert!(dev.content.contains(r#".setHint("Email address")"#));
    assert!(dev.content.contains("doweFloatingInput("));
    assert!(
        dev.content
            .contains(r#""Name", "Full name", DOWE_BACKGROUND_TEXT"#)
    );
    let dev_floating_input = dev
        .content
        .find("= doweFloatingInput(")
        .map(|start| &dev.content[start..(start + 320).min(dev.content.len())])
        .expect("dev floating input");
    assert!(dev_floating_input.contains("setMinimumHeight(doweDp(40))"));
    assert!(dev.content.contains("doweUpdateFloatingInputLabel"));
    assert!(
        dev.content
            .contains(r#"doweControlLabel("Department", DOWE_BACKGROUND_TEXT"#)
    );
    assert!(dev.content.contains("doweFloatingSelect("));
    let dev_floating_select = dev
        .content
        .find("= doweFloatingSelect(")
        .map(|start| &dev.content[start..(start + 320).min(dev.content.len())])
        .expect("dev floating select");
    assert!(dev_floating_select.contains("setMinimumHeight(doweDp(56))"));
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
    assert!(
        dev.content
            .contains("Math.min(content.getMeasuredHeight(), doweDp(260))")
    );
    assert!(
        dev.content
            .contains("ScrollView optionsScroll = new ScrollView(this)")
    );
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

