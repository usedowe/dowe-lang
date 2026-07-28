fn required_http_base_prop(
    node: &SourceNode,
    environment: &EnvironmentConfig,
) -> DoweResult<HttpConnectionValue> {
    let prop = node
        .prop("base")
        .ok_or_else(|| node_error(node, "missing `base`"))?;
    match &prop.value {
        SourceValue::String(value) => {
            if !value.starts_with("https://") && !value.starts_with("http://") {
                return Err(prop_error(prop, "`base` must be an http or https URL"));
            }
            Ok(HttpConnectionValue::Static(value.clone()))
        }
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(prop_error(
                    prop,
                    "`base` must be a quoted URL or server env reference",
                ));
            };
            let variable = environment.variable(env_name).ok_or_else(|| {
                prop_error(prop, format!("unknown environment variable `{env_name}`"))
            })?;
            if variable.visibility != EnvironmentVisibility::Server {
                return Err(prop_error(
                    prop,
                    format!("environment variable `{env_name}` must be server-only"),
                ));
            }
            Ok(HttpConnectionValue::Environment(env_name.to_string()))
        }
        _ => Err(prop_error(
            prop,
            "`base` must be a quoted URL or server env reference",
        )),
    }
}

fn required_http_path_prop(node: &SourceNode) -> DoweResult<String> {
    let prop = node
        .prop("path")
        .ok_or_else(|| node_error(node, "missing `path`"))?;
    let value = required_static_string_prop(prop)?;
    if !value.starts_with('/') {
        return Err(prop_error(prop, "`path` must start with `/`"));
    }
    Ok(value)
}

fn optional_http_mode_prop(node: &SourceNode) -> DoweResult<HttpResponseMode> {
    let Some(prop) = node.prop("mode") else {
        return Ok(HttpResponseMode::Json);
    };
    let value = required_static_string_prop(prop)?;
    match value.as_str() {
        "json" => Ok(HttpResponseMode::Json),
        "proxy" => Ok(HttpResponseMode::Proxy),
        "bytes" => Ok(HttpResponseMode::Bytes),
        _ => Err(prop_error(
            prop,
            "`mode` must be `json`, `proxy`, or `bytes`",
        )),
    }
}

fn optional_http_redirect_prop(node: &SourceNode) -> DoweResult<HttpRedirectPolicy> {
    let Some(prop) = node.prop("redirect") else {
        return Ok(HttpRedirectPolicy::Follow);
    };
    let value = required_static_string_prop(prop)?;
    match value.as_str() {
        "follow" => Ok(HttpRedirectPolicy::Follow),
        "manual" => Ok(HttpRedirectPolicy::Manual),
        "error" => Ok(HttpRedirectPolicy::Error),
        _ => Err(prop_error(
            prop,
            "`redirect` must be `follow`, `manual`, or `error`",
        )),
    }
}

fn optional_positive_u32_prop(node: &SourceNode, name: &str) -> DoweResult<Option<u32>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    let value = prop
        .value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    let value = value
        .parse::<u32>()
        .map_err(|_| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    if value == 0 {
        return Err(prop_error(
            prop,
            format!("`{name}` must be a positive integer"),
        ));
    }
    Ok(Some(value))
}

fn optional_positive_u64_prop(node: &SourceNode, name: &str) -> DoweResult<Option<u64>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    let value = prop
        .value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    let value = value
        .parse::<u64>()
        .map_err(|_| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    if value == 0 {
        return Err(prop_error(
            prop,
            format!("`{name}` must be a positive integer"),
        ));
    }
    Ok(Some(value))
}

fn optional_positive_usize_prop(node: &SourceNode, name: &str) -> DoweResult<Option<usize>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    let value = prop
        .value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    let value = value
        .parse::<usize>()
        .map_err(|_| prop_error(prop, format!("`{name}` must be a positive integer")))?;
    if value == 0 {
        return Err(prop_error(
            prop,
            format!("`{name}` must be a positive integer"),
        ));
    }
    Ok(Some(value))
}

fn optional_bool_prop(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    match &prop.value {
        SourceValue::Boolean(value) => Ok(Some(*value)),
        _ => Err(prop_error(prop, format!("`{name}` must be a boolean"))),
    }
}

fn required_bool_value(prop: &SourceProp, value: &SourceValue, name: &str) -> DoweResult<bool> {
    match value {
        SourceValue::Boolean(value) => Ok(*value),
        _ => Err(prop_error(prop, format!("`{name}` must be a boolean"))),
    }
}

fn required_u64_value(prop: &SourceProp, value: &SourceValue, name: &str) -> DoweResult<u64> {
    let Some(value) = value.as_string_like() else {
        return Err(prop_error(
            prop,
            format!("`{name}` must be a positive integer"),
        ));
    };
    value
        .parse::<u64>()
        .map_err(|_| prop_error(prop, format!("`{name}` must be a positive integer")))
}

fn required_reference_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    prop.value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, format!("`{name}` must be a binding reference")))
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|name| *name == prop.name) {
            return Err(prop_error(
                prop,
                format!("unknown prop `{}` on `{}`", prop.name, node.name),
            ));
        }
    }
    Ok(())
}

fn assignment(node: &SourceNode) -> Option<(String, String)> {
    if node.args.len() < 3 {
        return None;
    }
    let binding = node.args[0].as_string_like()?;
    let equals = node.args[1].as_string_like()?;
    let expression = node.args[2].as_string_like()?;
    (equals == "=").then_some((binding, expression))
}

fn required_header_name_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    let value = match &prop.value {
        SourceValue::String(value) => value.clone(),
        _ => {
            return Err(prop_error(
                prop,
                format!("`{name}` must be a quoted static string literal"),
            ));
        }
    };
    normalize_http_header_name(&value).ok_or_else(|| prop_error(prop, "invalid header name"))
}

fn required_cookie_name_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    let SourceValue::String(value) = &prop.value else {
        return Err(prop_error(
            prop,
            format!("`{name}` must be a quoted static string literal"),
        ));
    };
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(prop_error(prop, "invalid cookie name"));
    }
    Ok(value.clone())
}

fn required_http_method_prop(node: &SourceNode) -> DoweResult<HttpMethod> {
    let prop = node
        .prop("method")
        .ok_or_else(|| node_error(node, "missing `method`"))?;
    let value = required_static_string_prop(prop)?;
    match value.as_str() {
        "get" => Ok(HttpMethod::Get),
        "post" => Ok(HttpMethod::Post),
        "put" => Ok(HttpMethod::Put),
        "patch" => Ok(HttpMethod::Patch),
        "delete" => Ok(HttpMethod::Delete),
        _ => Err(prop_error(
            prop,
            "`method` must be get, post, put, patch, or delete",
        )),
    }
}

fn optional_http_headers_prop(
    node: &SourceNode,
    environment: &EnvironmentConfig,
) -> DoweResult<Vec<OutboundHttpHeader>> {
    let Some(prop) = node.prop("headers") else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(
            prop,
            "`headers` must be an array of { name:\"Header\" value:\"literal\" } objects",
        ));
    };
    values
        .iter()
        .map(|value| parse_http_header_value(prop, value, environment))
        .collect()
}

fn parse_http_header_value(
    prop: &SourceProp,
    value: &SourceValue,
    environment: &EnvironmentConfig,
) -> DoweResult<OutboundHttpHeader> {
    let SourceValue::Object(entries) = value else {
        return Err(prop_error(prop, "`headers` entries must be objects"));
    };
    let mut name = None;
    let mut header_value = None;
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "`headers` entries do not support spread"));
        };
        match key.as_str() {
            "name" => {
                let SourceValue::String(value) = value else {
                    return Err(prop_error(prop, "header `name` must be a quoted string"));
                };
                name = Some(value.clone());
            }
            "value" => header_value = Some(parse_http_header_binding(prop, value, environment)?),
            _ => return Err(prop_error(prop, format!("unknown header field `{key}`"))),
        }
    }
    let name = name.ok_or_else(|| prop_error(prop, "header entry missing `name`"))?;
    let name =
        normalize_http_header_name(&name).ok_or_else(|| prop_error(prop, "invalid header name"))?;
    if is_restricted_outbound_header(&name) {
        return Err(prop_error(
            prop,
            format!("header `{name}` is not allowed in outbound request headers"),
        ));
    }
    let value = header_value.ok_or_else(|| prop_error(prop, "header entry missing `value`"))?;
    Ok(OutboundHttpHeader { name, value })
}

fn parse_http_header_binding(
    prop: &SourceProp,
    value: &SourceValue,
    environment: &EnvironmentConfig,
) -> DoweResult<HttpHeaderValue> {
    match value {
        SourceValue::String(value) => Ok(HttpHeaderValue::Static(value.clone())),
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(prop_error(
                    prop,
                    "header `value` must be a quoted string or server env reference",
                ));
            };
            let variable = environment.variable(env_name).ok_or_else(|| {
                prop_error(prop, format!("unknown environment variable `{env_name}`"))
            })?;
            if variable.visibility != EnvironmentVisibility::Server {
                return Err(prop_error(
                    prop,
                    format!("environment variable `{env_name}` must be server-only"),
                ));
            }
            Ok(HttpHeaderValue::Environment(env_name.to_string()))
        }
        _ => Err(prop_error(
            prop,
            "header `value` must be a quoted string or server env reference",
        )),
    }
}

fn is_restricted_outbound_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "authorization"
            | "cookie"
            | "set-cookie"
            | "host"
            | "connection"
            | "content-length"
            | "transfer-encoding"
            | "upgrade"
            | "proxy-authorization"
            | "proxy-authenticate"
            | "te"
            | "trailer"
    )
}

fn required_secret_prop(
    node: &SourceNode,
    name: &str,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerSecret> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    let Some(value) = prop.value.as_string_like() else {
        return Err(prop_error(
            prop,
            format!("`{name}` must be an env reference"),
        ));
    };
    let Some(env_name) = value.strip_prefix("env.") else {
        return Err(prop_error(
            prop,
            format!("`{name}` must use a server env variable"),
        ));
    };
    let variable = environment
        .variable(env_name)
        .ok_or_else(|| prop_error(prop, format!("unknown environment variable `{env_name}`")))?;
    if variable.visibility != EnvironmentVisibility::Server {
        return Err(prop_error(
            prop,
            format!("environment variable `{env_name}` must be server-only"),
        ));
    }
    Ok(ServerSecret::Environment(env_name.to_string()))
}

fn required_algorithm_prop(node: &SourceNode, name: &str, allowed: &[&str]) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    let value = required_static_string_prop(prop)?;
    if value == "none" {
        return Err(prop_error(prop, "`alg:\"none\"` is not supported"));
    }
    if allowed.iter().any(|allowed| *allowed == value) {
        Ok(value)
    } else {
        Err(prop_error(prop, format!("unsupported algorithm `{value}`")))
    }
}

fn required_store_literal_prop(node: &SourceNode, name: &str) -> DoweResult<StoreLiteral> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    store_literal(&prop.value)
}

fn handler_request_name(_node: &SourceNode) -> Option<&'static str> {
    Some("req")
}

fn handler_action_context(node: &SourceNode) -> ActionContext<'_> {
    ActionContext::HttpHandler {
        async_handler: true,
        request: handler_request_name(node),
    }
}

fn reject_explicit_handler_async(node: &SourceNode) -> DoweResult<()> {
    if node
        .args
        .iter()
        .any(|arg| arg.as_string_like().as_deref() == Some("async"))
    {
        return Err(node_error(
            node,
            "handlers are asynchronous by default; remove `async`",
        ));
    }
    Ok(())
}

fn child_named<'a>(node: &'a SourceNode, name: &str) -> Option<&'a SourceNode> {
    node.children.iter().find(|child| child.name == name)
}

