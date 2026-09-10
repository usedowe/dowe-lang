fn insert_header(headers: &mut HeaderMap, name: &str, value: String) -> AgentResult<()> {
    let name = HeaderName::try_from(name).map_err(|error| AgentError::new(error.to_string()))?;
    let value =
        HeaderValue::from_str(&value).map_err(|error| AgentError::new(error.to_string()))?;
    headers.insert(name, value);
    Ok(())
}

fn percent_encode_query(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn is_retryable(status: StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 425 | 429 | 500 | 502 | 503 | 504)
}

fn redact_secret(value: &str, secret: Option<&str>) -> String {
    match secret.filter(|secret| !secret.is_empty()) {
        Some(secret) => value.replace(secret, "[REDACTED]"),
        None => value.to_string(),
    }
}

fn status_text(status: StatusCode) -> String {
    status
        .canonical_reason()
        .map(|reason| format!("{} {reason}", status.as_u16()))
        .unwrap_or_else(|| status.as_u16().to_string())
}

