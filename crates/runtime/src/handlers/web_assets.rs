pub(crate) const IMMUTABLE_CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

pub(crate) fn chunk_response(
    web: &WebOutput,
    path: &str,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Option<Response> {
    let prefix = "/chunks/";
    let chunk_path = path.strip_prefix(prefix)?;
    let relative = std::path::Path::new("web/chunks").join(chunk_path);
    if let Some(chunk) = web
        .runtime_chunks()
        .into_iter()
        .find(|chunk| chunk.relative_path == relative)
    {
        return Some(cacheable_text_response(
            chunk.content,
            "application/javascript; charset=utf-8",
            request_headers,
            cache_control,
        ));
    }
    if let Some(chunk) = web
        .translation_chunks
        .iter()
        .find(|chunk| chunk.relative_path == relative)
    {
        return Some(cacheable_text_response(
            chunk.content.clone(),
            "application/javascript; charset=utf-8",
            request_headers,
            cache_control,
        ));
    }
    let chunk = web
        .chunks
        .iter()
        .find(|chunk| chunk.relative_path == relative || chunk.css_relative_path == relative)?;
    let content_type = if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else {
        "application/javascript; charset=utf-8"
    };
    let content = if path.ends_with(".css") {
        chunk.css_content.clone()
    } else {
        chunk.content.clone()
    };

    Some(cacheable_text_response(
        content,
        content_type,
        request_headers,
        cache_control,
    ))
}

pub(crate) fn cacheable_design_css_response(
    project: &CompiledProject,
    relative_path: &str,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Response {
    let path = project.root.join(".dowe").join(relative_path);
    match fs::read_to_string(path) {
        Ok(css) => cacheable_text_response(
            css,
            "text/css; charset=utf-8",
            request_headers,
            cache_control,
        ),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(crate) fn design_css_chunk_relative_path(path: &str) -> Option<String> {
    let file_name = path.strip_prefix("/chunks/design/")?;
    if file_name.is_empty() || file_name.contains('/') || !file_name.ends_with(".css") {
        return None;
    }
    Some(format!("web/chunks/design/{file_name}"))
}

pub(crate) fn font_response(project: &CompiledProject, path: &str) -> Option<Response> {
    let font_path = path.strip_prefix("/fonts/")?;
    let relative = Path::new(font_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Some(StatusCode::NOT_FOUND.into_response());
    }

    let path = project.root.join(".dowe/fonts").join(relative);
    let Ok(content) = fs::read(path) else {
        return Some(StatusCode::NOT_FOUND.into_response());
    };

    Some(
        (
            StatusCode::OK,
            [
                (CONTENT_TYPE, "font/ttf"),
                (CACHE_CONTROL, "public, max-age=31536000"),
            ],
            content,
        )
            .into_response(),
    )
}

pub(crate) fn project_asset_response(
    project: &CompiledProject,
    path: &str,
    cache_control: &'static str,
) -> Option<Response> {
    let (directory, relative) = if let Some(relative) = path.strip_prefix("/assets/") {
        ("assets", relative)
    } else if let Some(relative) = path.strip_prefix("/icons/") {
        ("icons", relative)
    } else {
        return None;
    };
    let Some(path) = safe_project_asset_path(&project.root, directory, relative) else {
        return Some(StatusCode::NOT_FOUND.into_response());
    };
    let Ok(content) = fs::read(&path) else {
        return Some(StatusCode::NOT_FOUND.into_response());
    };
    let content_type = asset_content_type(&path);
    Some(
        (
            StatusCode::OK,
            [(CONTENT_TYPE, content_type), (CACHE_CONTROL, cache_control)],
            content,
        )
            .into_response(),
    )
}

fn safe_project_asset_path(
    root: &Path,
    directory: &str,
    relative: &str,
) -> Option<std::path::PathBuf> {
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    let base = root.join(directory);
    if fs::symlink_metadata(&base).ok()?.file_type().is_symlink() {
        return None;
    }
    let mut current = base;
    for component in relative.components() {
        let Component::Normal(value) = component else {
            return None;
        };
        current.push(value);
        let metadata = fs::symlink_metadata(&current).ok()?;
        if metadata.file_type().is_symlink() {
            return None;
        }
    }
    current.is_file().then_some(current)
}

fn asset_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|value| value.to_str()) {
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("svg") => "image/svg+xml",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("json") => "application/json",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn render_page(page: &ViewPage) -> Response {
    with_cache_control(
        Html(inject_dev_client(&page.html_document)).into_response(),
        "no-store",
    )
}

pub(crate) fn inspector_selection_response(project: &CompiledProject, body: &Bytes) -> Response {
    if body.len() > 64 * 1024 {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let Ok(value) = serde_json::from_slice::<Value>(body) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(node) = value.get("node").and_then(Value::as_object) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(path) = node.get("path").and_then(Value::as_str) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if !safe_inspector_source_path(path) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let mut selected_node = Map::new();
    for key in ["id", "kind", "path", "startLine", "endLine"] {
        if let Some(value) = node.get(key)
            && matches!(value, Value::String(_) | Value::Number(_))
        {
            selected_node.insert(key.to_string(), value.clone());
        }
    }
    if let Some(usages) = node.get("usages").and_then(Value::as_array) {
        let usages = usages
            .iter()
            .filter_map(|usage| {
                let usage = usage.as_object()?;
                let path = usage.get("path").and_then(Value::as_str)?;
                if !safe_inspector_source_path(path) {
                    return None;
                }
                let line = usage.get("line").and_then(Value::as_u64)?;
                let column = usage.get("column").and_then(Value::as_u64)?;
                Some(json!({"path": path, "line": line, "column": column}))
            })
            .collect::<Vec<_>>();
        selected_node.insert("usages".to_string(), Value::Array(usages));
    }
    let selection = json!({"node": selected_node});
    let selection_root = project.root.join(".dowe/dev");
    if let Err(error) = fs::create_dir_all(&selection_root) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("inspector selection directory failed: {error}"),
        )
            .into_response();
    }
    let content = match serde_json::to_vec(&selection) {
        Ok(content) => content,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let staged = selection_root.join(".inspector-selection.json.tmp");
    let target = selection_root.join("inspector-selection.json");
    if fs::write(&staged, content).is_err() || fs::rename(&staged, &target).is_err() {
        let _ = fs::remove_file(&staged);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    json_response_text(r#"{"ok":true}"#.to_string())
}

fn safe_inspector_source_path(path: &str) -> bool {
    let path = Path::new(path);
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn dev_client_response(inspector_enabled: bool, server_inspector_url: Option<&str>) -> Response {
    javascript_response(dev_client_script(inspector_enabled, server_inspector_url))
}

pub(crate) fn javascript_response(content: String) -> Response {
    web_text_response(
        content,
        "application/javascript; charset=utf-8",
        Some("no-store"),
    )
}

pub(crate) fn cacheable_javascript_response(
    content: String,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Response {
    cacheable_text_response(
        content,
        "application/javascript; charset=utf-8",
        request_headers,
        cache_control,
    )
}

pub(crate) fn json_response_text(content: String) -> Response {
    web_text_response(content, "application/json; charset=utf-8", Some("no-store"))
}

fn cacheable_text_response(
    content: String,
    content_type: &'static str,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Response {
    let etag = content_etag(content.as_bytes());
    if request_headers
        .get(IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|values| values.split(',').any(|value| value.trim() == etag))
    {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        response.headers_mut().insert(
            ETAG,
            HeaderValue::from_str(&etag).expect("valid generated etag"),
        );
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(cache_control));
        return response;
    }
    let mut response = web_text_response(content, content_type, Some(cache_control));
    response.headers_mut().insert(
        ETAG,
        HeaderValue::from_str(&etag).expect("valid generated etag"),
    );
    response
}

fn web_text_response(
    content: String,
    content_type: &'static str,
    cache_control: Option<&'static str>,
) -> Response {
    let mut response = (StatusCode::OK, content).into_response();
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
    if let Some(cache_control) = cache_control {
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(cache_control));
    }
    response
}

fn content_etag(content: &[u8]) -> String {
    format!(r#"W/"{:x}""#, Sha256::digest(content))
}

pub(crate) fn generated_json_response(project: &CompiledProject, relative_path: &str) -> Response {
    let path = project.root.join(".dowe").join(relative_path);
    match fs::read_to_string(path) {
        Ok(content) => (
            StatusCode::OK,
            [
                (CONTENT_TYPE, "application/json; charset=utf-8"),
                (CACHE_CONTROL, "no-store"),
            ],
            content,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(crate) fn production_json_response(project: &CompiledProject, relative_path: &str) -> Response {
    let path = project.root.join(".dowe").join(relative_path);
    match fs::read_to_string(path) {
        Ok(content) => {
            web_text_response(content, "application/json; charset=utf-8", Some("no-cache"))
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(crate) fn with_cache_control(mut response: Response, cache_control: &'static str) -> Response {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(cache_control));
    response
}

pub(crate) fn dev_module_response(project: &CompiledProject, path: &str) -> Option<Response> {
    let relative = path.strip_prefix("/_dowe/dev/modules/")?;
    let relative = Path::new(relative);
    let components = relative.components().collect::<Vec<_>>();
    if components.len() != 2
        || components
            .iter()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Some(StatusCode::NOT_FOUND.into_response());
    }
    let file = project.root.join(".dowe/dev/modules").join(relative);
    let Ok(content) = fs::read(file) else {
        return Some(StatusCode::NOT_FOUND.into_response());
    };

    Some(
        (
            StatusCode::OK,
            [
                (CONTENT_TYPE, "application/octet-stream"),
                (CACHE_CONTROL, "no-store"),
            ],
            content,
        )
            .into_response(),
    )
}

fn inject_dev_client(html: &str) -> String {
    let script = r#"<script type="module" src="/_dowe/dev/client.js"></script>"#;
    if html.contains(script) {
        return html.to_string();
    }

    if let Some(index) = html.rfind("</body>") {
        let mut output = String::with_capacity(html.len() + script.len());
        output.push_str(&html[..index]);
        output.push_str(script);
        output.push_str(&html[index..]);
        output
    } else {
        format!("{html}{script}")
    }
}

fn dev_client_script(inspector_enabled: bool, server_inspector_url: Option<&str>) -> String {
    let refresh = if inspector_enabled {
        "window.__doweInspectorRefresh?.();"
    } else {
        ""
    };
    let hmr = format!(
        r#"const protocol=location.protocol==="https:"?"wss":"ws";let active=true;let hmrQueue=Promise.resolve();function queueHotUpdate(version){{hmrQueue=hmrQueue.then(async()=>{{if(typeof window.__doweHotUpdate==="function"){{try{{await window.__doweHotUpdate(version||"");{refresh}return;}}catch(error){{}}}}location.reload();}}).catch(()=>{{}});}}function connect(){{if(!active)return;const socket=new WebSocket(`${{protocol}}://${{location.host}}/_dowe/dev/ws`);socket.onmessage=async(event)=>{{try{{const message=JSON.parse(event.data);if(message.type==="module_update"&&message.target==="web"){{queueHotUpdate(message.version||"");return;}}if(message.type==="reload"&&(message.target==="web"||message.target==="desktop"))location.reload();if(message.type==="shutdown")active=false;}}catch(error){{}}}};socket.onclose=()=>{{if(active)setTimeout(connect,250);}};}}connect();"#
    );
    if inspector_enabled {
        let icon = serde_json::to_string(include_str!("../dowe_inspector_icon.svg"))
            .expect("Dowe inspector icon must be JSON encodable");
        let server_inspector_url = server_inspector_url
            .map(|url| {
                serde_json::to_string(url).expect("Server inspector URL must be JSON encodable")
            })
            .unwrap_or_else(|| "null".to_string());
        let client = include_str!("../dev_inspector_client.js")
            .replace("\"__DOWE_INSPECTOR_ICON_SVG__\"", &icon)
            .replace("\"__DOWE_SERVER_INSPECTOR_URL__\"", &server_inspector_url);
        format!("{hmr}\n{client}")
    } else {
        hmr.to_string()
    }
}

#[cfg(test)]
mod project_asset_tests {
    use super::{
        asset_content_type, dev_client_script, safe_inspector_source_path, safe_project_asset_path,
    };
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn resolves_regular_project_assets_and_rejects_traversal() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("icons/web")).expect("icons");
        fs::write(temp.path().join("icons/web/favicon-32x32.png"), "png").expect("asset");

        assert!(safe_project_asset_path(temp.path(), "icons", "web/favicon-32x32.png").is_some());
        assert!(safe_project_asset_path(temp.path(), "icons", "../main.dowe").is_none());
        assert!(safe_project_asset_path(temp.path(), "icons", "/etc/passwd").is_none());
        assert_eq!(asset_content_type(Path::new("favicon.png")), "image/png");
    }

    #[test]
    fn inspector_client_is_only_included_for_dev_web_output() {
        let inspector = dev_client_script(true, Some("http://127.0.0.1:8081/_dowe/dev/server/"));
        assert!(inspector.contains("Dowe inspect"));
        assert!(inspector.contains("setPointerCapture"));
        assert!(inspector.contains("dowe-inspector-position"));
        assert!(inspector.contains("left:\"16px\""));
        assert!(inspector.contains("right:\"auto\""));
        assert!(inspector.contains("Number.isFinite(top)"));
        assert!(inspector.contains("dowe-inspector-enabled"));
        assert!(inspector.contains("dowe-inspector-hidden"));
        assert!(inspector.contains("dowe-inspector-panel-open"));
        assert!(inspector.contains("KeyD"));
        assert!(inspector.contains("KeyR"));
        assert!(inspector.contains("#1f3a5f"));
        assert!(inspector.contains("#6bc670"));
        assert!(inspector.contains("rgb(31,58,95)"));
        assert!(!inspector.contains("__DOWE_INSPECTOR_ICON_SVG__"));
        assert!(inspector.contains("function solarIcon"));
        assert!(inspector.contains("Open Dowe Server Inspector"));
        assert!(inspector.contains("http://127.0.0.1:8081/_dowe/dev/server/"));
        assert!(inspector.contains("aria-label"));
        assert!(inspector.contains("Routes"));
        assert!(inspector.contains("Show details"));
        assert!(inspector.contains("loadManifest();"));
        assert!(!inspector.contains("inspectorPreview"));
        assert!(!inspector.contains("<iframe"));
        assert!(dev_client_script(true, None).contains("const SERVER_INSPECTOR_URL=null;"));
        assert!(!dev_client_script(false, None).contains("Dowe inspect"));
        assert!(!dev_client_script(false, None).contains("Inspector"));
        assert!(safe_inspector_source_path("views/pages/home.dowe"));
        assert!(!safe_inspector_source_path(""));
        assert!(!safe_inspector_source_path("../main.dowe"));
        assert!(!safe_inspector_source_path("/tmp/main.dowe"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_project_assets() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("icons")).expect("icons");
        fs::write(temp.path().join("outside.png"), "outside").expect("outside");
        std::os::unix::fs::symlink(
            temp.path().join("outside.png"),
            temp.path().join("icons/favicon.png"),
        )
        .expect("symlink");

        assert!(safe_project_asset_path(temp.path(), "icons", "favicon.png").is_none());
    }
}
