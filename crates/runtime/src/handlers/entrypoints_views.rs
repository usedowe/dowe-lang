pub async fn views_handler(
    State(state): State<DevRuntimeState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if uri.path() == "/_dowe/dev/ws" {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let project = state.project.read().await;
    let web = if project.web.pages.is_empty() {
        &project.desktop_web
    } else {
        &project.web
    };
    let inspector_enabled = web.chunks.iter().any(|chunk| chunk.inspector.is_some());

    if uri.path() == "/_dowe/dev/client.js" {
        if project.studio_preview {
            return studio_preview_client_response();
        }
        if project.app_config.bundle == "dev.dowe.studio" {
            return studio_host_client_response();
        }
        let server_inspector_url = project.server_inspector.as_ref().and_then(|_| {
            state
                .dev_origins
                .last()
                .map(|origin| format!("{origin}/_dowe/dev/server/"))
        });
        return dev_client_response(inspector_enabled, server_inspector_url.as_deref());
    }

    if uri.path() == "/_dowe/dev/inspector-selection" {
        if !inspector_enabled {
            return StatusCode::NOT_FOUND.into_response();
        }
        if method != Method::POST {
            return StatusCode::METHOD_NOT_ALLOWED.into_response();
        }
        return inspector_selection_response(&project, &body);
    }

    if uri.path() == "/_dowe/dev/modules/manifest.json" {
        return generated_json_response(&project, "dev/modules/manifest.json");
    }

    if uri.path() == "/_dowe/dev/inspector.json" {
        return generated_json_response(&project, "web/inspector.json");
    }

    if let Some(response) = dev_module_response(&project, uri.path()) {
        return response;
    }

    let design_file_name = uri
        .path()
        .strip_prefix('/')
        .filter(|file_name| web.has_design_file_name(file_name));
    if design_file_name.is_some() {
        return cacheable_dev_design_css_response(
            &project,
            web,
            &format!("web/{}", web.design_file_name()),
            &headers,
            "no-store",
        );
    }

    if let Some(relative_path) = design_css_chunk_relative_path(uri.path()) {
        return cacheable_dev_design_css_response(
            &project,
            web,
            &relative_path,
            &headers,
            "no-store",
        );
    }

    if uri.path() == "/router.js" || uri.path() == format!("/{}", web.router_file_name()) {
        return javascript_response(web.router_js.clone());
    }

    if uri.path() == "/env.json" {
        return json_response_text(project.environment_config.client_json());
    }

    if uri.path() == "/manifest.json" {
        return generated_web_manifest_response(&project, web);
    }

    if let Some(response) = font_response(&project, uri.path()) {
        return response;
    }

    if let Some(response) = project_asset_response(&project, uri.path(), "no-store") {
        return response;
    }

    if let Some(response) = chunk_response(web, uri.path(), &headers, "no-store") {
        return response;
    }

    if let Some(page) = web.pages.iter().find(|page| page.route_path == uri.path()) {
        return render_page(page);
    }

    StatusCode::NOT_FOUND.into_response()
}
