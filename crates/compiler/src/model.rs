use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

pub use dowe_components::{DesignConfig, FontConfig, TranslationCatalog, ViewNode, ViewRoute};
pub use dowe_generator_web::{ChunkKind, GeneratedChunk, ViewPage, WebOutput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledProject {
    pub root: PathBuf,
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
    pub apps: AppOutput,
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
    pub required: bool,
    pub default_value: Option<String>,
    pub resolved_source: EnvironmentValueSource,
    pub resolved_value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentVisibility {
    Server,
    Client,
}

impl EnvironmentVisibility {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "server" => Some(Self::Server),
            "client" => Some(Self::Client),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Server => "server",
            Self::Client => "client",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentValueSource {
    DotEnv,
    Os,
    Default,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub port: u16,
    pub endpoints: Vec<Endpoint>,
    pub websockets: Vec<WebSocketRoute>,
    pub init_action: ServerAction,
    pub cors: CorsConfig,
}

impl ServerConfig {
    pub fn find_endpoint(&self, method: &HttpMethod, path: &str) -> Option<MatchedEndpoint> {
        self.endpoints.iter().find_map(|endpoint| {
            if &endpoint.method != method {
                return None;
            }

            match_route(&endpoint.path, path).map(|params| MatchedEndpoint {
                endpoint: endpoint.clone(),
                params,
            })
        })
    }

    pub fn has_endpoint_path(&self, path: &str) -> bool {
        self.endpoints
            .iter()
            .any(|endpoint| match_route(&endpoint.path, path).is_some())
    }

    pub fn methods_for_path(&self, path: &str) -> Vec<HttpMethod> {
        let mut methods = Vec::new();
        for endpoint in &self.endpoints {
            if match_route(&endpoint.path, path).is_some() && !methods.contains(&endpoint.method) {
                methods.push(endpoint.method);
            }
        }
        methods
    }

    pub fn has_websocket(&self, path: &str) -> bool {
        self.find_websocket(path).is_some()
    }

    pub fn find_websocket(&self, path: &str) -> Option<WebSocketRoute> {
        self.websockets
            .iter()
            .find(|websocket| websocket.path.as_str() == path)
            .cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectServerConfig {
    pub backend_cors: CorsConfig,
    pub desktop_cors: CorsConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorsConfig {
    pub enabled: bool,
    pub origins: Vec<String>,
    pub allow_wildcard_origin: bool,
    pub allow_dev_origins: bool,
    pub methods: Vec<String>,
    pub headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub credentials: bool,
    pub max_age: Option<u32>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            origins: Vec::new(),
            allow_wildcard_origin: false,
            allow_dev_origins: false,
            methods: Vec::new(),
            headers: Vec::new(),
            expose_headers: Vec::new(),
            credentials: false,
            max_age: None,
        }
    }
}

impl CorsConfig {
    pub fn allowed_origin(&self, origin: &str, dev_origins: &[String]) -> Option<String> {
        if !self.enabled {
            return None;
        }
        if self.allow_wildcard_origin {
            return Some("*".to_string());
        }
        let origin = normalize_cors_origin(origin)?;
        if self.origins.iter().any(|allowed| allowed == &origin) {
            return Some(origin);
        }
        if self.allow_dev_origins
            && dev_origins.iter().any(|allowed| {
                normalize_cors_origin(allowed)
                    .as_ref()
                    .is_some_and(|allowed| allowed == &origin)
            })
        {
            return Some(origin);
        }
        None
    }

    pub fn allows_method(&self, method: &str) -> bool {
        let Some(method) = normalize_cors_method(method) else {
            return false;
        };
        self.methods.is_empty() || self.methods.iter().any(|allowed| allowed == method)
    }

    pub fn allows_headers(&self, headers: &[String]) -> bool {
        headers.iter().all(|header| {
            normalize_http_header_name(header).is_some_and(|header| {
                self.headers
                    .iter()
                    .any(|allowed| allowed.eq_ignore_ascii_case(&header))
            })
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub method: HttpMethod,
    pub path: String,
    pub behavior: EndpointBehavior,
    pub action: ServerAction,
    pub middlewares: Vec<ServerMiddleware>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchedEndpoint {
    pub endpoint: Endpoint,
    pub params: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointBehavior {
    StaticText(String),
    TextTemplate(String),
    UserGreeting,
    CreatePostJson,
    HttpProxy(HttpProxyEndpoint),
    HttpActionJson(HttpActionJsonEndpoint),
    AgentResponse(AgentResponseEndpoint),
    StoreInsertJson(StoreInsertEndpoint),
    StoreQueryJson(StoreQueryEndpoint),
    StoreTransactionJson(StoreTransactionEndpoint),
    StoreActionJson(StoreActionJsonEndpoint),
    KvActionJson(KvActionJsonEndpoint),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerMiddleware {
    pub name: String,
    pub action: ServerMiddlewareAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerMiddlewareAction {
    pub statements: Vec<ServerMiddlewareStatement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerMiddlewareStatement {
    Log(ServerLog),
    Header {
        binding: String,
        name: String,
    },
    Bearer {
        binding: String,
        source: String,
    },
    JwtVerify {
        binding: String,
        token: String,
        secret: ServerSecret,
        algorithm: String,
    },
    JwtSign {
        binding: String,
        claims: StoreLiteral,
        secret: ServerSecret,
        algorithm: String,
    },
    JwtDecrypt {
        binding: String,
        token: String,
        key: ServerSecret,
        algorithm: String,
        encryption: String,
    },
    JwtEncrypt {
        binding: String,
        claims: StoreLiteral,
        key: ServerSecret,
        algorithm: String,
        encryption: String,
    },
    IfValid {
        binding: String,
        statements: Vec<ServerMiddlewareStatement>,
    },
    Continue {
        context: Option<StoreLiteral>,
    },
    Response {
        status: u16,
        body: ServerMiddlewareResponseBody,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerMiddlewareResponseBody {
    Text(String),
    Json(StoreLiteral),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerSecret {
    Environment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpProxyEndpoint {
    pub binding: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpActionJsonEndpoint {
    pub status: u16,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentResponseEndpoint {
    pub upstream: String,
    pub request: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundHttpRequest {
    pub binding: String,
    pub method: HttpMethod,
    pub base: HttpConnectionValue,
    pub path: String,
    pub bearer: Option<ServerSecret>,
    pub json: Option<StoreLiteral>,
    pub mode: HttpResponseMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpConnectionValue {
    Static(String),
    Environment(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpResponseMode {
    Json,
    Proxy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentChatTransform {
    pub binding: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketJsonStatement {
    pub binding: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketSendJsonStatement {
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketSseBridgeStatement {
    pub upstream: String,
    pub request_id: String,
    pub request_type: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreInsertEndpoint {
    pub connection: StoreConnection,
    pub table: String,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreQueryEndpoint {
    pub connection: StoreConnection,
    pub sql: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreConnection {
    pub database: String,
    pub remote: Option<StoreRemoteConnection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRemoteConnection {
    pub host: StoreConnectionValue,
    pub user: StoreConnectionValue,
    pub credential: StoreCredential,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreCredential {
    Token(StoreConnectionValue),
    Password(StoreConnectionValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreConnectionValue {
    Static(String),
    Environment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreTransactionEndpoint {
    pub database: String,
    pub operations: Vec<StoreTransactionOperation>,
    pub return_binding: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreActionJsonEndpoint {
    pub status: u16,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvActionJsonEndpoint {
    pub status: u16,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvConnection {
    pub database: String,
    pub persist: bool,
    pub remote: Option<KvRemoteConnection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvRemoteConnection {
    pub host: KvConnectionValue,
    pub user: KvConnectionValue,
    pub credential: KvCredential,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KvCredential {
    Token(KvConnectionValue),
    Password(KvConnectionValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KvConnectionValue {
    Static(String),
    Environment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreTransactionOperation {
    Insert {
        binding: String,
        table: String,
        value: StoreLiteral,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketRoute {
    pub path: String,
    pub handlers: WebSocketHandlers,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WebSocketHandlers {
    pub open: ServerAction,
    pub message: ServerAction,
    pub close: ServerAction,
    pub drain: ServerAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerAction {
    pub statements: Vec<ServerStatement>,
}

impl ServerAction {
    pub fn empty() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerStatement {
    Log(ServerLog),
    RequestJson {
        binding: String,
        schema: Option<DoweType>,
    },
    Http(OutboundHttpRequest),
    AgentChat(AgentChatTransform),
    WebSocketJson(WebSocketJsonStatement),
    WebSocketSendJson(WebSocketSendJsonStatement),
    WebSocketSseBridge(WebSocketSseBridgeStatement),
    Store(ServerStoreStatement),
    Kv(ServerKvStatement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerStoreStatement {
    Handle {
        binding: String,
        database: String,
        remote: Option<StoreRemoteConnection>,
    },
    Insert {
        binding: String,
        handle: String,
        table: String,
        value: StoreLiteral,
        required: Vec<String>,
    },
    List {
        binding: String,
        handle: String,
        table: String,
    },
    Read {
        binding: String,
        handle: String,
        table: String,
        filter: StoreFilter,
        required: bool,
    },
    Update {
        binding: String,
        handle: String,
        table: String,
        filter: StoreFilter,
        value: StoreLiteral,
        required: bool,
        matches: Vec<StoreMatchField>,
    },
    Delete {
        binding: String,
        handle: String,
        table: String,
        filter: StoreFilter,
        required: bool,
    },
    Query {
        binding: String,
        handle: String,
        sql: String,
    },
    Transaction {
        binding: String,
        handle: String,
        operations: Vec<StoreTransactionOperation>,
        return_binding: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerKvStatement {
    Handle {
        binding: String,
        database: String,
        persist: bool,
        remote: Option<KvRemoteConnection>,
    },
    Get {
        binding: String,
        handle: String,
        key: String,
        required: bool,
    },
    Set {
        binding: String,
        handle: String,
        key: String,
        value: StoreLiteral,
    },
    Delete {
        binding: String,
        handle: String,
        key: String,
    },
    Keys {
        binding: String,
        handle: String,
        prefix: Option<String>,
    },
    Clear {
        binding: String,
        handle: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreFilter {
    pub field: String,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreMatchField {
    pub field: String,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreLiteral {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Reference(String),
    Array(Vec<StoreLiteral>),
    Object(Vec<(String, StoreLiteral)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoweType {
    Unknown,
    Null,
    Bool,
    Number,
    String,
    Array(Box<DoweType>),
    Object(Vec<DoweTypeField>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoweTypeField {
    pub name: String,
    pub value: DoweType,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLog {
    pub level: ServerLogLevel,
    pub values: Vec<ServerLogValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerLogLevel {
    Log,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerLogValue {
    String(String),
    Reference(String),
    Number(String),
    Boolean(bool),
    Null,
    JsonLiteral(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
        }
    }
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            _ => Err(()),
        }
    }
}

pub fn normalize_cors_method(value: &str) -> Option<&'static str> {
    match value.to_ascii_uppercase().as_str() {
        "GET" => Some("GET"),
        "POST" => Some("POST"),
        "PUT" => Some("PUT"),
        "DELETE" => Some("DELETE"),
        "PATCH" => Some("PATCH"),
        "HEAD" => Some("HEAD"),
        _ => None,
    }
}

pub fn normalize_http_header_name(value: &str) -> Option<String> {
    if !is_http_header_name(value) {
        return None;
    }
    Some(
        value
            .split('-')
            .map(|part| {
                let mut chars = part.chars();
                let Some(first) = chars.next() else {
                    return String::new();
                };
                let mut output = String::new();
                output.push(first.to_ascii_uppercase());
                output.push_str(&chars.as_str().to_ascii_lowercase());
                output
            })
            .collect::<Vec<_>>()
            .join("-"),
    )
}

pub fn normalize_cors_origin(value: &str) -> Option<String> {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return None;
    }
    let (scheme, rest) = if let Some(rest) = value.strip_prefix("http://") {
        ("http", rest)
    } else if let Some(rest) = value.strip_prefix("https://") {
        ("https", rest)
    } else {
        return None;
    };
    if rest.is_empty()
        || rest.contains('/')
        || rest.contains('?')
        || rest.contains('#')
        || rest.contains('@')
    {
        return None;
    }
    if let Some((host, port)) = rest.rsplit_once(':') {
        if host.is_empty() || port.is_empty() {
            return None;
        }
        let Ok(port) = port.parse::<u16>() else {
            return None;
        };
        Some(format!("{scheme}://{}:{port}", host.to_ascii_lowercase()))
    } else {
        Some(format!("{scheme}://{}", rest.to_ascii_lowercase()))
    }
}

fn is_http_header_name(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|value| {
            value.is_ascii_alphanumeric()
                || matches!(
                    value,
                    '!' | '#'
                        | '$'
                        | '%'
                        | '&'
                        | '\''
                        | '*'
                        | '+'
                        | '-'
                        | '.'
                        | '^'
                        | '_'
                        | '`'
                        | '|'
                        | '~'
                )
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppOutput {
    pub files: Vec<GeneratedFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewTargetRoutes {
    pub web: Vec<ViewRoute>,
    pub desktop: Vec<ViewRoute>,
    pub android: Vec<ViewRoute>,
    pub ios: Vec<ViewRoute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ViewPlatform {
    Web,
    Desktop,
    Android,
    Ios,
}

impl ViewPlatform {
    pub fn all() -> &'static [Self] {
        &[Self::Web, Self::Desktop, Self::Android, Self::Ios]
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "web" => Some(Self::Web),
            "desktop" => Some(Self::Desktop),
            "android" => Some(Self::Android),
            "ios" => Some(Self::Ios),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Desktop => "desktop",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFile {
    pub relative_path: PathBuf,
    pub content: String,
    pub kind: String,
    pub target: String,
}

fn match_route(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_segments = pattern.trim_matches('/').split('/').collect::<Vec<_>>();
    let path_segments = path.trim_matches('/').split('/').collect::<Vec<_>>();

    if pattern == "/" && path == "/" {
        return Some(HashMap::new());
    }

    if pattern_segments.len() != path_segments.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (pattern_segment, path_segment) in pattern_segments.iter().zip(path_segments.iter()) {
        if let Some(param_name) = pattern_segment.strip_prefix(':') {
            params.insert(param_name.to_string(), (*path_segment).to_string());
        } else if pattern_segment != path_segment {
            return None;
        }
    }

    Some(params)
}

fn escape_json_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::{Endpoint, EndpointBehavior, HttpMethod, ServerAction, ServerConfig};

    #[test]
    fn matches_dynamic_routes() {
        let server = ServerConfig {
            port: 8080,
            endpoints: vec![Endpoint {
                method: HttpMethod::Get,
                path: "/users/:id".to_string(),
                behavior: EndpointBehavior::UserGreeting,
                action: ServerAction::empty(),
                middlewares: Vec::new(),
            }],
            websockets: Vec::new(),
            init_action: ServerAction::empty(),
            cors: super::CorsConfig::default(),
        };

        let matched = server
            .find_endpoint(&HttpMethod::Get, "/users/123")
            .expect("endpoint");

        assert_eq!(matched.params.get("id"), Some(&"123".to_string()));
    }
}
