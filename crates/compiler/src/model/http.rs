use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
        }
    }
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            _ => Err(()),
        }
    }
}

pub fn normalize_cors_method(value: &str) -> Option<&'static str> {
    match value.to_ascii_uppercase().as_str() {
        "GET" => Some("GET"),
        "POST" => Some("POST"),
        "PUT" => Some("PUT"),
        "DELETE" => Some("DELETE"),
        "PATCH" => Some("PATCH"),
        "HEAD" => Some("HEAD"),
        _ => None,
    }
}

pub fn normalize_http_header_name(value: &str) -> Option<String> {
    if !is_http_header_name(value) {
        return None;
    }
    Some(
        value
            .split('-')
            .map(|part| {
                let mut chars = part.chars();
                let Some(first) = chars.next() else {
                    return String::new();
                };
                let mut output = String::new();
                output.push(first.to_ascii_uppercase());
                output.push_str(&chars.as_str().to_ascii_lowercase());
                output
            })
            .collect::<Vec<_>>()
            .join("-"),
    )
}

pub fn normalize_cors_origin(value: &str) -> Option<String> {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return None;
    }
    let (scheme, rest) = if let Some(rest) = value.strip_prefix("http://") {
        ("http", rest)
    } else if let Some(rest) = value.strip_prefix("https://") {
        ("https", rest)
    } else {
        return None;
    };
    if rest.is_empty()
        || rest.contains('/')
        || rest.contains('?')
        || rest.contains('#')
        || rest.contains('@')
    {
        return None;
    }
    if let Some((host, port)) = rest.rsplit_once(':') {
        if host.is_empty() || port.is_empty() {
            return None;
        }
        let Ok(port) = port.parse::<u16>() else {
            return None;
        };
        Some(format!("{scheme}://{}:{port}", host.to_ascii_lowercase()))
    } else {
        Some(format!("{scheme}://{}", rest.to_ascii_lowercase()))
    }
}

fn is_http_header_name(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|value| {
            value.is_ascii_alphanumeric()
                || matches!(
                    value,
                    '!' | '#'
                        | '$'
                        | '%'
                        | '&'
                        | '\''
                        | '*'
                        | '+'
                        | '-'
                        | '.'
                        | '^'
                        | '_'
                        | '`'
                        | '|'
                        | '~'
                )
        })
}
