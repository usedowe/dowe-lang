async fn http_result_response(result: HttpActionResult) -> Response {
    match result {
        HttpActionResult::Buffered {
            status,
            content_type,
            raw,
            ..
        } => body_response(status, content_type, raw),
        HttpActionResult::Proxy(response) => {
            let status = status_from_reqwest(response.status());
            let content_type = response_content_type(&response);
            if content_type.as_deref().is_some_and(is_sse_content_type) {
                return streaming_body_response(status, content_type, response.bytes_stream());
            }
            match response.bytes().await {
                Ok(body) => body_response(status, content_type, body),
                Err(_) => json_error(
                    StatusCode::BAD_GATEWAY,
                    "http_error",
                    "Outbound HTTP response failed",
                ),
            }
        }
    }
}

async fn agent_proxy_response(request: Value, response: reqwest::Response) -> Response {
    let status = status_from_reqwest(response.status());
    match response.bytes().await {
        Ok(body) if status.is_success() => json_response(
            StatusCode::OK,
            agent_http_success(request, json_from_bytes(&body)),
        ),
        Ok(body) => json_response(status, openrouter_error(json_from_bytes(&body))),
        Err(_) => json_error(
            StatusCode::BAD_GATEWAY,
            "http_error",
            "Outbound HTTP response failed",
        ),
    }
}

fn body_response(status: StatusCode, content_type: Option<String>, body: Bytes) -> Response {
    let mut response = (status, body).into_response();
    if let Some(content_type) = content_type {
        insert_header(&mut response, "content-type", &content_type);
    }
    response
}

fn bytes_endpoint_response(
    context: &StoreActionContext<'_>,
    endpoint: &HttpBytesEndpoint,
    body: Bytes,
) -> Response {
    let mut response = body_response(
        status_from_u16(endpoint.status),
        endpoint.content_type.clone(),
        body,
    );
    for header in &endpoint.headers {
        if let Some(value) = context
            .evaluate(&header.value)
            .ok()
            .and_then(ResolvedValue::into_json)
        {
            insert_dynamic_header(&mut response, &header.name, &json_text(&value));
        }
    }
    for cookie in &endpoint.cookies {
        if let Some(value) = context
            .evaluate(&cookie.value)
            .ok()
            .and_then(ResolvedValue::into_json)
        {
            append_header(
                &mut response,
                "set-cookie",
                &cookie_header(cookie, &json_text(&value)),
            );
        }
    }
    response
}

fn cookie_header(cookie: &ResponseCookie, value: &str) -> String {
    let mut output = format!("{}={}", cookie.name, cookie_value(value));
    if let Some(path) = &cookie.path {
        output.push_str("; Path=");
        output.push_str(path);
    }
    if let Some(max_age) = cookie.max_age {
        output.push_str("; Max-Age=");
        output.push_str(&max_age.to_string());
    }
    if let Some(same_site) = &cookie.same_site {
        output.push_str("; SameSite=");
        output.push_str(same_site);
    }
    if cookie.http_only {
        output.push_str("; HttpOnly");
    }
    if cookie.secure {
        output.push_str("; Secure");
    }
    output
}

fn cookie_value(value: &str) -> String {
    value
        .chars()
        .filter(|value| {
            value.is_ascii()
                && !matches!(
                    value,
                    '\u{0}'..='\u{20}' | '\u{7f}' | '"' | ',' | ';' | '\\'
                )
        })
        .collect()
}

fn json_text(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Null => String::new(),
        _ => value.to_string(),
    }
}

fn streaming_body_response(
    status: StatusCode,
    content_type: Option<String>,
    stream: impl futures_util::Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> Response {
    let mut response = Response::new(Body::from_stream(stream));
    *response.status_mut() = status;
    if let Some(content_type) = content_type {
        insert_header(&mut response, "content-type", &content_type);
    }
    response
}

fn status_from_reqwest(status: reqwest::StatusCode) -> StatusCode {
    StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::OK)
}

fn response_content_type(response: &reqwest::Response) -> Option<String> {
    response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

fn response_location(headers: &reqwest::header::HeaderMap) -> Option<String> {
    headers
        .get("location")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

fn response_headers_json(headers: &reqwest::header::HeaderMap) -> Value {
    let mut output = Map::new();
    for (name, value) in headers {
        let name = name.as_str();
        if name.eq_ignore_ascii_case("set-cookie") {
            continue;
        }
        if let Ok(value) = value.to_str() {
            output.insert(name.to_string(), Value::String(value.to_string()));
        }
    }
    Value::Object(output)
}

fn is_sse_content_type(value: &str) -> bool {
    value
        .split(';')
        .next()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("text/event-stream"))
}

fn json_from_bytes(body: &Bytes) -> Value {
    serde_json::from_slice::<Value>(body)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(body).to_string()))
}

fn http_binding_json(
    status: StatusCode,
    content_type: Option<String>,
    body: Option<Value>,
    url: String,
    redirected: bool,
    headers: Value,
    location: Option<String>,
) -> Value {
    let mut output = Map::new();
    output.insert(
        "status".to_string(),
        Value::Number(u64::from(status.as_u16()).into()),
    );
    output.insert("ok".to_string(), Value::Bool(status.is_success()));
    output.insert("url".to_string(), Value::String(url));
    output.insert("redirected".to_string(), Value::Bool(redirected));
    output.insert("headers".to_string(), headers);
    if let Some(content_type) = content_type {
        output.insert("contentType".to_string(), Value::String(content_type));
    }
    if let Some(location) = location {
        output.insert("location".to_string(), Value::String(location));
    }
    if let Some(body) = body {
        output.insert("json".to_string(), body);
    }
    Value::Object(output)
}

