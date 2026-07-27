use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use dowe_stdlib::StdlibCall;

pub use dowe_components::{DesignConfig, FontConfig, TranslationCatalog, ViewNode, ViewRoute};
pub use dowe_generator_web::{ChunkKind, GeneratedChunk, ViewPage, WebOutput};

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
    pub local_databases: bool,
    pub web: WebOutput,
    pub desktop_web: WebOutput,
    pub view_routes: ViewTargetRoutes,
    pub apps: AppOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectCapabilities {
    pub server: bool,
    pub views: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub port: u16,
    pub tls: Option<TlsConfig>,
    pub endpoints: Vec<Endpoint>,
    pub websockets: Vec<WebSocketRoute>,
    pub transports: Vec<ServerTransport>,
    pub rtp: Option<RtpConfig>,
    pub models: Vec<ServerModel>,
    pub init_action: ServerAction,
    pub cors: CorsConfig,
    pub database_service: bool,
    pub cache_service: bool,
    pub vector_service: bool,
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

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            tls: None,
            endpoints: Vec::new(),
            websockets: Vec::new(),
            transports: Vec::new(),
            rtp: None,
            models: Vec::new(),
            init_action: ServerAction::empty(),
            cors: CorsConfig::default(),
            database_service: false,
            cache_service: false,
            vector_service: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsMode {
    Acme,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsConfig {
    pub mode: TlsMode,
    pub domains: Vec<String>,
    pub email: Option<String>,
    pub staging: bool,
    pub cache: String,
    pub domains_from: Option<TlsDomainsSource>,
    pub refresh_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsDomainsSource {
    Kv {
        database: String,
        key: String,
    },
    Database {
        database: String,
        table: String,
        field: String,
    },
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
    HttpBytes(HttpBytesEndpoint),
    HttpActionJson(HttpActionJsonEndpoint),
    AgentResponse(AgentResponseEndpoint),
    StoreInsertJson(StoreInsertEndpoint),
    StoreQueryJson(StoreQueryEndpoint),
    StoreTransactionJson(StoreTransactionEndpoint),
    StoreActionJson(StoreActionJsonEndpoint),
    KvActionJson(KvActionJsonEndpoint),
    VectorActionJson(VectorActionJsonEndpoint),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerMiddleware {
    pub name: String,
    pub params: StoreLiteral,
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
    Jwt(ServerJwtStatement),
    SessionVerify {
        binding: String,
        cache: CacheConnection,
        database: StoreConnection,
        token: String,
        max_age_seconds: u64,
    },
    IfValid {
        binding: String,
        statements: Vec<ServerMiddlewareStatement>,
    },
    Call(ServerCallStatement),
    Next {
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
pub struct HttpBytesEndpoint {
    pub status: u16,
    pub binding: String,
    pub content_type: Option<String>,
    pub headers: Vec<ResponseHeader>,
    pub cookies: Vec<ResponseCookie>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseHeader {
    pub name: String,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseCookie {
    pub name: String,
    pub value: StoreLiteral,
    pub path: Option<String>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: Option<String>,
    pub max_age: Option<u64>,
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
    pub headers: Vec<OutboundHttpHeader>,
    pub json: Option<StoreLiteral>,
    pub mode: HttpResponseMode,
    pub redirect: HttpRedirectPolicy,
    pub max_redirects: Option<u32>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundHttpHeader {
    pub name: String,
    pub value: HttpHeaderValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpHeaderValue {
    Static(String),
    Environment(String),
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
    Bytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpRedirectPolicy {
    Follow,
    Manual,
    Error,
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
    pub binding: String,
    pub provider: DatabaseProvider,
    pub database: String,
    pub host: Option<StoreConnectionValue>,
    pub port: Option<StoreConnectionValue>,
    pub account: Option<StoreConnectionValue>,
    pub secret: Option<StoreConnectionValue>,
    pub entities: Vec<DatabaseEntity>,
    pub seeders: Vec<DatabaseSeeder>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseBinding {
    pub binding: String,
    pub connection: StoreConnection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseProvider {
    Postgres,
    D1,
    Dowe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreConnectionValue {
    Static(String),
    Environment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseEntity {
    pub binding: String,
    pub table: String,
    pub fields: Vec<DatabaseEntityField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseEntityField {
    pub name: String,
    pub field_type: DatabaseFieldType,
    pub primary: bool,
    pub required: bool,
    pub unique: bool,
    pub index: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseFieldType {
    String,
    Bool,
    Int,
    Number,
    Decimal,
    Timestamp,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseSeeder {
    pub binding: String,
    pub fingerprint: String,
    pub inserts: Vec<DatabaseSeedInsert>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseSeedInsert {
    pub entity: String,
    pub table: String,
    pub value: StoreLiteral,
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
pub struct VectorActionJsonEndpoint {
    pub status: u16,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheConnection {
    pub binding: String,
    pub provider: CacheProvider,
    pub host: CacheConnectionValue,
    pub port: CacheConnectionValue,
    pub account: CacheConnectionValue,
    pub secret: CacheConnectionValue,
    pub name: CacheConnectionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheProvider {
    CloudflareKv,
    Redis,
    Dowe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheConnectionValue {
    Static(String),
    Environment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorConnection {
    pub binding: String,
    pub provider: VectorProvider,
    pub host: VectorConnectionValue,
    pub port: VectorConnectionValue,
    pub account: VectorConnectionValue,
    pub secret: VectorConnectionValue,
    pub name: VectorConnectionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorProvider {
    Dowe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VectorConnectionValue {
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
    pub middlewares: Vec<ServerMiddleware>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WebSocketHandlers {
    pub open: ServerAction,
    pub message: ServerAction,
    pub close: ServerAction,
    pub drain: ServerAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerTransport {
    pub name: String,
    pub protocol: ServerTransportProtocol,
    pub bind: String,
    pub port: u16,
    pub action: ServerAction,
    pub binding: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerTransportProtocol {
    Tcp,
    Udp,
}

impl ServerTransportProtocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tcp => "tcp",
            Self::Udp => "udp",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpConfig {
    pub bind: String,
    pub min: u16,
    pub max: u16,
}

impl RtpConfig {
    pub fn contains(&self, port: u16) -> bool {
        port >= self.min && port <= self.max
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerModel {
    pub name: String,
    pub kind: ServerModelKind,
    pub engine: ServerModelEngine,
    pub format: ServerModelFormat,
    pub source: Option<PathBuf>,
    pub sample_rates: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerModelKind {
    VadSilero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerModelEngine {
    Candle,
    Energy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerModelFormat {
    Onnx,
    Builtin,
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
    RequestQuery {
        binding: String,
    },
    RequestRawQuery {
        binding: String,
    },
    RequestHeader {
        binding: String,
        name: String,
    },
    RequestCookie {
        binding: String,
        name: String,
    },
    Stdlib(ServerStdlibStatement),
    Http(OutboundHttpRequest),
    Spawn(ServerSpawnStatement),
    CryptoAesCtr(ServerCryptoAesCtrStatement),
    CryptoCencAesCtr(ServerCryptoCencAesCtrStatement),
    Jwt(ServerJwtStatement),
    AgentChat(AgentChatTransform),
    WebSocketJson(WebSocketJsonStatement),
    WebSocketSendJson(WebSocketSendJsonStatement),
    WebSocketSseBridge(WebSocketSseBridgeStatement),
    Store(ServerStoreStatement),
    Kv(ServerKvStatement),
    Vector(ServerVectorStatement),
    Call(ServerCallStatement),
    Go(ServerBackgroundJob),
    Cron(ServerBackgroundJob),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerBackgroundJob {
    pub id: String,
    pub target: String,
    pub args: StoreLiteral,
    pub action: Box<ServerFunctionAction>,
    pub schedule: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerJwtStatement {
    Verify {
        binding: String,
        token: String,
        secret: ServerSecret,
        algorithm: String,
    },
    Sign {
        binding: String,
        claims: StoreLiteral,
        secret: ServerSecret,
        algorithm: String,
    },
    Decrypt {
        binding: String,
        token: String,
        key: ServerSecret,
        algorithm: String,
        encryption: String,
    },
    Encrypt {
        binding: String,
        claims: StoreLiteral,
        key: ServerSecret,
        algorithm: String,
        encryption: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerStdlibStatement {
    pub binding: String,
    pub call: StdlibCall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerSpawnStatement {
    pub binding: String,
    pub command: StoreLiteral,
    pub args: StoreLiteral,
    pub cwd: Option<StoreLiteral>,
    pub timeout_ms: Option<u64>,
    pub max_output_bytes: Option<usize>,
    pub background: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCryptoAesCtrStatement {
    pub binding: String,
    pub data: String,
    pub key: StoreLiteral,
    pub iv: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCryptoCencAesCtrStatement {
    pub binding: String,
    pub data: String,
    pub key: StoreLiteral,
    pub iv: StoreLiteral,
    pub subsamples: Option<StoreLiteral>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCallStatement {
    pub binding: String,
    pub target: String,
    pub args: StoreLiteral,
    pub action: Box<ServerFunctionAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerFunctionAction {
    pub params: Vec<ServerFunctionParameter>,
    pub return_type: Option<ServerFunctionReturn>,
    pub statements: Vec<ServerStatement>,
    pub return_value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerFunctionParameter {
    pub name: String,
    pub type_name: String,
    pub schema: DoweType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerFunctionReturn {
    pub type_name: String,
    pub schema: DoweType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerStoreStatement {
    Handle {
        connection: StoreConnection,
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
        params: Vec<StoreLiteral>,
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
        connection: CacheConnection,
    },
    Get {
        binding: String,
        handle: String,
        key: StoreLiteral,
        required: bool,
    },
    Set {
        binding: String,
        handle: String,
        key: StoreLiteral,
        value: StoreLiteral,
    },
    Delete {
        binding: String,
        handle: String,
        key: StoreLiteral,
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
pub enum ServerVectorStatement {
    Handle {
        connection: VectorConnection,
    },
    Upsert {
        binding: String,
        handle: String,
        id: StoreLiteral,
        vector: StoreLiteral,
        metadata: Option<StoreLiteral>,
    },
    Search {
        binding: String,
        handle: String,
        vector: StoreLiteral,
        limit: usize,
        min_score: String,
        filter: Option<StoreLiteral>,
    },
    Read {
        binding: String,
        handle: String,
        id: StoreLiteral,
        required: bool,
    },
    Delete {
        binding: String,
        handle: String,
        id: StoreLiteral,
    },
    List {
        binding: String,
        handle: String,
        limit: usize,
        filter: Option<StoreLiteral>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreFilter {
    pub field: String,
    pub value: StoreLiteral,
    pub additional: Vec<StoreMatchField>,
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

    let mut params = HashMap::new();
    let splat = pattern_segments
        .last()
        .and_then(|segment| segment.strip_prefix('*'));
    let fixed_pattern_len = if splat.is_some() {
        pattern_segments.len().saturating_sub(1)
    } else {
        pattern_segments.len()
    };

    if let Some(splat_name) = splat {
        if splat_name.is_empty() || path_segments.len() <= fixed_pattern_len {
            return None;
        }
    } else if pattern_segments.len() != path_segments.len() {
        return None;
    }

    for (pattern_segment, path_segment) in pattern_segments
        .iter()
        .take(fixed_pattern_len)
        .zip(path_segments.iter())
    {
        if let Some(param_name) = pattern_segment.strip_prefix(':') {
            params.insert(param_name.to_string(), (*path_segment).to_string());
        } else if pattern_segment != path_segment {
            return None;
        }
    }

    if let Some(splat_name) = splat {
        params.insert(
            splat_name.to_string(),
            path_segments[fixed_pattern_len..].join("/"),
        );
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
            tls: None,
            endpoints: vec![Endpoint {
                method: HttpMethod::Get,
                path: "/users/:id".to_string(),
                behavior: EndpointBehavior::UserGreeting,
                action: ServerAction::empty(),
                middlewares: Vec::new(),
            }],
            websockets: Vec::new(),
            transports: Vec::new(),
            rtp: None,
            models: Vec::new(),
            init_action: ServerAction::empty(),
            cors: super::CorsConfig::default(),
            database_service: false,
            cache_service: false,
            vector_service: false,
        };

        let matched = server
            .find_endpoint(&HttpMethod::Get, "/users/123")
            .expect("endpoint");

        assert_eq!(matched.params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn matches_final_splat_routes() {
        let server = ServerConfig {
            port: 8080,
            tls: None,
            endpoints: vec![Endpoint {
                method: HttpMethod::Get,
                path: "/dash/:name/*segment".to_string(),
                behavior: EndpointBehavior::UserGreeting,
                action: ServerAction::empty(),
                middlewares: Vec::new(),
            }],
            websockets: Vec::new(),
            transports: Vec::new(),
            rtp: None,
            models: Vec::new(),
            init_action: ServerAction::empty(),
            cors: super::CorsConfig::default(),
            database_service: false,
            cache_service: false,
            vector_service: false,
        };

        let matched = server
            .find_endpoint(&HttpMethod::Get, "/dash/news/video/0001.m4s")
            .expect("endpoint");

        assert_eq!(matched.params.get("name"), Some(&"news".to_string()));
        assert_eq!(
            matched.params.get("segment"),
            Some(&"video/0001.m4s".to_string())
        );
        assert!(
            server
                .find_endpoint(&HttpMethod::Get, "/dash/news")
                .is_none()
        );
    }
}
