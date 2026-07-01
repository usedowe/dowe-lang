use crate::error::{DoweError, DoweResult};
use crate::language::model::{
    LanguageDiagnostic, LanguageDiagnosticSeverity, LanguageDocument, LanguageRange,
};
use crate::model::{
    DoweType, DoweTypeField, EnvironmentConfig, EnvironmentValueSource, EnvironmentVariable,
    EnvironmentVisibility,
};
use crate::parser::{
    SourceFile, SourceNode, SourceObjectEntry, SourceValue, parse_config_file,
    parse_project_config, parse_server_source, parse_source_file, parse_translation_catalog,
    parse_views_file, reference_fields_for_type, resolve_import, type_from_source_value,
    validate_server_module_source, validate_shared_type_source, validate_translation_source,
    validate_view_source,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn analyze_document(root: &Path, document: &LanguageDocument) -> Vec<LanguageDiagnostic> {
    let normalized_root = document_workspace_root(root, &document.path);
    let mut diagnostics = Vec::new();
    let file = match parse_source_file(&normalized_root, &document.path, document.source.clone()) {
        Ok(file) => file,
        Err(error) => {
            diagnostics.push(diagnostic_from_error(&error, &document.path));
            return diagnostics;
        }
    };

    diagnostics.extend(import_diagnostics(&normalized_root, &file));
    diagnostics.extend(surface_diagnostics(&normalized_root, &file));
    diagnostics
}

pub(crate) fn document_workspace_root(root: &Path, document_path: &Path) -> PathBuf {
    let normalized_document = normalize_path(document_path.to_path_buf());
    if let Some(candidate) = find_workspace_root(&normalized_document) {
        let src = normalize_path(candidate.join("src"));
        if normalized_document.starts_with(&src) {
            return candidate;
        }
    }
    let normalized_root = normalize_path(root.to_path_buf());
    if normalized_root.join("src").is_dir() {
        normalized_root
    } else {
        find_workspace_root(&normalized_document).unwrap_or(normalized_root)
    }
}

pub fn find_workspace_root(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };
    loop {
        let src = current.join("src");
        if src.is_dir() && contains_dowe_source(&src) {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn contains_dowe_source(path: &Path) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && contains_dowe_source(&path) {
            return true;
        }
        if path.extension().and_then(|value| value.to_str()) == Some("dowe") {
            return true;
        }
    }
    false
}

fn import_diagnostics(root: &Path, file: &SourceFile) -> Vec<LanguageDiagnostic> {
    let mut diagnostics = Vec::new();
    for import in &file.imports {
        let resolved = match resolve_import(root, &file.path, import) {
            Ok(path) => path,
            Err(error) => {
                diagnostics.push(diagnostic_from_error(&error, &file.path));
                continue;
            }
        };
        match read_source_file(root, &resolved) {
            Ok(target) => {
                if !exports_symbol(&target, &import.local) {
                    diagnostics.push(LanguageDiagnostic {
                        code: "DOWE_IMPORT_EXPORT".to_string(),
                        message: format!("import target does not export `{}`", import.local),
                        severity: LanguageDiagnosticSeverity::Error,
                        range: range_from_location(
                            import.location.line,
                            import.location.column,
                            import.local.len(),
                        ),
                    });
                }
            }
            Err(error) => diagnostics.push(diagnostic_from_error(&error, &resolved)),
        }
    }
    diagnostics
}

fn surface_diagnostics(root: &Path, file: &SourceFile) -> Vec<LanguageDiagnostic> {
    let mut diagnostics = Vec::new();
    let environment = environment_config(root).unwrap_or_default();
    let result = match source_surface(file) {
        SourceSurface::Config => validate_config_shape(root, file),
        SourceSurface::ViewModule => validate_view_source(root, file, &environment).map(|_| ()),
        SourceSurface::Views => validate_views_shape(root, file, &environment),
        SourceSurface::Translations => validate_translation_source(file),
        SourceSurface::SharedTypes => validate_shared_type_source(root, file),
        SourceSurface::Server => validate_server_shape(root, file),
        SourceSurface::LegacyServer => Err(DoweError::at_path(
            &file.path,
            "`src/server.dowe` has been renamed to `src/main.dowe`",
        )),
        SourceSurface::Middleware => validate_middleware_shape(root, file),
        SourceSurface::Handler => validate_handler_shape(root, file),
        SourceSurface::Service => validate_service_shape(root, file),
        SourceSurface::Repository => validate_repository_shape(root, file),
        SourceSurface::Unknown => Ok(()),
    };
    if let Err(error) = result {
        diagnostics.push(diagnostic_from_error(&error, &file.path));
    }
    diagnostics
}

fn source_surface(file: &SourceFile) -> SourceSurface {
    let relative = file.relative_path.to_string_lossy().replace('\\', "/");
    if relative == "src/config.dowe" {
        SourceSurface::Config
    } else if relative == "src/views.dowe" {
        SourceSurface::Views
    } else if relative == "src/main.dowe" {
        SourceSurface::Server
    } else if relative.starts_with("src/i18n/") {
        SourceSurface::Translations
    } else if relative.starts_with("src/types/") {
        SourceSurface::SharedTypes
    } else if relative == "src/server.dowe" {
        SourceSurface::LegacyServer
    } else if relative.starts_with("src/middlewares/") {
        SourceSurface::Middleware
    } else if relative.starts_with("src/services/") {
        SourceSurface::Service
    } else if relative.starts_with("src/repositories/") {
        SourceSurface::Repository
    } else if file.nodes.iter().any(|node| node.name == "middleware") {
        SourceSurface::Middleware
    } else if file.nodes.iter().any(|node| node.name == "service") {
        SourceSurface::Service
    } else if file.nodes.iter().any(|node| node.name == "repository") {
        SourceSurface::Repository
    } else if file
        .nodes
        .iter()
        .any(|node| matches!(node.name.as_str(), "page" | "layout"))
        && file
            .nodes
            .iter()
            .all(|node| matches!(node.name.as_str(), "type" | "page" | "layout"))
    {
        SourceSurface::ViewModule
    } else if file.nodes.iter().any(|node| node.name == "handler")
        && file
            .nodes
            .iter()
            .all(|node| matches!(node.name.as_str(), "type" | "handler"))
    {
        SourceSurface::Handler
    } else {
        SourceSurface::Unknown
    }
}

enum SourceSurface {
    Config,
    ViewModule,
    Views,
    Translations,
    SharedTypes,
    Server,
    LegacyServer,
    Middleware,
    Handler,
    Service,
    Repository,
    Unknown,
}

fn read_source_file(root: &Path, path: &Path) -> DoweResult<SourceFile> {
    let source =
        fs::read_to_string(path).map_err(|error| DoweError::at_path(path, error.to_string()))?;
    parse_source_file(root, path, source)
}

fn exports_symbol(file: &SourceFile, name: &str) -> bool {
    file.nodes.iter().any(|node| {
        matches!(
            node.name.as_str(),
            "layout"
                | "page"
                | "component"
                | "action"
                | "handler"
                | "middleware"
                | "service"
                | "repository"
                | "type"
        ) && node
            .args
            .first()
            .and_then(SourceValue::as_required_string)
            .is_some_and(|value| value == name)
    })
}

fn validate_config_shape(root: &Path, file: &SourceFile) -> DoweResult<()> {
    parse_config_file(root, file).map(|_| ())
}

fn validate_views_shape(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<()> {
    if file.nodes.len() != 1 || file.nodes[0].name != "views" {
        return Err(DoweError::at_path(
            &file.path,
            "`src/views.dowe` must declare one `views` block",
        ));
    }
    let translations = parse_translation_catalog(root)?;
    parse_views_file(root, file, environment, &translations)?;
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
            "`src/main.dowe` must declare one `main` block",
        ));
    }
    let environment = environment_config(root).unwrap_or_default();
    parse_server_source(root, file, &environment)?;
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

fn validate_service_shape(root: &Path, file: &SourceFile) -> DoweResult<()> {
    let environment = environment_config(root).unwrap_or_default();
    validate_server_module_source(root, file, &environment)
}

fn validate_repository_shape(root: &Path, file: &SourceFile) -> DoweResult<()> {
    let environment = environment_config(root).unwrap_or_default();
    validate_server_module_source(root, file, &environment)
}

pub(crate) fn environment_config(root: &Path) -> DoweResult<EnvironmentConfig> {
    parse_project_config(root)
        .map(|config| config.environment_config)
        .or_else(|_| manual_environment_config(root))
}

fn manual_environment_config(root: &Path) -> DoweResult<EnvironmentConfig> {
    let path = root.join("src/config.dowe");
    if !path.is_file() {
        return Ok(EnvironmentConfig::default());
    }
    let source =
        fs::read_to_string(&path).map_err(|error| DoweError::at_path(&path, error.to_string()))?;
    let file = parse_source_file(root, &path, source)?;
    let Some(config) = file.nodes.iter().find(|node| node.name == "config") else {
        return Ok(EnvironmentConfig::default());
    };
    let Some(env) = config.children.iter().find(|node| node.name == "env") else {
        return Ok(EnvironmentConfig::default());
    };
    let mut variables = Vec::new();
    for node in &env.children {
        if node.name != "variable" {
            continue;
        }
        let Some(name) = prop_string(node, "name") else {
            continue;
        };
        let visibility = prop_string(node, "visibility")
            .as_deref()
            .and_then(EnvironmentVisibility::from_name)
            .unwrap_or(EnvironmentVisibility::Server);
        variables.push(EnvironmentVariable {
            name,
            visibility,
            required: false,
            default_value: prop_string(node, "default"),
            resolved_source: EnvironmentValueSource::Missing,
            resolved_value: None,
        });
    }
    Ok(EnvironmentConfig { variables })
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
    } else if message.contains("unknown action") {
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
    find_reference_fields(&file.nodes, &tables, &types, reference_root)
        .or_else(|| find_each_item_fields(&file.nodes, &file.nodes, &types, reference_root))
        .unwrap_or_default()
}

fn collect_store_table_fields(nodes: &[SourceNode], tables: &mut HashMap<String, DoweType>) {
    for node in nodes {
        if let Some((_, fields)) = store_binding_fields(node, tables)
            && assignment_expression(node)
                .is_some_and(|(_, expression)| expression.ends_with(".insert"))
            && let Some(table) = prop_string(node, "table")
        {
            tables.insert(table, fields);
        }
        collect_store_table_fields(&node.children, tables);
    }
}

fn find_reference_fields(
    nodes: &[SourceNode],
    tables: &HashMap<String, DoweType>,
    types: &crate::parser::TypeRegistry,
    reference_root: &str,
) -> Option<Vec<String>> {
    for node in nodes {
        if node.name == "signal"
            && node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|name| name == reference_root)
        {
            return signal_type(node, types)
                .map(|value| reference_fields_for_type(&value))
                .or_else(|| node.prop("value").map(|prop| signal_fields(&prop.value)));
        }
        if let Some((binding, fields)) = request_json_binding_fields(node, types)
            && binding == reference_root
        {
            return Some(reference_fields_for_type(&fields));
        }
        if let Some((binding, fields)) = store_binding_fields(node, tables)
            && binding == reference_root
        {
            return Some(reference_fields_for_type(&fields));
        }
        if let Some((binding, fields)) = kv_binding_fields(node)
            && binding == reference_root
        {
            return Some(reference_fields_for_type(&fields));
        }
        if let Some(fields) = find_reference_fields(&node.children, tables, types, reference_root) {
            return Some(fields);
        }
    }
    None
}

fn store_binding_fields(
    node: &SourceNode,
    tables: &HashMap<String, DoweType>,
) -> Option<(String, DoweType)> {
    let (binding, expression) = assignment_expression(node)?;
    if expression.ends_with(".insert") {
        let value = node.prop("value")?;
        let mut schema = type_from_source_value(&value.value);
        if let DoweType::Object(fields) = &mut schema
            && !fields.iter().any(|field| field.name == "id")
        {
            fields.push(DoweTypeField {
                name: "id".to_string(),
                value: DoweType::String,
                optional: false,
            });
        }
        return Some((binding, schema));
    }
    if expression.ends_with(".read") {
        let table = prop_string(node, "table")?;
        return tables.get(&table).cloned().map(|fields| (binding, fields));
    }
    if expression.ends_with(".update") || expression.ends_with(".delete") {
        return Some((
            binding,
            DoweType::Object(vec![DoweTypeField {
                name: "changed".to_string(),
                value: DoweType::Number,
                optional: false,
            }]),
        ));
    }
    None
}

fn kv_binding_fields(node: &SourceNode) -> Option<(String, DoweType)> {
    let (binding, expression) = assignment_expression(node)?;
    if expression.ends_with(".set") {
        return Some((
            binding,
            DoweType::Object(vec![
                DoweTypeField {
                    name: "ok".to_string(),
                    value: DoweType::Bool,
                    optional: false,
                },
                DoweTypeField {
                    name: "key".to_string(),
                    value: DoweType::String,
                    optional: false,
                },
            ]),
        ));
    }
    if expression.ends_with(".delete") {
        return Some((
            binding,
            DoweType::Object(vec![DoweTypeField {
                name: "deleted".to_string(),
                value: DoweType::Bool,
                optional: false,
            }]),
        ));
    }
    if expression.ends_with(".clear") {
        return Some((
            binding,
            DoweType::Object(vec![DoweTypeField {
                name: "cleared".to_string(),
                value: DoweType::Number,
                optional: false,
            }]),
        ));
    }
    None
}

fn request_json_binding_fields(
    node: &SourceNode,
    types: &crate::parser::TypeRegistry,
) -> Option<(String, DoweType)> {
    if node.name != "let" || node.args.len() != 4 {
        return None;
    }
    let binding = node.args[0].as_string_like()?;
    let (_, type_name) = binding.split_once(':')?;
    if node.args[1].as_string_like()?.as_str() != "="
        || node.args[2].as_string_like()?.as_str() != "await"
        || node.args[3].as_string_like()?.as_str() != "req.json()"
    {
        return None;
    }
    let (binding, _) = binding.split_once(':')?;
    types
        .resolve(node, type_name)
        .ok()
        .map(|value| (binding.to_string(), value))
}

fn signal_type(node: &SourceNode, types: &crate::parser::TypeRegistry) -> Option<DoweType> {
    let name = prop_string(node, "type")?;
    types.resolve(node, &name).ok()
}

fn find_each_item_fields(
    nodes: &[SourceNode],
    root_nodes: &[SourceNode],
    types: &crate::parser::TypeRegistry,
    reference_root: &str,
) -> Option<Vec<String>> {
    for node in nodes {
        if node.name == "each"
            && node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|name| name == reference_root)
        {
            let collection = node.args.get(2).and_then(SourceValue::as_required_string)?;
            let collection_type = find_signal_type(root_nodes, types, &collection)?;
            if let DoweType::Array(item) = collection_type {
                return Some(reference_fields_for_type(&item));
            }
        }
        if let Some(fields) =
            find_each_item_fields(&node.children, root_nodes, types, reference_root)
        {
            return Some(fields);
        }
    }
    None
}

fn find_signal_type(
    nodes: &[SourceNode],
    types: &crate::parser::TypeRegistry,
    signal: &str,
) -> Option<DoweType> {
    for node in nodes {
        if node.name == "signal"
            && node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|name| name == signal)
        {
            return signal_type(node, types);
        }
        if let Some(value) = find_signal_type(&node.children, types, signal) {
            return Some(value);
        }
    }
    None
}

fn assignment_expression(node: &SourceNode) -> Option<(String, String)> {
    if node.name != "let" || node.args.len() < 3 {
        return None;
    }
    let binding = node.args[0].as_string_like()?;
    if node.args[1].as_string_like()?.as_str() != "=" {
        return None;
    }
    Some((binding, node.args[2].as_string_like()?))
}
