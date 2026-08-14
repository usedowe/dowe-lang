use crate::error::{DoweError, DoweResult};
use crate::model::{
    AppConfig, CompileEnvironment, DatabaseBinding, EnvironmentConfig, ProjectCapabilities,
    ServerConfig, ViewPlatform, ViewTargetRoutes, WebOutput,
};
use crate::parser::source_config::parse_app;
use crate::parser::source_config::parse_project_config_for;
use crate::parser::source_i18n::parse_translation_catalog;
use crate::parser::source_parser::parse_source_file;
use crate::parser::source_server::{parse_server_source, parse_server_source_without_seeders};
use crate::parser::source_views::{client_environment_names, parse_views_entry};
use dowe_components::{DesignConfig, FontConfig, TranslationCatalog};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub struct ParsedProject {
    pub capabilities: ProjectCapabilities,
    pub app_config: AppConfig,
    pub font_config: FontConfig,
    pub design_config: DesignConfig,
    pub environment_config: EnvironmentConfig,
    pub translations: TranslationCatalog,
    pub backend: ServerConfig,
    pub desktop_server: Option<ServerConfig>,
    pub databases: Vec<DatabaseBinding>,
    pub web: WebOutput,
    pub desktop_web: WebOutput,
    pub view_routes: ViewTargetRoutes,
}

pub(crate) fn parse_project_for(
    root: &Path,
    environment: CompileEnvironment,
    include_seeders: bool,
    compile_server: bool,
    compile_views: bool,
    selected_platforms: Option<&BTreeSet<ViewPlatform>>,
) -> DoweResult<ParsedProject> {
    let legacy_main_path = root.join("src/main.dowe");
    if legacy_main_path.exists() {
        return Err(DoweError::at_path(
            &legacy_main_path,
            "`src/main.dowe` has moved to project-root `main.dowe`",
        ));
    }
    let legacy_server_path = root.join("src/server.dowe");
    if compile_server && legacy_server_path.exists() {
        return Err(DoweError::at_path(
            &legacy_server_path,
            "`src/server.dowe` has been replaced by project-root `main.dowe`",
        ));
    }
    let server_path = root.join("main.dowe");
    let views_path = root.join("src/views.dowe");
    if compile_views && views_path.exists() {
        return Err(DoweError::at_path(
            &views_path,
            "`src/views.dowe` has been replaced by root `main.dowe` with `views:<name>`",
        ));
    }
    let server = parse_required_file(root, &server_path)?;
    let capabilities = capabilities_from_main(&server)?;
    let config = parse_project_config_for(root, environment, compile_views)?;
    let mut environment_config = config.environment_config;
    let translations = if compile_views {
        parse_translation_catalog(root)?
    } else {
        TranslationCatalog::default()
    };
    let app_config = parse_main_app_config(&server)?.unwrap_or(config.app_config);
    let view_platforms = selected_platforms
        .map(|platforms| platforms.iter().copied().collect::<Vec<_>>())
        .unwrap_or_else(|| ViewPlatform::all().to_vec());
    let views = (compile_views && capabilities.views && !view_platforms.is_empty())
        .then(|| {
            parse_views_entry(
                root,
                &server,
                &environment_config,
                &translations,
                &config.design_config,
                &view_platforms,
            )
        })
        .transpose()?;
    if let Some(views) = &views {
        for name in client_environment_names(&views.routes) {
            environment_config.expose_to_client(&name);
        }
    }
    let server_root = (compile_server && has_server_configuration(&server))
        .then(|| {
            if include_seeders {
                parse_server_source(root, &server, &environment_config)
            } else {
                parse_server_source_without_seeders(root, &server, &environment_config)
            }
        })
        .transpose()?;

    let databases = server_root
        .as_ref()
        .map(|server| server.databases.clone())
        .unwrap_or_default();
    Ok(ParsedProject {
        capabilities,
        app_config,
        font_config: config.font_config,
        design_config: config.design_config,
        environment_config,
        translations,
        backend: server_root
            .as_ref()
            .map(|server| server.backend.clone())
            .unwrap_or_default(),
        desktop_server: server_root.and_then(|server| server.desktop_server),
        databases,
        web: views
            .as_ref()
            .map(|views| views.web.clone())
            .unwrap_or_else(empty_web_output),
        desktop_web: views
            .as_ref()
            .map(|views| views.desktop_web.clone())
            .unwrap_or_else(empty_web_output),
        view_routes: views.map(|views| views.routes).unwrap_or_default(),
    })
}

fn has_server_configuration(file: &crate::parser::source_ast::SourceFile) -> bool {
    file.nodes
        .iter()
        .find(|node| node.name == "main")
        .is_some_and(|main| {
            main.children.iter().any(|node| node.name == "server")
                || main.children.iter().any(|node| {
                    node.name == "desktop"
                        && node.children.iter().any(|child| child.name == "server")
                })
        })
}

pub fn inspect_project_capabilities(root: &Path) -> DoweResult<ProjectCapabilities> {
    let legacy_path = root.join("src/main.dowe");
    if legacy_path.exists() {
        return Err(DoweError::at_path(
            &legacy_path,
            "`src/main.dowe` has moved to project-root `main.dowe`",
        ));
    }
    let path = root.join("main.dowe");
    let file = parse_required_file(root, &path)?;
    capabilities_from_main(&file)
}

fn capabilities_from_main(
    file: &crate::parser::source_ast::SourceFile,
) -> DoweResult<ProjectCapabilities> {
    let mains = file
        .nodes
        .iter()
        .filter(|node| node.name == "main")
        .collect::<Vec<_>>();
    let [main] = mains.as_slice() else {
        return Err(DoweError::at_path(
            &file.path,
            "`main.dowe` must declare exactly one `main` block",
        ));
    };
    Ok(ProjectCapabilities {
        server: main.children.iter().any(|node| node.name == "server"),
        views: main.children.iter().any(|node| node.name == "views"),
    })
}

fn empty_web_output() -> WebOutput {
    WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    }
}

fn parse_main_app_config(
    file: &crate::parser::source_ast::SourceFile,
) -> DoweResult<Option<AppConfig>> {
    let main = file
        .nodes
        .iter()
        .find(|node| node.name == "main")
        .ok_or_else(|| {
            DoweError::at_path(&file.path, "`main.dowe` must declare one `main` block")
        })?;
    let apps = main
        .children
        .iter()
        .filter(|node| node.name == "app")
        .collect::<Vec<_>>();
    match apps.as_slice() {
        [] => Ok(None),
        [app] => Ok(Some(parse_app(app)?)),
        _ => Err(DoweError::at_path(
            &file.path,
            "`main.dowe` must declare at most one `app` block",
        )),
    }
}

fn parse_required_file(
    root: &Path,
    path: &Path,
) -> DoweResult<crate::parser::source_ast::SourceFile> {
    let source =
        fs::read_to_string(path).map_err(|error| DoweError::at_path(path, error.to_string()))?;
    parse_source_file(root, path, source)
}
