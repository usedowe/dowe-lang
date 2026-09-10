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

    assert!(
        views
            .content
            .contains("val active = focused || value.isNotEmpty()")
    );
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
    assert!(
        dev.content
            .contains("setPadding(doweDp(44), 0, doweDp(44), 0)")
    );
    assert!(
        dev.content
            .contains("setPadding(doweDp(44), doweDp(10), doweDp(44), 0)")
    );
    assert!(
        dev.content
            .contains("boolean active = input.hasFocus() || input.getText().length() > 0;")
    );
    assert!(
        dev.content
            .contains("startIcon.setVisibility(active ? View.VISIBLE : View.GONE);")
    );
    assert!(
        dev.content
            .contains("endIcon.setVisibility(active ? View.VISIBLE : View.GONE);")
    );
    assert!(
        dev.content
            .contains("labelParams.leftMargin = doweDp(active && startIcon != null ? 44 : 12);")
    );
    assert!(
        dev.content
            .contains("labelParams.rightMargin = doweDp(active && endIcon != null ? 44 : 12);")
    );
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
    assert!(views.content.contains("MediaPlayer()"));
    assert!(views.content.contains("repeat(50)"));
    assert!(views.content.contains("doweAudioTime"));
    assert!(views.content.contains("awaitEachGesture"));
    assert!(views.content.contains("animateFloatAsState"));
    assert!(views.content.contains("private val doweAudioWaveform"));
    assert!(dev.content.contains("doweAudio(\""));
    assert!(dev.content.contains("class DoweAudioWaveView"));
    assert!(dev.content.contains("DOWE_AUDIO_WAVEFORM"));

    let mut card_route = media_display_form_route();
    card_route.page_tree = ViewNode::RadioGroup {
        props: RadioGroupProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Outlined),
                color: Some(ColorFamily::Primary),
                element: ElementProps {
                    bind: Some("workspace".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            size: ButtonSize::Md,
            orientation: RadioGroupOrientation::Horizontal,
            presentation: RadioGroupPresentation::Card,
            name: Some("workspace".to_string()),
            info: None,
            error: None,
        },
        options: vec![RadioOption {
            value: "local".to_string(),
            label: "Local".to_string(),
            description: Some("Edit files on your computer".to_string()),
            icon: Some(solar_control_icon("laptop").expect("laptop icon")),
            disabled: false,
        }],
    };
    let card_output = generate_android(
        &[card_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let card_views = card_output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("card views");
    let card_dev = dev_java_source(&card_output);
    assert!(card_views.content.contains("DoweRadioCard("));
    assert!(card_views.content.contains("DoweRadioCardOption("));
    assert!(card_views.content.contains("Edit files on your computer"));
    assert!(card_dev.content.contains("setBackground(doweInputBackground"));
    assert!(card_dev.content.contains("doweWrite(\"workspace\", \"local\")"));
    assert!(views.content.contains("private fun DoweImage("));
    assert!(
        views
            .content
            .contains("doweLoadImageBitmap(context, source)")
    );
    assert!(
        views
            .content
            .contains("DOWE_IMAGE_MEMORY_CACHE_BYTES = 24 * 1024 * 1024")
    );
    assert!(
        views
            .content
            .contains("DOWE_IMAGE_DISK_CACHE_BYTES = 64L * 1024L * 1024L")
    );
    assert!(
        views
            .content
            .contains("LruCache<String, android.graphics.Bitmap>")
    );
    assert!(
        views
            .content
            .contains("doweImageLoadLocks = ConcurrentHashMap<String, Mutex>()")
    );
    assert!(views.content.contains("lock.withLock"));
    assert!(
        views
            .content
            .contains("File(context.cacheDir, \"dowe-images\")")
    );
    assert!(views.content.contains("doweTrimImageDiskCache(directory)"));
    assert!(
        views
            .content
            .contains("imageOpacity by animateFloatAsState")
    );
    assert!(
        views
            .content
            .contains("bitmap == null || imageOpacity < 1f")
    );
    assert!(
        views
            .content
            .contains("Modifier.matchParentSize().background(DoweDesign.surface)")
    );
    assert!(
        views
            .content
            .contains("graphicsLayer { alpha = imageOpacity }")
    );
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
    assert!(
        !views
            .content
            .contains("DoweCoverBox(modifier = Modifier.matchParentSize(), source = source")
    );
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
    assert!(
        !views
            .content
            .contains("__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__")
    );
    assert!(views.content.contains("rotationZ = if (open) 90f else 0f"));
    assert!(views.content.contains("variant = \"ghost\""));
    assert!(
        views
            .content
            .contains("padding(if (variant == \"ghost\" || variant == \"line\") 0.dp else 4.dp)")
    );
    assert!(
        views
            .content
            .contains("radius = 0.dp, onToggle = { toggleItem(\"intro\") }")
    );
    assert!(!views.content.contains("radius = if (variant == \"ghost\")"));
    assert!(views.content.contains("fontWeight = FontWeight.Bold"));
    assert!(
        !views
            .content
            .contains("background(LocalContentColor.current.copy(alpha = 0.12f))")
    );
    assert!(!views.content.contains("Text(if (open) \"^\" else \"v\")"));
    assert!(
        dev.content
            .contains("doweAccordion(true, \"ghost\", Color.TRANSPARENT, DOWE_SURFACE, null")
    );
    assert!(dev.content.contains("doweAlpha(DOWE_SURFACE, 0.22f)"));
    assert!(!dev.content.contains("arrow.setBackgroundColor"));
    assert!(
        dev.content
            .contains("doweText(label, accordionState.contentColor, 15f, 700, 0f, 1.2f, font)")
    );
    assert!(
        !dev.content
            .contains("doweText(label, accordionState.contentColor, 15f, 700, 0f, 20f, font)")
    );
    assert!(dev.content.contains("doweAccordionItem("));
    assert!(dev.content.contains("\"Intro\", false, true"));
    assert!(dev.content.contains("private LinearLayout doweAccordion("));
    assert!(
        dev.content
            .contains("private LinearLayout doweAccordionItem(")
    );
    assert!(dev.content.contains("private void doweSetAccordionOpen("));
    assert!(dev.content.contains("setOnClickListener(target ->"));
    assert!(
        dev.content
            .matches("m19.704 12l-8.491-8.727a.75.75")
            .count()
            >= 2
    );
    assert!(!dev.content.contains("__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__"));
    assert!(
        dev.content
            .contains("item.arrow.animate().rotation(open ? 90f : 0f)")
    );
    assert!(views.content.contains("DoweCarousel("));
    assert!(views.content.contains("variant = \"snapping\""));
    assert!(views.content.contains("DoweCarouselSlideSpec(id ="));
    assert!(views.content.contains("LazyRow("));
    assert!(views.content.contains("rememberSnapFlingBehavior"));
    assert!(views.content.contains("if (showNavigation)"));
    assert!(
        views
            .content
            .contains("enabled = !disableLoop || currentIndex > 0")
    );
    assert!(views.content.contains("orientation = orientation"));
    assert!(views.content.contains("phase = slidePhase(index)"));
    assert!(views.content.contains("rotationY = phase * 24f"));
    assert!(views.content.contains("ButtonDefaults.buttonColors"));
    assert!(
        views
            .content
            .contains("TextButton(modifier = Modifier.size(36.dp)")
    );
    assert!(
        views
            .content
            .contains("ButtonDefaults.textButtonColors(contentColor = accentColor)")
    );
    assert!(dev.content.contains("android.widget.HorizontalScrollView"));
    assert!(
        dev.content
            .contains("setBackgroundColor(Color.TRANSPARENT)")
    );
    assert!(dev.content.contains("setMinimumHeight(doweDp(32))"));
    assert!(
        dev.content
            .contains("setContentDescription(\"Previous slide\")")
    );
    assert!(
        dev.content
            .contains("setContentDescription(\"Next slide\")")
    );
    assert!(dev.content.contains("ArrayList<Button>"));
    assert!(dev.content.contains("setOnScrollChangeListener"));
    assert!(dev.content.contains("setRotationY(phase * 24f)"));
    assert!(dev.content.contains("setRotation(phase * 1.5f)"));
    assert!(!dev.content.contains("setRotationZ("));
    assert!(
        dev.content
            .contains("final int selectedIndex = targetIndex")
    );
    assert!(
        dev.content
            .contains("final int selectedNextIndex = targetIndex")
    );
    assert!(dev.content.contains("smoothScrollTo(slide.getLeft(), 0)"));
    assert!(
        dev.content.contains(
            "doweImage(\"https://example.com/photo.jpg\", \"Photo\", \"square\", \"cover\""
        )
    );
    assert!(dev.content.contains("private FrameLayout doweImage("));
    assert!(
        dev.content
            .contains("private final LruCache<String, Bitmap> doweImageMemoryCache")
    );
    assert!(
        dev.content
            .contains("doweImageLoadLocks = new ConcurrentHashMap<>()")
    );
    assert!(
        dev.content
            .contains("private Bitmap doweLoadImageBitmap(String source)")
    );
    assert!(
        dev.content
            .contains("new File(getCacheDir(), \"dowe-images\")")
    );
    assert!(dev.content.contains("doweTrimImageDiskCache(directory)"));
    assert!(
        dev.content
            .contains("doweBackground(DOWE_SURFACE, DOWE_RADIUS)")
    );
    assert!(dev.content.contains("image.setImageBitmap(bitmap);"));
    assert!(
        dev.content
            .contains("view.setBackground(loadedBackground);")
    );
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
    assert!(
        views
            .content
            .contains("doweControlHeight(size) + if (floating) 8.dp else 0.dp")
    );
    assert!(
        views
            .content
            .contains("DoweColorSwatch(canonical, size, contentColor)")
    );
    assert!(
        views
            .content
            .contains("Box(modifier = Modifier.weight(1f)) {\n                        Text(label")
    );
    assert!(views.content.contains("DoweDateField("));
    assert!(views.content.contains("DoweDateRangeField("));
    assert!(views.content.contains("fontSize: TextUnit"));
    assert!(
        dev.content
            .contains("doweFluidTextSize(12f, 11.2f, 0.2f, 14f)")
    );
    assert!(
        dev.content
            .contains("doweFluidTextSize(16f, 15.2f, 0.3f, 18f)")
    );
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
    assert!(
        dev.content
            .contains("setButtonTintList(ColorStateList.valueOf(")
    );
    assert!(dev.content.contains("Color.parseColor("));
    assert!(dev.content.contains("doweBindColor("));
    assert!(dev.content.contains("private void doweColorPopup("));
    assert!(dev.content.contains("private String doweColorOklchText("));
    assert!(
        dev.content
            .contains("popup.setHeight(Math.min(content.getMeasuredHeight(), doweDp(480)))")
    );
    assert!(dev.content.contains("doweControlLabel(\"Theme\""));
    assert!(dev.content.contains("doweControlLabel(\"Ship date\""));
    assert!(dev.content.contains("doweDatePopup("));
    assert!(!dev.content.contains("new android.widget.GridLayout.Spec("));
    assert!(
        dev.content
            .contains("android.widget.GridLayout.spec(index / 7)")
    );
    assert!(
        dev.content
            .contains("android.widget.GridLayout.spec(index % 7)")
    );
    assert!(dev.content.contains("android.widget.RadioGroup"));
    assert!(dev.content.contains("android.widget.RadioGroup.HORIZONTAL"));
    assert!(dev.content.contains("android.widget.Switch"));
    assert!(dev.content.contains("doweText(\"Off\""));
    assert!(dev.content.contains("doweText(\"On\""));
}

