fn reject_legacy_root_file(root: &Path, file_name: &str) -> DoweResult<()> {
    let legacy_path = root.join("src").join(file_name);
    if legacy_path.exists() {
        return Err(DoweError::at_path(
            &legacy_path,
            format!("`src/{file_name}` has moved to project-root `{file_name}`"),
        ));
    }
    Ok(())
}

pub(crate) fn parse_config_file(_root: &Path, file: &SourceFile) -> DoweResult<ParsedConfig> {
    if !file.imports.is_empty() {
        return Err(DoweError::at_path(
            &file.imports[0].location.path,
            format!(
                "{}:{}: `src/config.dowe` does not support imports",
                file.imports[0].location.line, file.imports[0].location.column
            ),
        ));
    }
    if file.nodes.len() != 1 || file.nodes[0].name != "config" {
        return Err(DoweError::at_path(
            &file.path,
            "`src/config.dowe` must declare one `config` block",
        ));
    }

    let config_root = &file.nodes[0];
    if !config_root.args.is_empty() || !config_root.props.is_empty() {
        return Err(node_error(
            config_root,
            "`config` does not accept args or props",
        ));
    }

    let mut app_config = AppConfig::default();
    let mut font_config = FontConfig::default();
    let mut design_config = DesignConfig::default();
    let mut _server_config = ProjectServerConfig::default();
    let mut app_seen = false;
    let mut fonts_seen = false;
    let mut design_seen = false;
    let mut server_seen = false;

    for child in &config_root.children {
        match child.name.as_str() {
            "app" => {
                if app_seen {
                    return Err(node_error(child, "duplicate `app` block"));
                }
                app_seen = true;
                app_config = parse_app(child)?;
            }
            "fonts" => {
                if fonts_seen {
                    return Err(node_error(child, "duplicate `fonts` block"));
                }
                fonts_seen = true;
                font_config = parse_fonts(child)?;
            }
            "design" => {
                if design_seen {
                    return Err(node_error(child, "duplicate `design` block"));
                }
                design_seen = true;
                design_config = parse_design(child)?;
            }
            "env" => {
                return Err(node_error(
                    child,
                    "`env` blocks are no longer supported; use `.env.example` and `.env`",
                ));
            }
            "server" => {
                if server_seen {
                    return Err(node_error(child, "duplicate `server` block"));
                }
                server_seen = true;
                _server_config = parse_server_config(child)?;
            }
            _ => {
                return Err(node_error(
                    child,
                    format!("`{}` is not valid in `src/config.dowe`", child.name),
                ));
            }
        }
    }

    Ok(ParsedConfig {
        app_config,
        font_config,
        design_config,
        environment_config: EnvironmentConfig::default(),
    })
}

pub(crate) fn parse_theme_file(
    file: &SourceFile,
) -> DoweResult<(AppConfig, FontConfig, DesignConfig)> {
    if !file.imports.is_empty() {
        return Err(DoweError::at_path(
            &file.imports[0].location.path,
            format!(
                "{}:{}: `theme.dowe` does not support imports",
                file.imports[0].location.line, file.imports[0].location.column
            ),
        ));
    }
    if file.nodes.len() != 1 || file.nodes[0].name != "theme" {
        return Err(DoweError::at_path(
            &file.path,
            "`theme.dowe` must declare one `theme` block",
        ));
    }
    let theme_root = &file.nodes[0];
    if !theme_root.args.is_empty() || !theme_root.props.is_empty() {
        return Err(node_error(
            theme_root,
            "`theme` does not accept args or props",
        ));
    }

    let mut font_config = FontConfig::default();
    let mut design_config = DesignConfig::default();
    let mut fonts_seen = false;
    let mut design_seen = false;

    for child in &theme_root.children {
        match child.name.as_str() {
            "app" => {
                return Err(node_error(
                    child,
                    "`app` metadata belongs in root `main.dowe` under `main`",
                ));
            }
            "fonts" => {
                if fonts_seen {
                    return Err(node_error(child, "duplicate `fonts` block"));
                }
                fonts_seen = true;
                font_config = parse_fonts(child)?;
            }
            "design" => {
                if design_seen {
                    return Err(node_error(child, "duplicate `design` block"));
                }
                design_seen = true;
                design_config = parse_design(child)?;
            }
            _ => {
                return Err(node_error(
                    child,
                    format!("`{}` is not valid in `theme.dowe`", child.name),
                ));
            }
        }
    }

    Ok((AppConfig::default(), font_config, design_config))
}

pub(crate) fn parse_app(node: &SourceNode) -> DoweResult<AppConfig> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`app` only accepts `name` and `bundle` props",
        ));
    }
    reject_unknown_props(node, &["name", "bundle"])?;
    let name = match node.prop("name") {
        Some(prop) => parse_app_name_prop(prop)?,
        None => AppConfig::default().name,
    };
    let bundle = match node.prop("bundle") {
        Some(prop) => parse_app_bundle_prop(prop)?,
        None => AppConfig::default().bundle,
    };
    Ok(AppConfig { name, bundle })
}

fn parse_server_config(node: &SourceNode) -> DoweResult<ProjectServerConfig> {
    if !node.args.is_empty() || !node.props.is_empty() {
        return Err(node_error(node, "`server` does not accept args or props"));
    }
    let mut config = ProjectServerConfig::default();
    let mut backend_seen = false;
    let mut desktop_seen = false;
    for child in &node.children {
        if child.name != "cors" {
            return Err(node_error(
                child,
                format!("`{}` is not valid inside `server`", child.name),
            ));
        }
        let raw = parse_cors(child)?;
        match raw.target {
            CorsTarget::Server => {
                if backend_seen {
                    return Err(node_error(&raw.node, "duplicate CORS policy for `server`"));
                }
                backend_seen = true;
                config.backend_cors = raw.config;
            }
            CorsTarget::Desktop => {
                if desktop_seen {
                    return Err(node_error(&raw.node, "duplicate CORS policy for `desktop`"));
                }
                desktop_seen = true;
                config.desktop_cors = raw.config;
            }
            CorsTarget::All => {
                if backend_seen {
                    return Err(node_error(&raw.node, "duplicate CORS policy for `server`"));
                }
                if desktop_seen {
                    return Err(node_error(&raw.node, "duplicate CORS policy for `desktop`"));
                }
                backend_seen = true;
                desktop_seen = true;
                config.backend_cors = raw.config.clone();
                config.desktop_cors = raw.config;
            }
        }
    }
    Ok(config)
}

fn parse_cors(node: &SourceNode) -> DoweResult<RawCors> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(node, "`cors` only accepts props"));
    }
    reject_unknown_props(
        node,
        &[
            "target",
            "origins",
            "devOrigins",
            "methods",
            "headers",
            "exposeHeaders",
            "credentials",
            "maxAge",
            "enabled",
        ],
    )?;
    let target = match node.prop("target") {
        Some(prop) => parse_cors_target(prop)?,
        None => CorsTarget::Server,
    };
    let enabled = match node.prop("enabled") {
        Some(prop) => parse_boolean_prop(prop, "cors.enabled")?,
        None => true,
    };
    let credentials = match node.prop("credentials") {
        Some(prop) => parse_boolean_prop(prop, "cors.credentials")?,
        None => false,
    };
    let (origins, allow_wildcard_origin) = match node.prop("origins") {
        Some(prop) => parse_cors_origins(prop, credentials)?,
        None => (Vec::new(), false),
    };
    let allow_dev_origins = match node.prop("devOrigins") {
        Some(prop) => parse_boolean_prop(prop, "cors.devOrigins")?,
        None => false,
    };
    if enabled && origins.is_empty() && !allow_wildcard_origin && !allow_dev_origins {
        return Err(node_error(
            node,
            "enabled CORS policy requires `origins` or `devOrigins:true`",
        ));
    }
    let methods = match node.prop("methods") {
        Some(prop) => parse_cors_methods(prop)?,
        None => Vec::new(),
    };
    let headers = match node.prop("headers") {
        Some(prop) => parse_header_array(prop, "cors.headers")?,
        None => vec!["Content-Type".to_string()],
    };
    let expose_headers = match node.prop("exposeHeaders") {
        Some(prop) => parse_header_array(prop, "cors.exposeHeaders")?,
        None => Vec::new(),
    };
    let max_age = match node.prop("maxAge") {
        Some(prop) => Some(parse_max_age(prop)?),
        None => None,
    };
    Ok(RawCors {
        node: node.clone(),
        target,
        config: CorsConfig {
            enabled,
            origins,
            allow_wildcard_origin,
            allow_dev_origins,
            methods,
            headers,
            expose_headers,
            credentials,
            max_age,
        },
    })
}

pub(crate) fn parse_server_cors_config(node: &SourceNode) -> DoweResult<CorsConfig> {
    let raw = parse_cors(node)?;
    match raw.target {
        CorsTarget::Server | CorsTarget::All => Ok(raw.config),
        CorsTarget::Desktop => Err(node_error(
            node,
            "`cors target:\"desktop\"` belongs inside `desktop.server`",
        )),
    }
}

pub(crate) fn parse_desktop_cors_config(node: &SourceNode) -> DoweResult<CorsConfig> {
    let raw = parse_cors(node)?;
    match raw.target {
        CorsTarget::Desktop | CorsTarget::All => Ok(raw.config),
        CorsTarget::Server => Err(node_error(
            node,
            "`cors target:\"server\"` belongs inside `main.server`",
        )),
    }
}

fn parse_cors_target(prop: &SourceProp) -> DoweResult<CorsTarget> {
    let value = required_static_string_prop(prop)?;
    match value.as_str() {
        "server" => Ok(CorsTarget::Server),
        "desktop" => Ok(CorsTarget::Desktop),
        "all" => Ok(CorsTarget::All),
        _ => Err(prop_error(
            prop,
            "`cors.target` must be `server`, `desktop`, or `all`",
        )),
    }
}

fn parse_cors_origins(prop: &SourceProp, credentials: bool) -> DoweResult<(Vec<String>, bool)> {
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(
            prop,
            "`cors.origins` must be an array of strings",
        ));
    };
    let mut origins = Vec::new();
    let mut seen = BTreeSet::new();
    let mut allow_wildcard_origin = false;
    for value in values {
        let origin = required_static_string_value(prop, value)?;
        if origin == "*" {
            if credentials {
                return Err(prop_error(
                    prop,
                    "`cors.origins` cannot contain `*` when `credentials:true`",
                ));
            }
            if allow_wildcard_origin {
                return Err(prop_error(prop, "duplicate CORS origin `*`"));
            }
            allow_wildcard_origin = true;
            continue;
        }
        let normalized = normalize_cors_origin(&origin)
            .ok_or_else(|| prop_error(prop, format!("invalid CORS origin `{origin}`")))?;
        if !seen.insert(normalized.clone()) {
            return Err(prop_error(
                prop,
                format!("duplicate CORS origin `{normalized}`"),
            ));
        }
        origins.push(normalized);
    }
    origins.sort();
    Ok((origins, allow_wildcard_origin))
}

fn parse_cors_methods(prop: &SourceProp) -> DoweResult<Vec<String>> {
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(prop, "`cors.methods` must be an array"));
    };
    let mut methods = BTreeSet::new();
    for value in values {
        let method = required_static_string_value(prop, value)?;
        let normalized = normalize_cors_method(&method)
            .ok_or_else(|| prop_error(prop, format!("invalid CORS method `{method}`")))?;
        methods.insert(normalized.to_string());
    }
    Ok(methods.into_iter().collect())
}

fn parse_header_array(prop: &SourceProp, field: &str) -> DoweResult<Vec<String>> {
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(
            prop,
            format!("`{field}` must be an array of strings"),
        ));
    };
    let mut headers = Vec::new();
    let mut seen = BTreeSet::new();
    for value in values {
        let header = required_static_string_value(prop, value)?;
        let normalized = normalize_http_header_name(&header)
            .ok_or_else(|| prop_error(prop, format!("invalid HTTP header `{header}`")))?;
        if !seen.insert(normalized.to_ascii_lowercase()) {
            return Err(prop_error(
                prop,
                format!("duplicate HTTP header `{normalized}`"),
            ));
        }
        headers.push(normalized);
    }
    headers.sort();
    Ok(headers)
}

fn parse_boolean_prop(prop: &SourceProp, field: &str) -> DoweResult<bool> {
    match &prop.value {
        SourceValue::Boolean(value) => Ok(*value),
        _ => Err(prop_error(prop, format!("`{field}` must be a boolean"))),
    }
}

fn parse_max_age(prop: &SourceProp) -> DoweResult<u32> {
    let value = prop
        .value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, "`cors.maxAge` must be a non-negative integer"))?;
    value
        .parse::<u32>()
        .map_err(|_| prop_error(prop, "`cors.maxAge` must be a non-negative integer"))
}

