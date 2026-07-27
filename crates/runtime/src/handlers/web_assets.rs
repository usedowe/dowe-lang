pub(crate) fn chunk_response(web: &WebOutput, path: &str) -> Option<Response> {
    let prefix = "/chunks/";
    let chunk_path = path.strip_prefix(prefix)?;
    let relative = std::path::Path::new("web/chunks").join(chunk_path);
    if let Some(chunk) = web
        .translation_chunks
        .iter()
        .find(|chunk| chunk.relative_path == relative)
    {
        return Some(javascript_response(chunk.content.clone()));
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

    Some((StatusCode::OK, [(CONTENT_TYPE, content_type)], content).into_response())
}

pub(crate) fn design_css_response(project: &CompiledProject, relative_path: &str) -> Response {
    let path = project.root.join(".dowe").join(relative_path);
    match fs::read_to_string(path) {
        Ok(css) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/css; charset=utf-8")],
            css,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
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
    let relative = path.strip_prefix("/assets/")?;
    let Some(path) = safe_project_asset_path(&project.root, relative) else {
        return Some(StatusCode::NOT_FOUND.into_response());
    };
    let Ok(content) = fs::read(&path) else {
        return Some(StatusCode::NOT_FOUND.into_response());
    };
    let content_type = asset_content_type(&path);
    Some(
        (
            StatusCode::OK,
            [
                (CONTENT_TYPE, content_type),
                (CACHE_CONTROL, cache_control),
            ],
            content,
        )
            .into_response(),
    )
}

fn safe_project_asset_path(root: &Path, relative: &str) -> Option<std::path::PathBuf> {
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    let assets = root.join("assets");
    if fs::symlink_metadata(&assets).ok()?.file_type().is_symlink() {
        return None;
    }
    let mut current = assets;
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
    Html(inject_dev_client(&page.html_document)).into_response()
}

fn dev_client_response() -> Response {
    javascript_response(dev_client_script())
}

pub(crate) fn javascript_response(content: String) -> Response {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        content,
    )
        .into_response()
}

pub(crate) fn json_response_text(content: String) -> Response {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json; charset=utf-8")],
        content,
    )
        .into_response()
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

fn dev_client_script() -> String {
    r#"const protocol=location.protocol==="https:"?"wss":"ws";let active=true;function connect(){if(!active)return;const socket=new WebSocket(`${protocol}://${location.host}/_dowe/dev/ws`);socket.onmessage=async(event)=>{try{const message=JSON.parse(event.data);if(message.type==="module_update"&&message.target==="web"){if(typeof window.__doweHotUpdate==="function"){try{await window.__doweHotUpdate(message.version||"");return;}catch(error){}}location.reload();}if(message.type==="reload"&&(message.target==="web"||message.target==="desktop"))location.reload();if(message.type==="shutdown")active=false;}catch(error){}};socket.onclose=()=>{if(active)setTimeout(connect,250);};}connect();"#
        .to_string()
}

#[cfg(test)]
mod project_asset_tests {
    use super::{asset_content_type, safe_project_asset_path};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn resolves_regular_project_assets_and_rejects_traversal() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("assets/icons/web")).expect("assets");
        fs::write(
            temp.path().join("assets/icons/web/favicon-32x32.png"),
            "png",
        )
        .expect("asset");

        assert!(
            safe_project_asset_path(temp.path(), "icons/web/favicon-32x32.png").is_some()
        );
        assert!(safe_project_asset_path(temp.path(), "../main.dowe").is_none());
        assert!(safe_project_asset_path(temp.path(), "/etc/passwd").is_none());
        assert_eq!(
            asset_content_type(Path::new("favicon.png")),
            "image/png"
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_project_assets() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("assets/icons")).expect("assets");
        fs::write(temp.path().join("outside.png"), "outside").expect("outside");
        std::os::unix::fs::symlink(
            temp.path().join("outside.png"),
            temp.path().join("assets/icons/favicon.png"),
        )
        .expect("symlink");

        assert!(safe_project_asset_path(temp.path(), "icons/favicon.png").is_none());
    }
}
