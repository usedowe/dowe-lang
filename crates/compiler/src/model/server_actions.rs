use super::*;
use dowe_database_query::SelectQuery;
use dowe_stdlib::StdlibCall;
use std::path::PathBuf;

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
    RequestBytes {
        binding: String,
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
    Queue(ServerQueueStatement),
    File(ServerFileStatement),
    Password(ServerPasswordStatement),
    Call(ServerCallStatement),
    Task(ServerBackgroundJob),
    Cron(ServerBackgroundJob),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerPasswordStatement {
    Hash {
        binding: String,
        value: StoreLiteral,
    },
    Verify {
        binding: String,
        value: StoreLiteral,
        hash: StoreLiteral,
        required: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerFileStatement {
    Write {
        binding: String,
        root: StoreLiteral,
        path: StoreLiteral,
        data: String,
        sha256: Option<StoreLiteral>,
    },
    Read {
        binding: String,
        root: StoreLiteral,
        path: StoreLiteral,
    },
    Exists {
        binding: String,
        root: StoreLiteral,
        path: StoreLiteral,
    },
    Delete {
        binding: String,
        root: StoreLiteral,
        path: StoreLiteral,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerTaskTiming {
    Immediate,
    ResponseHeaders,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerBackgroundJob {
    pub id: String,
    pub target: Option<String>,
    pub args: StoreLiteral,
    pub action: Box<ServerFunctionAction>,
    pub schedule: Option<String>,
    pub timing: ServerTaskTiming,
    pub source_path: PathBuf,
    pub source_line: usize,
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
        query: SelectQuery,
        params: Vec<StoreLiteral>,
    },
    Transaction {
        binding: String,
        handle: String,
        operations: Vec<StoreTransactionOperation>,
        return_binding: Option<String>,
        rollback: bool,
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
pub enum ServerQueueStatement {
    Handle {
        connection: QueueConnection,
    },
    Publish {
        binding: String,
        handle: String,
        queue: StoreLiteral,
        payload: StoreLiteral,
    },
}
