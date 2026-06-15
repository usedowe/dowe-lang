#[test]
fn generates_swiftui_code_with_copy_and_theme_tokens() {
    let output = generate_ios(
        &[code_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweCodeView: View"));
    assert!(views.contains("UIPasteboard.general.string = source"));
    assert!(views.contains("DoweCodeView(source: \"page docsPage\\n  Card variant:\\\"soft\\\" p:4 show:true\\n    Text\\n      Documentation\""));
    assert!(views.contains("DoweDesign.primary"));
    assert!(views.contains("DoweDesign.info"));
    assert!(views.contains("DoweDesign.tertiary"));
    assert!(views.contains("DoweDesign.success"));
    assert!(views.contains("DoweDesign.warning"));
    assert!(views.contains("DoweDesign.danger"));
}

#[test]
fn generates_swiftui_video_with_native_hls_player() {
    let output = generate_ios(
        &[video_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("import AVKit"));
    assert!(views.contains("struct DoweVideoView: View"));
    assert!(views.contains("VideoPlayer(player: player)"));
    assert!(views.contains("AVPlayer(url: URL(string: source)!)"));
    assert!(views.contains("https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8"));
    assert!(views.contains("poster: \"/images/video.jpg\""));
    assert!(views.contains("aspect: \"vertical\""));
}

#[test]
fn generates_swiftui_candlestick_with_canvas_and_stream() {
    let output = generate_ios(
        &[candlestick_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweCandlestickView: View"));
    assert!(views.contains("Canvas { context, size in"));
    assert!(views.contains("URLSession.shared.bytes(from: url)"));
    assert!(
        views.contains("state.upsertCandles(dataPath, payload: payload, maxPoints: maxPoints)")
    );
    assert!(views.contains(
        "DoweCandlestickView(state: state, dataPath: \"candles\", stream: \"/api/candles\""
    ));
    assert!(views.contains("emptyLabel: \"Market closed\""));
}

#[test]
fn generates_swiftui_charts_with_canvas_runtime() {
    let output = generate_ios(
        &[charts_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweChartView: View"));
    assert!(views.contains("drawPointChart(series, context: &context, size: size)"));
    assert!(views.contains("drawPieChart(categories, context: &context, size: size)"));
    assert!(views.contains(
        "return DoweChartLegendItem(id: item.id, label: item.label, color: chartColor(index, explicit: item.color))"
    ));
    assert!(views.contains(
        "DoweChartView(state: state, chartType: \"arc\", dataPath: \"segments\""
    ));
    assert!(views.contains(
        "DoweChartView(state: state, chartType: \"line\", dataPath: \"points\""
    ));
}

#[test]
fn generates_swiftui_table_with_columns_and_scheme() {
    let output = generate_ios(
        &[table_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweTableView: View"));
    assert!(views.contains("DoweTableView(state: state, dataPath: \"users\""));
    assert!(views.contains(
        "DoweTableColumn(field: \"status\", label: \"Status\", align: .end, width: \"8rem\")"
    ));
    assert!(views.contains("size: .lg"));
    assert!(views.contains("striped: true, bordered: true, dividers: true"));
    assert!(views.contains("emptyTitle: \"No users\""));
    assert!(views.contains("backgroundColor: DoweDesign.surface"));
    assert!(views.contains("contentColor: DoweDesign.onSurface"));
    assert!(views.contains("state.rows(dataPath)"));
}

#[test]
fn generates_swiftui_divider_with_native_shape() {
    let output = generate_ios(
        &[divider_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("Rectangle()"));
    assert!(views.contains(".fill(DoweDesign.primary)"));
    assert!(views.contains(".frame(width: CGFloat(1))"));
    assert!(views.contains(".frame(maxHeight: .infinity)"));
}

#[test]
fn generates_swiftui_responsive_runtime_values() {
    let output = generate_ios(
        &[responsive_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("GeometryReader { geometry in"));
    assert!(views.contains("let viewportWidth: CGFloat"));
    assert!(views.contains(
        ".padding(doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(32)) ?? CGFloat(0))"
    ));
    assert!(
        views.contains(".padding(doweResponsive(viewportWidth, md: CGFloat(32)) ?? CGFloat(0))")
    );
    assert!(views.contains(
            ".font(doweFont(.inter, size: doweResponsive(viewportWidth, md: doweTextSize(viewportWidth, min: CGFloat(16), preferredBase: CGFloat(15.2), preferredViewport: CGFloat(0.3), max: CGFloat(18))) ?? doweTextSize(viewportWidth, min: CGFloat(14), preferredBase: CGFloat(13.12), preferredViewport: CGFloat(0.25), max: CGFloat(16))))"
        ));
    assert!(views.contains(
            ".fontWeight(doweResponsive(viewportWidth, xs: Font.Weight.ultraLight, md: Font.Weight.thin, lg: Font.Weight.black) ?? Font.Weight.regular)"
        ));
    assert!(!views.contains(".padding(16)\n"));
}

#[test]
fn generates_swiftui_show_visibility_conditions() {
    let output = generate_ios(
        &[show_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("if doweResponsive(viewportWidth, xs: false, md: true) ?? true {"));
    assert!(views.contains("if state.bool(\"ready01\") {"));
    assert!(views.contains("if state.bool(\"item.ready\", item: row.value) {"));
}
