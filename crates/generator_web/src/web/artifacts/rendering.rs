pub fn render_page_body(layout_tree: &ViewNode, page_tree: &ViewNode) -> String {
    render_page_body_with_inspector(layout_tree, page_tree, None, None)
}

pub fn render_page_body_with_inspector(
    layout_tree: &ViewNode,
    page_tree: &ViewNode,
    layout_inspector: Option<&ViewInspectorMap>,
    page_inspector: Option<&ViewInspectorMap>,
) -> String {
    let page_html = render_html_with_inspector(page_tree, None, page_inspector);
    render_html_with_inspector(layout_tree, Some(&page_html), layout_inspector)
}

pub fn render_routed_page_body(
    layout_tree: &ViewNode,
    page_tree: &ViewNode,
    layout_chunk_ids: &[String],
    page_chunk_id: &str,
) -> String {
    render_routed_page_body_with_inspector(
        layout_tree,
        page_tree,
        layout_chunk_ids,
        page_chunk_id,
        None,
        None,
    )
}

pub fn render_routed_page_body_with_inspector(
    layout_tree: &ViewNode,
    page_tree: &ViewNode,
    layout_chunk_ids: &[String],
    page_chunk_id: &str,
    layout_inspector: Option<&ViewInspectorMap>,
    page_inspector: Option<&ViewInspectorMap>,
) -> String {
    let page_html = format!(
        r#"<div data-dowe-boundary="page:{page_chunk_id}">{}</div>"#,
        render_html_with_inspector(page_tree, None, page_inspector)
    );
    let body = render_html_with_inspector(layout_tree, Some(&page_html), layout_inspector);

    if let Some(layout_chunk_id) = layout_chunk_ids.first() {
        format!(r#"<div data-dowe-boundary="layout:{layout_chunk_id}">{body}</div>"#)
    } else {
        page_html
    }
}

pub fn render_page_document(page: &ViewPage) -> String {
    render_page_document_with_icons(page, None, None)
}

pub fn render_page_document_with_icons(
    page: &ViewPage,
    favicon: Option<&str>,
    apple_touch_icon: Option<&str>,
) -> String {
    let css_links = page
        .css_chunks
        .iter()
        .map(|path| {
            let path = escape_attr(path);
            format!(r#"<link data-dowe-css="{path}" rel="stylesheet" href="/{path}">"#)
        })
        .collect::<String>();
    let chunk_scripts = page
        .js_chunks
        .iter()
        .map(|path| format!(r#"<script type="module" src="/{path}"></script>"#))
        .collect::<String>();
    let runtime_preloads = page
        .runtime_chunks
        .iter()
        .map(|path| format!(r#"<link rel="modulepreload" href="/{path}">"#))
        .collect::<String>();
    let theme_script = theme_bootstrap_script();
    let icon_links = icon_links(favicon, apple_touch_icon);
    let metadata = metadata_head(&page.metadata);

    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=5, viewport-fit=cover, interactive-widget=resizes-content">{icon_links}{metadata}{theme_script}<link data-dowe-design rel="stylesheet" href="/{}">{css_links}{runtime_preloads}<script data-dowe-router type="module" src="/{}"></script>{chunk_scripts}</head><body><div id="dowe-app" data-dowe-route="{}">{}</div></body></html>"#,
        escape_attr(&page.design_file_name),
        escape_attr(&page.router_file_name),
        escape_attr(&page.route_path),
        page.body_html
    )
}

fn metadata_head(metadata: &[ViewMetadata]) -> String {
    let mut head = String::new();
    let mut has_title = false;
    for entry in metadata {
        match entry.name.as_str() {
            "title" => {
                has_title = true;
                head.push_str(&format!(
                    r#"<title data-dowe-meta>{}</title>"#,
                    escape_html_text(&entry.content)
                ));
            }
            "canonical" => head.push_str(&format!(
                r#"<link data-dowe-meta rel="canonical" href="{}">"#,
                escape_attr(&entry.content)
            )),
            name if name.starts_with("og:") => head.push_str(&format!(
                r#"<meta data-dowe-meta property="{}" content="{}">"#,
                escape_attr(name),
                escape_attr(&entry.content)
            )),
            name => head.push_str(&format!(
                r#"<meta data-dowe-meta name="{}" content="{}">"#,
                escape_attr(name),
                escape_attr(&entry.content)
            )),
        }
    }
    if !has_title {
        head.insert_str(0, "<title>Dowe</title>");
    }
    head
}

fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn icon_links(favicon: Option<&str>, apple_touch_icon: Option<&str>) -> String {
    let mut links = match favicon {
        Some(path) => format!(
            r#"<link rel="icon" type="image/png" sizes="32x32" href="{}">"#,
            escape_attr(path)
        ),
        None => r#"<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' rx='16' fill='%23111827'/%3E%3Cpath d='M18 16h12c12 0 20 6 20 16s-8 16-20 16H18zm10 8v16h3c7 0 11-3 11-8s-4-8-11-8z' fill='%23fff'/%3E%3C/svg%3E">"#.to_string(),
    };
    if let Some(path) = apple_touch_icon {
        links.push_str(&format!(
            r#"<link rel="apple-touch-icon" href="{}">"#,
            escape_attr(path)
        ));
    }
    links
}

fn theme_bootstrap_script() -> &'static str {
    r#"<script>!function(){document.documentElement.classList.add("dowe-entrance-pending");try{var k="theme-preference",t=localStorage.getItem(k);if(!t){t=window.matchMedia&&window.matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light";localStorage.setItem(k,t)}if(t&&t!=="light")document.documentElement.setAttribute("data-dowe-theme",t);else document.documentElement.removeAttribute("data-dowe-theme")}catch(e){}}();</script>"#
}

