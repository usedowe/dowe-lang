#[derive(Clone, Copy)]
enum SourceSurface {
    Config,
    Theme,
    RemovedEnvironment,
    ViewModule,
    ViewStore,
    Views,
    Translations,
    SharedTypes,
    ServerConfigModule,
    Server,
    LegacyServer,
    LegacyMain,
    LegacyTheme,
    Middleware,
    Handler,
    Function,
    LegacyServerFunction,
    Test,
    Unknown,
}

fn read_source_file(root: &Path, path: &Path) -> DoweResult<SourceFile> {
    let source =
        fs::read_to_string(path).map_err(|error| DoweError::at_path(path, error.to_string()))?;
    parse_source_file(root, path, source)
}

pub(super) fn exports_symbol(file: &SourceFile, name: &str) -> bool {
    file.nodes.iter().any(|node| {
        matches!(
            node.name.as_str(),
            "layout"
                | "page"
                | "component"
                | "fn"
                | "handler"
                | "middleware"
                | "views"
                | "endpoints"
                | "type"
                | "database"
                | "cache"
                | "vector"
                | "queue"
                | "entity"
                | "seeder"
                | "store"
        ) && node
            .args
            .first()
            .and_then(SourceValue::as_required_string)
            .is_some_and(|value| value == name)
            || node.name == "let"
                && node
                    .args
                    .first()
                    .and_then(SourceValue::as_required_string)
                    .is_some_and(|value| value == name)
    })
}

fn validate_config_shape(root: &Path, file: &SourceFile) -> DoweResult<()> {
    parse_config_file(root, file).map(|_| ())
}

fn validate_theme_shape(file: &SourceFile) -> DoweResult<()> {
    parse_theme_file(file).map(|_| ())
}

fn validate_views_shape(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<()> {
    if file.nodes.len() != 1 || file.nodes[0].name != "views" {
        return Err(DoweError::at_path(
            &file.path,
            "views modules must declare one `views` block",
        ));
    }
    let translations = parse_translation_catalog(root)?;
    parse_views_file(
        root,
        file,
        environment,
        &translations,
        &DesignConfig::default(),
    )?;
    Ok(())
}

fn validate_server_shape(root: &Path, file: &SourceFile) -> DoweResult<()> {
    let main_count = file.nodes.iter().filter(|node| node.name == "main").count();
    if main_count != 1
        || file
            .nodes
            .iter()
            .any(|node| !matches!(node.name.as_str(), "type" | "main"))
    {
        return Err(DoweError::at_path(
            &file.path,
            "`main.dowe` must declare one `main` block",
        ));
    }
    let environment = environment_config(root).unwrap_or_default();
    let main = file
        .nodes
        .iter()
        .find(|node| node.name == "main")
        .expect("validated main block");
    if main.children.iter().any(|node| node.name == "server")
        || main.children.iter().any(|node| {
            node.name == "desktop" && node.children.iter().any(|child| child.name == "server")
        })
        || main.children.iter().any(|node| node.name == "ipc")
    {
        parse_server_source(root, file, &environment)?;
    }
    Ok(())
}

fn validate_handler_shape(root: &Path, file: &SourceFile) -> DoweResult<()> {
    let environment = environment_config(root).unwrap_or_default();
    validate_server_module_source(root, file, &environment)
}

fn validate_middleware_shape(root: &Path, file: &SourceFile) -> DoweResult<()> {
    let environment = environment_config(root).unwrap_or_default();
    validate_server_module_source(root, file, &environment)
}

fn validate_server_config_module_shape(root: &Path, file: &SourceFile) -> DoweResult<()> {
    let environment = environment_config(root).unwrap_or_default();
    validate_server_module_source(root, file, &environment)
}

pub(crate) fn environment_config(root: &Path) -> DoweResult<EnvironmentConfig> {
    parse_environment_files(root)
}

fn prop_string(node: &SourceNode, name: &str) -> Option<String> {
    node.prop(name)
        .and_then(|prop| prop.value.as_required_string())
}

pub(crate) fn diagnostic_from_error(error: &DoweError, fallback_path: &Path) -> LanguageDiagnostic {
    let message = error.message().to_string();
    let range = parse_error_range(&message).unwrap_or_else(|| LanguageRange::single_line(1, 1, 1));
    LanguageDiagnostic {
        code: diagnostic_code(&message).to_string(),
        message: strip_path_prefix(&message, fallback_path),
        severity: LanguageDiagnosticSeverity::Error,
        range,
    }
}

fn parse_error_range(message: &str) -> Option<LanguageRange> {
    let parts = message.split(':').collect::<Vec<_>>();
    for index in 0..parts.len().saturating_sub(2) {
        let Ok(line) = parts[index + 1].trim().parse::<usize>() else {
            continue;
        };
        let Ok(column) = parts[index + 2].trim().parse::<usize>() else {
            continue;
        };
        return Some(LanguageRange::single_line(
            line,
            column,
            diagnostic_token_length(message).unwrap_or(1),
        ));
    }
    None
}

fn diagnostic_token_length(message: &str) -> Option<usize> {
    let (_, after_open) = message.split_once('`')?;
    let (value, _) = after_open.split_once('`')?;
    let length = value.chars().count();
    if length == 0 { None } else { Some(length) }
}

fn strip_path_prefix(message: &str, path: &Path) -> String {
    let prefix = format!("{}: ", path.display());
    message.strip_prefix(&prefix).unwrap_or(message).to_string()
}

fn diagnostic_code(message: &str) -> &'static str {
    if message.contains("import") {
        "DOWE_IMPORT"
    } else if message.contains("indentation") || message.contains("tabs") {
        "DOWE_INDENT"
    } else if message.contains("unknown component") {
        "DOWE_COMPONENT"
    } else if message.contains("unknown prop")
        || message.contains("invalid prop")
        || message.contains("invalid value for prop")
    {
        "DOWE_PROP"
    } else if message.contains("environment variable") {
        "DOWE_ENV"
    } else if message.contains("unknown fn") {
        "DOWE_ACTION"
    } else {
        "DOWE_SOURCE"
    }
}

fn range_from_location(line: usize, column: usize, length: usize) -> LanguageRange {
    LanguageRange::single_line(line, column, length)
}

pub(crate) fn normalize_path(path: PathBuf) -> PathBuf {
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                output.pop();
            }
            _ => output.push(component.as_os_str()),
        }
    }
    output
}

pub(crate) fn signal_fields(value: &SourceValue) -> Vec<String> {
    match value {
        SourceValue::Object(entries) => entries
            .iter()
            .filter_map(|entry| match entry {
                SourceObjectEntry::KeyValue { key, .. } => Some(key.clone()),
                SourceObjectEntry::Spread(_) => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn reference_fields(
    root: &Path,
    document: &LanguageDocument,
    reference_root: &str,
) -> Vec<String> {
    let root = document_workspace_root(root, &document.path);
    let Ok(file) = parse_source_file(&root, &document.path, document.source.clone()) else {
        return Vec::new();
    };
    let types = crate::parser::TypeRegistry::parse_file(&root, &file).unwrap_or_default();
    let mut tables = HashMap::new();
    collect_store_table_fields(&file.nodes, &mut tables);
    imported_view_store_fields(&root, &file, reference_root)
        .or_else(|| find_reference_fields(&file.nodes, &tables, &types, reference_root))
        .or_else(|| find_each_item_fields(&file.nodes, &file.nodes, &types, reference_root))
        .unwrap_or_default()
}

