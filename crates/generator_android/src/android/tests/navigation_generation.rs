#[test]
fn generates_native_android_translation_resources() {
    let mut localized_route = route();
    localized_route.page_tree = ViewNode::Title {
        props: TextProps {
            i18n: Some("home.hero.title".to_string()),
            ..Default::default()
        },
        value: "Dowe builds systems.".to_string(),
    };
    let output = generate_android_with_translations(
        &[localized_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &translations(),
    );
    let resource = dowe_components::translation_resource_name("home.hero.title");
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(
        views
            .content
            .contains(&format!("stringResource(R.string.{resource})"))
    );
    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains(&format!("getString(R.string.{resource})"))
    );
    let default_strings = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("res/values/strings.xml"))
        .expect("default strings");
    assert!(default_strings.content.contains("Dowe builds systems."));
    let spanish_strings = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("res/values-es/strings.xml"))
        .expect("spanish strings");
    assert!(spanish_strings.content.contains("Dowe construye sistemas."));
}

#[test]
fn generates_android_code_with_copy_and_theme_tokens() {
    let output = generate_android(
        &[code_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweCode("));
    assert!(
        views
            .content
            .contains("clipboard.setText(AnnotatedString(source))")
    );
    assert!(views.content.contains("DoweCode(source = \"page docsPage\\n  Card variant:\\\"soft\\\" p:4 show:true\\n    Text\\n      Documentation\""));
    assert!(views.content.contains("DoweDesign.primary"));
    assert!(views.content.contains("DoweDesign.info"));
    assert!(views.content.contains("DoweDesign.tertiary"));
    assert!(views.content.contains("DoweDesign.success"));
    assert!(views.content.contains("DoweDesign.warning"));
    assert!(views.content.contains("DoweDesign.danger"));
    assert!(views.content.contains(
        "Modifier.fillMaxWidth().clipToBounds().horizontalScroll(rememberScrollState())"
    ));
    assert!(views.content.contains(
        "Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(contentColor.copy(alpha = 0.24f)))"
    ));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("private LinearLayout doweCode("));
    assert!(dev.content.contains("ClipboardManager clipboard"));
    assert!(dev.content.contains("view.setClipChildren(true);"));
    assert!(dev.content.contains("doweRound(view, DOWE_RADIUS);"));
    assert!(dev.content.contains("divider.setBackgroundColor(doweAlpha(contentColor, 0.24f));"));
    assert!(dev.content.contains("scroll.setFillViewport(true);"));
    assert!(
        dev.content
            .contains("new ForegroundColorSpan(tokenColors[index])")
    );
}

#[test]
fn generates_android_video_with_native_hls_player() {
    let output = generate_android(
        &[video_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweVideo("));
    assert!(views.content.contains("VideoView(context)"));
    assert!(!views.content.contains("MediaController(context)"));
    assert!(views.content.contains("DoweVideoControls("));
    assert!(views.content.contains("icons.pictureInPicture"));
    assert!(views.content.contains("icons.fullscreen"));
    assert!(views.content.contains("enterPictureInPictureMode"));
    assert!(views.content.contains("doweVideoPictureInPictureOverlay"));
    assert!(views.content.contains("doweHandleVideoPictureInPictureMode"));
    assert!(views.content.contains("Dialog("));
    assert!(views.content.contains("doweLoadImageBitmap(context, poster)"));
    assert!(views.content.contains("contentAlignment = Alignment.Center"));
    assert!(
        views
            .content
            .contains("https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8")
    );
    assert!(views.content.contains("poster = \"/images/video.jpg\""));
    assert!(views.content.contains("aspect = \"vertical\""));
    let app_manifest = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("app/src/main/AndroidManifest.xml"))
        .expect("app manifest");
    assert!(
        app_manifest
            .content
            .contains("android:supportsPictureInPicture=\"true\"")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("private FrameLayout doweVideo("));
    assert!(
        dev.content
            .contains("VideoView video = new VideoView(this)")
    );
    assert!(!dev.content.contains("MediaController controls"));
    assert!(dev.content.contains("doweVideoControls("));
    assert!(dev.content.contains("doweEnterVideoPictureInPicture"));
    assert!(dev.content.contains("handlePictureInPictureMode"));
    assert!(dev.content.contains("setControlsVisible(false)"));
    assert!(dev.content.contains("doweToggleVideoFullscreen"));
    assert!(dev.content.contains("doweLoadImageBitmap(poster)"));
    assert!(dev.content.contains("setMediaAspect"));
    let dev_manifest = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("dev/AndroidManifest.xml"))
        .expect("dev manifest");
    assert!(
        dev_manifest
            .content
            .contains("android:supportsPictureInPicture=\"true\"")
    );
    assert!(
        dev.content
            .contains("https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8")
    );
}

#[test]
fn generates_android_iframe_with_hardened_webview() {
    let output = generate_android(&[iframe_route()], &FontConfig::default(), &DesignConfig::default(), &[]);
    let views = output.files.iter().find(|file| file.relative_path.ends_with("DowePages.kt")).expect("views");
    assert!(views.content.contains("private fun DoweIframe("));
    assert!(views.content.contains("WebView(context)"));
    assert!(views.content.contains("settings.allowFileAccess = false"));
    assert!(views.content.contains("sandbox = listOf(\"scripts\", \"same-origin\")"));
    assert!(views.content.contains("autoplay = true"));
    let dev = dev_java_source(&output);
    assert!(dev.content.contains("private FrameLayout doweIframe("));
    assert!(dev.content.contains("setAllowFileAccess(false)"));
    assert!(dev.content.contains("doweIframe(\"https://example.com/embed\""));
}

#[test]
fn generates_android_canvas_for_compose_and_dev_runtime() {
    let output = generate_android(
        &[canvas_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweCanvas("));
    assert!(views.content.contains(
        "DoweCanvas(state = state, scenePath = \"scene\", viewWidth = 640f, viewHeight = 360f, fit = \"cover\", fps = 30, autoplay = false, pixelated = true"
    ));
    assert!(views.content.contains(
        "filterQuality = if (pixelated) FilterQuality.None else FilterQuality.Low"
    ));
    assert!(views.content.contains(
        ".border(doweResponsive(viewportWidth, xs = 1.dp) ?: 0.dp, DoweDesign.primary, RoundedCornerShape(doweResponsive(viewportWidth, xs = 8.dp) ?: DoweDesign.radius))"
    ));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("private DoweCanvasView doweCanvas("));
    assert!(dev.content.contains("private int doweCanvasColor(Object value)"));
    assert!(dev.content.contains("doweLoadCanvasImage"));
    assert!(dev.content.contains("requestDisallowInterceptTouchEvent(true)"));
    assert!(dev.content.contains("doweReleaseCanvasGesture();\n            pointers.clear();"));
    assert!(dev.content.contains("doweFocusedCanvasKeyAction"));
    assert!(dev.content.contains("void doweRunCanvasAction("));
    assert!(dev.content.contains("doweRunCanvasAction(onMotion, item)"));
    assert!(!dev.content.contains("doweRunAction(onMotion, item)"));
    assert!(dev.content.contains("canvas.drawBitmap(bitmap, source, destination, paint)"));
    assert!(dev.content.contains(
        "DoweCanvasView view0 = doweCanvas(\"scene\", 640f, 360f, \"cover\", 30, false, true, DOWE_BACKGROUND, \"Animated scene\", null, null, null, 30, doweResponsiveInt(viewportWidth, 1, null, null, null, null), DOWE_PRIMARY, doweFloat(doweResponsiveFloat(viewportWidth, 8f, null, null, null, null), DOWE_RADIUS))"
    ));
    assert!(dev.content.contains("doweDrawCanvasBorder(canvas);"));
    assert!(dev.content.contains("paint.setFilterBitmap(!pixelated);"));
    assert!(dev.content.contains("doweAdd(root, view0);"));
}

#[test]
fn generates_android_candlestick_with_canvas_and_stream_runtime() {
    let output = generate_android(
        &[candlestick_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweCandlestick("));
    assert!(
        views
            .content
            .contains("Canvas(modifier = Modifier.matchParentSize())")
    );
    assert!(
        views
            .content
            .contains("doweConnectCandlestickStream(stream, dataPath, maxPoints, state)")
    );
    assert!(
        views
            .content
            .contains("state.upsertCandles(dataPath, payload, maxPoints)")
    );
    assert!(views.content.contains(
        "DoweCandlestick(state = state, dataPath = \"candles\", stream = \"/api/candles\""
    ));
    assert!(views.content.contains("emptyLabel = \"Market closed\""));

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("private DoweCandlestickView doweCandlestick(")
    );
    assert!(dev.content.contains("private void doweUpsertCandles("));
    assert!(dev.content.contains(
        "HttpURLConnection connection = (HttpURLConnection) new URL(address).openConnection()"
    ));
    assert!(dev.content.contains("DoweCandlestickView"));
}

#[test]
fn generates_android_charts_with_canvas_runtime() {
    let output = generate_android(
        &[charts_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweChart("));
    assert!(views.content.contains("doweDrawPointChart"));
    assert!(views.content.contains("doweDrawPieChart"));
    assert!(views.content.contains(
        "DoweChart(state = state, chartType = \"arc\", dataPath = \"segments\""
    ));
    assert!(views.content.contains(
        "DoweChart(state = state, chartType = \"line\", dataPath = \"points\""
    ));

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("private DoweChartView doweChart(")
    );
    assert!(dev.content.contains("DoweChartView"));
    assert!(dev.content.contains("doweDrawPieChart"));
}
