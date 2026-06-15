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
    r#"const protocol=location.protocol==="https:"?"wss":"ws";let active=true;function connect(){if(!active)return;const socket=new WebSocket(`${protocol}://${location.host}/_dowe/dev/ws`);socket.onmessage=(event)=>{try{const message=JSON.parse(event.data);if(message.type==="reload")location.reload();if(message.type==="shutdown")active=false;}catch(error){}};socket.onclose=()=>{if(active)setTimeout(connect,250);};}connect();"#
        .to_string()
}
