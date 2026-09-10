#[test]
fn incremental_artifacts_use_safe_names_when_prepared_names_are_empty() {
    let web = super::WebOutput {
        chunks: Vec::new(),
        pages: vec![Arc::new(super::ViewPage {
            id: "index".to_string(),
            route_path: "/".to_string(),
            source_path: Path::new("/project/views/pages/index.dowe").to_path_buf(),
            layout_tree: ViewNode::Children,
            page_tree: text("Index"),
            body_html: "<p>Index</p>".to_string(),
            html_document: String::new(),
            layout_text: String::new(),
            page_text: "Index".to_string(),
            layout_chunk_id: String::new(),
            page_chunk_id: String::new(),
            layout_chunk_ids: Vec::new(),
            js_chunks: Vec::new(),
            css_chunks: Vec::new(),
            runtime_chunks: Vec::new(),
            design_file_name: String::new(),
            router_file_name: String::new(),
            boundaries: Vec::new(),
            sections: Vec::new(),
            navigation_actions: Vec::new(),
            metadata: Vec::new(),
        })],
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: "export {}".to_string(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    };

    assert_eq!(web.design_file_name(), "design.css");
    let update = super::web_artifact_update(&web, None, String::new());

    assert!(
        update
            .files
            .iter()
            .any(|file| file.relative_path == Path::new("web/design.css"))
    );
    assert!(
        update
            .files
            .iter()
            .any(|file| file.relative_path == Path::new("web/router.js"))
    );
    assert!(
        !update
            .files
            .iter()
            .any(|file| file.relative_path == Path::new("web"))
    );
}

#[test]
fn incremental_artifacts_publish_each_active_page_design_name() {
    let index = super::ViewPage {
        id: "index".to_string(),
        route_path: "/".to_string(),
        source_path: Path::new("/project/views/pages/index.dowe").to_path_buf(),
        layout_tree: ViewNode::Children,
        page_tree: text("Index"),
        body_html: "<p>Index</p>".to_string(),
        html_document: String::new(),
        layout_text: String::new(),
        page_text: "Index".to_string(),
        layout_chunk_id: String::new(),
        page_chunk_id: String::new(),
        layout_chunk_ids: Vec::new(),
        js_chunks: Vec::new(),
        css_chunks: Vec::new(),
        runtime_chunks: Vec::new(),
        design_file_name: "design.css".to_string(),
        router_file_name: "router.js".to_string(),
        boundaries: Vec::new(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
        metadata: Vec::new(),
    };
    let mut login = index.clone();
    login.id = "login".to_string();
    login.route_path = "/login".to_string();
    login.source_path = Path::new("/project/views/pages/login.dowe").to_path_buf();
    login.design_file_name = "design-hot-reload.css".to_string();
    let previous = super::WebOutput {
        chunks: Vec::new(),
        pages: vec![Arc::new(index.clone())],
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: "export {}".to_string(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    };
    let web = super::WebOutput {
        chunks: Vec::new(),
        pages: vec![Arc::new(index), Arc::new(login)],
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: "export {}".to_string(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    };

    let update = super::web_artifact_update(&web, Some(&previous), "body{}".to_string());

    for path in ["web/design.css", "web/design-hot-reload.css"] {
        assert!(
            update
                .expected_paths
                .iter()
                .any(|expected| expected == Path::new(path)),
            "missing {path}"
        );
    }
    assert!(
        update
            .files
            .iter()
            .any(|file| file.relative_path == Path::new("web/design-hot-reload.css"))
    );
}

#[test]
fn incremental_design_preparation_repairs_reused_page_capability_styles() {
    let page = super::ViewPage {
        id: "index".to_string(),
        route_path: "/".to_string(),
        source_path: Path::new("/project/views/pages/index.dowe").to_path_buf(),
        layout_tree: navigation_shell_tree(),
        page_tree: text("Home"),
        body_html: "<p>Home</p>".to_string(),
        html_document: String::new(),
        layout_text: String::new(),
        page_text: "Home".to_string(),
        layout_chunk_id: "layout".to_string(),
        page_chunk_id: "page".to_string(),
        layout_chunk_ids: vec!["layout".to_string()],
        js_chunks: vec![
            "chunks/layouts/layout.js".to_string(),
            "chunks/pages/page.js".to_string(),
        ],
        css_chunks: vec![
            "chunks/layouts/layout.css".to_string(),
            "chunks/pages/page.css".to_string(),
        ],
        runtime_chunks: vec!["chunks/runtime/controls.js".to_string()],
        design_file_name: "design.css".to_string(),
        router_file_name: "router.js".to_string(),
        boundaries: vec!["layout:layout".to_string(), "page:page".to_string()],
        sections: Vec::new(),
        navigation_actions: Vec::new(),
        metadata: Vec::new(),
    };
    let mut initial = super::WebOutput {
        chunks: Vec::new(),
        pages: vec![Arc::new(page)],
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    };
    super::prepare_dev_design_asset(
        &mut initial,
        &FontConfig::default(),
        &DesignConfig::default(),
    );

    let mut stale_page = initial.pages[0].as_ref().clone();
    stale_page
        .css_chunks
        .retain(|path| !path.starts_with("chunks/design/"));
    stale_page.html_document = super::render_page_document(&stale_page);
    let previous = super::WebOutput {
        chunks: Vec::new(),
        pages: vec![Arc::new(stale_page)],
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: initial.router_js.clone(),
        render_report: initial.render_report.clone(),
    };
    let mut web = previous.clone();

    let design_css = super::prepare_incremental_dev_design_asset(
        &mut web,
        &previous,
        &FontConfig::default(),
        &DesignConfig::default(),
    );
    let navigation_css = web.pages[0]
        .css_chunks
        .iter()
        .find(|path| path.starts_with("chunks/design/navigation-"))
        .expect("navigation capability CSS");

    assert!(web.pages[0].html_document.contains(&format!(
        r#"data-dowe-css="{navigation_css}" rel="stylesheet" href="/{navigation_css}""#
    )));
    assert!(super::manifest(&web).contains(navigation_css));

    let update = super::web_artifact_update(&web, Some(&previous), design_css);
    assert!(
        update
            .files
            .iter()
            .any(|file| file.relative_path == Path::new("web/index.html"))
    );
}

