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
    let digest = Sha256::digest(content)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!(r#"W/"{digest}""#)
}

pub(crate) fn generated_web_manifest_response(
    project: &CompiledProject,
    web: &WebOutput,
) -> Response {
    let path = project.root.join(".dowe/web/manifest.json");
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
        Err(_) => dev_web_artifact_response(
            project,
            web,
            "web/manifest.json",
            &HeaderMap::new(),
            "no-store",
        )
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response()),
    }
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

