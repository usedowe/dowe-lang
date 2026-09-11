const ROUTER_RUNTIME_MODULES: &[&str] = &[
    include_str!("router_runtime/bootstrap_start.js"),
    include_str!("router_runtime/bootstrap_end.js"),
    include_str!("router_runtime/capability_bridges.js"),
    include_str!("router_runtime/hydration_and_appbar.js"),
    include_str!("router_runtime/layout_navigation_and_device_rendering.js"),
    include_str!("router_runtime/dynamic_component_rendering.js"),
    include_str!("router_runtime/chat_text_and_record_rendering.js"),
    include_str!("router_runtime/table_forms_and_avatar_rendering.js"),
    include_str!("router_runtime/native_and_action_execution.js"),
    include_str!("router_runtime/svg_and_runtime_actions.js"),
    include_str!("router_runtime/value_and_data_actions.js"),
    include_str!("router_runtime/notifications.js"),
    include_str!("router_runtime/routing.js"),
    include_str!("router_runtime/form_events.js"),
    include_str!("router_runtime/interaction_events.js"),
    include_str!("router_runtime/navigation_and_chat_events.js"),
];

pub fn router_js(web: &WebOutput) -> String {
    let mut script = router_config_js(web);
    script.reserve(
        ROUTER_RUNTIME_MODULES
            .iter()
            .map(|module| module.len())
            .sum(),
    );
    for module in ROUTER_RUNTIME_MODULES {
        script.push_str(module);
        script.push('\n');
    }
    minify_js(&script)
}

fn router_config_js(web: &WebOutput) -> String {
    let routes = web
        .pages
        .iter()
        .map(|page| {
            let layout_chunks = page
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
                .map(|path| format!(r#""{path}""#))
                .collect::<Vec<_>>()
                .join(",");
            let metadata = metadata_json(&page.metadata);
            format!(
                r#""{}":{{id:"{}",path:"{}",layoutChunks:[{layout_chunks}],pageChunk:"{}",jsChunks:[{js_chunks}],cssChunks:[{css_chunks}],runtimeChunks:[{runtime_chunks}],metadata:{metadata}}}"#,
                escape_js(&page.route_path),
                escape_js(&page.id),
                escape_js(&page.route_path),
                escape_js(&page.page_chunk_id)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let initial_path = web
        .pages
        .first()
        .map(|page| escape_js(&page.route_path))
        .unwrap_or_else(|| "/".to_string());
    let locale_chunks = web
        .translation_chunks
        .iter()
        .map(|chunk| {
            format!(
                r#""{}":"{}""#,
                escape_js(&chunk.locale),
                escape_js(
                    chunk
                        .relative_path
                        .strip_prefix("web")
                        .unwrap_or(&chunk.relative_path)
                        .to_string_lossy()
                        .trim_start_matches('/')
                )
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let default_locale = web
        .default_locale
        .as_deref()
        .map(escape_js)
        .unwrap_or_default();
    let icon_catalog = dowe_components::dynamic_icon_catalog_for_trees(
        web.pages
            .iter()
            .flat_map(|page| [&page.layout_tree, &page.page_tree]),
    )
    .expect("computed icon names must be validated before generation")
    .iter()
    .map(|(name, payload)| format!(r#""{}":"{}""#, escape_js(&name), escape_js(&payload)))
    .collect::<Vec<_>>()
    .join(",");

    format!(
        "let routes={{{routes}}};const initialPath=\"{initial_path}\";const localeChunks={{{locale_chunks}}};const defaultLocale=\"{default_locale}\";const doweIconCatalog={{{icon_catalog}}};"
    )
}
