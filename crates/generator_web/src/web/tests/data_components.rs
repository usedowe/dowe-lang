#[test]
fn renders_code_markup_theme_classes_and_copy_runtime() {
    let tree = code_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains("data-dowe-code"));
    assert!(html.contains("data-dowe-code-copy"));
    assert!(html.contains("code-token-keyword"));
    assert!(html.contains("code-token-type"));
    assert!(html.contains("docsPage"));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/docs.dowe"),
        "page docsPage",
        &tree,
    );
    assert!(chunk.css_content.contains(".code-block.is-solid.is-surface"));
    assert!(
        super::router_js(&super::WebOutput {
            chunks: Vec::new(),
            pages: Vec::new(),
            translation_chunks: Vec::new(),
            default_locale: None,
            router_js: String::new(),
            render_report: dowe_components::RenderReport::new(dowe_components::RenderTarget::Web, Vec::new()),
        })
        .contains("navigator.clipboard")
    );
}

#[test]
fn renders_video_markup_theme_classes_and_hls_runtime() {
    let tree = video_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains("data-dowe-video"));
    assert!(html.contains("data-dowe-video-controls"));
    assert!(html.contains("data-dowe-video-play"));
    assert!(html.contains("data-dowe-video-mute"));
    assert!(html.contains("data-dowe-video-progress"));
    assert!(!html.contains(" controls playsinline"));
    assert!(html.contains(r#"poster="/images/video.jpg""#));
    assert!(html.contains(r#"class="video horizontal is-solid is-surface""#));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/video.dowe"),
        "page videoPage",
        &tree,
    );
    assert!(chunk.css_content.contains(".video.is-solid.is-surface"));
    let runtime = super::media_runtime_chunk().content;
    assert!(runtime.contains("application/vnd.apple.mpegurl"));
    assert!(runtime.contains("https://cdn.jsdelivr.net/npm/hls.js@1/dist/hls.min.js"));
    assert!(runtime.contains("hls.loadSource(source)"));
    assert!(runtime.contains("video.controls=false"));
    assert!(runtime.contains("requestPictureInPicture"));
    assert!(runtime.contains("requestFullscreen"));
}

#[test]
fn renders_iframe_markup_and_security_policy() {
    let tree = iframe_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains("<iframe"));
    assert!(html.contains(r#"src="https://example.com/embed""#));
    assert!(html.contains(r#"title="Example embed""#));
    assert!(html.contains(r#"loading="eager""#));
    assert!(html.contains(r#"allow="fullscreen; autoplay""#));
    assert!(html.contains(r#"sandbox="allow-scripts allow-same-origin""#));
    assert!(html.contains(" allowfullscreen"));
}

#[test]
fn renders_candlestick_markup_theme_classes_and_stream_runtime() {
    let tree = candlestick_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains("data-dowe-candlestick"));
    assert!(html.contains(r#"data-dowe-candlestick-data="candles""#));
    assert!(html.contains(r#"data-dowe-candlestick-stream="/api/candles""#));
    assert!(html.contains(r#"class="candlestick"#));
    assert!(html.contains("is-solid"));
    assert!(html.contains("is-surface"));
    assert!(html.contains("Market closed"));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/market.dowe"),
        "page marketPage",
        &tree,
    );
    assert!(
        chunk
            .css_content
            .contains(".candlestick.is-solid.is-surface")
    );
    let runtime = super::visualization_runtime_chunk().content;
    assert!(runtime.contains("new EventSource(stream)"));
    assert!(runtime.contains("upsertCandles"));
    assert!(runtime.contains("renderCandlestick"));
}

#[test]
fn renders_chart_markup_css_and_runtime() {
    let tree = charts_tree();
    let runtime_chunks = super::runtime_chunks_for_trees(&ViewNode::Children, &tree);
    assert_eq!(runtime_chunks.len(), 1);
    assert_eq!(runtime_chunks[0].name, "visualization");
    assert!(
        runtime_chunks[0]
            .browser_path()
            .starts_with("chunks/runtime/visualization-")
    );
    let html = render_page_body(&ViewNode::Children, &tree);
    for chart_type in ["arc", "area", "bar", "line", "pie"] {
        assert!(html.contains(&format!(r#"data-dowe-chart-type="{chart_type}""#)));
    }
    assert!(html.contains(r#"data-dowe-chart-data="segments""#));
    assert!(html.contains(r#"data-dowe-chart-data="points""#));
    assert!(html.contains("dowe-chart-svg"));
    assert!(html.contains("dowe-chart-legend"));
    assert!(html.contains(r#"preserveAspectRatio="xMidYMid meet""#));
    assert!(html.contains("data-dowe-chart-show-inline-labels=\"true\""));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/charts.dowe"),
        "page chartsPage",
        &tree,
    );
    assert!(chunk.css_content.contains(".arc-chart-container"));
    assert!(chunk.css_content.contains(".line-chart-container"));
    assert!(chunk.css_content.contains(".dowe-chart-svg"));
    let design_css = super::design_css();
    assert!(design_css.contains(".arc-chart-container .dowe-chart-arc-viewport"));
    assert!(design_css.contains(".dowe-chart-inline-label"));
    let runtime = super::visualization_runtime_chunk().content;
    assert!(runtime.contains("function renderCharts"));
    assert!(runtime.contains("renderPieArcChart"));
    assert!(runtime.contains("renderLineAreaChart"));
    assert!(runtime.contains("chart.dataset.doweChartDonut"));
    assert!(runtime.contains("chart.dataset.doweChartHideLabels"));
    assert!(runtime.contains("Math.abs(sweep)>=359.999"));
    assert!(runtime.contains("chart.dataset.doweChartShowInlineLabels"));
}

#[test]
fn renders_table_markup_css_and_runtime() {
    let tree = table_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains(
        r#"class="table is-lg is-outlined is-primary is-striped is-bordered has-dividers""#
    ));
    assert!(html.contains(r#"data-dowe-table-data="users""#));
    assert!(html.contains(r#"data-dowe-table-field="status""#));
    assert!(html.contains(r#"data-dowe-table-align="end""#));
    assert!(html.contains(r#"style="text-align:end;width:8rem""#));
    assert!(html.contains("No users"));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/users.dowe"),
        "page usersPage",
        &tree,
    );
    assert!(chunk.css_content.contains(
        ".table.is-outlined.is-primary{background-color:transparent;color:var(--dowe-primary);border:1px solid var(--dowe-primary);}"
    ));
    assert!(chunk.css_content.contains(".table-container"));
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(dowe_components::RenderTarget::Web, Vec::new()),
    });
    assert!(router.contains("renderTable"));
    assert!(router.contains("tableCellValue"));
}

#[test]
fn renders_tree_markup_css_and_runtime() {
    let tree = tree_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains(r#"class="tree is-ghost is-surface""#));
    assert!(html.contains(r#"role="tree""#));
    assert!(html.contains(r#"aria-label="Application files""#));
    assert!(html.contains(r#"data-dowe-tree-data="fileTree""#));
    assert!(html.contains(r#"data-dowe-tree-bind="selectedFile""#));
    assert!(html.contains(r#"data-dowe-tree-on-select="selectFile""#));
    assert!(html.contains(r#"data-dowe-tree-content"#));
    assert!(html.contains("data-dowe-tree-icon=\"folder\""));
    assert!(html.contains("data-dowe-tree-icon=\"file\""));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/tree.dowe"),
        "page treePage",
        &tree,
    );
    assert!(chunk.css_content.contains(".tree.is-ghost.is-surface"));
    assert!(super::design_css().contains(".tree-row"));
    let runtime_chunks = super::runtime_chunks_for_trees(&ViewNode::Children, &tree);
    assert_eq!(runtime_chunks.iter().map(|chunk| chunk.name).collect::<Vec<_>>(), ["tree"]);
    let runtime = super::tree_runtime_chunk().content;
    assert!(runtime.contains("function renderTrees"));
    assert!(runtime.contains("function updateTreeSelection"));
    assert!(runtime.contains("doweTreeRenderedData"));
    assert!(runtime.contains("function treeNode"));
    assert!(runtime.contains("data-dowe-tree-toggle"));
    assert!(runtime.contains("wrapper.dataset.doweTreeNode=\"\""));
    assert_javascript_syntax(&runtime);
}

#[test]
fn renders_divider_markup_orientation_and_scheme_css() {
    let tree = divider_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains(r#"class="divider divider-vertical is-primary""#));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/divider.dowe"),
        "page dividerPage",
        &tree,
    );
    assert!(super::design_css().contains(".divider{--dowe-component-display:block;"));
    assert!(chunk.css_content.contains(
        ".divider.is-primary{background-color:var(--dowe-primary);color:var(--dowe-primary);}"
    ));
}

