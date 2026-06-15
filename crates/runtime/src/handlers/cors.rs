pub(crate) fn is_preflight(headers: &HeaderMap) -> bool {
    headers.contains_key(ORIGIN) && headers.contains_key(ACCESS_CONTROL_REQUEST_METHOD)
}

fn cors_preflight_response(
    server: &ServerConfig,
    dev_origins: &[String],
    path: &str,
    headers: &HeaderMap,
) -> Response {
    let cors = &server.cors;
    let Some(origin) = origin_header(headers) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(allowed_origin) = cors.allowed_origin(origin, dev_origins) else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let Some(requested_method) = request_method_header(headers) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(requested_method) = normalize_cors_method(requested_method) else {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    };
    let route_methods = server.methods_for_path(path);
    if route_methods.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }
    if !route_methods
        .iter()
        .any(|method| method.as_str() == requested_method)
        || !cors.allows_method(requested_method)
    {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let Some(requested_headers) = requested_header_names(headers) else {
        return StatusCode::FORBIDDEN.into_response();
    };
    if !cors.allows_headers(&requested_headers) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let mut response = StatusCode::NO_CONTENT.into_response();
    apply_cors_base_headers(cors, &allowed_origin, &mut response);
    insert_header(
        &mut response,
        "access-control-allow-methods",
        &allow_methods_value(cors, &route_methods),
    );
    insert_header(
        &mut response,
        "access-control-allow-headers",
        &allow_headers_value(cors, &requested_headers),
    );
    if let Some(max_age) = cors.max_age {
        insert_header(
            &mut response,
            "access-control-max-age",
            &max_age.to_string(),
        );
    }
    response
}

fn cors_actual_response(
    cors: &CorsConfig,
    dev_origins: &[String],
    headers: &HeaderMap,
    mut response: Response,
) -> Response {
    let Some(origin) = origin_header(headers) else {
        return response;
    };
    let Some(allowed_origin) = cors.allowed_origin(origin, dev_origins) else {
        return response;
    };
    apply_cors_base_headers(cors, &allowed_origin, &mut response);
    if !cors.expose_headers.is_empty() {
        insert_header(
            &mut response,
            "access-control-expose-headers",
            &cors.expose_headers.join(", "),
        );
    }
    response
}

fn apply_cors_base_headers(cors: &CorsConfig, allowed_origin: &str, response: &mut Response) {
    insert_header(response, "access-control-allow-origin", allowed_origin);
    if cors.credentials {
        insert_header(response, "access-control-allow-credentials", "true");
    }
    if allowed_origin != "*" {
        append_vary_origin(response);
    }
}

fn origin_header(headers: &HeaderMap) -> Option<&str> {
    headers.get(ORIGIN).and_then(|value| value.to_str().ok())
}

fn request_method_header(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(ACCESS_CONTROL_REQUEST_METHOD)
        .and_then(|value| value.to_str().ok())
}

fn requested_header_names(headers: &HeaderMap) -> Option<Vec<String>> {
    let Some(value) = headers.get(ACCESS_CONTROL_REQUEST_HEADERS) else {
        return Some(Vec::new());
    };
    let value = value.to_str().ok()?;
    let mut output = Vec::new();
    for header in value.split(',') {
        let header = header.trim();
        if header.is_empty() {
            continue;
        }
        output.push(normalize_http_header_name(header)?);
    }
    Some(output)
}

fn allow_methods_value(cors: &CorsConfig, route_methods: &[HttpMethod]) -> String {
    route_methods
        .iter()
        .filter_map(|method| {
            let method = method.as_str();
            if cors.allows_method(method) {
                Some(method)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn allow_headers_value(cors: &CorsConfig, requested_headers: &[String]) -> String {
    if requested_headers.is_empty() {
        cors.headers.join(", ")
    } else {
        requested_headers.join(", ")
    }
}

fn insert_header(response: &mut Response, name: &'static str, value: &str) {
    if let Ok(value) = HeaderValue::from_str(value) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(name), value);
    }
}

fn append_vary_origin(response: &mut Response) {
    let next = match response
        .headers()
        .get(VARY)
        .and_then(|value| value.to_str().ok())
    {
        Some(current)
            if current
                .split(',')
                .any(|value| value.trim().eq_ignore_ascii_case("origin")) =>
        {
            return;
        }
        Some(current) if !current.is_empty() => format!("{current}, Origin"),
        _ => "Origin".to_string(),
    };
    insert_header(response, "vary", &next);
}
