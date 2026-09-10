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

