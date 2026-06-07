use crate::error::{DoweError, DoweResult};
use crate::model::{
    AppConfig, CorsConfig, EnvironmentConfig, EnvironmentValueSource, EnvironmentVariable,
    EnvironmentVisibility, ProjectServerConfig, normalize_cors_method, normalize_cors_origin,
    normalize_http_header_name,
};
use crate::parser::source_ast::{SourceFile, SourceNode, SourceProp, SourceValue};
use crate::parser::source_parser::parse_source_file;
use dowe_components::{
    ColorToken, DesignConfig, DesignRadii, DesignTheme, FontConfig, FontFamily,
    integrated_design_theme,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;

pub struct ParsedConfig {
    pub app_config: AppConfig,
    pub font_config: FontConfig,
    pub design_config: DesignConfig,
    pub environment_config: EnvironmentConfig,
    pub server_config: ProjectServerConfig,
}

struct RawEnvironmentVariable {
    node: SourceNode,
    name: String,
    visibility: EnvironmentVisibility,
    required: bool,
    default_value: Option<String>,
}

#[derive(Clone)]
struct RawCors {
    node: SourceNode,
    target: CorsTarget,
    config: CorsConfig,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CorsTarget {
    Server,
    Desktop,
    All,
}

#[derive(Clone)]
struct RawTheme {
    node: SourceNode,
    name: String,
    extends: Option<String>,
    colors: BTreeMap<ColorToken, String>,
    radii: PartialRadii,
}

#[derive(Clone, Default)]
struct PartialRadii {
    radius: Option<u16>,
    radius_box: Option<u16>,
    radius_ui: Option<u16>,
}

pub fn parse_project_config(root: &Path) -> DoweResult<ParsedConfig> {
    let json_path = root.join("dowe.json");
    if json_path.exists() {
        return Err(DoweError::at_path(
            &json_path,
            "`dowe.json` is no longer supported; move project configuration to `src/config.dowe`",
        ));
    }

    let config_path = root.join("src/config.dowe");
    if !config_path.exists() {
        return Ok(ParsedConfig {
            app_config: AppConfig::default(),
            font_config: FontConfig::default(),
            design_config: DesignConfig::default(),
            environment_config: EnvironmentConfig::default(),
            server_config: ProjectServerConfig::default(),
        });
    }

    let source = fs::read_to_string(&config_path)
        .map_err(|error| DoweError::at_path(&config_path, error.to_string()))?;
    let file = parse_source_file(root, &config_path, source)?;
    parse_config_file(root, &file)
}

pub(crate) fn parse_config_file(root: &Path, file: &SourceFile) -> DoweResult<ParsedConfig> {
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
    let mut environment_config = EnvironmentConfig::default();
    let mut server_config = ProjectServerConfig::default();
    let mut app_seen = false;
    let mut fonts_seen = false;
    let mut design_seen = false;
    let mut env_seen = false;
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
                if env_seen {
                    return Err(node_error(child, "duplicate `env` block"));
                }
                env_seen = true;
                environment_config = parse_env(root, child)?;
            }
            "server" => {
                if server_seen {
                    return Err(node_error(child, "duplicate `server` block"));
                }
                server_seen = true;
                server_config = parse_server_config(child)?;
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
        environment_config,
        server_config,
    })
}

fn parse_app(node: &SourceNode) -> DoweResult<AppConfig> {
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

fn parse_env(root: &Path, node: &SourceNode) -> DoweResult<EnvironmentConfig> {
    if !node.args.is_empty() || !node.props.is_empty() {
        return Err(node_error(node, "`env` does not accept args or props"));
    }
    let dotenv = parse_dotenv(root)?;
    let mut names = HashSet::new();
    let mut variables = Vec::new();
    for child in &node.children {
        if child.name != "variable" {
            return Err(node_error(
                child,
                format!("`{}` is not valid inside `env`", child.name),
            ));
        }
        let raw = parse_raw_environment_variable(child)?;
        if !names.insert(raw.name.clone()) {
            return Err(node_error(
                child,
                format!("duplicate environment variable `{}`", raw.name),
            ));
        }
        variables.push(resolve_environment_variable(&dotenv, raw)?);
    }
    variables.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(EnvironmentConfig { variables })
}

fn parse_raw_environment_variable(node: &SourceNode) -> DoweResult<RawEnvironmentVariable> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(node, "`variable` only accepts props"));
    }
    reject_unknown_props(node, &["name", "visibility", "required", "default"])?;
    let name = node
        .prop("name")
        .ok_or_else(|| node_error(node, "`variable` requires `name`"))
        .and_then(|prop| parse_environment_name_prop(prop, "variable.name"))?;
    let visibility = match node.prop("visibility") {
        Some(prop) => {
            let value = required_static_string_prop(prop)?;
            EnvironmentVisibility::from_name(&value).ok_or_else(|| {
                prop_error(prop, "`variable.visibility` must be `server` or `client`")
            })?
        }
        None => EnvironmentVisibility::Server,
    };
    let required = match node.prop("required") {
        Some(prop) => match &prop.value {
            SourceValue::Boolean(value) => *value,
            _ => return Err(prop_error(prop, "`variable.required` must be a boolean")),
        },
        None => false,
    };
    let default_value = match node.prop("default") {
        Some(prop) => match &prop.value {
            SourceValue::String(value) => Some(value.clone()),
            _ => {
                return Err(prop_error(
                    prop,
                    "`variable.default` must be a string literal",
                ));
            }
        },
        None => None,
    };
    Ok(RawEnvironmentVariable {
        node: node.clone(),
        name,
        visibility,
        required,
        default_value,
    })
}

fn resolve_environment_variable(
    dotenv: &BTreeMap<String, String>,
    raw: RawEnvironmentVariable,
) -> DoweResult<EnvironmentVariable> {
    let (resolved_source, resolved_value) = if let Some(value) = dotenv.get(&raw.name) {
        (EnvironmentValueSource::DotEnv, Some(value.clone()))
    } else if let Ok(value) = std::env::var(&raw.name) {
        (EnvironmentValueSource::Os, Some(value))
    } else if let Some(value) = raw.default_value.clone() {
        (EnvironmentValueSource::Default, Some(value))
    } else {
        (EnvironmentValueSource::Missing, None)
    };
    if raw.required && resolved_value.is_none() {
        return Err(node_error(
            &raw.node,
            format!(
                "required environment variable `{}` is not configured",
                raw.name
            ),
        ));
    }
    Ok(EnvironmentVariable {
        name: raw.name,
        visibility: raw.visibility,
        required: raw.required,
        default_value: raw.default_value,
        resolved_source,
        resolved_value,
    })
}

fn parse_dotenv(root: &Path) -> DoweResult<BTreeMap<String, String>> {
    let path = root.join(".env");
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let source =
        fs::read_to_string(&path).map_err(|error| DoweError::at_path(&path, error.to_string()))?;
    let mut values = BTreeMap::new();
    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            return Err(DoweError::at_path(
                &path,
                format!("{line_number}:1: malformed `.env` entry"),
            ));
        };
        let key = key.trim();
        if !is_valid_environment_name(key) {
            return Err(DoweError::at_path(
                &path,
                format!("{line_number}:1: invalid environment variable name `{key}`"),
            ));
        }
        if values.contains_key(key) {
            return Err(DoweError::at_path(
                &path,
                format!("{line_number}:1: duplicate `.env` key `{key}`"),
            ));
        }
        values.insert(key.to_string(), parse_dotenv_value(value.trim()));
    }
    Ok(values)
}

fn parse_dotenv_value(value: &str) -> String {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn parse_fonts(node: &SourceNode) -> DoweResult<FontConfig> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`fonts` only accepts `default` and `install` props",
        ));
    }
    reject_unknown_props(node, &["default", "install"])?;

    let default_family = match node.prop("default") {
        Some(prop) => parse_font_token(prop, "fonts.default")?,
        None => FontFamily::Inter,
    };
    let install = match node.prop("install") {
        Some(prop) => parse_install_fonts(prop)?,
        None => Vec::new(),
    };

    Ok(FontConfig {
        default_family,
        install,
    })
}

fn parse_install_fonts(prop: &SourceProp) -> DoweResult<Vec<FontFamily>> {
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(
            prop,
            "`fonts.install` must be an array of font tokens",
        ));
    };
    let mut seen = BTreeSet::new();
    let mut fonts = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let token = required_static_string_value(prop, value)?;
        let family = FontFamily::from_name(&token).ok_or_else(|| {
            prop_error(
                prop,
                format!("unknown font token `{token}` in `fonts.install[{index}]`"),
            )
        })?;
        if !seen.insert(family) {
            return Err(prop_error(
                prop,
                format!("duplicate font token `{token}` in `fonts.install`"),
            ));
        }
        fonts.push(family);
    }
    Ok(fonts)
}

fn parse_font_token(prop: &SourceProp, field: &str) -> DoweResult<FontFamily> {
    let token = required_static_string_prop(prop)?;
    FontFamily::from_name(&token)
        .ok_or_else(|| prop_error(prop, format!("unknown font token `{token}` in `{field}`")))
}

fn parse_design(node: &SourceNode) -> DoweResult<DesignConfig> {
    if !node.args.is_empty() {
        return Err(node_error(node, "`design` does not accept args"));
    }
    reject_unknown_props(node, &["defaultTheme"])?;
    let default_theme = match node.prop("defaultTheme") {
        Some(prop) => parse_theme_name_prop(prop, "design.defaultTheme")?,
        None => "light".to_string(),
    };

    let mut themes = HashMap::new();
    for child in &node.children {
        if child.name != "theme" {
            return Err(node_error(
                child,
                format!("`{}` is not valid inside `design`", child.name),
            ));
        }
        let theme = parse_raw_theme(child)?;
        if themes.insert(theme.name.clone(), theme.clone()).is_some() {
            return Err(node_error(
                child,
                format!("duplicate theme `{}`", theme.name),
            ));
        }
    }

    if !themes.contains_key(&default_theme) {
        return Err(node_error(
            node,
            format!("default theme `{default_theme}` is not declared"),
        ));
    }

    let mut resolving = HashSet::new();
    let mut resolved = HashMap::new();
    let mut names = themes.keys().cloned().collect::<Vec<_>>();
    names.sort();
    let mut output = Vec::new();
    for name in names {
        output.push(resolve_theme(
            &name,
            &themes,
            &mut resolving,
            &mut resolved,
        )?);
    }

    Ok(DesignConfig {
        default_theme,
        themes: output,
    })
}

fn parse_raw_theme(node: &SourceNode) -> DoweResult<RawTheme> {
    if !node.args.is_empty() {
        return Err(node_error(node, "`theme` does not accept args"));
    }
    reject_unknown_props(node, &["name", "extends"])?;
    let name_prop = node
        .prop("name")
        .ok_or_else(|| node_error(node, "`theme` requires `name`"))?;
    let name = parse_theme_name_prop(name_prop, "theme.name")?;
    let extends = match node.prop("extends") {
        Some(prop) => Some(parse_theme_name_prop(prop, "theme.extends")?),
        None => None,
    };
    let mut colors = BTreeMap::new();
    let mut radii = PartialRadii::default();

    for child in &node.children {
        match child.name.as_str() {
            "colors" => parse_colors(child, &mut colors)?,
            "radii" => parse_radii(child, &mut radii)?,
            _ => {
                return Err(node_error(
                    child,
                    format!("`{}` is not valid inside `theme`", child.name),
                ));
            }
        }
    }

    Ok(RawTheme {
        node: node.clone(),
        name,
        extends,
        colors,
        radii,
    })
}

fn parse_colors(node: &SourceNode, colors: &mut BTreeMap<ColorToken, String>) -> DoweResult<()> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(node, "`colors` only accepts color token props"));
    }
    for prop in &node.props {
        let token = ColorToken::from_name(&prop.name)
            .ok_or_else(|| prop_error(prop, format!("unknown color token `{}`", prop.name)))?;
        if colors.contains_key(&token) {
            return Err(prop_error(
                prop,
                format!("duplicate color token `{}`", prop.name),
            ));
        }
        let value = required_static_string_prop(prop)?;
        colors.insert(token, normalize_hex_color(prop, &value)?);
    }
    Ok(())
}

fn parse_radii(node: &SourceNode, radii: &mut PartialRadii) -> DoweResult<()> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(node, "`radii` only accepts radius token props"));
    }
    reject_unknown_props(node, &["radius", "radiusBox", "radiusUi"])?;
    for prop in &node.props {
        let value = parse_radius(prop)?;
        match prop.name.as_str() {
            "radius" => set_radius(prop, &mut radii.radius, value)?,
            "radiusBox" => set_radius(prop, &mut radii.radius_box, value)?,
            "radiusUi" => set_radius(prop, &mut radii.radius_ui, value)?,
            _ => {}
        }
    }
    Ok(())
}

fn set_radius(prop: &SourceProp, slot: &mut Option<u16>, value: u16) -> DoweResult<()> {
    if slot.replace(value).is_some() {
        return Err(prop_error(
            prop,
            format!("duplicate radius token `{}`", prop.name),
        ));
    }
    Ok(())
}

fn parse_radius(prop: &SourceProp) -> DoweResult<u16> {
    let value = prop.value.as_string_like().ok_or_else(|| {
        prop_error(
            prop,
            format!("radius token `{}` must be a number", prop.name),
        )
    })?;
    value.parse::<u16>().map_err(|_| {
        prop_error(
            prop,
            format!(
                "radius token `{}` must be a non-negative integer",
                prop.name
            ),
        )
    })
}

fn resolve_theme(
    name: &str,
    raw: &HashMap<String, RawTheme>,
    resolving: &mut HashSet<String>,
    resolved: &mut HashMap<String, DesignTheme>,
) -> DoweResult<DesignTheme> {
    if let Some(theme) = resolved.get(name) {
        return Ok(theme.clone());
    }
    let theme = raw
        .get(name)
        .ok_or_else(|| DoweError::new(format!("theme `{name}` is not declared")))?;
    if !resolving.insert(name.to_string()) {
        return Err(node_error(
            &theme.node,
            format!("theme `{name}` has cyclic inheritance"),
        ));
    }

    let base = match theme.extends.as_deref() {
        Some(parent) => {
            if raw.contains_key(parent) {
                Some(resolve_theme(parent, raw, resolving, resolved)?)
            } else {
                integrated_design_theme(parent)
                    .ok_or_else(|| {
                        node_error(
                            &theme.node,
                            format!("theme `{name}` extends unknown theme `{parent}`"),
                        )
                    })?
                    .into()
            }
        }
        None => integrated_design_theme(&theme.name),
    };

    let mut colors = base
        .as_ref()
        .map(|theme| theme.colors.clone())
        .unwrap_or_default();
    for (token, value) in &theme.colors {
        colors.insert(*token, value.clone());
    }

    let base_radii = base.as_ref().map(|theme| theme.radii);
    let radii = DesignRadii {
        radius: theme
            .radii
            .radius
            .or_else(|| base_radii.map(|value| value.radius))
            .ok_or_else(|| {
                node_error(&theme.node, format!("theme `{name}` is missing `radius`"))
            })?,
        radius_box: theme
            .radii
            .radius_box
            .or_else(|| base_radii.map(|value| value.radius_box))
            .ok_or_else(|| {
                node_error(
                    &theme.node,
                    format!("theme `{name}` is missing `radiusBox`"),
                )
            })?,
        radius_ui: theme
            .radii
            .radius_ui
            .or_else(|| base_radii.map(|value| value.radius_ui))
            .ok_or_else(|| {
                node_error(&theme.node, format!("theme `{name}` is missing `radiusUi`"))
            })?,
    };

    for token in ColorToken::all() {
        if !colors.contains_key(token) {
            return Err(node_error(
                &theme.node,
                format!("theme `{name}` is missing `{}`", token.as_str()),
            ));
        }
    }

    resolving.remove(name);
    let resolved_theme = DesignTheme {
        name: name.to_string(),
        colors,
        radii,
    };
    resolved.insert(name.to_string(), resolved_theme.clone());
    Ok(resolved_theme)
}

fn parse_theme_name_prop(prop: &SourceProp, field: &str) -> DoweResult<String> {
    let value = required_static_string_prop(prop)?;
    if is_valid_theme_name(&value) {
        Ok(value)
    } else {
        Err(prop_error(
            prop,
            format!("`{field}` must use lowercase letters, numbers, and hyphens"),
        ))
    }
}

fn parse_app_name_prop(prop: &SourceProp) -> DoweResult<String> {
    let value = required_static_string_prop(prop)?;
    if !value.trim().is_empty() && !value.contains('\n') && !value.contains('\r') {
        Ok(value)
    } else {
        Err(prop_error(
            prop,
            "`app.name` must be a non-empty single-line string",
        ))
    }
}

fn parse_app_bundle_prop(prop: &SourceProp) -> DoweResult<String> {
    let value = required_static_string_prop(prop)?;
    if is_valid_app_bundle(&value) {
        Ok(value)
    } else {
        Err(prop_error(
            prop,
            "`app.bundle` must be a reverse-DNS identifier such as `com.example.app`",
        ))
    }
}

fn parse_environment_name_prop(prop: &SourceProp, field: &str) -> DoweResult<String> {
    let value = required_static_string_prop(prop)?;
    if is_valid_environment_name(&value) {
        Ok(value)
    } else {
        Err(prop_error(
            prop,
            format!("`{field}` must use uppercase letters, numbers, and underscores"),
        ))
    }
}

fn is_valid_environment_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_uppercase())
        && chars.all(|value| value.is_ascii_uppercase() || value.is_ascii_digit() || value == '_')
}

fn is_valid_theme_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && chars.all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == '-')
}

fn is_valid_app_bundle(value: &str) -> bool {
    let segments = value.split('.').collect::<Vec<_>>();
    segments.len() >= 2
        && segments
            .iter()
            .all(|segment| is_valid_bundle_segment(segment))
}

fn is_valid_bundle_segment(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}

fn normalize_hex_color(prop: &SourceProp, value: &str) -> DoweResult<String> {
    let Some(raw) = value.strip_prefix('#') else {
        return Err(prop_error(
            prop,
            format!("color token `{}` must be a hex color", prop.name),
        ));
    };
    if !matches!(raw.len(), 3 | 6 | 8) || !raw.chars().all(|value| value.is_ascii_hexdigit()) {
        return Err(prop_error(
            prop,
            format!(
                "color token `{}` must be `#RGB`, `#RRGGBB`, or `#RRGGBBAA`",
                prop.name
            ),
        ));
    }
    let output = if raw.len() == 3 {
        raw.chars()
            .flat_map(|value| [value.to_ascii_lowercase(), value.to_ascii_lowercase()])
            .collect::<String>()
    } else {
        raw.to_ascii_lowercase()
    };
    Ok(format!("#{output}"))
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.contains(&prop.name.as_str()) {
            return Err(prop_error(
                prop,
                format!("`{}` is not valid on `{}`", prop.name, node.name),
            ));
        }
    }
    Ok(())
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}

fn prop_error(prop: &SourceProp, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &prop.location.path,
        format!(
            "{}:{}: {}",
            prop.location.line,
            prop.location.column,
            message.as_ref()
        ),
    )
}

fn required_static_string_prop(prop: &SourceProp) -> DoweResult<String> {
    required_static_string_value(prop, &prop.value)
}

fn required_static_string_value(prop: &SourceProp, value: &SourceValue) -> DoweResult<String> {
    match value {
        SourceValue::String(value) => Ok(value.clone()),
        _ => Err(quoted_static_string_error(prop)),
    }
}

fn quoted_static_string_error(prop: &SourceProp) -> DoweError {
    prop_error(
        prop,
        format!(
            "invalid value for prop `{}`: expected quoted static string literal",
            prop.name
        ),
    )
}
