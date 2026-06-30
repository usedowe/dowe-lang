#[test]
fn creates_stable_chunk_ids() {
    let root = Path::new("/project");
    let source = "page loginPage\n  Text\n    Login";
    let page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![text("Login")],
    };
    let first = build_page_chunk(
        root,
        Path::new("/project/src/pages/login.dowe"),
        source,
        &page_tree,
    );
    let second = build_page_chunk(
        root,
        Path::new("/project/src/pages/login.dowe"),
        source,
        &page_tree,
    );

    assert_eq!(first.id, second.id);
    assert_eq!(first.id.len(), 8);
    assert!(
        first
            .id
            .chars()
            .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit())
    );
}

#[test]
fn creates_locale_chunks_and_browser_translation_runtime() {
    let catalog = translations();
    let first = build_translation_chunks(Path::new("/project"), &catalog);
    let second = build_translation_chunks(Path::new("/project"), &catalog);
    assert_eq!(first, second);
    assert_eq!(first.len(), 2);
    assert!(first[0].relative_path.starts_with("web/chunks/i18n"));
    assert!(
        first
            .iter()
            .any(|chunk| chunk.content.contains("Dowe construye sistemas."))
    );

    let tree = ViewNode::Title {
        props: TextProps {
            i18n: Some("home.hero.title".to_string()),
            ..Default::default()
        },
        value: "Dowe builds systems.".to_string(),
    };
    assert!(
        render_page_body(&ViewNode::Children, &tree)
            .contains(r#"data-dowe-i18n="home.hero.title""#)
    );

    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: first,
        default_locale: Some("en".to_string()),
        router_js: String::new(),
    });
    assert!(router.contains("navigator.languages"));
    assert!(router.contains("localeChunks"));
    assert!(router.contains("hydrateTranslations"));
}

#[test]
fn separates_layout_and_page_chunks() {
    let root = Path::new("/project");
    let layout_tree = layout_tree();
    let page_tree = page_tree();
    let layout = build_layout_chunk(
        root,
        Path::new("/project/src/layouts/auth.dowe"),
        "layout",
        &layout_tree,
    );
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/login.dowe"),
        "page",
        &page_tree,
    );

    assert_eq!(layout.kind, ChunkKind::Layout);
    assert_eq!(page.kind, ChunkKind::Page);
    assert_ne!(layout.relative_path, page.relative_path);
}

#[test]
fn renders_box_and_text_as_div_and_paragraph() {
    assert_eq!(
        render_page_body(&layout_tree(), &page_tree()),
        r#"<div class="box"><p class="text-md">Layout</p><div class="box"><p class="text-md">Login</p></div></div>"#
    );
}

#[test]
fn renders_section_markup_and_background_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Section {
        props: StyleProps {
            text: Some(ResponsiveValue::scalar(ColorToken::OnBackground)),
            background: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: SectionBackground::Soft,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: SectionBackground::Aurora,
                },
            ])),
            ..Default::default()
        },
        children: vec![text("Hero")],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    let html = render_page_body(&ViewNode::Children, &page_tree);

    assert!(html.contains("<section"));
    assert!(html.contains(
        "section color-onBackground has-background background-soft md:background-aurora"
    ));
    assert!(page.css_content.contains(
        "background-image:linear-gradient(135deg,var(--dowe-surface),var(--dowe-background));"
    ));
    assert!(page.css_content.contains("background-image:linear-gradient(135deg,var(--dowe-softPrimary),var(--dowe-softSecondary),var(--dowe-softTertiary));"));
    assert!(page.css_content.contains("@media (min-width:768px)"));
}

#[test]
fn scopes_layout_and_page_reactivity_by_generated_id() {
    let root = Path::new("/project");
    let layout_tree = reactive_tree("layout01", "action01", true);
    let page_tree = reactive_tree("page0001", "action02", false);
    let layout = build_layout_chunk(
        root,
        Path::new("/project/src/layouts/auth.dowe"),
        "layout reactive",
        &layout_tree,
    );
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/login.dowe"),
        "page reactive",
        &page_tree,
    );
    let html = render_page_body(&layout_tree, &page_tree);

    assert!(layout.content.contains("doweLayout"));
    assert!(layout.content.contains(r#""id":"layout01""#));
    assert!(layout.content.contains(r#""target":"layout01""#));
    assert!(page.content.contains(r#""id":"page0001""#));
    assert!(page.content.contains(r#""target":"page0001""#));
    assert!(html.contains(r#"data-dowe-bind="layout01.message""#));
    assert!(html.contains(r#"data-dowe-bind="page0001.message""#));
    assert!(html.contains(r#"data-dowe-click="action01""#));
    assert!(html.contains(r#"data-dowe-click="action02""#));
}

#[test]
fn emits_web_manifest_and_html_artifacts() {
    let root = Path::new("/project");
    let layout_tree = layout_tree();
    let page_tree = page_tree();
    let layout = build_layout_chunk(
        root,
        Path::new("/project/src/layouts/auth.dowe"),
        "layout",
        &layout_tree,
    );
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/login.dowe"),
        "page",
        &page_tree,
    );
    let layout_js = strip_web_for_test(&layout.relative_path);
    let page_js = strip_web_for_test(&page.relative_path);
    let layout_css = strip_web_for_test(&layout.css_relative_path);
    let page_css = strip_web_for_test(&page.css_relative_path);
    let body_html =
        super::render_routed_page_body(&layout_tree, &page_tree, &[layout.id.clone()], &page.id);
    let mut view_page = super::ViewPage {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        source_path: Path::new("/project/src/pages/login.dowe").to_path_buf(),
        layout_tree,
        page_tree,
        body_html,
        html_document: String::new(),
        layout_text: "Layout".to_string(),
        page_text: "Login".to_string(),
        layout_chunk_id: layout.id.clone(),
        page_chunk_id: page.id.clone(),
        layout_chunk_ids: vec![layout.id.clone()],
        js_chunks: vec![layout_js, page_js],
        css_chunks: vec![layout_css, page_css],
        boundaries: vec![format!("layout:{}", layout.id), format!("page:{}", page.id)],
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    view_page.html_document = super::render_page_document(&view_page);
    assert!(
        view_page
            .html_document
            .contains(r#"<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=5, viewport-fit=cover, interactive-widget=resizes-content">"#)
    );
    let mut web = super::WebOutput {
        chunks: vec![layout, page],
        pages: vec![view_page],
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    };
    web.router_js = super::router_js(&web);
    let artifacts = web_artifacts(&web, &FontConfig::default(), &DesignConfig::default());

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.relative_path == Path::new("web/manifest.json"))
    );
    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.relative_path == Path::new("web/pages/login.html"))
    );
    let index = artifacts
        .iter()
        .find(|artifact| artifact.relative_path == Path::new("web/index.html"))
        .expect("index");
    assert!(index.content.contains(r#"href="design.css""#));
    assert!(index.content.contains(r#"src="chunks/layouts/"#));
    let page = artifacts
        .iter()
        .find(|artifact| artifact.relative_path == Path::new("web/pages/login.html"))
        .expect("page");
    assert!(page.content.contains(r#"href="../design.css""#));
    assert!(page.content.contains(r#"src="../chunks/layouts/"#));
    assert!(web.pages[0].html_document.contains(r#"href="/design.css""#));
    assert!(super::manifest(&web).contains(r#""staticFile":"web/pages/login.html""#));
    assert!(web.router_js.contains("staticMode"));
    assert!(web.router_js.contains("doweHref"));
    assert!(web.router_js.contains("function positionSelect(control)"));
    assert!(
        web.router_js
            .contains("function mountSelectPopover(control)")
    );
    assert!(web.router_js.contains("document.body.appendChild(popover)"));
    assert!(web.router_js.contains("popover.__doweControl"));
    assert!(
        web.router_js
            .contains("const above=bottom<Math.min(height,224)&&top>bottom")
    );
    assert!(
        web.router_js
            .contains("scrollIntoView({behavior:reduce?\"auto\":\"smooth\",block:\"start\"})")
    );
    assert!(
        web.router_js
            .contains("new RegExp(\"^https?:/{2}\",\"i\").test(source)")
    );
    assert!(
        web.router_js
            .contains("const boundary=document.querySelector('[data-dowe-boundary^=\"page:\"]')")
    );
    assert!(
        web.router_js
            .contains("boundary.outerHTML=wrapPage(route,page.render())")
    );
    assert!(web.router_js.contains("history.pushState"));
}

#[test]
fn emits_container_refactor_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Grid {
        props: GridProps {
            columns: Some(ResponsiveValue::scalar(GridTracks::Count(3))),
            rows: Some(ResponsiveValue::scalar(GridTracks::Template(
                "100px auto".to_string(),
            ))),
            justify: Some(ResponsiveValue::scalar(GridAlignment::Center)),
            gap: Some(ResponsiveValue::scalar(GapValue::Pair(
                GapSize::Px(10),
                GapSize::Px(20),
            ))),
            ..Default::default()
        },
        children: vec![
            ViewNode::Box {
                props: StyleProps {
                    cover: Some(ResponsiveValue::ordered(vec![
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Xs,
                            value: CoverSource("/mobile.jpg".to_string()),
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: CoverSource("/desktop.jpg".to_string()),
                        },
                    ])),
                    overlay: Some(ResponsiveValue::scalar(OverlayPaint::BlackOpacity(
                        "0.6".to_string(),
                    ))),
                    grid_item: dowe_components::GridItemProps {
                        col_span: Some(ResponsiveValue::scalar(GridSpan(2))),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Hero")],
            },
            ViewNode::Card {
                props: VariantProps {
                    variant: Some(ComponentVariant::Soft),
                    color: Some(dowe_components::ColorFamily::Surface),
                    ..Default::default()
                },
                children: vec![text("Card")],
            },
        ],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("grid-cols-3"));
    assert!(page.content.contains("grid-justify-center"));
    assert!(page.content.contains("col-span-2"));
    assert!(page.content.contains("has-cover"));
    assert!(page.content.contains("has-overlay"));
    assert!(
        page.css_content
            .contains("grid-template-columns:repeat(3,minmax(0,1fr));")
    );
    assert!(page.css_content.contains("grid-template-rows:100px auto;"));
    assert!(page.css_content.contains("row-gap:10px;column-gap:20px;"));
    assert!(
        page.css_content
            .contains("background-image:url(\"/mobile.jpg\")")
    );
    assert!(page.css_content.contains("@media (min-width:768px)"));
    assert!(page.css_content.contains("rgba(0,0,0,0.6)"));
    assert!(page.css_content.contains(".card.is-soft.is-surface"));
}

#[test]
fn emits_reset_and_font_css() {
    let css = super::design_css();

    assert!(css.contains("body{margin:0;"));
    assert!(css.contains("p,h1,h2,h3,h4,h5,h6{margin:0;"));
    assert!(css.contains("a{color:inherit;text-decoration:inherit;}"));
    assert!(css.contains("button,input,textarea,select{font:inherit;color:inherit;margin:0;}"));
    assert!(css.contains("--dowe-font-inter"));
    assert!(css.contains("@font-face{font-family:\"Dowe Inter\""));
    assert!(css.contains("src:url(\"/fonts/inter/inter-regular.ttf\") format(\"truetype\")"));
}

#[test]
fn rewrites_static_route_hrefs_for_desktop_fallback() {
    let document = r##"<a class="button" href="/signup#join" data-dowe-nav="push" data-dowe-href="/signup#join">Signup</a><a class="button" href="/" data-dowe-nav="push" data-dowe-href="/">Home</a><link rel="stylesheet" href="/design.css">"##;
    let index = super::static_html_document(document, "");
    let page = super::static_html_document(document, "../");

    assert!(index.contains(
        r##"href="pages/signup.html#join" data-dowe-nav="push" data-dowe-href="/signup#join""##
    ));
    assert!(index.contains(r##"href="index.html" data-dowe-nav="push" data-dowe-href="/""##));
    assert!(index.contains(r#"href="design.css""#));
    assert!(page.contains(
        r##"href="signup.html#join" data-dowe-nav="push" data-dowe-href="/signup#join""##
    ));
    assert!(page.contains(r##"href="../index.html" data-dowe-nav="push" data-dowe-href="/""##));
    assert!(page.contains(r#"href="../design.css""#));
}
