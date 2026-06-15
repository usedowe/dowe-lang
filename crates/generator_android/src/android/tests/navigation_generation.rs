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
    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
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

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
    assert!(dev.content.contains("private LinearLayout doweCode("));
    assert!(dev.content.contains("ClipboardManager clipboard"));
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
    assert!(views.content.contains("MediaController(context)"));
    assert!(
        views
            .content
            .contains("https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8")
    );
    assert!(views.content.contains("poster = \"/images/video.jpg\""));
    assert!(views.content.contains("aspect = \"vertical\""));

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
    assert!(dev.content.contains("private FrameLayout doweVideo("));
    assert!(
        dev.content
            .contains("VideoView video = new VideoView(this)")
    );
    assert!(
        dev.content
            .contains("MediaController controls = new MediaController(this)")
    );
    assert!(
        dev.content
            .contains("https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8")
    );
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

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
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

    let dev = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev");
    assert!(
        dev.content
            .contains("private DoweChartView doweChart(")
    );
    assert!(dev.content.contains("DoweChartView"));
    assert!(dev.content.contains("doweDrawPieChart"));
}
