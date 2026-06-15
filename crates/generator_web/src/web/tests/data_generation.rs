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
    assert!(chunk.css_content.contains(".code-block.is-soft.is-surface"));
    assert!(
        super::router_js(&super::WebOutput {
            chunks: Vec::new(),
            pages: Vec::new(),
            translation_chunks: Vec::new(),
            default_locale: None,
            router_js: String::new(),
        })
        .contains("navigator.clipboard")
    );
}

#[test]
fn renders_video_markup_theme_classes_and_hls_runtime() {
    let tree = video_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains("data-dowe-video"));
    assert!(html.contains("controls playsinline"));
    assert!(html.contains(r#"poster="/images/video.jpg""#));
    assert!(html.contains(r#"class="video horizontal is-solid is-surface""#));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/video.dowe"),
        "page videoPage",
        &tree,
    );
    assert!(chunk.css_content.contains(".video.is-solid.is-surface"));
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });
    assert!(router.contains("application/vnd.apple.mpegurl"));
    assert!(router.contains("https://cdn.jsdelivr.net/npm/hls.js@1/dist/hls.min.js"));
    assert!(router.contains("hls.loadSource(source)"));
}

#[test]
fn renders_candlestick_markup_theme_classes_and_stream_runtime() {
    let tree = candlestick_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains("data-dowe-candlestick"));
    assert!(html.contains(r#"data-dowe-candlestick-data="candles""#));
    assert!(html.contains(r#"data-dowe-candlestick-stream="/api/candles""#));
    assert!(html.contains(r#"class="candlestick"#));
    assert!(html.contains("is-soft"));
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
            .contains(".candlestick.is-soft.is-surface")
    );
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });
    assert!(router.contains("new EventSource(stream)"));
    assert!(router.contains("upsertCandles"));
    assert!(router.contains("renderCandlestick"));
}

#[test]
fn renders_chart_markup_css_and_runtime() {
    let tree = charts_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    for chart_type in ["arc", "area", "bar", "line", "pie"] {
        assert!(html.contains(&format!(r#"data-dowe-chart-type="{chart_type}""#)));
    }
    assert!(html.contains(r#"data-dowe-chart-data="segments""#));
    assert!(html.contains(r#"data-dowe-chart-data="points""#));
    assert!(html.contains("dowe-chart-svg"));
    assert!(html.contains("dowe-chart-legend"));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/charts.dowe"),
        "page chartsPage",
        &tree,
    );
    assert!(chunk.css_content.contains(".arc-chart-container"));
    assert!(chunk.css_content.contains(".line-chart-container"));
    assert!(chunk.css_content.contains(".dowe-chart-svg"));
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });
    assert!(router.contains("function renderCharts"));
    assert!(router.contains("renderPieArcChart"));
    assert!(router.contains("renderLineAreaChart"));
}

#[test]
fn renders_table_markup_css_and_runtime() {
    let tree = table_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(
        html.contains(
            r#"class="table is-lg is-soft is-surface is-striped is-bordered has-dividers""#
        )
    );
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
    assert!(chunk.css_content.contains(".table.is-soft.is-surface"));
    assert!(chunk.css_content.contains(".table-container"));
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });
    assert!(router.contains("renderTable"));
    assert!(router.contains("tableCellValue"));
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

#[test]
fn emits_view_motion_markup_and_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Box {
        props: StyleProps {
            animation: Some(ViewAnimation::FadeIn),
            ..Default::default()
        },
        children: vec![ViewNode::Card {
            props: VariantProps {
                style: StyleProps {
                    animation: Some(ViewAnimation::SlideUp),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Motion")],
        }],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/motion.dowe"),
        "page",
        &page_tree,
    );
    let css = super::design_css();

    assert!(page.content.contains("animate-fade-in"));
    assert!(page.content.contains("animate-slide-up"));
    assert!(
        page.css_content
            .contains(".animate-fade-in{animation:dowe-fade-in 220ms ease-out both;}")
    );
    assert!(
        page.css_content
            .contains(".animate-slide-up{animation:dowe-slide-up 220ms ease-out both;}")
    );
    assert!(css.contains("@keyframes dowe-fade-in"));
    assert!(css.contains("@media (prefers-reduced-motion:reduce)"));
}

#[test]
fn emits_button_size_and_variant_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Button {
        props: VariantProps {
            variant: Some(ComponentVariant::Soft),
            color: Some(ColorFamily::Warning),
            size: Some(ButtonSize::Lg),
            style: StyleProps {
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Warn")],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("button-lg"));
    assert!(page.content.contains("rounded-full"));
    assert!(page.content.contains("is-soft"));
    assert!(page.content.contains("is-warning"));
    assert!(
        page.css_content
            .contains(".button-lg{padding:0.75rem 1.25rem;min-height:2.75rem;}")
    );
    assert!(
        page.css_content
            .contains(".rounded-full{border-radius:9999px;}")
    );
    assert!(page.css_content.contains(".button.is-soft.is-warning"));
}

#[test]
fn emits_text_weight_override_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Text {
        props: TextProps {
            weight: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: TextWeight::Thin,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: TextWeight::Extralight,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Lg,
                    value: TextWeight::Black,
                },
            ])),
            ..Default::default()
        },
        value: "Weight".to_string(),
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("weight-thin"));
    assert!(page.content.contains("md:weight-extralight"));
    assert!(page.content.contains("lg:weight-black"));
    assert!(page.css_content.contains(".weight-thin{font-weight:100;}"));
    assert!(
        page.css_content
            .contains(".md\\:weight-extralight{font-weight:200;}")
    );
    assert!(
        page.css_content
            .contains(".lg\\:weight-black{font-weight:900;}")
    );
}

#[test]
fn emits_show_visibility_markup_and_css() {
    let root = Path::new("/project");
    let page_tree = show_tree();
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/ready.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("show-false md:show-true"));
    assert!(page.content.contains(r#"data-dowe-show=\"ready01\""#));
    assert!(!page.css_content.contains(".show-false{display:none;}"));
    assert!(!page.css_content.contains(".md\\:show-true"));
    let design_css = show_design_css();
    assert!(design_css.contains(".show-false{display:none;}"));
    assert!(design_css.contains(
        "@media (min-width:768px){.md\\:show-false{display:none;}.md\\:show-true{display:var(--dowe-component-display,revert);}}"
    ));
    assert!(
        super::router_js(&super::WebOutput {
            chunks: Vec::new(),
            pages: Vec::new(),
            translation_chunks: Vec::new(),
            default_locale: None,
            router_js: String::new(),
        })
        .contains("data-dowe-show")
    );
}

#[test]
fn keeps_nested_layout_visibility_rules_order_safe() {
    let root = Path::new("/project");
    let parent = build_layout_chunk(
        root,
        Path::new("/project/src/layouts/docs.dowe"),
        "docs",
        &show_tree(),
    );
    let child = build_layout_chunk(
        root,
        Path::new("/project/src/layouts/views.dowe"),
        "views",
        &ViewNode::Box {
            props: StyleProps {
                element: ElementProps {
                    show: Some(VisibilityCondition::Static(responsive_bool(&[
                        (Breakpoint::Xs, false),
                        (Breakpoint::Lg, true),
                    ]))),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Views")],
        },
    );
    let design_css = show_design_css();

    assert!(!parent.css_content.contains(".show-false"));
    assert!(!child.css_content.contains(".show-false"));
    assert!(design_css.contains(".show-false{display:none;}"));
    assert!(design_css.contains(".md\\:show-true{display:var(--dowe-component-display,revert);}"));
    assert!(design_css.contains(".lg\\:show-true{display:var(--dowe-component-display,revert);}"));
}
