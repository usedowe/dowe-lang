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
    assert!(views.content.contains("DoweCode(source = \"page docsPage\\n  Card variant:\\\"solid\\\" p:4 show:true\\n    Text\\n      Documentation\""));
    assert!(views.content.contains("DoweDesign.primary"));
    assert!(views.content.contains("DoweDesign.info"));
    assert!(views.content.contains("DoweDesign.accent"));
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
    assert!(
        dev.content
            .contains("divider.setBackgroundColor(doweAlpha(contentColor, 0.24f));")
    );
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
    assert!(
        views
            .content
            .contains("doweHandleVideoPictureInPictureMode")
    );
    assert!(views.content.contains("Dialog("));
    assert!(
        views
            .content
            .contains("doweLoadImageBitmap(context, poster)")
    );
    assert!(
        views
            .content
            .contains("contentAlignment = Alignment.Center")
    );
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
        .find(|file| {
            file.relative_path
                .ends_with("app/src/main/AndroidManifest.xml")
        })
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
    let output = generate_android(
        &[iframe_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private fun DoweIframe("));
    assert!(views.content.contains("WebView(context)"));
    assert!(views.content.contains("settings.allowFileAccess = false"));
    assert!(
        views
            .content
            .contains("sandbox = listOf(\"scripts\", \"same-origin\")")
    );
    assert!(views.content.contains("autoplay = true"));
    let dev = dev_java_source(&output);
    assert!(dev.content.contains("private FrameLayout doweIframe("));
    assert!(dev.content.contains("setAllowFileAccess(false)"));
    assert!(
        dev.content
            .contains("doweIframe(\"https://example.com/embed\"")
    );
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
    for contract in [
        "rememberUpdatedState(drawMode)",
        "points.size == 1 && doweCanvasSegmentDistance(point, points.first(), points.first()) <= max(6f, stroke)",
        "var layerSequence",
        "drawingPointer == change.id.value",
        "finally {",
        "doweCanvasLayerHit",
        "doweCanvasLayerBounds",
        "kind == \"move\" || kind == \"up\"",
    ] {
        assert!(
            views.content.contains(contract),
            "missing Compose Draw contract: {contract}"
        );
    }
    assert!(views.content.contains(
        "DoweCanvas(state = state, scenePath = \"scene\", viewWidth = 640f, viewHeight = 360f, fit = \"cover\", fps = 30, autoplay = false, pixelated = true"
    ));
    assert!(
        views
            .content
            .contains("filterQuality = if (pixelated) FilterQuality.None else FilterQuality.Low")
    );
    assert!(views.content.contains(
        ".border(doweResponsive(viewportWidth, xs = 1.dp) ?: 0.dp, DoweDesign.primary, RoundedCornerShape(doweResponsive(viewportWidth, xs = 8.dp) ?: DoweDesign.radius))"
    ));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("private DoweCanvasView doweCanvas("));
    for contract in [
        "long layerSequence",
        "drawingPointerId == id",
        "doweCancelCanvasLayer()",
        "doweCanvasLayerBounds",
        "doweCanvasSegmentDistance",
        "doweRefreshCanvasPage",
        "drawingMode = doweActiveCanvasDrawMode()",
    ] {
        assert!(
            dev.content.contains(contract),
            "missing launcher Draw contract: {contract}"
        );
    }
    assert!(
        dev.content
            .contains("private int doweCanvasColor(Object value)")
    );
    assert!(dev.content.contains("doweLoadCanvasImage"));
    assert!(
        dev.content
            .contains("requestDisallowInterceptTouchEvent(true)")
    );
    assert!(
        dev.content
            .contains("doweReleaseCanvasGesture();\n            pointers.clear();")
    );
    assert!(dev.content.contains("doweFocusedCanvasKeyAction"));
    assert!(dev.content.contains("void doweRunCanvasAction("));
    assert!(dev.content.contains("doweRunCanvasAction(onMotion, item)"));
    assert!(!dev.content.contains("doweRunAction(onMotion, item)"));
    assert!(
        dev.content
            .contains("canvas.drawBitmap(bitmap, source, destination, paint)")
    );
    assert!(dev.content.contains(
        "DoweCanvasView view0 = doweCanvas(\"scene\", 640f, 360f, \"cover\", 30, false, true, DOWE_BACKGROUND, \"Animated scene\", null, null, null, 30, false, \"pen\""
    ));
    assert!(dev.content.contains(
        "null, null, null, null, null, null, null, doweResponsiveInt(viewportWidth, 1, null, null, null, null)"
    ));
    assert!(dev.content.contains("doweDrawCanvasBorder(canvas);"));
    assert!(dev.content.contains("paint.setFilterBitmap(!pixelated);"));
    assert!(dev.content.contains("doweAdd(root, view0);"));
}

#[test]
#[ignore = "requires a JDK; run explicitly for Draw runtime validation"]
fn draw_android_launcher_executes_layer_contract() {
    let runtime = super::dev_activity_canvas_runtime();
    let methods = [
        "private String doweActiveCanvasDrawMode(",
        "private ArrayList<Map<String, Object>> doweCanvasLayers(",
        "private String doweCanvasLayerId(",
        "private boolean doweCanvasLayerHit(",
        "private void doweCanvasLayerEvent(",
        "private void doweRunCanvasAction(",
        "private void doweUpdateCanvasLayer(",
        "private void doweRemoveSelectedLayer(",
        "private String doweSelectedCanvasLayer(",
        "private void doweCancelCanvasLayer(",
        "private List<PointF> doweCanvasLayerPoints(",
        "private float doweCanvasSegmentDistance(",
        "private android.graphics.RectF doweCanvasLayerBounds(",
        "private float doweCanvasNumber(",
        "public boolean onTouchEvent(",
        "private PointF doweCanvasLogicalPoint(",
        "private PointF doweCanvasRawPoint(",
        "private boolean doweCanvasInside(",
    ]
    .map(|signature| diagram_java_method(runtime, signature))
    .join("\n");
    let source = include_str!("canvas_harness.java").replace(
        "__DOWE_METHODS__",
        &methods.replace("android.graphics.RectF", "RectF"),
    );
    let directory = std::env::temp_dir().join(format!("dowe-canvas-java-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let file = directory.join("CanvasHarness.java");
    std::fs::write(&file, source).unwrap();
    let compiled = std::process::Command::new("javac")
        .arg(&file)
        .output()
        .expect("JDK javac");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = std::process::Command::new("java")
        .arg("-cp")
        .arg(&directory)
        .arg("CanvasHarness")
        .output()
        .expect("JDK java");
    std::fs::remove_dir_all(directory).unwrap();
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
}

#[test]
#[ignore = "writes generated Android native validation artifacts"]
fn draw_android_native_validation_artifacts() {
    let output = generate_android(
        &[canvas_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.dowe/android-draw-native");
    for file in output.files {
        let path = directory.join(file.relative_path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, file.content).unwrap();
    }
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
    assert!(views.content.contains("donutWidth: Int = 60"));
    assert!(views.content.contains("aspectRatio(1f)"));
    assert!(views.content.contains("DoweChartLegend"));
    assert!(views.content.contains("doweDrawArcChart(categories, palette, contentColor, backgroundColor, thickness, gap, startAngle, endAngle.toFloat()"));
    assert!(
        views
            .content
            .contains("backgroundColor.copy(alpha = 0.94f), RoundedCornerShape(999.dp)")
    );
    assert!(
        views
            .content
            .contains("drawRoundRect(clampedLeft - horizontalPadding")
    );
    assert!(views.content.contains("centerText = \"Share\""));
    assert!(views.content.contains("showInlineLabels = true"));
    assert!(
        views
            .content
            .contains("DoweChart(state = state, chartType = \"arc\", dataPath = \"segments\"")
    );
    assert!(
        views
            .content
            .contains("DoweChart(state = state, chartType = \"line\", dataPath = \"points\"")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("private DoweChartView doweChart("));
    assert!(dev.content.contains("DoweChartView"));
    assert!(dev.content.contains("doweDrawPieChart"));
    assert!(dev.content.contains("int donutWidth"));
    assert!(dev.content.contains("float ringWidth"));
    assert!(dev.content.contains("doweDrawArcChart(canvas, categories"));
    assert!(dev.content.contains("doweDrawChartCenterBadge"));
    assert!(dev.content.contains("doweAlpha(backgroundColor, 0.94f)"));
    assert!(dev.content.contains("int thickness"));
    assert!(dev.content.contains("Float max"));
}
