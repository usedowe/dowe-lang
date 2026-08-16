use crate::handlers::{
    IMMUTABLE_CACHE_CONTROL, cacheable_design_css_response, cacheable_javascript_response,
    chunk_response, design_css_chunk_relative_path, font_response, is_preflight,
    json_response_text, production_json_response, project_asset_response, server_response,
    websocket_response, with_cache_control,
};
use crate::server::DevRuntimeState;
use axum::body::Bytes;
use axum::extract::{State, WebSocketUpgrade};
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use dowe_compiler::CompiledProject;

pub async fn production_handler(
    State(state): State<DevRuntimeState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let project = state.project.read().await;
    if project.backend.has_endpoint_path(uri.path())
        || method == Method::OPTIONS && is_preflight(&headers)
    {
        return server_response(
            &project,
            &project.backend,
            &[],
            state.cache_mode,
            method,
            uri.path(),
            uri.query(),
            headers,
            body,
        )
        .await;
    }

    if method == Method::GET {
        production_static_response(&project, uri.path(), &headers)
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn production_declared_websocket_handler(
    state: DevRuntimeState,
    upgrade: WebSocketUpgrade,
    uri: Uri,
    headers: HeaderMap,
    path: String,
) -> Response {
    let project = state.project.read().await;
    let Some(route) = project.backend.find_websocket(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let params = std::collections::HashMap::new();
    let body = Bytes::new();
    if let crate::handlers::MiddlewareFlow::Respond(response) =
        crate::handlers::execute_middlewares(
            &project,
            &project.root,
            &route.middlewares,
            &headers,
            &params,
            &body,
            uri.query(),
            state.cache_mode,
        )
        .await
    {
        return response;
    }
    websocket_response(upgrade, project.clone(), route.handlers, state.cache_mode)
}

fn production_static_response(
    project: &CompiledProject,
    path: &str,
    headers: &HeaderMap,
) -> Response {
    let design_path = format!("/{}", project.web.design_file_name());
    let design_relative_path = format!("web/{}", project.web.design_file_name());
    if path == design_path {
        return cacheable_design_css_response(
            project,
            &design_relative_path,
            headers,
            IMMUTABLE_CACHE_CONTROL,
        );
    }
    if path == "/design.css" {
        return cacheable_design_css_response(project, &design_relative_path, headers, "no-cache");
    }
    if let Some(relative_path) = design_css_chunk_relative_path(path) {
        return cacheable_design_css_response(
            project,
            &relative_path,
            headers,
            IMMUTABLE_CACHE_CONTROL,
        );
    }
    if path == format!("/{}", project.web.router_file_name()) {
        return cacheable_javascript_response(
            project.web.router_js.clone(),
            headers,
            IMMUTABLE_CACHE_CONTROL,
        );
    }
    if path == "/router.js" {
        return cacheable_javascript_response(project.web.router_js.clone(), headers, "no-cache");
    }
    if path == "/env.json" {
        return json_response_text(project.environment_config.client_json());
    }
    if path == "/manifest.json" {
        return production_json_response(project, "web/manifest.json");
    }
    if let Some(response) = font_response(project, path) {
        return response;
    }
    if let Some(response) = project_asset_response(project, path, "public, max-age=300") {
        return response;
    }
    if let Some(response) = chunk_response(&project.web, path, headers, IMMUTABLE_CACHE_CONTROL) {
        return response;
    }
    let Some(page) = project
        .web
        .pages
        .iter()
        .find(|page| page.route_path == path)
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    with_cache_control(Html(page.html_document.clone()).into_response(), "no-cache")
}
