fn parse_server_config(
    node: &SourceNode,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    target: ServerTarget,
) -> DoweResult<ServerConfig> {
    let port = required_port(node)?;
    let mut databases = Vec::new();
    let mut databases_seen = false;
    let mut endpoints = Vec::new();
    let mut websockets = Vec::new();
    let mut transports = Vec::new();
    let mut rtp = None;
    let mut models = Vec::new();
    let mut init_action = ServerAction::empty();
    let mut cors = CorsConfig::default();
    let mut cors_seen = false;
    let mut tls = None;
    let mut database_service = false;
    let mut cache_service = false;
    let mut vector_service = false;
    let mut queue_service = false;

    for child in &node.children {
        match child.name.as_str() {
            "databases" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`databases` registry is only supported by `main.server`",
                    ));
                }
                if databases_seen {
                    return Err(node_error(child, "duplicate `databases` registry"));
                }
                databases_seen = true;
                databases = parse_server_database_registry(child, imports)?;
            }
            "endpoints" => {
                for name in endpoint_group_references(child)? {
                    let group = imports.endpoint_groups.get(&name).ok_or_else(|| {
                        node_error(child, format!("missing endpoints import `{name}`"))
                    })?;
                    endpoints.extend(group.endpoints.clone());
                    websockets.extend(group.websockets.clone());
                }
            }
            "route" => endpoints.extend(parse_route(child, imports, types, environment)?),
            "endpoint" => {
                return Err(node_error(child, "`endpoint` has been renamed to `route`"));
            }
            "websocket" => websockets.push(parse_websocket(child, environment, imports, "", &[])?),
            "udp" => transports.push(parse_transport(
                child,
                ServerTransportProtocol::Udp,
                environment,
            )?),
            "tcp" => transports.push(parse_transport(
                child,
                ServerTransportProtocol::Tcp,
                environment,
            )?),
            "rtp" => {
                if rtp.is_some() {
                    return Err(node_error(child, "duplicate `rtp` block"));
                }
                rtp = Some(parse_rtp_config(child)?);
            }
            "model" => models.push(parse_server_model(child)?),
            "database" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`database service` is only supported by `main.server`",
                    ));
                }
                if database_service {
                    return Err(node_error(child, "duplicate `database service` block"));
                }
                parse_database_service(child)?;
                database_service = true;
            }
            "cache" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`cache service` is only supported by `main.server`",
                    ));
                }
                if cache_service {
                    return Err(node_error(child, "duplicate `cache service` block"));
                }
                parse_cache_service(child)?;
                cache_service = true;
            }
            "vector" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`vector service` is only supported by `main.server`",
                    ));
                }
                if vector_service {
                    return Err(node_error(child, "duplicate `vector service` block"));
                }
                parse_vector_service(child)?;
                vector_service = true;
            }
            "queue" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`queue service` is only supported by `main.server`",
                    ));
                }
                if queue_service {
                    return Err(node_error(child, "duplicate `queue service` block"));
                }
                parse_queue_service(child)?;
                queue_service = true;
            }
            "cors" => {
                if cors_seen {
                    return Err(node_error(child, "duplicate `cors` block"));
                }
                cors_seen = true;
                cors = match target {
                    ServerTarget::Server => parse_server_cors_config(child)?,
                    ServerTarget::Desktop => parse_desktop_cors_config(child)?,
                };
            }
            "tls" => {
                if !matches!(target, ServerTarget::Server) {
                    return Err(node_error(
                        child,
                        "`tls` is only supported by `main.server`",
                    ));
                }
                if tls.is_some() {
                    return Err(node_error(child, "duplicate `tls` block"));
                }
                tls = Some(parse_tls_config(child, environment, port)?);
            }
            "init" => {
                init_action = parse_action(child, ActionContext::Init, types, environment, imports)?
            }
            _ => return Err(node_error(child, "unsupported server block")),
        }
    }
    validate_unique_transport_names(node, &transports)?;
    validate_unique_model_names(node, &models)?;
    if database_service
        && (endpoints
            .iter()
            .any(|route| route.path == "/v1/databases/:name")
            || websockets
                .iter()
                .any(|route| route.path == "/v1/databases/:name"))
    {
        return Err(node_error(
            node,
            "`database service` reserves WebSocket path `/v1/databases/:name`",
        ));
    }
    if cache_service
        && (endpoints
            .iter()
            .any(|route| route.path == "/v1/caches/:name")
            || websockets
                .iter()
                .any(|route| route.path == "/v1/caches/:name"))
    {
        return Err(node_error(
            node,
            "`cache service` reserves WebSocket path `/v1/caches/:name`",
        ));
    }
    if vector_service
        && (endpoints
            .iter()
            .any(|route| route.path == "/v1/vectors/:name")
            || websockets
                .iter()
                .any(|route| route.path == "/v1/vectors/:name"))
    {
        return Err(node_error(
            node,
            "`vector service` reserves WebSocket path `/v1/vectors/:name`",
        ));
    }
    if queue_service
        && (endpoints
            .iter()
            .any(|route| route.path == "/v1/queues/:name")
            || websockets
                .iter()
                .any(|route| route.path == "/v1/queues/:name"))
    {
        return Err(node_error(
            node,
            "`queue service` reserves WebSocket path `/v1/queues/:name`",
        ));
    }

    Ok(ServerConfig {
        port,
        databases,
        tls,
        endpoints,
        websockets,
        transports,
        rtp,
        models,
        init_action,
        cors,
        database_service,
        cache_service,
        vector_service,
        queue_service,
    })
}

fn parse_server_database_registry(
    node: &SourceNode,
    imports: &ServerImports,
) -> DoweResult<Vec<StoreConnection>> {
    reject_unknown_props(node, &["databases"])?;
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(node, "`databases` accepts a list of imported Database handles"));
    }
    if node.prop("databases").is_none() {
        return Err(node_error(
            node,
            "`databases` must be an array of imported Database handles",
        ));
    }
    binding_array_prop(node, "databases")?
        .into_iter()
        .map(|name| {
            let Some(binding) = imports.config_bindings.get(&name) else {
                return Err(node_error(
                    node,
                    format!("unknown Database handle import `{name}`"),
                ));
            };
            match &binding.statement {
                ServerStatement::Store(ServerStoreStatement::Handle { connection }) => {
                    Ok(connection.clone())
                }
                _ => Err(node_error(
                    node,
                    format!("`databases` entry `{name}` must reference a Database handle"),
                )),
            }
        })
        .collect()
}

fn parse_database_service(node: &SourceNode) -> DoweResult<()> {
    if node.args.len() != 1
        || node.args[0].as_string_like().as_deref() != Some("service")
        || !node.props.is_empty()
        || !node.children.is_empty()
    {
        return Err(node_error(
            node,
            "the built-in Database server uses `database service`",
        ));
    }
    Ok(())
}

fn parse_cache_service(node: &SourceNode) -> DoweResult<()> {
    if node.args.len() != 1
        || node.args[0].as_string_like().as_deref() != Some("service")
        || !node.props.is_empty()
        || !node.children.is_empty()
    {
        return Err(node_error(
            node,
            "the built-in Dowe Cache server uses `cache service`",
        ));
    }
    Ok(())
}

fn parse_vector_service(node: &SourceNode) -> DoweResult<()> {
    if node.args.len() != 1
        || node.args[0].as_string_like().as_deref() != Some("service")
        || !node.props.is_empty()
        || !node.children.is_empty()
    {
        return Err(node_error(
            node,
            "the built-in Dowe Vector server uses `vector service`",
        ));
    }
    Ok(())
}

fn parse_queue_service(node: &SourceNode) -> DoweResult<()> {
    if node.args.len() != 1
        || node.args[0].as_string_like().as_deref() != Some("service")
        || !node.props.is_empty()
        || !node.children.is_empty()
    {
        return Err(node_error(
            node,
            "the built-in Dowe Queue server uses `queue service`",
        ));
    }
    Ok(())
}

fn parse_tls_config(
    node: &SourceNode,
    environment: &EnvironmentConfig,
    server_port: u16,
) -> DoweResult<TlsConfig> {
    reject_unknown_props(
        node,
        &[
            "mode",
            "domains",
            "email",
            "staging",
            "cache",
            "domainsFrom",
            "refreshSeconds",
            "httpPort",
        ],
    )?;
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(node, "`tls` accepts props only"));
    }
    let mode_prop = node
        .prop("mode")
        .ok_or_else(|| node_error(node, "missing `tls.mode`"))?;
    let mode = match required_static_string_prop(mode_prop)?.as_str() {
        "acme" => TlsMode::Acme,
        "local" => TlsMode::Local,
        _ => return Err(prop_error(mode_prop, "`mode` must be `acme` or `local`")),
    };
    let domains = node
        .prop("domains")
        .map(parse_tls_domains)
        .transpose()?
        .unwrap_or_default();
    let domains_from = node
        .prop("domainsFrom")
        .map(|prop| parse_tls_domains_source(prop, environment))
        .transpose()?;
    if domains.is_empty() && domains_from.is_none() {
        return Err(node_error(
            node,
            "`tls` requires `domains` or `domainsFrom`",
        ));
    }
    for domain in &domains {
        validate_tls_domain(node, mode, domain)?;
    }
    let email = node
        .prop("email")
        .map(required_static_string_prop)
        .transpose()?;
    if matches!(mode, TlsMode::Acme) && !email.as_deref().is_some_and(valid_tls_email) {
        return Err(node_error(node, "ACME TLS requires a valid `email`"));
    }
    if matches!(mode, TlsMode::Local) && email.is_some() {
        return Err(node_error(node, "`email` is only supported by ACME TLS"));
    }
    let staging = optional_bool_prop(node, "staging")?.unwrap_or(true);
    if matches!(mode, TlsMode::Local) && node.prop("staging").is_some() {
        return Err(node_error(node, "`staging` is only supported by ACME TLS"));
    }
    let cache = node
        .prop("cache")
        .map(required_static_string_prop)
        .transpose()?
        .unwrap_or_else(|| ".dowe/tls".to_string());
    validate_tls_cache(node, &cache)?;
    let refresh_seconds = match node.prop("refreshSeconds") {
        Some(prop) => {
            let value = required_u64_value(prop, &prop.value, "refreshSeconds")?;
            if !(30..=86_400).contains(&value) {
                return Err(prop_error(
                    prop,
                    "`refreshSeconds` must be between 30 and 86400",
                ));
            }
            value
        }
        None => 60,
    };
    if domains_from.is_none() && node.prop("refreshSeconds").is_some() {
        return Err(node_error(node, "`refreshSeconds` requires `domainsFrom`"));
    }
    let http_port = node
        .prop("httpPort")
        .map(|prop| required_u64_value(prop, &prop.value, "httpPort"))
        .transpose()?
        .map(|value| {
            u16::try_from(value)
                .ok()
                .filter(|value| *value != 0 && *value != server_port)
                .ok_or_else(|| {
                    node_error(
                        node,
                        "`tls.httpPort` must be a valid port different from `server.port`",
                    )
                })
        })
        .transpose()?;
    Ok(TlsConfig {
        mode,
        domains,
        email,
        staging,
        cache,
        domains_from,
        refresh_seconds,
        http_port,
    })
}

fn parse_tls_domains(prop: &SourceProp) -> DoweResult<Vec<String>> {
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(
            prop,
            "`domains` must be an array of quoted strings",
        ));
    };
    let mut domains = Vec::new();
    for value in values {
        let SourceValue::String(domain) = value else {
            return Err(prop_error(
                prop,
                "`domains` must be an array of quoted strings",
            ));
        };
        let domain = domain.trim().to_ascii_lowercase();
        if domain.is_empty() {
            return Err(prop_error(prop, "TLS domains cannot be empty"));
        }
        if !domains.contains(&domain) {
            domains.push(domain);
        }
    }
    Ok(domains)
}

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
