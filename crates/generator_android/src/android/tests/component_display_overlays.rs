#[test]
fn generates_native_dropzone_file_picker_hooks() {
    let output = generate_android(
        &[dropzone_picker_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains(
        "rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments())"
    ));
    assert!(views.content.contains("doweDropzoneMimeTypes(accept)"));
    assert!(views.content.contains("maxSize = 4096L"));
    assert!(views.content.contains("contentResolver.query"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("Intent.ACTION_OPEN_DOCUMENT"));
    assert!(dev.content.contains("Intent.EXTRA_ALLOW_MULTIPLE"));
    assert!(
        dev.content
            .contains("public void handleActivityResult(int requestCode")
    );
    assert!(dev.content.contains("doweDropzoneMaxSize"));
    let host = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevHostActivity.java"))
        .expect("dev host");
    assert!(host.content.contains("onActivityResult(int requestCode"));
}

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
            .contains("DoweAvatar(source = \"https://example.com/avatar.png\", name = \"Maya\"")
    );
    assert!(
        views
            .content
            .contains("withContext(Dispatchers.IO) { doweLoadImageBitmap(context, source) }")
    );
    assert!(views.content.contains("contentScale = ContentScale.Crop"));
    assert!(views.content.contains(
        "modifier = Modifier.doweShadow(radius = doweResponsive(viewportWidth, xs = 44.dp) ?: 0.dp, shape = RoundedCornerShape(999.dp), color = DoweDesign.accent, alpha = 0.28f)"
    ));
    assert!(
        views
            .content
            .contains("DoweBadge(text = \"3\", position = \"bottom-right\"")
    );
    assert!(views.content.contains(".doweBadgeCornerOffset(position)"));
    assert!(
        views
            .content
            .contains("private fun Modifier.doweBadgeCornerOffset(position: String)")
    );
    assert!(
        views
            .content
            .contains("DoweChip(text = \"Filter\", size = \"sm\"")
    );
    assert!(views.content.contains("private fun DoweChip(text: String, size: String, backgroundColor: Color, contentColor: Color, borderColor: Color?, modifier: Modifier, compact: Boolean"));
    assert!(
        views
            .content
            .contains("private object DoweGridCompactWidthModifier : ParentDataModifier")
    );
    assert!(views.content.contains(
        "val compactWidth = (measurable.parentData as? DoweGridItemData)?.compactWidth == true"
    ));
    assert!(views.content.contains(
        "Box(modifier = Modifier.doweGridCompactWidth(), contentAlignment = Alignment.CenterStart)"
    ));
    assert!(views.content.contains("surface(modifier)"));
    assert!(
        views
            .content
            .contains("DoweChip(text = \"Filter\", size = \"sm\"")
    );
    assert!(views.content.contains("modifier = Modifier.doweShadow"));
    assert!(views.content.contains("compact = true, onClose ="));
    assert!(views.content.contains("start = {"));
    assert!(views.content.contains("end = {"));
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
    assert!(views.content.contains(
        "backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, borderColor = null, confirmBackgroundColor = DoweDesign.danger, confirmContentColor = DoweDesign.dangerText"
    ));
    assert!(
        views
            .content
            .contains("DoweTooltip(label = \"More actions\", position = \"end\"")
    );
    assert!(views.content.contains("private fun DoweTooltip("));
    assert!(
        views
            .content
            .contains("private fun doweGridVerticalAlignment")
    );
    assert!(views.content.contains("is DoweSize.Percent -> this"));
    assert!(
        views
            .content
            .contains("maxImageWidth: Int?, maxImageHeight: Int?")
    );
    assert!(
        views
            .content
            .contains("DoweToast(visible = true, title = \"Saved\"")
    );
    assert!(views.content.contains(
        "position = \"top-right\", backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, borderColor = DoweDesign.warning"
    ));
    assert!(views.content.contains("paths = doweOverlayClosePaths"));
    assert!(
        views
            .content
            .contains("contentDescription = \"Close toast\"")
    );
    assert!(
        views
            .content
            .contains("DoweDropdown(backgroundColor = DoweDesign.surface")
    );
    assert!(views.content.contains("trigger = {"));
    assert!(views.content.contains("}, content = { close ->"));
    assert!(!views.content.contains("} content: { close ->"));
    let dropdown_runtime_start = views
        .content
        .find("private fun DoweDropdown(")
        .expect("dropdown runtime");
    let dropdown_runtime_end = views.content[dropdown_runtime_start..]
        .find("private fun DoweOverlayItem(")
        .map(|offset| dropdown_runtime_start + offset)
        .expect("overlay item after dropdown runtime");
    let dropdown_runtime = &views.content[dropdown_runtime_start..dropdown_runtime_end];
    assert!(dropdown_runtime.contains("popupMounted"));
    assert!(dropdown_runtime.contains("onGloballyPositioned { triggerHeight = it.size.height }"));
    assert!(dropdown_runtime.contains("DoweAnchoredPopover("));
    assert!(dropdown_runtime.contains("offset = popupOffset"));
    assert!(views.content.contains("private fun DoweAnchoredPopover("));
    assert!(views.content.contains(".heightIn(max = 260.dp)"));
    assert!(
        views
            .content
            .contains(".verticalScroll(rememberScrollState())")
    );
    assert!(
        views
            .content
            .contains("DoweCommand(open = state.bool(\"modal01\")")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains(
        ", doweResponsiveInt(viewportWidth, 44, null, null, null, null), DOWE_ACCENT, 999f, 0.28f);"
    ));
    assert!(
        dev.content
            .contains(".setLayoutParams(new LinearLayout.LayoutParams(doweDp(48), doweDp(48)));")
    );
    assert!(
        dev.content.contains(
            "doweAvatarImage(\"https://example.com/avatar.png\", \"Maya portrait\", \"M\""
        )
    );
    assert!(dev.content.contains("private FrameLayout doweAvatarImage("));
    assert!(
        dev.content
            .contains(".setBackground(doweBackground(DOWE_SUCCESS, 999f));")
    );
    assert!(dev.content.contains("BlurMaskFilter.Blur.NORMAL"));
    assert!(dev.content.contains("doweDrawChildShadows(this, canvas)"));
    assert!(dev.content.contains("FrameLayout.LayoutParams"));
    assert!(dev.content.contains("setTranslationX(v.getWidth() / 2f)"));
    assert!(dev.content.contains("setTranslationY(v.getHeight() / 2f)"));
    assert!(dev.content.contains("doweText(\"Search\""));
    assert!(dev.content.contains("DOWE_COMPACT_WIDTH_TAG = 0x7f0d0016"));
    assert!(
        dev.content
            .contains("setTag(DOWE_COMPACT_WIDTH_TAG, Boolean.TRUE)")
    );
    assert!(dev.content.contains(
        "MeasureSpec.makeMeasureSpec(cellWidth, compactWidth ? MeasureSpec.AT_MOST : MeasureSpec.EXACTLY)"
    ));
    assert!(
        dev.content
            .contains("align == DOWE_ALIGN_STRETCH && !compactWidth")
    );
    assert!(dev.content.contains("doweText(\"Docs\""));
    assert!(dev.content.contains("doweDp(14)"));
    assert!(dev.content.contains("if (doweBool(\"modal01\"))"));
    assert!(dev.content.contains("PopupWindow"));
    assert!(dev.content.contains("ScrollView view"));
    assert!(dev.content.contains(".setHeight(Math.min("));
    assert!(dev.content.contains(".setDuration(160).start();"));
    assert!(
        dev.content
            .contains("new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0);")
    );
    assert!(dev.content.contains("TriggerHeight = view"));
    assert!(dev.content.contains("HitParams.height = view"));
    assert!(dev.content.contains(".requestLayout();"));
    assert!(views.content.contains("content = { close ->"));
    assert!(
        views
            .content
            .contains("onClick = { close(); navigate(\"push\", \"/docs\", null) }")
    );
    assert!(dev.content.contains(".showAsDropDown(view"));
    assert!(dev.content.contains("LinearLayout view"));
    assert!(
        dev.content
            .contains("doweNavigate(\"push\", \"/docs\", null);")
    );
    assert!(dev.content.contains("setText(\"Menu\");"));
    assert!(dev.content.contains("setTextColor(DOWE_PRIMARY_TEXT);"));
    assert!(
        dev.content
            .contains("setBackground(doweInputBackground(DOWE_PRIMARY, null, DOWE_RADIUS));")
    );
    assert!(!dev.content.contains("doweText(\"More actions\""));
}

#[test]
fn generates_android_overlay_surface_action_and_close_parity() {
    let output = generate_android(
        &[overlay_parity_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains(
        "backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, borderColor = DoweDesign.warning"
    ));
    assert!(views.content.contains(
        "backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, borderColor = null"
    ));
    assert!(views.content.contains(
        "confirmBackgroundColor = DoweDesign.warning, confirmContentColor = DoweDesign.warningText"
    ));
    assert!(
        views
            .content
            .contains("private val doweOverlayClosePaths = listOf(")
    );
    assert!(views.content.contains(
        "DoweSvg(viewBox = doweOverlayCloseViewBox, modifier = Modifier.width(18.dp).height(18.dp), color = DoweDesign.mutedText, paths = doweOverlayClosePaths)"
    ));
    assert!(views.content.contains(
        "BoxWithConstraints(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center)"
    ));
    assert!(
        views
            .content
            .contains("val modalMaxWidth = (maxWidth * 0.95f).coerceAtMost(560.dp)")
    );
    assert!(
        views
            .content
            .contains(".width(modalMaxWidth)\n                    .padding(16.dp)")
    );
    assert!(
        !views
            .content
            .contains("val modalMaxWidth = LocalConfiguration.current.screenWidthDp.dp * 0.95f")
    );
    assert!(
        views
            .content
            .contains(".width(28.dp)\n                            .height(28.dp)")
    );
    assert!(
        views
            .content
            .contains("Column(modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp))")
    );
    assert!(views.content.contains(
        "val toastWidth = (viewportWidth - 32.dp).coerceAtLeast(1.dp).coerceAtMost(420.dp)"
    ));
    assert!(views.content.contains(".width(toastWidth)"));

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains(".setBackground(doweInputBackground(DOWE_WARNING, null, DOWE_RADIUS));")
    );
    assert!(
        dev.content
            .contains(".setBackground(doweInputBackground(DOWE_SURFACE, null, DOWE_RADIUS));")
    );
    assert!(dev.content.contains("DOWE_WARNING_TEXT"));
    assert!(
        dev.content
            .contains("setContentDescription(\"Close modal\")")
    );
    assert!(dev.content.contains("setPadding"));
    assert!(dev.content.contains(
        "doweDp(Math.max(1, Math.min(560, Math.min(Math.max(0, viewportWidth - 32), (viewportWidth * 95) / 100))))"
    ));
    assert!(dev.content.contains(
        "new FrameLayout.LayoutParams(doweDp(28), doweDp(28), Gravity.TOP | Gravity.END)"
    ));
    assert!(
        dev.content
            .contains("new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER)")
    );
    assert!(dev.content.contains(
        "new FrameLayout.LayoutParams(doweDp(Math.max(1, Math.min(560, Math.min(Math.max(0, viewportWidth - 32), (viewportWidth * 95) / 100)))), ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER)"
    ));
}

#[test]
fn generates_compose_modal_width_from_overlay_constraints() {
    let output = generate_android(
        &[overlay_parity_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains(
        "BoxWithConstraints(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center)"
    ));
    assert!(
        views
            .content
            .contains("val modalMaxWidth = (maxWidth * 0.95f).coerceAtMost(560.dp)")
    );
    assert!(
        views
            .content
            .contains(".width(modalMaxWidth)\n                    .padding(16.dp)")
    );
    assert!(
        !views
            .content
            .contains("val modalMaxWidth = LocalConfiguration.current.screenWidthDp.dp * 0.95f")
    );
    let dev = dev_java_source(&output);
    let width = "new FrameLayout.LayoutParams(doweDp(Math.max(1, Math.min(560, Math.min(Math.max(0, viewportWidth - 32), (viewportWidth * 95) / 100)))), ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER)";
    assert_eq!(dev.content.matches(width).count(), 2);
    assert!(
        !dev.content.contains(
            "Math.min(Math.max(0, viewportWidth - doweDp(32)), (viewportWidth * 95) / 100)"
        )
    );
}
