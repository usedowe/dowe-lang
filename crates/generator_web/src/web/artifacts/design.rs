pub fn prepare_design_asset(
    web: &mut WebOutput,
    font_config: &FontConfig,
    design_config: &DesignConfig,
) -> String {
    prepare_design_asset_with_router(web, None, font_config, design_config, false)
}

pub fn prepare_dev_design_asset(
    web: &mut WebOutput,
    font_config: &FontConfig,
    design_config: &DesignConfig,
) -> String {
    prepare_design_asset_with_router(web, None, font_config, design_config, true)
}

pub fn prepare_incremental_dev_design_asset(
    web: &mut WebOutput,
    previous: &WebOutput,
    font_config: &FontConfig,
    design_config: &DesignConfig,
) -> String {
    prepare_design_asset_with_router(web, Some(previous), font_config, design_config, true)
}

fn prepare_design_asset_with_router(
    web: &mut WebOutput,
    previous: Option<&WebOutput>,
    font_config: &FontConfig,
    design_config: &DesignConfig,
    stable_router: bool,
) -> String {
    let css = design_css_for_web(web, font_config, design_config);
    let file_name = design_css_file_name(&css);
    for page in &mut web.pages {
        let mut css_chunks = design_css_chunks(DesignCssFeatures::collect([
            &page.layout_tree,
            &page.page_tree,
        ]))
        .iter()
        .map(GeneratedDesignCssChunk::browser_path)
        .collect::<Vec<_>>();
        css_chunks.extend(
            page.css_chunks
                .iter()
                .filter(|path| !path.starts_with("chunks/design/"))
                .cloned(),
        );
        if page.css_chunks != css_chunks {
            Arc::make_mut(page).css_chunks = css_chunks;
        }
    }
    if stable_router {
        if let Some(previous) = previous {
            web.router_js.clone_from(&previous.router_js);
        } else {
            web.router_js = router_js(web);
        }
    } else {
        web.router_js = router_js(web);
    }
    let router_file_name = if stable_router {
        "router.js".to_string()
    } else {
        web.generated_router_file_name()
    };
    for page in &mut web.pages {
        if let Some(previous) = reusable_prepared_page(previous, page)
            && previous.design_file_name == file_name
            && previous.router_file_name == router_file_name
            && page_document_has_asset_references(page, &file_name, &router_file_name)
        {
            continue;
        }
        let page = Arc::make_mut(page);
        page.design_file_name.clone_from(&file_name);
        page.router_file_name.clone_from(&router_file_name);
        page.html_document = render_page_document(page);
    }
    css
}

fn page_document_has_asset_references(
    page: &ViewPage,
    design_file_name: &str,
    router_file_name: &str,
) -> bool {
    let design_reference = format!(
        r#"<link data-dowe-design rel="stylesheet" href="/{}">"#,
        escape_attr(design_file_name)
    );
    let router_reference = format!(
        r#"<script data-dowe-router type="module" src="/{}"></script>"#,
        escape_attr(router_file_name)
    );
    let css_references = page
        .css_chunks
        .iter()
        .map(|path| {
            let path = escape_attr(path);
            format!(r#"<link data-dowe-css="{path}" rel="stylesheet" href="/{path}">"#)
        })
        .collect::<String>();
    let runtime_preloads = page
        .runtime_chunks
        .iter()
        .map(|path| format!(r#"<link rel="modulepreload" href="/{path}">"#))
        .collect::<String>();
    page.html_document.contains(&format!(
        "{design_reference}{css_references}{runtime_preloads}{router_reference}"
    ))
}

fn reusable_prepared_page<'a>(
    previous: Option<&'a WebOutput>,
    page: &ViewPage,
) -> Option<&'a ViewPage> {
    previous?.pages.iter().map(Arc::as_ref).find(|candidate| {
        candidate.route_path == page.route_path
            && candidate.source_path == page.source_path
            && candidate.page_chunk_id == page.page_chunk_id
            && candidate.layout_chunk_ids == page.layout_chunk_ids
            && candidate.metadata == page.metadata
    })
}

