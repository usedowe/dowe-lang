fn parse_tls_domains_source(
    prop: &SourceProp,
    environment: &EnvironmentConfig,
) -> DoweResult<TlsDomainsSource> {
    let SourceValue::Object(entries) = &prop.value else {
        return Err(prop_error(
            prop,
            "`domainsFrom` must be a KV, Database, or endpoint object",
        ));
    };
    let mut values = HashMap::<&str, &SourceValue>::new();
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "`domainsFrom` does not support spread"));
        };
        if values.insert(key.as_str(), value).is_some() {
            return Err(prop_error(prop, format!("duplicate `domainsFrom.{key}`")));
        }
    }
    if values.contains_key("endpoint") {
        return parse_tls_endpoint_source(prop, values, environment);
    }
    let static_value = |name: &str| {
        values.get(name).and_then(|value| match value {
            SourceValue::String(value) if !value.is_empty() => Some((*value).clone()),
            _ => None,
        })
    };
    match (
        static_value("kv"),
        static_value("key"),
        static_value("db"),
        static_value("table"),
        static_value("field"),
        values.len(),
    ) {
        (Some(database), Some(key), None, None, None, 2) => {
            Ok(TlsDomainsSource::Kv { database, key })
        }
        (None, None, Some(database), Some(table), Some(field), 3) => {
            Ok(TlsDomainsSource::Database {
                database,
                table,
                field,
            })
        }
        _ => Err(prop_error(
            prop,
            "`domainsFrom` must be a KV, Database, or authenticated endpoint source",
        )),
    }
}

fn parse_tls_endpoint_source(
    prop: &SourceProp,
    values: HashMap<&str, &SourceValue>,
    environment: &EnvironmentConfig,
) -> DoweResult<TlsDomainsSource> {
    if !values
        .keys()
        .all(|key| matches!(*key, "endpoint" | "path" | "bearer" | "timeoutMs"))
    {
        return Err(prop_error(prop, "unknown endpoint domain source field"));
    }
    let base = match values.get("endpoint") {
        Some(SourceValue::String(value)) if value.starts_with("https://") => {
            HttpConnectionValue::Static((*value).clone())
        }
        Some(SourceValue::Bareword(value)) => {
            let name = value.strip_prefix("env.").ok_or_else(|| {
                prop_error(
                    prop,
                    "`domainsFrom.endpoint` must be HTTPS or a server env reference",
                )
            })?;
            let variable = environment.variable(name).ok_or_else(|| {
                prop_error(prop, format!("unknown environment variable `{name}`"))
            })?;
            if variable.visibility != EnvironmentVisibility::Server {
                return Err(prop_error(
                    prop,
                    "TLS endpoint environment must be server-only",
                ));
            }
            HttpConnectionValue::Environment(name.to_string())
        }
        _ => {
            return Err(prop_error(
                prop,
                "`domainsFrom.endpoint` must be HTTPS or a server env reference",
            ));
        }
    };
    let path = match values.get("path") {
        Some(SourceValue::String(value)) if value.starts_with('/') => (*value).clone(),
        _ => return Err(prop_error(prop, "`domainsFrom.path` must start with `/`")),
    };
    let bearer = match values
        .get("bearer")
        .and_then(|value| value.as_string_like())
    {
        Some(value) => {
            let name = value.strip_prefix("env.").ok_or_else(|| {
                prop_error(prop, "`domainsFrom.bearer` must use a server env variable")
            })?;
            let variable = environment.variable(name).ok_or_else(|| {
                prop_error(prop, format!("unknown environment variable `{name}`"))
            })?;
            if variable.visibility != EnvironmentVisibility::Server {
                return Err(prop_error(prop, "TLS endpoint bearer must be server-only"));
            }
            ServerSecret::Environment(name.to_string())
        }
        None => return Err(prop_error(prop, "missing `domainsFrom.bearer`")),
    };
    let timeout_ms = match values.get("timeoutMs") {
        Some(value) => value
            .as_string_like()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| (100..=30_000).contains(value))
            .ok_or_else(|| {
                prop_error(
                    prop,
                    "`domainsFrom.timeoutMs` must be between 100 and 30000",
                )
            })?,
        None => 5_000,
    };
    Ok(TlsDomainsSource::Endpoint {
        base,
        path,
        bearer,
        timeout_ms,
    })
}

fn validate_tls_domain(node: &SourceNode, mode: TlsMode, domain: &str) -> DoweResult<()> {
    let local = domain == "localhost"
        || domain.ends_with(".localhost")
        || matches!(domain, "127.0.0.1" | "::1");
    let valid_dns = domain.parse::<IpAddr>().is_err()
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
        && domain.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        });
    match mode {
        TlsMode::Acme if local || !valid_dns || !domain.contains('.') => Err(node_error(
            node,
            format!("invalid public ACME domain `{domain}`"),
        )),
        TlsMode::Local if !local => Err(node_error(
            node,
            format!("local TLS does not support public domain `{domain}`"),
        )),
        _ => Ok(()),
    }
}

fn valid_tls_email(value: &str) -> bool {
    let value = value.strip_prefix("mailto:").unwrap_or(value);
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.ends_with('.')
}

fn validate_tls_cache(node: &SourceNode, cache: &str) -> DoweResult<()> {
    let path = Path::new(cache);
    let inside_dowe = path
        .components()
        .next()
        .is_some_and(|component| component.as_os_str() == ".dowe");
    if path.is_absolute()
        || !inside_dowe
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(node_error(node, "`tls.cache` must stay inside `.dowe`"));
    }
    Ok(())
}
