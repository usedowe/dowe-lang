fn connection_for_handle(handles: &[CacheConnection], handle: &str) -> DoweResult<CacheConnection> {
    handles
        .iter()
        .find(|connection| connection.binding == handle)
        .cloned()
        .ok_or_else(|| DoweError::new(format!("Cache connection `{handle}` is not defined")))
}

fn required_provider(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<CacheProvider> {
    let prop = node
        .prop("provider")
        .ok_or_else(|| node_error(node, "Cache connection must declare `provider`"))?;
    match &prop.value {
        SourceValue::String(value) if value == "kv" => Ok(CacheProvider::CloudflareKv),
        SourceValue::String(value) if value == "redis" => Ok(CacheProvider::Redis),
        SourceValue::String(value) if value == "dowe" => Ok(CacheProvider::Dowe),
        SourceValue::String(value) => Err(node_error(
            node,
            format!("unsupported Cache provider `{value}`"),
        )),
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(node_error(
                    node,
                    "`provider` must be `kv`, `redis`, `dowe`, or a server env reference",
                ));
            };
            if let Some(environment) = environment {
                let variable = environment.variable(env_name).ok_or_else(|| {
                    node_error(node, format!("unknown environment variable `{env_name}`"))
                })?;
                if variable.visibility != EnvironmentVisibility::Server {
                    return Err(node_error(
                        node,
                        format!("environment variable `{env_name}` must be server-only"),
                    ));
                }
            }
            Ok(CacheProvider::Environment(env_name.to_string()))
        }
        _ => Err(node_error(
            node,
            "`provider` must be a quoted provider or a server env reference",
        )),
    }
}

fn required_connection_prop(
    node: &SourceNode,
    name: &str,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<CacheConnectionValue> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("Cache connection must declare `{name}`")))?;
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => {
            if name == "port" {
                validate_static_port(node, value)?;
            }
            if name == "name" {
                validate_cache_name(node, value)?;
            }
            Ok(CacheConnectionValue::Static(value.clone()))
        }
        SourceValue::Number(value) if name == "port" => {
            validate_static_port(node, value)?;
            Ok(CacheConnectionValue::Static(value.clone()))
        }
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(node_error(
                    node,
                    format!("`{name}` must be a literal or server env reference"),
                ));
            };
            if let Some(environment) = environment {
                let variable = environment.variable(env_name).ok_or_else(|| {
                    node_error(node, format!("unknown environment variable `{env_name}`"))
                })?;
                if variable.visibility != EnvironmentVisibility::Server {
                    return Err(node_error(
                        node,
                        format!("environment variable `{env_name}` must be server-only"),
                    ));
                }
            }
            Ok(CacheConnectionValue::Environment(env_name.to_string()))
        }
        _ => Err(node_error(
            node,
            format!("`{name}` must be a literal or server env reference"),
        )),
    }
}

fn validate_static_port(node: &SourceNode, value: &str) -> DoweResult<()> {
    if value.parse::<u16>().is_ok_and(|port| port > 0) {
        Ok(())
    } else {
        Err(node_error(node, "Cache `port` must be between 1 and 65535"))
    }
}

fn validate_cache_name(node: &SourceNode, value: &str) -> DoweResult<()> {
    if value.is_empty()
        || matches!(value, "." | ".." | "_auth")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || !value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
    {
        Err(node_error(node, format!("invalid Cache name `{value}`")))
    } else {
        Ok(())
    }
}

fn required_key_prop(node: &SourceNode) -> DoweResult<StoreLiteral> {
    let prop = node
        .prop("key")
        .ok_or_else(|| node_error(node, "Cache operation must declare `key`"))?;
    let value = store_literal(&prop.value)?;
    match &value {
        StoreLiteral::String(value) => {
            if value.is_empty()
                || matches!(value.as_str(), "." | "..")
                || value.contains('/')
                || value.contains('\\')
                || value.chars().any(char::is_control)
            {
                return Err(node_error(node, format!("invalid Cache key `{value}`")));
            }
        }
        StoreLiteral::Reference(value) if !value.is_empty() => {}
        _ => {
            return Err(node_error(
                node,
                "`key` must be a non-empty quoted string or a reference",
            ));
        }
    }
    Ok(value)
}

