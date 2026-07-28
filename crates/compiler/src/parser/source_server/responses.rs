fn return_content_type(node: &SourceNode) -> DoweResult<Option<String>> {
    node.children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("contentType"))
        .map(required_static_string_prop)
        .transpose()
}

fn return_headers(node: &SourceNode) -> DoweResult<Vec<ResponseHeader>> {
    let Some(prop) = node
        .children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("headers"))
    else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(prop, "`headers` must be an array"));
    };
    values
        .iter()
        .map(|value| parse_response_header(prop, value))
        .collect()
}

fn parse_response_header(prop: &SourceProp, value: &SourceValue) -> DoweResult<ResponseHeader> {
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
            "value" => header_value = Some(store_literal(value)?),
            _ => return Err(prop_error(prop, format!("unknown header field `{key}`"))),
        }
    }
    let name = name.ok_or_else(|| prop_error(prop, "header entry missing `name`"))?;
    let name =
        normalize_http_header_name(&name).ok_or_else(|| prop_error(prop, "invalid header name"))?;
    let value = header_value.ok_or_else(|| prop_error(prop, "header entry missing `value`"))?;
    Ok(ResponseHeader { name, value })
}

fn return_cookies(node: &SourceNode) -> DoweResult<Vec<ResponseCookie>> {
    let Some(prop) = node
        .children
        .iter()
        .find(|child| child.name == "return")
        .and_then(|child| child.prop("cookies"))
    else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(prop, "`cookies` must be an array"));
    };
    values
        .iter()
        .map(|value| parse_response_cookie(prop, value))
        .collect()
}

fn parse_response_cookie(prop: &SourceProp, value: &SourceValue) -> DoweResult<ResponseCookie> {
    let SourceValue::Object(entries) = value else {
        return Err(prop_error(prop, "`cookies` entries must be objects"));
    };
    let mut name = None;
    let mut cookie_value = None;
    let mut path = None;
    let mut http_only = false;
    let mut secure = false;
    let mut same_site = None;
    let mut max_age = None;
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "`cookies` entries do not support spread"));
        };
        match key.as_str() {
            "name" => {
                let SourceValue::String(value) = value else {
                    return Err(prop_error(prop, "cookie `name` must be a quoted string"));
                };
                name = Some(value.clone());
            }
            "value" => cookie_value = Some(store_literal(value)?),
            "path" => {
                let SourceValue::String(value) = value else {
                    return Err(prop_error(prop, "cookie `path` must be a quoted string"));
                };
                path = Some(value.clone());
            }
            "httpOnly" => http_only = required_bool_value(prop, value, "httpOnly")?,
            "secure" => secure = required_bool_value(prop, value, "secure")?,
            "sameSite" => {
                let SourceValue::String(value) = value else {
                    return Err(prop_error(
                        prop,
                        "cookie `sameSite` must be a quoted string",
                    ));
                };
                if !matches!(value.as_str(), "Lax" | "Strict" | "None") {
                    return Err(prop_error(
                        prop,
                        "cookie `sameSite` must be `Lax`, `Strict`, or `None`",
                    ));
                }
                same_site = Some(value.clone());
            }
            "maxAge" => max_age = Some(required_u64_value(prop, value, "maxAge")?),
            _ => return Err(prop_error(prop, format!("unknown cookie field `{key}`"))),
        }
    }
    let name = name.ok_or_else(|| prop_error(prop, "cookie entry missing `name`"))?;
    let value = cookie_value.ok_or_else(|| prop_error(prop, "cookie entry missing `value`"))?;
    Ok(ResponseCookie {
        name,
        value,
        path,
        http_only,
        secure,
        same_site,
        max_age,
    })
}

fn return_status(node: &SourceNode) -> DoweResult<u16> {
    let Some(return_node) = node.children.iter().find(|child| child.name == "return") else {
        return Ok(200);
    };
    return_status_from_node(return_node)
}

fn return_status_from_node(node: &SourceNode) -> DoweResult<u16> {
    let Some(prop) = node.prop("status") else {
        return Ok(200);
    };
    let Some(value) = prop.value.as_string_like() else {
        return Err(node_error(node, "`status` must be a number"));
    };
    value
        .parse::<u16>()
        .map_err(|_| node_error(node, "`status` must be a valid HTTP status"))
}

fn optional_prop_string(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    node.prop(name)
        .map(|prop| {
            prop.value
                .as_required_string()
                .ok_or_else(|| node_error(node, format!("`{name}` must be a string")))
        })
        .transpose()
}

