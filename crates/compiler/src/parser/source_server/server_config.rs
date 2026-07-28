fn parse_server_config(
    node: &SourceNode,
    imports: &ServerImports,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    target: ServerTarget,
) -> DoweResult<ServerConfig> {
    let port = required_port(node)?;
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

    for child in &node.children {
        match child.name.as_str() {
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
                tls = Some(parse_tls_config(child)?);
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

    Ok(ServerConfig {
        port,
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
    })
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

fn parse_tls_config(node: &SourceNode) -> DoweResult<TlsConfig> {
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
        .map(parse_tls_domains_source)
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
    Ok(TlsConfig {
        mode,
        domains,
        email,
        staging,
        cache,
        domains_from,
        refresh_seconds,
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

fn parse_tls_domains_source(prop: &SourceProp) -> DoweResult<TlsDomainsSource> {
    let SourceValue::Object(entries) = &prop.value else {
        return Err(prop_error(
            prop,
            "`domainsFrom` must be a KV or Database object",
        ));
    };
    let mut values = HashMap::new();
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "`domainsFrom` does not support spread"));
        };
        let SourceValue::String(value) = value else {
            return Err(prop_error(
                prop,
                "`domainsFrom` values must be quoted strings",
            ));
        };
        if values.insert(key.as_str(), value.clone()).is_some() {
            return Err(prop_error(prop, format!("duplicate `domainsFrom.{key}`")));
        }
    }
    match (
        values.remove("kv"),
        values.remove("key"),
        values.remove("db"),
        values.remove("table"),
        values.remove("field"),
        values.is_empty(),
    ) {
        (Some(database), Some(key), None, None, None, true)
            if !database.is_empty() && !key.is_empty() =>
        {
            Ok(TlsDomainsSource::Kv { database, key })
        }
        (None, None, Some(database), Some(table), Some(field), true)
            if !database.is_empty() && !table.is_empty() && !field.is_empty() =>
        {
            Ok(TlsDomainsSource::Database {
                database,
                table,
                field,
            })
        }
        _ => Err(prop_error(
            prop,
            "`domainsFrom` must be `{ kv:\"name\" key:\"key\" }` or `{ db:\"name\" table:\"table\" field:\"field\" }`",
        )),
    }
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

