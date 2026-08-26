use super::*;
use std::collections::HashMap;

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
    HttpReverseProxy(HttpReverseProxyEndpoint),
    HttpBytes(HttpBytesEndpoint),
    HttpActionJson(HttpActionJsonEndpoint),
    AgentResponse(AgentResponseEndpoint),
    StoreInsertJson(StoreInsertEndpoint),
    StoreQueryJson(StoreQueryEndpoint),
    StoreTransactionJson(StoreTransactionEndpoint),
    StoreActionJson(StoreActionJsonEndpoint),
    KvActionJson(KvActionJsonEndpoint),
    VectorActionJson(VectorActionJsonEndpoint),
    QueueActionJson(QueueActionJsonEndpoint),
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
pub struct HttpReverseProxyEndpoint {
    pub upstream: String,
    pub strategy: ReverseProxyStrategy,
    pub state: Option<String>,
    pub loading_url: Option<String>,
    pub error_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReverseProxyStrategy {
    Single,
    RoundRobin,
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
pub struct AiChatStatement {
    pub binding: String,
    pub prompt: StoreLiteral,
    pub files: StoreLiteral,
    pub model: Option<String>,
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
