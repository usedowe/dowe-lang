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
    let paths = [
        project.root.join(".dowe").join(relative_path),
        project.root.join(".dowe/apps/desktop").join(relative_path),
    ];
    match paths.iter().find_map(|path| fs::read_to_string(path).ok()) {
        Some(css) => cacheable_text_response(
            css,
            "text/css; charset=utf-8",
            request_headers,
            cache_control,
        ),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(crate) fn cacheable_dev_design_css_response(
    project: &CompiledProject,
    web: &WebOutput,
    relative_path: &str,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Response {
    let response = cacheable_design_css_response(
        project,
        relative_path,
        request_headers,
        cache_control,
    );
    if response.status() != StatusCode::NOT_FOUND {
        return response;
    }
    dev_web_artifact_response(project, web, relative_path, request_headers, cache_control)
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

pub(crate) fn dev_web_artifact_response(
    project: &CompiledProject,
    web: &WebOutput,
    relative_path: &str,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Option<Response> {
    let relative_path = Path::new(relative_path);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    let artifact = dowe_compiler::web_artifacts_for_target(
        web,
        &project.font_config,
        &project.design_config,
        Path::new(""),
        "web",
    )
    .into_iter()
    .find(|artifact| artifact.relative_path == relative_path)?;
    let content_type = if relative_path.extension().and_then(|value| value.to_str()) == Some("css") {
        "text/css; charset=utf-8"
    } else if relative_path.extension().and_then(|value| value.to_str()) == Some("json") {
        "application/json; charset=utf-8"
    } else if relative_path.extension().and_then(|value| value.to_str()) == Some("js") {
        "application/javascript; charset=utf-8"
    } else {
        "text/html; charset=utf-8"
    };
    Some(cacheable_text_response(
        artifact.content,
        content_type,
        request_headers,
        cache_control,
    ))
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

pub(crate) fn studio_host_client_response() -> Response {
    let mut script = dev_client_script(false, None);
    script.push('\n');
    script.push_str(include_str!("../studio_inspector_client.js"));
    javascript_response(script)
}

pub(crate) fn studio_preview_client_response() -> Response {
    javascript_response(studio_preview_client_script())
}

