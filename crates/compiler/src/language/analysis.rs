use crate::error::{DoweError, DoweResult};
use crate::language::model::{
    LanguageDiagnostic, LanguageDiagnosticSeverity, LanguageDocument, LanguageRange,
};
use crate::model::{DesignConfig, DoweType, DoweTypeField, EnvironmentConfig};
use crate::parser::{
    SourceFile, SourceNode, SourceObjectEntry, SourceValue, parse_config_file,
    parse_environment_files, parse_server_source, parse_source_file, parse_theme_file,
    parse_translation_catalog, parse_views_file, queue_publish_result_type,
    reference_fields_for_type, resolve_import, type_from_source_value,
    validate_server_module_source, validate_shared_type_source, validate_translation_source,
    validate_view_source, validate_view_store_source,
};
use crate::test_runner::validate_test_file;
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
        if normalized_document.starts_with(&candidate) {
            return candidate;
        }
    }
    let normalized_root = normalize_path(root.to_path_buf());
    if normalized_root.join("main.dowe").is_file() {
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
        if current.join("main.dowe").is_file() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn import_diagnostics(root: &Path, file: &SourceFile) -> Vec<LanguageDiagnostic> {
    let mut diagnostics = Vec::new();
    let surface = source_surface(file);
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
                if is_server_config_source(&target)
                    && !matches!(
                        surface,
                        SourceSurface::Server
                            | SourceSurface::Handler
                            | SourceSurface::Middleware
                            | SourceSurface::Function
                            | SourceSurface::ServerConfigModule
                    )
                {
                    diagnostics.push(LanguageDiagnostic {
                        code: "DOWE_IMPORT_SURFACE".to_string(),
                        message: "server config modules can only be imported by server source"
                            .to_string(),
                        severity: LanguageDiagnosticSeverity::Error,
                        range: range_from_location(
                            import.location.line,
                            import.location.column,
                            import.local.len(),
                        ),
                    });
                    continue;
                }
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

fn is_server_config_source(file: &SourceFile) -> bool {
    file.nodes.iter().any(|node| {
        matches!(
            node.name.as_str(),
            "database"
                | "entity"
                | "seeder"
                | "cache"
                | "kv"
                | "vector"
                | "emb"
                | "queue"
                | "let"
                | "query"
        )
    })
}

fn surface_diagnostics(root: &Path, file: &SourceFile) -> Vec<LanguageDiagnostic> {
    let mut diagnostics = Vec::new();
    let environment = environment_config(root).unwrap_or_default();
    let result = match source_surface(file) {
        SourceSurface::Config => validate_config_shape(root, file),
        SourceSurface::Theme => validate_theme_shape(file),
        SourceSurface::RemovedEnvironment => Err(DoweError::at_path(
            &file.path,
            "`env.dowe` is no longer supported; declare names in `.env.example`, values in `.env`, and keep using `env.NAME` in Dowe source",
        )),
        SourceSurface::ViewModule => validate_view_source(root, file, &environment).map(|_| ()),
        SourceSurface::ViewStore => validate_view_store_source(root, file),
        SourceSurface::Views => validate_views_shape(root, file, &environment),
        SourceSurface::Translations => validate_translation_source(file),
        SourceSurface::SharedTypes => validate_shared_type_source(root, file),
        SourceSurface::ServerConfigModule => validate_server_config_module_shape(root, file),
        SourceSurface::Server => validate_server_shape(root, file),
        SourceSurface::LegacyServer => Err(DoweError::at_path(
            &file.path,
            "`src/server.dowe` has been renamed to `main.dowe`",
        )),
        SourceSurface::LegacyMain => Err(DoweError::at_path(
            &file.path,
            "`src/main.dowe` has moved to project-root `main.dowe`",
        )),
        SourceSurface::LegacyTheme => Err(DoweError::at_path(
            &file.path,
            "`src/theme.dowe` has moved to project-root `theme.dowe`",
        )),
        SourceSurface::Middleware => validate_middleware_shape(root, file),
        SourceSurface::Handler => validate_handler_shape(root, file),
        SourceSurface::Function | SourceSurface::LegacyServerFunction => {
            validate_server_module_source(root, file, &environment)
        }
        SourceSurface::Test => validate_test_file(file).map(|_| ()),
        SourceSurface::Unknown => Ok(()),
    };
    if let Err(error) = result {
        diagnostics.push(diagnostic_from_error(&error, &file.path));
    }
    diagnostics
}

fn source_surface(file: &SourceFile) -> SourceSurface {
    let relative = file.relative_path.to_string_lossy().replace('\\', "/");
    if file.nodes.iter().any(|node| node.name == "test") {
        SourceSurface::Test
    } else if relative == "src/main.dowe" {
        SourceSurface::LegacyMain
    } else if relative == "src/theme.dowe" {
        SourceSurface::LegacyTheme
    } else if relative == "src/env.dowe" {
        SourceSurface::RemovedEnvironment
    } else if relative == "src/config.dowe" {
        SourceSurface::Config
    } else if relative == "theme.dowe" {
        SourceSurface::Theme
    } else if relative == "env.dowe" {
        SourceSurface::RemovedEnvironment
    } else if relative == "src/views.dowe" {
        SourceSurface::Views
    } else if file.nodes.iter().any(|node| node.name == "views") {
        SourceSurface::Views
    } else if relative == "main.dowe" {
        SourceSurface::Server
    } else if file.nodes.iter().any(|node| node.name == "translations") {
        SourceSurface::Translations
    } else if !file.nodes.is_empty() && file.nodes.iter().all(|node| node.name == "type") {
        SourceSurface::SharedTypes
    } else if file.nodes.iter().any(|node| node.name == "store") {
        SourceSurface::ViewStore
    } else if relative == "src/server.dowe" {
        SourceSurface::LegacyServer
    } else if file.nodes.iter().any(|node| node.name == "middleware") {
        SourceSurface::Middleware
    } else if file.nodes.iter().any(|node| node.name == "handler") {
        SourceSurface::Handler
    } else if file.nodes.iter().any(|node| node.name == "fn") {
        SourceSurface::Function
    } else if is_server_config_source(file) {
        SourceSurface::ServerConfigModule
    } else if file
        .nodes
        .iter()
        .any(|node| matches!(node.name.as_str(), "service" | "repository"))
    {
        SourceSurface::LegacyServerFunction
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

fn imported_view_store_fields(
    root: &Path,
    file: &SourceFile,
    reference_root: &str,
) -> Option<Vec<String>> {
    let import = file
        .imports
        .iter()
        .find(|import| import.local == reference_root)?;
    let path = resolve_import(root, &file.path, import).ok()?;
    let target = read_source_file(root, &path).ok()?;
    let store = target.nodes.iter().find(|node| {
        node.name == "store"
            && node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|name| name == reference_root)
    })?;
    let types = crate::parser::TypeRegistry::parse_file(root, &target).ok()?;
    store
        .prop("type")
        .and_then(|prop| prop.value.as_required_string())
        .and_then(|name| types.resolve(store, &name).ok())
        .map(|schema| reference_fields_for_type(&schema))
        .or_else(|| store.prop("value").map(|prop| signal_fields(&prop.value)))
}

fn collect_store_table_fields(nodes: &[SourceNode], tables: &mut HashMap<String, DoweType>) {
    for node in nodes {
        if let Some((_, fields)) = store_binding_fields(node, tables)
            && store_binding_expression(node)
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
        if matches!(node.name.as_str(), "signal" | "const")
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
        if let Some((binding, fields)) = vector_binding_fields(node)
            && binding == reference_root
        {
            return Some(reference_fields_for_type(&fields));
        }
        if let Some((binding, fields)) = queue_binding_fields(node)
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
    let (binding, expression) = store_binding_expression(node)?;
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

fn store_binding_expression(node: &SourceNode) -> Option<(String, String)> {
    if node.name == "query" {
        let binding = node.args.first()?.as_required_string()?;
        let expression = node.prop("db")?.value.as_required_string()?;
        return Some((binding, expression));
    }
    assignment_expression(node)
}

fn kv_binding_fields(node: &SourceNode) -> Option<(String, DoweType)> {
    let (binding, expression) = kv_binding_expression(node)?;
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

fn kv_binding_expression(node: &SourceNode) -> Option<(String, String)> {
    if node.name != "kv" {
        return None;
    }
    let binding = node.args.first()?.as_required_string()?;
    let expression = node.prop("conn")?.value.as_required_string()?;
    Some((binding, expression))
}

fn vector_binding_fields(node: &SourceNode) -> Option<(String, DoweType)> {
    if node.name != "emb" {
        return None;
    }
    let binding = node.args.first()?.as_required_string()?;
    let expression = node.prop("conn")?.value.as_required_string()?;
    let fields = if expression.ends_with(".upsert") {
        vec![
            DoweTypeField {
                name: "id".to_string(),
                value: DoweType::String,
                optional: false,
            },
            DoweTypeField {
                name: "dimensions".to_string(),
                value: DoweType::Number,
                optional: false,
            },
            DoweTypeField {
                name: "created".to_string(),
                value: DoweType::Bool,
                optional: false,
            },
        ]
    } else if expression.ends_with(".delete") {
        vec![DoweTypeField {
            name: "deleted".to_string(),
            value: DoweType::Bool,
            optional: false,
        }]
    } else if expression.ends_with(".read") {
        vec![
            DoweTypeField {
                name: "id".to_string(),
                value: DoweType::String,
                optional: false,
            },
            DoweTypeField {
                name: "vector".to_string(),
                value: DoweType::Array(Box::new(DoweType::Number)),
                optional: false,
            },
            DoweTypeField {
                name: "metadata".to_string(),
                value: DoweType::Unknown,
                optional: false,
            },
        ]
    } else {
        return None;
    };
    Some((binding, DoweType::Object(fields)))
}

fn queue_binding_fields(node: &SourceNode) -> Option<(String, DoweType)> {
    if node.name != "msg"
        || !node
            .prop("conn")?
            .value
            .as_required_string()?
            .ends_with(".publish")
    {
        return None;
    }
    let binding = node.args.first()?.as_required_string()?;
    Some((binding, queue_publish_result_type()))
}

fn request_json_binding_fields(
    node: &SourceNode,
    types: &crate::parser::TypeRegistry,
) -> Option<(String, DoweType)> {
    if node.name != "const" || node.args.len() != 1 {
        return None;
    }
    let binding = node.args[0].as_string_like()?;
    let (_, type_name) = binding.split_once(':')?;
    if node.prop("value")?.value.as_string_like()?.as_str() != "req.json" {
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
                .prop("as")
                .and_then(|prop| prop.value.as_required_string())
                .is_some_and(|name| name == reference_root)
        {
            let collection = node
                .prop("in")
                .and_then(|prop| prop.value.as_required_string())?;
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
