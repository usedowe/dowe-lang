mod error;
mod language;
mod model;
mod parser;
mod pipeline;
mod typecheck_artifacts;

pub use error::{DoweError, DoweResult};
pub use language::{
    LanguageCompletion, LanguageCompletionKind, LanguageDiagnostic, LanguageDiagnosticSeverity,
    LanguageDocument, LanguageDocumentSymbol, LanguageLocation, LanguagePosition, LanguageRange,
    LanguageSymbolKind, analyze_document, complete_document, definition_at, document_symbols,
    find_workspace_root, format_document, hover_at,
};
pub use model::{
    AgentChatTransform, AgentResponseEndpoint, AppOutput, ChunkKind, CompiledProject, CorsConfig,
    DesignConfig, DoweType, DoweTypeField, Endpoint, EndpointBehavior, EnvironmentConfig,
    EnvironmentValueSource, EnvironmentVariable, EnvironmentVisibility, GeneratedChunk,
    GeneratedFile, HttpActionJsonEndpoint, HttpConnectionValue, HttpMethod, HttpProxyEndpoint,
    HttpResponseMode, KvActionJsonEndpoint, KvConnection, KvConnectionValue, KvCredential,
    KvRemoteConnection, MatchedEndpoint, OutboundHttpRequest, ProjectServerConfig, ServerAction,
    ServerConfig, ServerKvStatement, ServerLog, ServerLogLevel, ServerLogValue, ServerMiddleware,
    ServerMiddlewareAction, ServerMiddlewareResponseBody, ServerMiddlewareStatement, ServerSecret,
    ServerStatement, ServerStoreStatement, StoreActionJsonEndpoint, StoreConnection,
    StoreConnectionValue, StoreCredential, StoreFilter, StoreInsertEndpoint, StoreLiteral,
    StoreMatchField, StoreQueryEndpoint, StoreRemoteConnection, StoreTransactionEndpoint,
    StoreTransactionOperation, ViewNode, ViewPage, ViewPlatform, ViewRoute, ViewTargetRoutes,
    WebOutput, WebSocketHandlers, WebSocketJsonStatement, WebSocketRoute,
    WebSocketSendJsonStatement, WebSocketSseBridgeStatement, normalize_cors_method,
    normalize_cors_origin, normalize_http_header_name,
};
pub use parser::validate_design_copilot_dowe;
pub use pipeline::compile_dev;
