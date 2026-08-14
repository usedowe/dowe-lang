fn dropzone_picker_route() -> ViewRoute {
    ViewRoute {
        id: "dropzone-picker".to_string(),
        route_path: "/dropzone-picker".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Dropzone {
            props: DropzoneProps {
                style: VariantProps {
                    label: Some("Assets".to_string()),
                    placeholder: Some("Choose files".to_string()),
                    variant: Some(ComponentVariant::Outlined),
                    color: Some(ColorFamily::Primary),
                    ..Default::default()
                },
                accept: Some("image/*".to_string()),
                multiple: true,
                max_size: Some(4096),
                size: ButtonSize::Md,
                name: Some("assets".to_string()),
                help_text: Some("Images only".to_string()),
                error_text: None,
                disabled: false,
            },
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn overlay_parity_route() -> ViewRoute {
    ViewRoute {
        id: "overlay-parity".to_string(),
        route_path: "/overlay-parity".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps::default(),
            children: vec![
                ViewNode::Modal {
                    props: ModalProps {
                        style: VariantProps {
                            variant: Some(ComponentVariant::Outlined),
                            color: Some(ColorFamily::Warning),
                            ..Default::default()
                        },
                        open: "modal01".to_string(),
                        on_close: None,
                        disable_overlay_close: false,
                        hide_close_button: false,
                    },
                    header: vec![text("Settings")],
                    body: vec![text("Body")],
                    footer: Vec::new(),
                },
                ViewNode::AlertDialog {
                    props: AlertDialogProps {
                        style: VariantProps {
                            variant: Some(ComponentVariant::Soft),
                            color: Some(ColorFamily::Warning),
                            ..Default::default()
                        },
                        open: "alert01".to_string(),
                        title: "Archive?".to_string(),
                        description: "Archive this project.".to_string(),
                        confirm_text: "Archive".to_string(),
                        cancel_text: "Cancel".to_string(),
                        on_confirm: None,
                        on_cancel: None,
                        loading: false,
                    },
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

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
    assert!(dev
        .content
        .contains("public void handleActivityResult(int requestCode"));
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
    assert!(views
        .content
        .contains("DoweAvatar(source = null, name = \"Ada\""));
    assert!(views
        .content
        .contains("DoweAvatar(source = \"https://example.com/avatar.png\", name = \"Maya\""));
    assert!(views
        .content
        .contains("withContext(Dispatchers.IO) { doweLoadImageBitmap(context, source) }"));
    assert!(views.content.contains("contentScale = ContentScale.Crop"));
    assert!(views.content.contains(
        "modifier = Modifier.doweShadow(radius = doweResponsive(viewportWidth, xs = 44.dp) ?: 0.dp, shape = RoundedCornerShape(999.dp), color = DoweDesign.tertiary, alpha = 0.28f)"
    ));
    assert!(views
        .content
        .contains("DoweBadge(text = \"3\", position = \"bottom-right\""));
    assert!(views.content.contains(".doweBadgeCornerOffset(position)"));
    assert!(views
        .content
        .contains("private fun Modifier.doweBadgeCornerOffset(position: String)"));
    assert!(views
        .content
        .contains("DoweChip(text = \"Filter\", size = \"sm\""));
    assert!(views
        .content
        .contains("DoweSkeleton(variant = \"rounded\", animation = \"pulse\""));
    assert!(views
        .content
        .contains("DoweModal(open = state.bool(\"modal01\")"));
    assert!(views
        .content
        .contains("DoweAlertDialog(open = state.bool(\"modal01\")"));
    assert!(views.content.contains(
        "backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, borderColor = null, confirmBackgroundColor = DoweDesign.danger, confirmContentColor = DoweDesign.dangerText"
    ));
    assert!(views
        .content
        .contains("DoweTooltip(label = \"More actions\", position = \"end\""));
    assert!(views.content.contains("private fun DoweTooltip("));
    assert!(!views.content.contains("private fun doweTooltipAlignment("));
    assert!(views
        .content
        .contains("DoweToast(visible = true, title = \"Saved\""));
    assert!(views.content.contains(
        "position = \"top-right\", backgroundColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText, borderColor = DoweDesign.warning"
    ));
    assert!(views.content.contains("paths = doweOverlayClosePaths"));
    assert!(views
        .content
        .contains("contentDescription = \"Close toast\""));
    assert!(views
        .content
        .contains("DoweDropdown(backgroundColor = DoweDesign.surface"));
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
    assert!(views
        .content
        .contains(".verticalScroll(rememberScrollState())"));
    assert!(views
        .content
        .contains("DoweCommand(open = state.bool(\"modal01\")"));

    let dev = dev_java_source(&output);
    assert!(dev
        .content
        .contains(", doweResponsiveInt(viewportWidth, 44, null, null, null, null), DOWE_TERTIARY, 999f, 0.28f);"));
    assert!(dev
        .content
        .contains(".setLayoutParams(new LinearLayout.LayoutParams(doweDp(48), doweDp(48)));"));
    assert!(dev
        .content
        .contains("doweAvatarImage(\"https://example.com/avatar.png\", \"Maya portrait\", \"M\""));
    assert!(dev.content.contains("private FrameLayout doweAvatarImage("));
    assert!(dev
        .content
        .contains(".setBackground(doweBackground(DOWE_SOFT_SUCCESS, 999f));"));
    assert!(dev.content.contains("BlurMaskFilter.Blur.NORMAL"));
    assert!(dev.content.contains("doweDrawChildShadows(this, canvas)"));
    assert!(dev.content.contains("FrameLayout.LayoutParams"));
    assert!(dev.content.contains("setTranslationX(v.getWidth() / 2f)"));
    assert!(dev.content.contains("setTranslationY(v.getHeight() / 2f)"));
    assert!(dev.content.contains("doweText(\"Search\""));
    assert!(dev.content.contains("doweText(\"Docs\""));
    assert!(dev.content.contains("if (doweBool(\"modal01\"))"));
    assert!(dev.content.contains("PopupWindow"));
    assert!(dev.content.contains("ScrollView view"));
    assert!(dev.content.contains(".setHeight(Math.min("));
    assert!(dev.content.contains(".setDuration(160).start();"));
    assert!(dev
        .content
        .contains("new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0);"));
    assert!(dev.content.contains("TriggerHeight = view"));
    assert!(dev.content.contains("HitParams.height = view"));
    assert!(dev.content.contains(".requestLayout();"));
    assert!(views.content.contains("content = { close ->"));
    assert!(views
        .content
        .contains("onClick = { close(); navigate(\"push\", \"/docs\", null) }"));
    assert!(dev.content.contains(".showAsDropDown(view"));
    assert!(dev.content.contains("LinearLayout view"));
    assert!(dev
        .content
        .contains("doweNavigate(\"push\", \"/docs\", null);"));
    assert!(dev.content.contains("setText(\"Menu\");"));
    assert!(dev.content.contains("setTextColor(DOWE_SOFT_PRIMARY_TEXT);"));
    assert!(dev
        .content
        .contains("setBackground(doweInputBackground(DOWE_SOFT_PRIMARY, null, DOWE_RADIUS));"));
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
    assert!(views
        .content
        .contains("private val doweOverlayClosePaths = listOf("));
    assert!(views.content.contains(
        "DoweSvg(viewBox = doweOverlayCloseViewBox, modifier = Modifier.width(18.dp).height(18.dp), color = DoweDesign.softMutedText, paths = doweOverlayClosePaths)"
    ));
    assert!(views.content.contains(
        "BoxWithConstraints(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center)"
    ));
    assert!(views
        .content
        .contains("val modalMaxWidth = (maxWidth * 0.95f).coerceAtMost(560.dp)"));
    assert!(views
        .content
        .contains(".width(modalMaxWidth)\n                    .padding(16.dp)"));
    assert!(!views
        .content
        .contains("val modalMaxWidth = LocalConfiguration.current.screenWidthDp.dp * 0.95f"));
    assert!(views
        .content
        .contains(".width(28.dp)\n                            .height(28.dp)"));
    assert!(views.content.contains(
        "val toastWidth = (viewportWidth - 32.dp).coerceAtLeast(1.dp).coerceAtMost(420.dp)"
    ));
    assert!(views.content.contains(".width(toastWidth)"));

    let dev = dev_java_source(&output);
    assert!(dev
        .content
        .contains(".setBackground(doweInputBackground(DOWE_SURFACE, DOWE_WARNING, DOWE_RADIUS));"));
    assert!(dev
        .content
        .contains(".setBackground(doweInputBackground(DOWE_SURFACE, null, DOWE_RADIUS));"));
    assert!(dev.content.contains("DOWE_WARNING_TEXT"));
    assert!(dev
        .content
        .contains("setContentDescription(\"Close modal\")"));
    assert!(dev.content.contains(
        "Math.min(doweDp(420), Math.max(doweDp(1), Math.max(0, viewportWidth - doweDp(32))))"
    ));
    assert!(dev.content.contains("popup.setWidth(Math.max(doweDp(1), Math.min(doweDp(420), Math.max(0, root.getWidth() - doweDp(32))))"));
    assert!(dev.content.contains(
        "new FrameLayout.LayoutParams(doweDp(28), doweDp(28), Gravity.TOP | Gravity.END)"
    ));
    assert!(dev
        .content
        .contains("new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER)"));
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
    assert!(views
        .content
        .contains("val modalMaxWidth = (maxWidth * 0.95f).coerceAtMost(560.dp)"));
    assert!(views
        .content
        .contains(".width(modalMaxWidth)\n                    .padding(16.dp)"));
    assert!(!views
        .content
        .contains("val modalMaxWidth = LocalConfiguration.current.screenWidthDp.dp * 0.95f"));
    let dev = dev_java_source(&output);
    let width = "new FrameLayout.LayoutParams(doweDp(Math.max(1, Math.min(560, Math.min(Math.max(0, viewportWidth - 32), (viewportWidth * 95) / 100)))), ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER)";
    assert_eq!(dev.content.matches(width).count(), 2);
    assert!(!dev
        .content
        .contains("Math.min(Math.max(0, viewportWidth - doweDp(32)), (viewportWidth * 95) / 100)"));
}

#[test]
fn generates_android_solar_icon_paints() {
    let stroke = SvgPathFill::Stroke {
        color: Some(ColorToken::Tertiary),
        opacity: 128,
        width: 150,
        line_cap: SvgLineCap::Round,
        line_join: SvgLineJoin::Round,
    };
    assert!(compose_svg_fill(stroke).contains("DoweSvgFill.Stroke(DoweDesign.tertiary"));
    assert!(dev_svg_path_details(stroke).contains("true, 128, 1.5f"));
    assert!(android_runtime_data_code_svg().contains("drawscope.Stroke"));
    assert!(dev_activity_svg_view().contains("Paint.Style.STROKE"));
}

#[test]
fn generates_android_svg_logo_literal_paints() {
    let fill = SvgPathFill::LiteralFill {
        red: 36,
        green: 41,
        blue: 47,
        opacity: 255,
        even_odd: false,
    };
    assert!(compose_svg_fill(fill).contains("Color(0xFF24292F)"));
    assert!(dev_svg_path_color(fill).contains("Color.rgb(36, 41, 47)"));
}

#[test]
fn generates_android_svg_logo_paths_for_compose_and_dev() {
    let logo = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("svg-logos:github-icon".to_string()),
    }])
    .expect("SVG logo");
    let output = generate_android(
        &[ViewRoute {
            id: "logo".to_string(),
            route_path: "/logo".to_string(),
            layout_tree: ViewNode::Children,
            page_tree: logo,
            sections: Vec::new(),
            navigation_actions: Vec::new(),
        }],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();

    assert!(generated.contains("DoweSvgFill.Fill(Color(0xFF"));
    assert!(generated.contains("Color.rgb("));
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
    assert!(views
        .content
        .contains("val visibleItems = maxCount?.let { items.take(it.coerceAtLeast(1)) } ?: items"));
    assert!(!views.content.contains("imageLoadFinished &&"));
    assert!(!views
        .content
        .contains("items.take((visibleLimit - 1).coerceAtLeast(0))"));
    assert!(views
        .content
        .contains("DoweAvatarGroup(items = doweAvatarGroupItems(state.rows(\"people\")"));
    assert!(views
        .content
        .contains("DoweChatBox(state = state, messagesPath = \"messages\""));
    assert!(views.content.contains("DoweEmpty(kind = \"result\""));
    assert!(views.content.contains("DoweMarquee(speed = \"fast\""));
    assert!(views
        .content
        .contains("DoweTypeWriter(texts = listOf(\"Hello\", \"World\")"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("doweAvatarGroup("));
    assert!(dev.content.contains("doweRows(dataPath)"));
    assert!(dev
        .content
        .contains("source = doweTextValue(\"item.src\", row);"));
    assert!(dev
        .content
        .contains("name = doweTextValue(\"item.name\", row);"));
    assert!(dev
        .content
        .contains("alt = doweTextValue(\"item.alt\", row);"));
    assert!(dev
        .content
        .contains("if (assetPath.startsWith(\"assets/\")) assetPath = assetPath.substring(7);"));
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
    assert!(views
        .content
        .contains("Layout(modifier = modifier, content = {"));
    assert!(views
        .content
        .contains("val childConstraints = constraints.copy(minWidth = 0, minHeight = 0)"));
    assert!(views
        .content
        .contains("val contentWidth = lines.maxOfOrNull { it.width } ?: 0"));
    assert!(views
        .content
        .contains("val layoutWidth = constraints.constrainWidth(contentWidth)"));
    assert!(views
        .content
        .contains("var lineLeft = ((layoutWidth - line.width) / 2).coerceAtLeast(0)"));
    assert!(views.content.contains("private fun DoweRichTextRun("));
    assert!(views.content.contains("textAlign = TextAlign.Center"));
    assert!(views
        .content
        .contains("var measuredTextWidth by remember(mark.text, fontFamily, fontSize)"));
    assert!(views.content.contains("onTextLayout = { layout ->"));
    assert!(views
        .content
        .contains("layout.getLineRight(index) - layout.getLineLeft(index)"));
    assert!(views
        .content
        .contains("decoration.then(measuredTextWidth?.let { Modifier.width(it) } ?: Modifier)"));
    assert!(views
        .content
        .contains("background(accent).padding(horizontal = 8.dp, vertical = 2.dp)"));
    assert!(views.content.contains("doweButtonTextFamily(mark.scheme)"));
    assert!(views.content.contains("doweButtonSoftFamily(mark.scheme)"));
    assert!(views.content.contains(
        "if (contentColor == Color.Unspecified) DoweDesign.backgroundText else contentColor"
    ));
    assert!(
        views
            .content
            .contains("DoweRichText(marks = listOf(DoweRichTextMark(text = \"Launch\", style = \"grad\", scheme = \"primary\")")
    );
    assert!(views.content.contains("DoweRecord(name = \"voice\""));
    assert!(views
        .content
        .contains("DoweToggleGroup(value = state.text(\"mode\")"));
    assert!(views
        .content
        .contains("DowePagination(value = state.text(\"page\")"));
    assert!(views.content.contains(
        "pageCount = maxOf(1, minOf(25, ((state.text(\"total\").toIntOrNull() ?: 0).coerceAtLeast(0) + 59) / 60))"
    ));
    assert!(views.content.contains("previousIcon = {"));
    assert!(views.content.contains("nextIcon = {"));
    assert!(views
        .content
        .contains("DoweCollapsible(label = \"Details\""));
    assert!(views.content.contains("arrowIcon = {"));
    assert!(!views
        .content
        .contains("Text(text = if (open) \"⌃\" else \"⌄\""));
    assert!(views
        .content
        .contains("DoweCountdown(target = \"2030-01-01T00:00:00Z\""));
    assert!(views
        .content
        .contains("fillMaxWidth().horizontalScroll(rememberScrollState())"));
    assert!(views.content.contains("Modifier.widthIn(min = width)"));
    assert!(views
        .content
        .contains("BoxWithConstraints(modifier = modifier.fillMaxWidth())"));
    assert!(views
        .content
        .contains("val displaySize = if (maxWidth < 480.dp && size != \"sm\") \"sm\" else size"));
    assert!(views.content.contains("while (!completed)"));
    assert!(views
        .content
        .contains("DoweMap(centerLat = \"4.7109\", centerLng = \"-74.0721\""));
    assert!(views.content.contains("DoweMapMarker(id = \"office\""));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("doweRichTextMark("));
    assert!(dev.content.contains("DoweFlexLayout"));
    assert!(dev
        .content
        .contains("DOWE_JUSTIFY_CENTER, DOWE_ALIGN_CENTER, 4"));
    assert!(dev.content.contains("view.setGravity(Gravity.CENTER);"));
    assert!(dev
        .content
        .contains("view.setBreakStrategy(android.text.Layout.BREAK_STRATEGY_SIMPLE);"));
    assert!(dev
        .content
        .contains("view.setHyphenationFrequency(android.text.Layout.HYPHENATION_FREQUENCY_NONE);"));
    assert!(dev
        .content
        .contains("private static final class DoweRichTextView extends TextView"));
    assert!(dev
        .content
        .contains("protected void onMeasure(int widthSpec, int heightSpec)"));
    assert!(dev
        .content
        .contains("if (MeasureSpec.getMode(widthSpec) == MeasureSpec.EXACTLY)"));
    assert!(dev.content.contains("layout.getLineWidth(index)"));
    assert!(dev.content.contains(
        "super.onMeasure(MeasureSpec.makeMeasureSpec(resolvedWidth, MeasureSpec.EXACTLY), heightSpec);"
    ));
    assert!(dev.content.contains("doweRichTextView(\"Launch\""));
    assert!(dev.content.contains("doweRichTextView(\"ready\""));
    assert!(dev.content.contains("doweText(\"voice\""));
    assert!(dev.content.contains("doweText(\"Details\""));
    assert!(dev
        .content
        .contains("doweCountdown(\"2030-01-01T00:00:00Z\""));
    assert!(dev
        .content
        .contains("private HorizontalScrollView doweCountdown("));
    assert!(dev.content.contains(
        "String displaySize = viewportWidth < 480 && !\"sm\".equals(size) ? \"sm\" : size;"
    ));
    assert!(dev.content.contains("doweWrapContentWidth(column);"));
    assert!(dev
        .content
        .contains("java.time.Instant.parse(target).toEpochMilli()"));
    assert!(dev.content.contains("if (deadline <= current)"));
    assert!(dev
        .content
        .contains("if (onComplete != null) onComplete.run();"));
    assert!(dev.content.contains("update[0].run();"));
    assert!(dev.content.contains("doweText(\"Office\""));
    assert!(dev
        .content
        .contains("setContentDescription(\"Previous page\")"));
    assert!(dev.content.contains("setContentDescription(\"Next page\")"));
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
    assert!(views
        .content
        .contains("columns = doweResponsive(viewportWidth, xs = 1, md = 2) ?: 1"));
    assert!(views
        .content
        .contains("horizontalGap = doweResponsive(viewportWidth, xs = 16.dp) ?: 0.dp"));
    assert!(views.content.contains("DoweInput("));
    assert!(views.content.contains("modifier = Modifier.weight(1f)"));
    assert!(views.content.contains("minHeight = 40.dp"));
    assert!(views.content.contains("horizontalPadding = 12.dp"));
    assert!(views
        .content
        .contains("contentColor = DoweDesign.secondary"));
    assert!(views.content.contains("borderColor = DoweDesign.muted"));
    assert!(views
        .content
        .contains("contentColor = DoweDesign.softMutedText"));
    assert!(views.content.contains(
            "CardDefaults.cardColors(containerColor = DoweDesign.surface, contentColor = DoweDesign.surfaceText), border = BorderStroke(1.dp, DoweDesign.surface)"
        ));
    assert!(
        views
            .content
            .contains("ButtonDefaults.buttonColors(containerColor = Color.Transparent, contentColor = DoweDesign.primary), border = BorderStroke(1.dp, DoweDesign.primary)")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("DoweGridLayout"));
    assert!(dev.content.contains(
            "doweGrid(doweResponsiveInt(viewportWidth, 1, null, 2, null, null), doweResponsiveInt(viewportWidth, 16, null, null, null, null), doweResponsiveInt(viewportWidth, 16, null, null, null, null))"
        ));
    assert!(dev.content.contains("setIncludeFontPadding(false)"));
    assert!(dev.content.contains("setMinHeight(doweDp(40))"));
    assert!(dev
        .content
        .contains("setPadding(doweDp(12), 0, doweDp(12), 0)"));
    assert!(dev
        .content
        .contains("background.setCornerRadius(doweDp(radius));"));
    assert!(dev.content.contains("private float doweDp(float value)"));
    assert!(dev.content.contains(
        "setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f))"
    ));
    assert!(dev.content.contains("doweCard(DOWE_SOFT_MUTED, null)"));
    assert!(dev.content.contains("doweCard(DOWE_SURFACE, DOWE_SURFACE)"));
    assert!(dev.content.contains(
        "setBackground(doweInputBackground(Color.TRANSPARENT, DOWE_PRIMARY, DOWE_RADIUS))"
    ));
    assert!(dev.content.contains("setBackgroundTintList(null)"));
    assert!(dev
        .content
        .contains("doweText(\"Surface\", DOWE_SURFACE_TEXT"));
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
    assert!(views
        .content
        .contains(r#"label = "Email", placeholder = "Email address", floating = false"#));
    assert!(views
        .content
        .contains(r#"label = "Name", placeholder = "Full name", floating = true"#));
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
    assert!(views
        .content
        .contains("targetValue = if (visible) 1f else 0f"));
    assert!(views.content.contains("Popup("));
    assert!(!views.content.contains("DropdownMenu("));
    assert!(!views.content.contains("DropdownMenuItem("));
    assert!(views
        .content
        .contains(r#"label = "Department", placeholder = "Choose department", floating = false"#));
    assert!(views
        .content
        .contains(r#"label = "Role", placeholder = "Choose role", floating = true"#));
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
    assert!(views
        .content
        .contains("DoweSvg(viewBox = doweSelectArrowViewBox"));
    assert!(views
        .content
        .contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4"));
    assert!(views
        .content
        .contains("val active = expanded || selected != null"));
    assert!(views
        .content
        .contains("if (selected != null || !floating || expanded)"));
    assert!(views.content.contains("Text(text = option.description"));
    assert!(views.content.contains(".heightIn(max = 260.dp)"));
    assert!(views
        .content
        .contains(".verticalScroll(rememberScrollState())"));

    let dev = dev_java_source(&output);
    assert!(dev
        .content
        .contains(r#"doweControlLabel("Email", DOWE_PRIMARY"#));
    assert!(dev.content.contains(r#".setHint("Email address")"#));
    assert!(dev.content.contains("doweFloatingInput("));
    assert!(dev.content.contains(r#""Name", "Full name", DOWE_PRIMARY"#));
    let dev_floating_input = dev
        .content
        .find("= doweFloatingInput(")
        .map(|start| &dev.content[start..(start + 320).min(dev.content.len())])
        .expect("dev floating input");
    assert!(dev_floating_input.contains("setMinimumHeight(doweDp(40))"));
    assert!(dev.content.contains("doweUpdateFloatingInputLabel"));
    assert!(dev
        .content
        .contains(r#"doweControlLabel("Department", DOWE_PRIMARY"#));
    assert!(dev.content.contains("doweFloatingSelect("));
    let dev_floating_select = dev
        .content
        .find("= doweFloatingSelect(")
        .map(|start| &dev.content[start..(start + 320).min(dev.content.len())])
        .expect("dev floating select");
    assert!(dev_floating_select.contains("setMinimumHeight(doweDp(56))"));
    assert!(dev.content.contains("doweUpdateFloatingSelectLabel"));
    assert!(dev.content.contains("expanded || hasSelection"));
    assert!(dev
        .content
        .contains("label.setTextSize(active ? 12f : baseSize);"));
    assert!(dev.content.contains("input.setPadding(input.getPaddingLeft(), active ? doweDp(10) : 0, input.getPaddingRight(), input.getPaddingBottom());"));
    assert!(dev.content.contains("doweSelectFrame("));
    assert!(dev.content.contains("doweSelectPopup("));
    assert!(dev.content.contains("PopupWindow popup = new PopupWindow"));
    assert!(dev
        .content
        .contains("Math.min(content.getMeasuredHeight(), doweDp(260))"));
    assert!(dev
        .content
        .contains("ScrollView optionsScroll = new ScrollView(this)"));
    assert!(dev.content.contains("doweSelectArrow("));
    assert!(dev
        .content
        .contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4"));
    assert!(!dev.content.contains("Spinner view"));
    assert!(!dev.content.contains("import android.widget.Spinner;"));
    assert!(dev.content.contains(r#"new String[]{"Admin"}"#));
    assert!(dev.content.contains(r#"new String[]{"Manages users"}"#));
    assert!(!dev.content.contains(r#".setPrompt("Role")"#));
}

#[test]
fn gates_floating_input_icons_on_focus_or_value() {
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

    assert!(views
        .content
        .contains("val active = focused || value.isNotEmpty()"));
    assert!(views.content.contains(
        "if (!floating || active) {\n                        startIcon?.invoke()\n                    }"
    ));
    assert!(views.content.contains(
        "if (!floating || active) {\n                        endIcon?.invoke()\n                    }"
    ));
}

#[test]
fn gates_dev_floating_input_icons_on_focus_or_value() {
    let output = generate_android(
        &[form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert_eq!(dev.content.matches("= doweInputFrame(").count(), 2);
    assert_eq!(dev.content.matches("= doweFloatingInput(").count(), 1);
    assert!(dev
        .content
        .contains("setPadding(doweDp(44), 0, doweDp(44), 0)"));
    assert!(dev
        .content
        .contains("setPadding(doweDp(44), doweDp(10), doweDp(44), 0)"));
    assert!(dev
        .content
        .contains("boolean active = input.hasFocus() || input.getText().length() > 0;"));
    assert!(dev
        .content
        .contains("startIcon.setVisibility(active ? View.VISIBLE : View.GONE);"));
    assert!(dev
        .content
        .contains("endIcon.setVisibility(active ? View.VISIBLE : View.GONE);"));
    assert!(dev
        .content
        .contains("labelParams.leftMargin = doweDp(active && startIcon != null ? 44 : 12);"));
    assert!(dev
        .content
        .contains("labelParams.rightMargin = doweDp(active && endIcon != null ? 44 : 12);"));
    let fixed_frame = dev
        .content
        .split("private FrameLayout doweInputFrame")
        .nth(1)
        .and_then(|body| body.split("private FrameLayout doweFloatingInput").next())
        .expect("fixed input frame helper");
    assert!(!fixed_frame.contains("setVisibility"));
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
    let dev = dev_java_source(&output);

    assert!(views.content.contains("private fun DoweAudio("));
    assert!(views.content.contains("DoweAudio(source ="));
    assert!(views.content.contains("private fun DoweImage("));
    assert!(views
        .content
        .contains("doweLoadImageBitmap(context, source)"));
    assert!(views
        .content
        .contains("DOWE_IMAGE_MEMORY_CACHE_BYTES = 24 * 1024 * 1024"));
    assert!(views
        .content
        .contains("DOWE_IMAGE_DISK_CACHE_BYTES = 64L * 1024L * 1024L"));
    assert!(views
        .content
        .contains("LruCache<String, android.graphics.Bitmap>"));
    assert!(views
        .content
        .contains("doweImageLoadLocks = ConcurrentHashMap<String, Mutex>()"));
    assert!(views.content.contains("lock.withLock"));
    assert!(views
        .content
        .contains("File(context.cacheDir, \"dowe-images\")"));
    assert!(views.content.contains("doweTrimImageDiskCache(directory)"));
    assert!(views
        .content
        .contains("imageOpacity by animateFloatAsState"));
    assert!(views
        .content
        .contains("bitmap == null || imageOpacity < 1f"));
    assert!(views
        .content
        .contains("Modifier.matchParentSize().background(DoweDesign.surface)"));
    assert!(views
        .content
        .contains("graphicsLayer { alpha = imageOpacity }"));
    let image_runtime = views
        .content
        .split("private fun DoweImage(")
        .nth(1)
        .expect("image runtime")
        .split("private const val DOWE_IMAGE_MEMORY_CACHE_BYTES")
        .next()
        .expect("image cache after runtime");
    assert!(image_runtime.contains("contentDescription = alt.takeIf { it.isNotEmpty() }"));
    assert!(!image_runtime.contains("Text("));
    assert!(!image_runtime.contains("hideControls"));
    assert!(views.content.contains("ContentScale.Crop"));
    assert!(views.content.contains("ContentScale.Fit"));
    assert!(views.content.contains("ContentScale.FillBounds"));
    assert!(views.content.contains("ContentScale.None"));
    assert!(!views
        .content
        .contains("DoweCoverBox(modifier = Modifier.matchParentSize(), source = source"));
    assert!(views.content.contains("DoweAccordion("));
    assert!(views.content.contains("defaultOpenIds = setOf(\"intro\")"));
    assert!(views.content.contains("openIds, toggleItem ->"));
    assert!(views.content.contains("open = openIds.contains(\"intro\")"));
    assert!(views.content.contains("arrowIcon = {"));
    assert!(views.content.contains("DoweSvg(viewBox ="));
    assert!(
        views
            .content
            .matches("m19.704 12l-8.491-8.727a.75.75")
            .count()
            >= 2
    );
    assert!(!views
        .content
        .contains("__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__"));
    assert!(views.content.contains("rotationZ = if (open) 90f else 0f"));
    assert!(!views.content.contains("Text(if (open) \"^\" else \"v\")"));
    assert!(dev
        .content
        .contains("doweAccordion(true, DOWE_SURFACE, DOWE_SURFACE_TEXT"));
    assert!(dev.content.contains("doweAccordionItem("));
    assert!(dev.content.contains("\"Intro\", false, true"));
    assert!(dev.content.contains("private LinearLayout doweAccordion("));
    assert!(dev
        .content
        .contains("private LinearLayout doweAccordionItem("));
    assert!(dev.content.contains("private void doweSetAccordionOpen("));
    assert!(dev.content.contains("setOnClickListener(target ->"));
    assert!(
        dev.content
            .matches("m19.704 12l-8.491-8.727a.75.75")
            .count()
            >= 2
    );
    assert!(!dev.content.contains("__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__"));
    assert!(dev
        .content
        .contains("item.arrow.animate().rotation(open ? 90f : 0f)"));
    assert!(views.content.contains("DoweCarousel("));
    assert!(views.content.contains("variant = \"snapping\""));
    assert!(views.content.contains("DoweCarouselSlideSpec(id ="));
    assert!(views.content.contains("LazyRow("));
    assert!(views.content.contains("rememberSnapFlingBehavior"));
    assert!(dev.content.contains("android.widget.HorizontalScrollView"));
    assert!(dev
        .content
        .contains("doweImage(\"https://example.com/photo.jpg\", \"Photo\", \"square\", \"cover\""));
    assert!(dev.content.contains("private FrameLayout doweImage("));
    assert!(dev
        .content
        .contains("private final LruCache<String, Bitmap> doweImageMemoryCache"));
    assert!(dev
        .content
        .contains("doweImageLoadLocks = new ConcurrentHashMap<>()"));
    assert!(dev
        .content
        .contains("private Bitmap doweLoadImageBitmap(String source)"));
    assert!(dev
        .content
        .contains("new File(getCacheDir(), \"dowe-images\")"));
    assert!(dev.content.contains("doweTrimImageDiskCache(directory)"));
    assert!(dev
        .content
        .contains("doweBackground(DOWE_SURFACE, DOWE_RADIUS)"));
    assert!(dev.content.contains("image.setImageBitmap(bitmap);"));
    assert!(dev
        .content
        .contains("view.setBackground(loadedBackground);"));
    assert!(dev.content.contains("ImageView.ScaleType.CENTER_CROP"));
    assert!(dev.content.contains("ImageView.ScaleType.FIT_CENTER"));
    assert!(!dev.content.contains("Image: Photo"));
    assert!(dev.content.contains("setHorizontalScrollBarEnabled(false)"));
    assert!(dev.content.contains("setOnTouchListener"));
    assert!(views.content.contains("rotationY"));
    for variant in [
        "coverFlow",
        "stories",
        "smartStack",
        "cardStack",
        "flipbook",
        "slideshow",
        "masonry",
        "rtl",
        "controls",
        "dots",
        "thumbnails",
    ] {
        assert!(views.content.contains(variant));
    }
    assert!(views.content.contains("DoweCheckbox("));
    assert!(views.content.contains("DoweColorField("));
    assert!(views.content.contains("fontSize = doweTextSize(viewportWidth, min = 12f, preferredBase = 11.2f, preferredViewport = 0.2f, max = 14f)"));
    assert!(views
        .content
        .contains("doweControlHeight(size) + if (floating) 8.dp else 0.dp"));
    assert!(views
        .content
        .contains("DoweColorSwatch(canonical, size, contentColor)"));
    assert!(views
        .content
        .contains("Box(modifier = Modifier.weight(1f)) {\n                        Text(label"));
    assert!(views.content.contains("DoweDateField("));
    assert!(views.content.contains("DoweDateRangeField("));
    assert!(views.content.contains("fontSize: TextUnit"));
    assert!(dev
        .content
        .contains("doweFluidTextSize(12f, 11.2f, 0.2f, 14f)"));
    assert!(dev
        .content
        .contains("doweFluidTextSize(16f, 15.2f, 0.3f, 18f)"));
    assert!(views.content.contains("\"sm\" -> 32.dp"));
    assert!(
        views
            .content
            .matches("doweControlHeight(size) + if (floating) 8.dp else 0.dp")
            .count()
            >= 3
    );
    assert!(views.content.contains("DoweDateCalendar("));
    assert!(views.content.contains("DoweAnchoredPopover("));
    assert!(views.content.contains("DoweRadioGroup("));
    assert!(views.content.contains("DoweToggle("));
    assert!(views.content.contains("RoundedCornerShape(4.dp)"));
    assert!(views.content.contains("DoweInput(value = value"));
    assert!(views.content.contains("private fun DoweColorPickerPanel("));
    assert!(views.content.contains("doweColorFromHsv(next)"));
    assert!(views.content.contains("doweColorCmykText(rgb)"));
    assert!(views.content.contains("doweColorOklchText(rgb)"));
    assert!(views.content.contains("maxHeight = 480.dp"));
    assert!(views.content.contains("BasicTextField("));
    assert!(views.content.contains("orientation = \"horizontal\""));
    assert!(views.content.contains("DoweRadioGroupOption("));
    assert!(views.content.contains("SwitchDefaults.colors"));
    assert!(dev.content.contains("android.widget.CheckBox"));
    assert!(dev
        .content
        .contains("setButtonTintList(ColorStateList.valueOf("));
    assert!(dev.content.contains("Color.parseColor("));
    assert!(dev.content.contains("doweBindColor("));
    assert!(dev.content.contains("private void doweColorPopup("));
    assert!(dev.content.contains("private String doweColorOklchText("));
    assert!(dev
        .content
        .contains("popup.setHeight(Math.min(content.getMeasuredHeight(), doweDp(480)))"));
    assert!(dev.content.contains("doweControlLabel(\"Theme\""));
    assert!(dev.content.contains("doweControlLabel(\"Ship date\""));
    assert!(dev.content.contains("doweDatePopup("));
    assert!(!dev.content.contains("new android.widget.GridLayout.Spec("));
    assert!(dev
        .content
        .contains("android.widget.GridLayout.spec(index / 7)"));
    assert!(dev
        .content
        .contains("android.widget.GridLayout.spec(index % 7)"));
    assert!(dev.content.contains("android.widget.RadioGroup"));
    assert!(dev.content.contains("android.widget.RadioGroup.HORIZONTAL"));
    assert!(dev.content.contains("android.widget.Switch"));
    assert!(dev.content.contains("doweText(\"Off\""));
    assert!(dev.content.contains("doweText(\"On\""));
}

#[test]
fn generates_android_slider_with_block_width_and_bound_initial_value() {
    let output = generate_android(
        &[slider_signal_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let dev = dev_java_source(&output);

    assert!(views
        .content
        .contains("DoweSliderField(value = state.text(\"volume\").toFloatOrNull() ?: 0f"));
    assert!(views
        .content
        .contains("Column(modifier = modifier.fillMaxWidth()"));
    assert!(views.content.contains("modifier = Modifier.fillMaxWidth()"));
    assert!(dev.content.contains("dowePutInitial(\"volume\", 40);"));
    assert!(dev.content.contains("SeekBar"));
    assert!(dev.content.contains(
        ".setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));"
    ));
    assert!(dev
        .content
        .contains("Double.parseDouble(doweTextValue(\"volume\", null))"));
    assert!(dev
        .content
        .contains("BoundValue = Math.max(0, Math.min(100,"));
    assert!(dev.content.contains(".setText(String.valueOf("));
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
    let dev = dev_java_source(&output);

    assert!(views.content.contains("private fun DoweComboBox("));
    assert!(views
        .content
        .contains("DoweComboBox(value = state.text(\"profile.role\")"));
    let combo_box = views
        .content
        .lines()
        .find(|line| line.contains("DoweComboBox(value = state.text(\"profile.role\")"))
        .expect("combo box call");
    assert!(combo_box.contains("minHeight = 40.dp"), "{combo_box}");
    assert!(combo_box.contains("fontSize = doweTextSize(viewportWidth, min = 12f, preferredBase = 11.2f, preferredViewport = 0.2f, max = 14f)"), "{combo_box}");
    assert!(views
        .content
        .contains("DoweSelectOption(\"admin\", \"Admin\", \"Full access\")"));
    assert!(views.content.contains("private data class DoweCsvColumn"));
    assert!(views.content.contains("DoweCsvField(label = \"Import\""));
    assert!(views
        .content
        .contains("DoweCsvColumn(\"email\", \"Email\")"));
    assert!(views.content.contains("private data class DoweDragGroup"));
    assert!(views.content.contains("DoweDragDrop(label = \"Tasks\""));
    assert!(views
        .content
        .contains("DoweDragItem(\"draft\", \"Draft\", \"Prepare\", false)"));
    assert!(views
        .content
        .contains("DoweEditorField(value = state.text(\"profile.notes\")"));
    assert!(views
        .content
        .contains("DoweImageCropper(value = state.text(\"profile.avatar\")"));
    assert!(views
        .content
        .contains("DowePassword(value = state.text(\"profile.password\")"));
    let password_call = views
        .content
        .lines()
        .find(|line| line.contains("DowePassword(value = state.text(\"profile.password\")"))
        .expect("password call");
    assert!(
        password_call.contains("minHeight = 48.dp"),
        "{password_call}"
    );
    assert!(
        password_call.contains("showIcon = { DoweSvg("),
        "{password_call}"
    );
    assert!(
        password_call.contains("hideIcon = { DoweSvg("),
        "{password_call}"
    );
    assert!(
        password_call.contains("Modifier.width(20.dp).height(20.dp)"),
        "{password_call}"
    );
    assert!(password_call.contains("fontSize = doweTextSize(viewportWidth, min = 14f, preferredBase = 13.12f, preferredViewport = 0.25f, max = 16f)"), "{password_call}");
    assert!(views.content.contains("value.any { it.isLowerCase() }"));
    assert!(views.content.contains("Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp))"));
    assert!(views.content.contains("DoweDesign.danger"));
    assert!(views.content.contains("DoweDesign.warning"));
    assert!(views.content.contains("DoweDesign.success"));
    assert!(views.content.contains("PasswordVisualTransformation()"));
    assert!(views
        .content
        .contains("contentDescription = if (visible) \"Hide password\" else \"Show password\""));
    assert!(dev
        .content
        .contains("PasswordTransformationMethod.getInstance()"));
    assert!(dev.content.contains("final View[]"));
    let dev_password = dev
        .content
        .find("= doweFloatingInput(")
        .map(|start| &dev.content[start..(start + 320).min(dev.content.len())])
        .expect("dev password");
    assert!(dev_password.contains("setMinimumHeight(doweDp(48))"));
    assert!(dev.content.contains("DOWE_DANGER"));
    assert!(dev.content.contains("DOWE_WARNING"));
    assert!(dev.content.contains("DOWE_SUCCESS"));
    assert!(dev
        .content
        .contains("setContentDescription(\"Show password\")"));
    assert!(dev.content.contains("setContentDescription("));
    assert!(dev.content.contains("DoweSvgPathEntry"));
    assert!(views
        .content
        .contains("DowePhone(value = state.text(\"profile.phone\")"));
    let phone_call = views
        .content
        .lines()
        .find(|line| line.contains("DowePhone(value = state.text(\"profile.phone\")"))
        .expect("phone call");
    assert!(phone_call.contains("minHeight = 56.dp"), "{phone_call}");
    assert!(phone_call.contains("fontSize = doweTextSize(viewportWidth, min = 16f, preferredBase = 15.2f, preferredViewport = 0.3f, max = 18f)"), "{phone_call}");
    assert!(views.content.contains("countries = dowePhoneCountries"));
    assert!(views
        .content
        .contains("private fun dowePhoneCountries0(): List<DowePhoneCountry>"));
    assert!(views
        .content
        .contains("private val dowePhoneCountries: List<DowePhoneCountry> = buildList"));
    let phone = views
        .content
        .split("private fun DowePhone(")
        .nth(1)
        .expect("phone runtime");
    assert!(phone.contains("DoweAnchoredPopover"));
    assert!(phone.contains("searchPlaceholder"));
    assert!(phone.contains("DoweSvg(viewBox = selected.viewBox"));
    assert!(phone.contains("Modifier.size(24.dp).align(Alignment.CenterVertically)"));
    assert!(phone.contains("modifier = Modifier.align(Alignment.CenterVertically)"));
    assert!(phone.contains("minWidth = 280.dp, maxWidth = 384.dp, maxHeight = 380.dp"));
    assert!(phone.contains("Text(item.name, modifier = Modifier.weight(1f)"));
    assert!(phone.contains("Text(\"+${item.dialCode}\", fontWeight = FontWeight.Bold"));
    assert!(phone.contains("var localValue by remember(value)"));
    assert!(phone.contains("char.isDigit()"));
    assert!(phone.contains("keyboardType = KeyboardType.Number"));
    assert!(dev.content.contains("dowePhonePopup"));
    assert!(dev.content.contains("setIncludeFontPadding(false)"));
    assert!(dev
        .content
        .contains("Params.gravity = Gravity.CENTER_VERTICAL"));
    assert!(dev.content.contains("Params.leftMargin = doweDp(6)"));
    assert!(dev.content.contains(
        "Math.min(doweDp(384), getResources().getDisplayMetrics().widthPixels - doweDp(16))"
    ));
    assert!(dev.content.contains("nameParams.leftMargin = doweDp(10)"));
    assert!(dev.content.contains("setMinimumHeight(doweDp(56))"));
    assert!(dev
        .content
        .contains("android.text.method.DigitsKeyListener.getInstance(\"0123456789\")"));
    let phone_route = output
        .files
        .iter()
        .find(|file| {
            file.relative_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("DoweDevRoute"))
                && file.content.contains("dowePhonePopup")
        })
        .expect("phone route shard");
    assert!(phone_route.content.contains("runtime.dowePhoneFlag("));
    let phone_flag_shards = output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("DoweDevPhoneFlags"))
        })
        .collect::<Vec<_>>();
    assert!(phone_flag_shards.len() > 2);
    assert!(phone_flag_shards
        .iter()
        .all(|file| file.content.len() < 512_000));
    assert!(phone_flag_shards
        .iter()
        .filter(|file| file
            .relative_path
            .file_name()
            .is_some_and(|name| { name.to_string_lossy() != "DoweDevPhoneFlags.java" }))
        .all(|file| file
            .content
            .contains("int viewportWidth = runtime.viewportWidth;")));
    assert!(views
        .content
        .contains("DowePin(value = state.text(\"profile.pin\")"));
    let pin = views
        .content
        .split("private fun DowePin(")
        .nth(1)
        .expect("pin field runtime");
    assert!(pin.contains("FocusRequester"));
    assert!(pin.contains("requestFocus()"));
    assert!(pin.contains("KeyEvent.KEYCODE_DEL"));
    assert!(pin.contains("fontSize: TextUnit"));
    assert!(pin.contains("lineHeight: TextUnit"));
    assert!(pin.contains(".height(doweControlHeight(size))"));
    assert!(pin.contains("BoxWithConstraints(modifier = Modifier.fillMaxWidth())"));
    assert!(pin.contains("val responsiveCellWidth = minOf(cellWidth"));
    assert!(pin.contains(".width(responsiveCellWidth)"));
    assert!(!pin.contains("horizontalScroll(rememberScrollState())"));
    assert!(pin
        .contains("TextStyle(color = contentColor, fontSize = fontSize, lineHeight = lineHeight"));
    assert!(dev.content.contains("PinCells = new EditText["));
    assert!(dev
        .content
        .contains("PinUpdating = new boolean[] { false }"));
    assert!(dev
        .content
        .contains("int accepted = Math.min(next.length()"));
    assert!(dev.content.contains("setOnKeyListener"));
    assert!(dev.content.contains("doweWrite(\"profile.pin\""));
    assert!(dev.content.contains("doweDp(44), doweDp(40)"));
    assert!(dev.content.contains("addOnLayoutChangeListener"));
    assert!(dev
        .content
        .contains("int availableCellWidth = Math.max(doweDp(1)"));
    assert!(dev
        .content
        .contains("Math.min(doweDp(44), availableCellWidth)"));
    assert!(dev
        .content
        .contains("setTextSize(doweFluidTextSize(14f, 13.12f, 0.25f, 16f))"));
    assert!(views
        .content
        .contains("DoweTextarea(value = state.text(\"profile.bio\")"));
    let textarea_call = views
        .content
        .lines()
        .find(|line| line.contains("DoweTextarea(value = state.text(\"profile.bio\")"))
        .expect("textarea call");
    assert!(textarea_call.contains("fontSize = doweTextSize(viewportWidth, min = 14f, preferredBase = 13.12f, preferredViewport = 0.25f, max = 16f)"), "{textarea_call}");
    let textarea = views
        .content
        .split("private fun DoweTextarea(")
        .nth(1)
        .expect("textarea runtime");
    assert!(textarea.contains("var focused by remember { mutableStateOf(false) }"));
    assert!(textarea.contains("fontSize: TextUnit"));
    assert!(textarea
        .contains("TextStyle(color = contentColor, fontSize = fontSize, lineHeight = lineHeight)"));
    assert!(textarea
        .contains("if (value.isEmpty() && placeholder.isNotEmpty() && (!floating || focused))"));
    assert!(textarea.contains("modifier = Modifier.align(Alignment.TopStart)"));
    assert!(dev
        .content
        .contains("setGravity(Gravity.TOP | Gravity.START)"));
    assert!(dev.content.contains("doweFloatingTextarea("));
}

#[test]
fn generates_unique_dev_pin_cell_arrays_for_multiple_fields() {
    let mut route = advanced_form_route();
    let duplicate = match &route.page_tree {
        ViewNode::Box { children, .. } => children
            .iter()
            .find(|node| matches!(node, ViewNode::Pin { .. }))
            .cloned()
            .expect("pin field"),
        _ => panic!("advanced form container"),
    };
    match &mut route.page_tree {
        ViewNode::Box { children, .. } => children.push(duplicate),
        _ => panic!("advanced form container"),
    }
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);
    let arrays = dev
        .content
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("EditText[] ")?
                .split_once(" = new EditText[")
                .map(|(name, _)| name)
        })
        .collect::<Vec<_>>();

    assert_eq!(arrays.len(), 2);
    assert_ne!(arrays[0], arrays[1]);
    assert!(!arrays.contains(&"pinCells"));
}

fn slider_signal_route() -> ViewRoute {
    ViewRoute {
        id: "slider".to_string(),
        route_path: "/slider".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scope {
            constants: Vec::new(),
            signals: vec![ViewSignal {
                id: "volume".to_string(),
                name: "volume".to_string(),
                storage_key: "volume".to_string(),
                scope: dowe_components::ViewSignalScope::Page,
                storage: dowe_components::ViewSignalStorage::None,
                initial: ViewSignalValue::Number("40".to_string()),
                schema: None,
            }],
            actions: Vec::new(),
            children: vec![ViewNode::Slider {
                props: SliderProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Warning),
                        label: Some("Volume".to_string()),
                        element: ElementProps {
                            bind: Some("volume".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    value: "0".to_string(),
                    min: "0".to_string(),
                    max: "100".to_string(),
                    step: Some("5".to_string()),
                    size: ButtonSize::Lg,
                    name: Some("volume".to_string()),
                    hide_label: false,
                },
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

#[test]
fn generates_svg_compose_and_dev_views() {
    let output = generate_android(
        &[svg_route(), runtime_svg_route()],
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
    assert!(views
        .content
        .contains("DoweRuntimeSvg(payload = state.json(\"iconData01\")"));
    assert!(views
        .content
        .contains("private fun doweRuntimeSvgRecord(payload: String)"));
    assert!(views.content.contains("DoweSvgViewBox(0f, 0f, 24f, 24f)"));
    assert!(views.content.contains("DoweSvgFill.CurrentColor"));
    assert!(views.content.contains(
        "doweResponsive(viewportWidth, xs = DoweDesign.tertiary) ?: LocalContentColor.current"
    ));
    assert!(views
        .content
        .contains("PathParser().parsePathString(entry.data).toPath()"));
    assert!(views
        .content
        .contains("DoweSvgTransform(2f, 0f, 0f, 2f, 4f, 6f)"));
    assert!(views.content.contains("private object DoweSvgImporter"));
    assert!(views
        .content
        .contains("private fun rectangle(attrs: Map<String, String>): String"));
    assert!(views
        .content
        .contains("private fun sameColor(left: String, right: String): Boolean"));
    assert!(views
        .content
        .contains("private fun originalFill(value: String?): String"));
    assert!(views
        .content
        .contains("val evenOdd = when (fillRule?.trim()?.lowercase())"));
    assert!(views
        .content
        .contains("if (path.evenOdd) value.put(\"evenOdd\", true)"));
    assert!(views
        .content
        .contains("if (path.evenOdd) \" fillRule:\\\"evenodd\\\"\" else \"\""));
    assert!(views
        .content
        .contains("\"parse.svg\" -> DoweSvgImporter.convert(text(\"value\"), text(\"colors\")"));

    let dev = dev_java_source(&output);
    assert!(dev
        .content
        .contains("private static final class DoweSvgView extends View"));
    assert!(dev
        .content
        .contains("doweRuntimeSvg(doweTextValue(\"iconData01\", null)"));
    assert!(dev.content.contains(
        "private DoweSvgView doweRuntimeSvg(String payload, int currentColor, boolean animated)"
    ));
    assert!(dev
        .content
        .contains("private static final class DoweSvgPathParser"));
    assert!(dev
        .content
        .contains("Path path = DoweSvgPathParser.parse(entry.data)"));
    assert!(dev.content.contains("if (entry.transform != null)"));
    assert!(
        dev.content
            .contains("private static Object doweParseSvg(String source, Object fallback, String colorsMode, String format)")
    );
    let parse_svg_start = dev
        .content
        .find("private static Object doweParseSvg")
        .expect("parse.svg development runtime");
    let parse_svg_end = dev.content[parse_svg_start..]
        .find("private static DoweSvgImportMatrix doweSvgMatrix")
        .expect("parse.svg runtime boundary");
    let parse_svg_runtime = &dev.content[parse_svg_start..parse_svg_start + parse_svg_end];
    assert!(parse_svg_runtime.contains("catch (Exception error)"));
    assert!(!parse_svg_runtime.contains("catch (RuntimeException error)"));
    assert!(parse_svg_runtime.contains("boolean evenOdd = parent.evenOdd;"));
    assert!(parse_svg_runtime.contains("if (pathEvenOdds.get(index)) path.put(\"evenOdd\", true);"));
    assert!(parse_svg_runtime
        .contains("pathEvenOdds.get(index) ? \" fillRule:\\\"evenodd\\\"\" : \"\""));
    assert!(dev
        .content
        .contains("private static String doweSvgRectangle(HashMap<String, String> attrs)"));
    assert!(dev
        .content
        .contains("private static boolean doweSvgSameColor(String left, String right)"));
    assert!(dev
        .content
        .contains("if (\"parse.svg\".equals(name)) return doweParseSvg"));
    assert!(dev.content.contains(
        "Integer fill = entry.currentColor ? Integer.valueOf(currentColor) : entry.color;"
    ));
    assert!(!dev.content.contains("import android.graphics.PathParser;"));
    assert!(dev.content.contains("new DoweSvgView(this, 0f, 0f, 24f, 24f, doweColor(doweResponsiveInt(viewportWidth, DOWE_TERTIARY, null, null, null, null), DOWE_BACKGROUND_TEXT)"));
}

#[test]
fn generates_animated_svg_spinner_views() {
    let spinner = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("svg-spinners:3-dots-bounce".to_string()),
    }])
    .expect("spinner");
    let route = ViewRoute {
        id: "spinner".to_string(),
        route_path: "/spinner".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: spinner,
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let dev = dev_java_source(&output);

    assert!(views.content.contains("animated = true"));
    assert!(views.content.contains("rememberInfiniteTransition"));
    assert!(dev.content.contains("ValueAnimator.areAnimatorsEnabled()"));
    assert!(dev.content.contains(", true);"));
}

#[test]
fn generates_loading_button_with_animated_spinner_and_disabled_state() {
    let route = ViewRoute {
        id: "loading-button".to_string(),
        route_path: "/loading-button".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Button {
            props: VariantProps {
                loading_icon: Some(
                    svg_spinner_control_icon("3-dots-move").expect("button spinner"),
                ),
                reactive: ReactiveVariantProps {
                    loading: Some("saving".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Save")],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views
        .content
        .contains("enabled = !(state.bool(\"saving\", true))"));
    assert!(views.content.contains("if (state.bool(\"saving\", true))"));
    assert!(views.content.contains("animated = true"));
}

#[test]
fn generates_android_viewport_minus_height() {
    let route = ViewRoute {
        id: "viewport".to_string(),
        route_path: "/viewport".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                sizing: dowe_components::SizingProps {
                    h: Some(ResponsiveValue::scalar(SizeValue::ViewportMinus(
                        ScaleValue::from_half_steps(32),
                    ))),
                    min_h: Some(ResponsiveValue::scalar(SizeValue::ViewportMinus(
                        ScaleValue::from_half_steps(40),
                    ))),
                    max_w: Some(ResponsiveValue::scalar(SizeValue::Scale(
                        ScaleValue::from_half_steps(128),
                    ))),
                    max_h: Some(ResponsiveValue::scalar(SizeValue::ViewportMinus(
                        ScaleValue::from_half_steps(48),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let dev = dev_java_source(&output);

    assert!(views.content.contains("DoweSize.ViewportMinus(64.dp)"));
    assert!(views.content.contains("DoweSize.ViewportMinus(80.dp)"));
    assert!(views
        .content
        .contains(".doweMaxWidth(doweResponsive(viewportWidth, xs = DoweSize.Fixed(256.dp)))"));
    assert!(views.content.contains(
        ".doweMaxHeight(doweResponsive(viewportWidth, xs = DoweSize.ViewportMinus(96.dp)))"
    ));
    assert!(views
        .content
        .contains("LocalConfiguration.current.screenHeightDp.dp - value.inset"));
    assert!(dev
        .content
        .contains("Math.max(0, getResources().getConfiguration().screenHeightDp - 64)"));
    assert!(dev
        .content
        .contains("Math.max(0, getResources().getConfiguration().screenHeightDp - 80)"));
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
                    style: bound_style_with_size(
                        "profile.role",
                        "Role",
                        "Choose role",
                        ButtonSize::Sm,
                    ),
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
            ViewNode::Password {
                props: PasswordProps {
                    style: bound_style_with_size(
                        "profile.password",
                        "Password",
                        "Create password",
                        ButtonSize::Md,
                    ),
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
            ViewNode::Phone {
                props: PhoneProps {
                    style: bound_style_with_size(
                        "profile.phone",
                        "Phone",
                        "Phone number",
                        ButtonSize::Lg,
                    ),
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
            ViewNode::Pin {
                props: PinProps {
                    style: bound_style("profile.pin", "Code", ""),
                    value: None,
                    length: 6,
                    kind: PinKind::Number,
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

fn bound_style_with_size(
    bind: &str,
    label: &str,
    placeholder: &str,
    size: ButtonSize,
) -> VariantProps {
    let mut style = bound_style(bind, label, placeholder);
    style.size = Some(size);
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

    assert!(views
        .content
        .contains("private enum class DoweAnimationPreset"));
    assert!(views
        .content
        .contains(".doweAnimation(DoweAnimationPreset.FadeIn)"));
    assert!(views
        .content
        .contains(".doweAnimation(DoweAnimationPreset.SlideUp)"));
    assert!(views.content.contains("animateFloatAsState("));
    assert!(views
        .content
        .contains("rotationZ = doweResponsive(viewportWidth, xs = -7f) ?: 0f"));
    assert!(views
        .content
        .contains("scaleX = doweResponsive(viewportWidth, xs = 1.05f) ?: 1f"));
    assert!(views
        .content
        .contains("translationX = (doweResponsive(viewportWidth, xs = -6.dp) ?: 0.dp).toPx()"));
    assert!(views
        .content
        .contains(".doweGesture(DoweGesturePreset.Lift, DoweTransitionPreset.Spring)"));
    assert!(views
        .content
        .contains("ValueAnimator.areAnimatorsEnabled()"));
    assert!(views.content.contains("scaleX = 1f - 0.06f * progress"));
    assert!(views
        .content
        .contains("awaitPointerEvent(PointerEventPass.Initial)"));
    assert!(views
        .content
        .contains("awaitPointerEventScope {\n                while (true)"));
    assert!(views
        .content
        .contains("pressed = event.changes.any { change ->"));
    assert!(views
        .content
        .contains("change.position.x <= size.width.toFloat()"));
    assert!(views
        .content
        .contains("change.position.y <= size.height.toFloat()"));
    assert!(views
        .content
        .contains("finally {\n                pressed = false"));
    assert!(views
        .content
        .contains("var routeRevision by remember { mutableIntStateOf(0) }"));
    assert!(views
        .content
        .contains("if (operation == \"replace\") routeRevision += 1"));
    assert!(views
        .content
        .contains("key(currentEntry.path, routeRevision)"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("baseScaleX * 0.94f"));
    assert!(dev
        .content
        .contains("stateAnimator.addState(new int[]{android.R.attr.state_pressed}"));
    assert!(dev
        .content
        .contains("view.setStateListAnimator(stateAnimator);"));
    assert!(dev
        .content
        .contains("DOWE_GESTURE_ANIMATORS.put(view, stateAnimator);"));
    assert!(dev
        .content
        .contains("if (!DOWE_GESTURE_ANIMATORS.containsKey(view))"));
    assert!(dev.content.contains(r#"doweAnimate(view0, "fadeIn");"#));
    assert!(dev.content.contains(r#"doweAnimate(view1, "slideUp");"#));
    assert!(dev
        .content
        .contains("view1.setRotation(doweResponsiveFloat(viewportWidth"));
    assert!(dev
        .content
        .contains(r#"doweGesture(view1, "lift", "spring");"#));
    assert!(dev.content.contains("renderCurrentRoute();"));
}

#[test]
fn generates_compose_form_validation_contract() {
    let mut props = VariantProps {
        label: Some("Email".to_string()),
        variant: Some(ComponentVariant::Outlined),
        ..Default::default()
    };
    let validation = props.element.form_validation_mut();
    validation.help_text = Some("Use your work email".to_string());
    validation.rules = vec![
        dowe_components::form_validation_rule("required", "Email is required").expect("rule"),
        dowe_components::form_validation_rule("email", "Enter a valid email").expect("rule"),
    ];
    let route = ViewRoute {
        id: "validation".to_string(),
        route_path: "/validation".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Input { props },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let source = all_android_source(&output);

    assert!(source.contains("private data class DoweValidationRule"));
    assert!(source.contains("private fun doweValidationError"));
    assert!(source.contains("message = \"Email is required\""));
    assert!(source.contains("helpText = \"Use your work email\""));
    assert!(source.contains("if (touched) doweValidationError"));
    assert!(source.contains("DoweDesign.danger"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("private final class DoweValidationBinding"));
    assert!(dev.content.contains("doweTouchedValidations"));
    assert!(dev
        .content
        .contains("new String[]{\"required\", null, \"Email is required\"}"));
    assert!(dev.content.contains("Validation.watchText();"));
    assert!(dev.content.contains("DOWE_DANGER"));
}
