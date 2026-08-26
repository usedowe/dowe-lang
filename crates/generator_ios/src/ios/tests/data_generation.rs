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
    assert!(views.contains("DoweCodeView(source: \"page docsPage\\n  Card variant:\\\"solid\\\" p:4 show:true\\n    Text\\n      Documentation\""));
    assert!(views.contains("DoweDesign.primary"));
    assert!(views.contains("DoweDesign.info"));
    assert!(views.contains("DoweDesign.accent"));
    assert!(views.contains("DoweDesign.success"));
    assert!(views.contains("DoweDesign.warning"));
    assert!(views.contains("DoweDesign.danger"));
    assert!(views.contains(".fixedSize(horizontal: true, vertical: true)"));
    assert!(views.contains(".frame(minWidth: 1, minHeight: 1, alignment: .leading)"));
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

    assert!(views.contains("import AVFoundation"));
    assert!(views.contains("struct DoweVideoView: View"));
    assert!(views.contains("struct DoweVideoSurface: UIViewRepresentable"));
    assert!(views.contains("AVPlayerLayer"));
    assert!(!views.contains("VideoPlayer(player: player)"));
    assert!(views.contains("DoweVideoControls("));
    assert!(views.contains("icons.pictureInPicture"));
    assert!(views.contains("icons.fullscreen"));
    assert!(views.contains("AVPictureInPictureController"));
    assert!(views.contains("startPictureInPicture(attempts:"));
    assert!(views.contains("DispatchQueue.main.asyncAfter"));
    assert!(!views.contains("enabled: AVPictureInPictureController.isPictureInPictureSupported()"));
    assert!(views.contains(".fullScreenCover(isPresented: $fullscreen)"));
    assert!(views.contains("videoGravity = .resizeAspect"));
    assert!(views.contains("AVPlayer(url: URL(string: source)!)"));
    assert!(views.contains("https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8"));
    assert!(views.contains("poster: \"/images/video.jpg\""));
    assert!(views.contains("aspect: \"vertical\""));
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");
    assert!(plist.content.contains("<key>UIBackgroundModes</key>"));
    assert!(plist.content.contains("<string>audio</string>"));
}

#[test]
fn generates_swiftui_iframe_with_hardened_webview() {
    let output = generate_ios(&[iframe_route()], &FontConfig::default(), &DesignConfig::default(), &[]);
    let views = swift_content(&output);
    assert!(views.contains("import WebKit"));
    assert!(views.contains("struct DoweIframeView: UIViewRepresentable"));
    assert!(views.contains("url.scheme == \"https\""));
    assert!(views.contains("sandbox: Optional([\"scripts\", \"same-origin\"])"));
    assert!(views.contains("autoplay: true"));
}

#[test]
fn generates_swiftui_diagram_with_viewport_and_connection_runtime() {
    let output = generate_ios(
        &[diagram_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweDiagramView: View"));
    assert!(views.contains(
        "DoweDiagramView(state: state, nodesPath: \"flowNodes\", edgesPath: \"flowEdges\", fitView: true, panOnDrag: true, zoomOnScroll: true, controls: true, minimap: true"
    ));
    assert!(views.contains("func fitViewport(_ size: CGSize)"));
    assert!(views.contains("func borderPoint(_ node: [String: Any], toward: CGPoint) -> CGPoint"));
    assert!(views.contains("func edgeGeometry(_ source: [String: Any], _ target: [String: Any], type: String?)"));
    assert!(views.contains("private func isConnectionTarget(_ node: [String: Any]) -> Bool"));
    assert!(views.contains("StrokeStyle(lineWidth: 2, dash: [6, 4])"));
    assert!(views.contains("func persistConnection(source: String, target: String)"));
    assert!(views.contains("func moveNode(_ node: [String: Any], to point: CGPoint)"));
    assert!(views.contains("func moveViewport(minimapPoint: CGPoint, canvasSize: CGSize)"));
    assert!(views.contains("func diagramControlButton(_ label: String, action: @escaping () -> Void)"));
    assert!(views.contains("MagnificationGesture()"));
    assert!(views.contains("SpatialTapGesture(coordinateSpace: .named(\"doweDiagramCanvas\"))"));
    assert!(views.contains("state.run(onNodeClick, item: node)"));
    assert!(views.contains("state.run(onConnect, item: [\"source\": id, \"target\": targetId])"));
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
    assert!(views.contains("chartLayout(availableWidth: geometry.size.width)"));
    assert!(views.contains(
        "private func chartCanvas(availableWidth: CGFloat) -> some View {\n        let chartWidth = min(CGFloat(360), max(CGFloat(1), availableWidth))\n        return ZStack {"
    ));
    assert!(views.contains("let donutWidth: Int"));
    assert!(views.contains("centerText: nil, thickness: 16, gap: 8, endAngle: 270"));
    assert!(views.contains(".aspectRatio(chartType == \"pie\" || chartType == \"arc\""));
    assert!(views.contains("thickness: 18"));
    assert!(views.contains("showInlineLabels: true"));
    assert!(views.contains("drawArcChart(categories, context: &context, size: size)"));
    assert!(views.contains("let max: Double?"));
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
    assert!(views.contains("backgroundColor: Color.clear"));
    assert!(views.contains("contentColor: DoweDesign.primary"));
    assert!(views.contains("borderColor: Optional(DoweDesign.primary)"));
    assert!(views.contains(".background(DoweDesign.muted)"));
    assert!(views.contains("DoweDesign.surfaceText.opacity(0.12)"));
    assert!(views.contains("DoweDesign.surfaceText.opacity(0.28)"));
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
    assert!(views.contains(".padding(EdgeInsets(top: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(32)) ?? CGFloat(0), leading: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(32)) ?? CGFloat(0), bottom: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(32)) ?? CGFloat(0), trailing: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(32)) ?? CGFloat(0)))"));
    assert!(views.contains("top: doweResponsive(viewportWidth, md: CGFloat(32)) ?? CGFloat(0)"));
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
