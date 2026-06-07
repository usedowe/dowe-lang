use crate::error::{DoweError, DoweResult};
use crate::model::{AppConfig, EnvironmentConfig, ServerConfig, ViewTargetRoutes, WebOutput};
use crate::parser::source_config::parse_project_config;
use crate::parser::source_discovery::discover_dowe_sources;
use crate::parser::source_i18n::parse_translation_catalog;
use crate::parser::source_parser::parse_source_file;
use crate::parser::source_server::parse_server_source;
use crate::parser::source_views::parse_views_file;
use dowe_components::{DesignConfig, FontConfig, TranslationCatalog};
use std::fs;
use std::path::Path;

pub struct ParsedProject {
    pub app_config: AppConfig,
    pub font_config: FontConfig,
    pub design_config: DesignConfig,
    pub environment_config: EnvironmentConfig,
    pub translations: TranslationCatalog,
    pub backend: ServerConfig,
    pub desktop_server: Option<ServerConfig>,
    pub web: WebOutput,
    pub desktop_web: WebOutput,
    pub view_routes: ViewTargetRoutes,
}

pub fn parse_project(root: &Path) -> DoweResult<ParsedProject> {
    let config = parse_project_config(root)?;
    discover_dowe_sources(root)?;
    let legacy_server_path = root.join("src/server.dowe");
    if legacy_server_path.exists() {
        return Err(DoweError::at_path(
            &legacy_server_path,
            "`src/server.dowe` has been renamed to `src/main.dowe`",
        ));
    }
    let server_path = root.join("src/main.dowe");
    let views_path = root.join("src/views.dowe");
    let server = parse_required_file(root, &server_path)?;
    let views = parse_required_file(root, &views_path)?;
    let translations = parse_translation_catalog(root)?;
    let mut server_root = parse_server_source(root, &server, &config.environment_config)?;
    let views = parse_views_file(root, &views, &config.environment_config, &translations)?;
    server_root.backend.cors = config.server_config.backend_cors;
    if let Some(desktop_server) = &mut server_root.desktop_server {
        desktop_server.cors = config.server_config.desktop_cors;
    }

    Ok(ParsedProject {
        app_config: config.app_config,
        font_config: config.font_config,
        design_config: config.design_config,
        environment_config: config.environment_config,
        translations,
        backend: server_root.backend,
        desktop_server: server_root.desktop_server,
        web: views.web,
        desktop_web: views.desktop_web,
        view_routes: views.routes,
    })
}

fn parse_required_file(
    root: &Path,
    path: &Path,
) -> DoweResult<crate::parser::source_ast::SourceFile> {
    let source =
        fs::read_to_string(path).map_err(|error| DoweError::at_path(path, error.to_string()))?;
    parse_source_file(root, path, source)
}
