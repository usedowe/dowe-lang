fn parse_log(node: &SourceNode) -> DoweResult<ServerLog> {
    let level = match node.name.as_str() {
        "log" => ServerLogLevel::Log,
        "info" => ServerLogLevel::Info,
        "warn" => ServerLogLevel::Warn,
        "error" => ServerLogLevel::Error,
        _ => return Err(node_error(node, "unsupported log action")),
    };
    let values = node
        .args
        .iter()
        .map(log_value)
        .collect::<DoweResult<Vec<_>>>()?;
    Ok(ServerLog { level, values })
}

fn log_value(value: &SourceValue) -> DoweResult<ServerLogValue> {
    match value {
        SourceValue::String(value) => Ok(ServerLogValue::String(value.clone())),
        SourceValue::Bareword(value) => Ok(ServerLogValue::Reference(value.clone())),
        SourceValue::Number(value) => Ok(ServerLogValue::Number(value.clone())),
        SourceValue::Boolean(value) => Ok(ServerLogValue::Boolean(*value)),
        SourceValue::Null => Ok(ServerLogValue::Null),
        SourceValue::Array(_) | SourceValue::Object(_) => {
            Ok(ServerLogValue::JsonLiteral(value.to_source()))
        }
    }
}

fn required_port(node: &SourceNode) -> DoweResult<u16> {
    let prop = node
        .prop("port")
        .ok_or_else(|| node_error(node, "missing server port"))?;
    let value = prop
        .value
        .as_string_like()
        .ok_or_else(|| node_error(node, "invalid server port"))?;
    value
        .parse::<u16>()
        .map_err(|_| node_error(node, "invalid server port"))
}

fn required_name_prop(node: &SourceNode) -> DoweResult<String> {
    let prop = node
        .prop("name")
        .ok_or_else(|| node_error(node, "missing `name`"))?;
    let value = required_static_string_prop(prop)?;
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(prop_error(prop, "`name` must be a stable ASCII name"));
    }
    Ok(value)
}

fn optional_bind_prop(node: &SourceNode) -> DoweResult<String> {
    let Some(prop) = node.prop("bind") else {
        return Ok("127.0.0.1".to_string());
    };
    let value = required_static_string_prop(prop)?;
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return Err(prop_error(prop, "`bind` must be an IP address or host"));
    }
    Ok(value)
}

fn required_transport_port(node: &SourceNode) -> DoweResult<u16> {
    reject_unknown_props(node, &["name", "bind", "port"])?;
    required_port_prop(node, "port")
}

fn required_port_prop(node: &SourceNode, name: &str) -> DoweResult<u16> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))?;
    let value = prop
        .value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, format!("`{name}` must be a port")))?;
    value
        .parse::<u16>()
        .map_err(|_| prop_error(prop, format!("`{name}` must be a port")))
}

fn required_model_kind_prop(node: &SourceNode) -> DoweResult<ServerModelKind> {
    let prop = node
        .prop("kind")
        .ok_or_else(|| node_error(node, "missing `kind`"))?;
    match required_static_string_prop(prop)?.as_str() {
        "vad.silero" => Ok(ServerModelKind::VadSilero),
        _ => Err(prop_error(prop, "unsupported model kind")),
    }
}

fn required_model_engine_prop(node: &SourceNode) -> DoweResult<ServerModelEngine> {
    let prop = node
        .prop("engine")
        .ok_or_else(|| node_error(node, "missing `engine`"))?;
    match required_static_string_prop(prop)?.as_str() {
        "candle" => Ok(ServerModelEngine::Candle),
        "energy" => Ok(ServerModelEngine::Energy),
        _ => Err(prop_error(prop, "`engine` must be `candle` or `energy`")),
    }
}

fn required_model_format_prop(node: &SourceNode) -> DoweResult<ServerModelFormat> {
    let prop = node
        .prop("format")
        .ok_or_else(|| node_error(node, "missing `format`"))?;
    match required_static_string_prop(prop)?.as_str() {
        "onnx" => Ok(ServerModelFormat::Onnx),
        "builtin" => Ok(ServerModelFormat::Builtin),
        _ => Err(prop_error(prop, "`format` must be `onnx` or `builtin`")),
    }
}

fn optional_model_source_prop(
    node: &SourceNode,
    format: ServerModelFormat,
) -> DoweResult<Option<std::path::PathBuf>> {
    let Some(prop) = node.prop("source") else {
        return match format {
            ServerModelFormat::Builtin => Ok(None),
            ServerModelFormat::Onnx => Err(node_error(node, "missing `source`")),
        };
    };
    let value = required_static_string_prop(prop)?;
    if !value.starts_with("assets/")
        || value.is_empty()
        || value.starts_with('/')
        || value.contains("..")
        || value.chars().any(char::is_whitespace)
    {
        return Err(prop_error(prop, "`source` must be under `assets/`"));
    }
    Ok(Some(std::path::PathBuf::from(value)))
}

fn optional_sample_rates_prop(node: &SourceNode) -> DoweResult<Vec<u32>> {
    let Some(prop) = node.prop("sampleRates") else {
        return Ok(vec![8_000, 16_000]);
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(prop, "`sampleRates` must be an array"));
    };
    let mut rates = Vec::new();
    for value in values {
        let Some(rate) = value.as_string_like() else {
            return Err(prop_error(prop, "`sampleRates` entries must be numbers"));
        };
        let rate = rate
            .parse::<u32>()
            .map_err(|_| prop_error(prop, "`sampleRates` entries must be numbers"))?;
        match rate {
            8_000 | 16_000 => rates.push(rate),
            _ => return Err(prop_error(prop, "Silero VAD supports 8000 and 16000 Hz")),
        }
    }
    if rates.is_empty() {
        return Err(prop_error(prop, "`sampleRates` cannot be empty"));
    }
    rates.sort_unstable();
    rates.dedup();
    Ok(rates)
}

fn required_path_arg(node: &SourceNode, label: &str) -> DoweResult<String> {
    let path = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| node_error(node, format!("{label} must declare a path string")))?;
    if !path.starts_with('/') {
        return Err(node_error(
            node,
            format!("{label} path must start with `/`"),
        ));
    }
    Ok(path)
}

fn required_text_prop(node: &SourceNode) -> DoweResult<String> {
    let prop = node
        .prop("text")
        .ok_or_else(|| node_error(node, "response must declare text"))?;
    required_static_string_prop(prop)
}

