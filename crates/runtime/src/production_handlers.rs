use crate::handlers::{
    chunk_response, design_css_response, font_response, is_preflight, javascript_response,
    json_response_text, project_asset_response, server_response, websocket_response,
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
        production_static_response(&project, uri.path())
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

fn production_static_response(project: &CompiledProject, path: &str) -> Response {
    if path == "/design.css" {
        return design_css_response(project, "web/design.css");
    }
    if path == "/router.js" {
        return javascript_response(project.web.router_js.clone());
    }
    if path == "/env.json" {
        return json_response_text(project.environment_config.client_json());
    }
    if let Some(response) = font_response(project, path) {
        return response;
    }
    if let Some(response) = project_asset_response(project, path, "public, max-age=300") {
        return response;
    }
    if let Some(response) = chunk_response(&project.web, path) {
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
    Html(page.html_document.clone()).into_response()
}
