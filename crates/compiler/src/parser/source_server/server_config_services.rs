fn parse_database_registry(
    node: &SourceNode,
    imports: &ServerImports,
) -> DoweResult<Vec<StoreConnection>> {
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

