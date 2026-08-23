use super::*;
use serde::Serialize;
use std::path::PathBuf;

fn escape_json_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledProject {
    pub root: PathBuf,
    pub capabilities: ProjectCapabilities,
    pub app_config: AppConfig,
    pub font_config: FontConfig,
    pub design_config: DesignConfig,
    pub environment_config: EnvironmentConfig,
    pub translations: TranslationCatalog,
    pub backend: ServerConfig,
    pub desktop_server: Option<ServerConfig>,
    pub databases: Vec<DatabaseBinding>,
    pub server_inspector: Option<ServerInspectorManifest>,
    pub local_databases: bool,
    pub web: WebOutput,
    pub desktop_web: WebOutput,
    pub view_routes: ViewTargetRoutes,
    pub apps: AppOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorManifest {
    pub schema_version: u32,
    pub port: u16,
    pub routes: Vec<ServerInspectorRoute>,
    pub websockets: Vec<ServerInspectorWebSocket>,
    pub nodes: Vec<ServerInspectorNode>,
    pub edges: Vec<ServerInspectorEdge>,
    pub resources: Vec<ServerInspectorResource>,
    pub entities: Vec<ServerInspectorEntity>,
    pub jobs: Vec<ServerInspectorJob>,
    pub services: Vec<ServerInspectorService>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorSource {
    pub path: String,
    pub line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorRoute {
    pub id: String,
    pub method: String,
    pub path: String,
    pub behavior: String,
    pub source: Option<ServerInspectorSource>,
    pub handler: Option<String>,
    pub parameters: Vec<ServerInspectorParameter>,
    pub headers: Vec<ServerInspectorHeader>,
    pub body: Option<ServerInspectorBody>,
    pub middleware: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorWebSocket {
    pub id: String,
    pub path: String,
    pub source: Option<ServerInspectorSource>,
    pub middleware: Vec<String>,
    pub message_format: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorParameter {
    pub name: String,
    pub location: String,
    pub required: bool,
    pub field_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorHeader {
    pub name: String,
    pub required: bool,
    pub sensitive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorBody {
    pub content_type: String,
    pub required: bool,
    pub fields: Vec<ServerInspectorBodyField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorBodyField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorNode {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub source: Option<ServerInspectorSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorResource {
    pub id: String,
    pub kind: String,
    pub binding: String,
    pub provider: String,
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorEntity {
    pub id: String,
    pub binding: String,
    pub database: String,
    pub table: String,
    pub fields: Vec<String>,
    pub field_details: Vec<ServerInspectorEntityField>,
    pub provider: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorEntityField {
    pub name: String,
    pub field_type: String,
    pub primary: bool,
    pub required: bool,
    pub unique: bool,
    pub index: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorJob {
    pub id: String,
    pub kind: String,
    pub target: Option<String>,
    pub schedule: Option<String>,
    pub source: Option<ServerInspectorSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerInspectorService {
    pub kind: String,
    pub enabled: bool,
    pub endpoint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectCapabilities {
    pub server: bool,
    pub views: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileEnvironment {
    Development,
    Live,
    Stage,
    Uat,
}

impl CompileEnvironment {
    pub fn file_name(self) -> &'static str {
        match self {
            Self::Development => ".env",
            Self::Live => ".env.live",
            Self::Stage => ".env.stage",
            Self::Uat => ".env.uat",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub name: String,
    pub bundle: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Dowe Dev".to_string(),
            bundle: "dev.dowe.generated".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnvironmentConfig {
    pub variables: Vec<EnvironmentVariable>,
}

impl EnvironmentConfig {
    pub fn variable(&self, name: &str) -> Option<&EnvironmentVariable> {
        self.variables.iter().find(|variable| variable.name == name)
    }

    pub fn expose_to_client(&mut self, name: &str) {
        if let Some(variable) = self
            .variables
            .iter_mut()
            .find(|variable| variable.name == name)
        {
            variable.visibility = EnvironmentVisibility::Client;
        }
    }

    pub fn client_values(&self) -> Vec<(String, String)> {
        self.variables
            .iter()
            .filter(|variable| variable.visibility == EnvironmentVisibility::Client)
            .map(|variable| {
                (
                    variable.name.clone(),
                    variable.resolved_value.clone().unwrap_or_default(),
                )
            })
            .collect()
    }

    pub fn client_json(&self) -> String {
        let values = self
            .client_values()
            .into_iter()
            .map(|(name, value)| {
                format!(
                    r#""{}":"{}""#,
                    escape_json_string(&name),
                    escape_json_string(&value)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("{{{values}}}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentVariable {
    pub name: String,
    pub visibility: EnvironmentVisibility,
    pub resolved_source: EnvironmentValueSource,
    pub resolved_value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentVisibility {
    Server,
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentValueSource {
    DotEnv,
    Os,
    Missing,
}
