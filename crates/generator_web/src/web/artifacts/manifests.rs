pub fn manifest(web: &WebOutput) -> String {
    let chunks = web
        .chunks
        .iter()
        .map(|chunk| {
            let kind = match chunk.kind {
                ChunkKind::Layout => "layout",
                ChunkKind::Page => "page",
            };

            format!(
                r#"{{"kind":"{kind}","id":"{}","file":"{}","source":"{}"}}"#,
                chunk.id,
                chunk.relative_path.display(),
                chunk.source_path.display()
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let routes = web
        .pages
        .iter()
        .map(|page| {
            let file_name = page_file_name(page);
            let layout_stack = page
                .layout_chunk_ids
                .iter()
                .map(|id| format!(r#""{id}""#))
                .collect::<Vec<_>>()
                .join(",");
            let js_chunks = page
                .js_chunks
                .iter()
                .map(|path| format!(r#""{path}""#))
                .collect::<Vec<_>>()
                .join(",");
            let css_chunks = page
                .css_chunks
                .iter()
                .map(|path| format!(r#""{path}""#))
                .collect::<Vec<_>>()
                .join(",");
            let runtime_chunks = page
                .runtime_chunks
                .iter()
                .map(|path| format!(r#""{}""#, escape_json(path)))
                .collect::<Vec<_>>()
                .join(",");
            let boundaries = page
                .boundaries
                .iter()
                .map(|boundary| format!(r#""{boundary}""#))
                .collect::<Vec<_>>()
                .join(",");
            let sections = page
                .sections
                .iter()
                .map(|section| format!(r#""{}""#, escape_json(&section.id)))
                .collect::<Vec<_>>()
                .join(",");
            let navigation_actions = page
                .navigation_actions
                .iter()
                .map(navigation_action_json)
                .collect::<Vec<_>>()
                .join(",");
            let metadata = metadata_json(&page.metadata);
            format!(
                r#"{{"id":"{}","path":"{}","layoutChunk":"{}","pageChunk":"{}","layoutStack":[{layout_stack}],"jsChunks":[{js_chunks}],"cssChunks":[{css_chunks}],"runtimeChunks":[{runtime_chunks}],"boundaries":[{boundaries}],"sections":[{sections}],"navigationActions":[{navigation_actions}],"metadata":{metadata},"staticFile":"web/pages/{file_name}.html"}}"#,
                escape_json(&page.id),
                escape_json(&page.route_path),
                escape_json(&page.layout_chunk_id),
                escape_json(&page.page_chunk_id)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let translation_chunks = web
        .translation_chunks
        .iter()
        .map(|chunk| {
            format!(
                r#"{{"locale":"{}","id":"{}","file":"{}","source":"{}"}}"#,
                escape_json(&chunk.locale),
                escape_json(&chunk.id),
                chunk.relative_path.display(),
                chunk.source_path.display()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let default_locale = json_optional_string(web.default_locale.as_deref());
    let design_css = escape_json(web.design_file_name());

    format!(
        r#"{{"chunks":[{chunks}],"translationChunks":[{translation_chunks}],"defaultLocale":{default_locale},"designCss":"{design_css}","routes":[{routes}],"history":{{"push":true,"replace":true,"back":true}},"externalPolicies":{{"web":["self","blank"],"desktop":["system","webview"],"android":["system","webview"],"ios":["system","webview"]}},"deepLinks":{{"scheme":"dowe-dev","routesFromManifest":true}}}}"#
    )
}

pub fn inspector_manifest(web: &WebOutput) -> String {
    let mut nodes = Vec::new();
    let mut seen = BTreeSet::new();
    for chunk in &web.chunks {
        let Some(inspector) = &chunk.inspector else {
            continue;
        };
        for node in &inspector.nodes {
            if seen.insert(node.id.clone()) {
                nodes.push(format!(
                    r#"{{"id":"{}","kind":"{}","path":"{}","startLine":{},"endLine":{},"usages":[{}],"props":[{}],"signals":[{}],"actions":[{}]}}"#,
                    escape_json(&node.id),
                    escape_json(&node.kind),
                    escape_json(&node.source_path),
                    node.start_line,
                    node.end_line,
                    node.usages
                        .iter()
                        .map(|usage| format!(
                            r#"{{"path":"{}","line":{},"column":{}}}"#,
                            escape_json(&usage.path),
                            usage.line,
                            usage.column
                        ))
                        .collect::<Vec<_>>()
                        .join(","),
                    node.props
                        .iter()
                        .map(|prop| {
                            format!(
                                r#"{{"name":"{}","value":"{}"}}"#,
                                escape_json(&prop.name),
                                escape_json(&prop.value)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(","),
                    node.signals
                        .iter()
                        .map(|signal| {
                            format!(
                                r#"{{"id":"{}","name":"{}","scope":"{}","storage":"{}","initial":{}}}"#,
                                escape_json(&signal.id),
                                escape_json(&signal.name),
                                escape_json(&signal.scope),
                                escape_json(&signal.storage),
                                signal.initial_json
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(","),
                    node.actions
                        .iter()
                        .map(|action| {
                            format!(
                                r#"{{"id":"{}","name":"{}","kind":"{}","detail":"{}"}}"#,
                                escape_json(&action.id),
                                escape_json(&action.name),
                                escape_json(&action.kind),
                                escape_json(&action.detail)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
        }
    }
    let routes = web
        .pages
        .iter()
        .map(|page| inspector_route_json(web, page))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"version":2,"nodes":[{}],"routes":[{}],"breakpoints":[{{"name":"xs","minWidth":0}},{{"name":"sm","minWidth":640}},{{"name":"md","minWidth":768}},{{"name":"lg","minWidth":1024}},{{"name":"xl","minWidth":1280}}]}}"#,
        nodes.join(","),
        routes
    )
}

fn inspector_route_json(web: &WebOutput, page: &ViewPage) -> String {
    let page_chunk = web
        .chunks
        .iter()
        .find(|chunk| chunk.id == page.page_chunk_id);
    let page_source = page_chunk
        .map(|chunk| chunk.source_path.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let page_span = page_chunk.and_then(|chunk| inspector_line_span(chunk));
    let layouts = page
        .layout_chunk_ids
        .iter()
        .filter_map(|id| web.chunks.iter().find(|chunk| chunk.id == *id))
        .map(|chunk| {
            let span = inspector_line_span(chunk);
            format!(
                r#"{{"source":"{}","chunk":"{}","startLine":{},"endLine":{}}}"#,
                escape_json(&chunk.source_path.to_string_lossy()),
                escape_json(&chunk.id),
                span.map(|span| span.0).unwrap_or(0),
                span.map(|span| span.1).unwrap_or(0)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"path":"{}","page":{{"source":"{}","chunk":"{}","startLine":{},"endLine":{}}},"layouts":[{}]}}"#,
        escape_json(&page.route_path),
        escape_json(&page_source),
        escape_json(&page.page_chunk_id),
        page_span.map(|span| span.0).unwrap_or(0),
        page_span.map(|span| span.1).unwrap_or(0),
        layouts
    )
}

fn inspector_line_span(chunk: &GeneratedChunk) -> Option<(usize, usize)> {
    let nodes = chunk.inspector.as_ref()?.nodes.as_slice();
    let start = nodes.iter().map(|node| node.start_line).min()?;
    let end = nodes.iter().map(|node| node.end_line).max()?;
    Some((start, end))
}

fn metadata_json(metadata: &[ViewMetadata]) -> String {
    let entries = metadata
        .iter()
        .map(|entry| {
            format!(
                r#"{{"name":"{}","content":"{}"}}"#,
                escape_json(&entry.name),
                escape_json(&entry.content)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{entries}]")
}

pub fn design_css() -> String {
    design_css_for_fonts(
        &BTreeSet::new(),
        &FontConfig::default(),
        &DesignConfig::default(),
    )
}
