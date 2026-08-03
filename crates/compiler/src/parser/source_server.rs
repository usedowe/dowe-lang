use crate::CronSchedule;
use crate::error::{DoweError, DoweResult};
use crate::model::{
    AgentChatTransform, AgentResponseEndpoint, CacheConnection, CorsConfig, DatabaseBinding,
    DatabaseEntity, DatabaseSeeder, DoweType, DoweTypeField, Endpoint, EndpointBehavior,
    EnvironmentConfig, EnvironmentVisibility, HttpActionJsonEndpoint, HttpBytesEndpoint,
    HttpConnectionValue, HttpHeaderValue, HttpMethod, HttpProxyEndpoint, HttpRedirectPolicy,
    HttpResponseMode, HttpReverseProxyEndpoint, OutboundHttpHeader, OutboundHttpRequest,
    ResponseCookie, ResponseHeader, ReverseProxyStrategy, RtpConfig, ServerAction,
    ServerBackgroundJob, ServerCallStatement, ServerConfig, ServerCryptoAesCtrStatement,
    ServerCryptoCencAesCtrStatement, ServerFileStatement, ServerFunctionAction,
    ServerFunctionParameter, ServerFunctionReturn, ServerJwtStatement, ServerKvStatement,
    ServerLog, ServerLogLevel, ServerLogValue, ServerMiddleware, ServerMiddlewareAction,
    ServerMiddlewareResponseBody, ServerMiddlewareStatement, ServerModel, ServerModelEngine,
    ServerModelFormat, ServerModelKind, ServerPasswordStatement, ServerQueueStatement,
    ServerSecret, ServerSpawnStatement, ServerStatement, ServerStdlibStatement, ServerTransport,
    ServerTransportProtocol, ServerVectorStatement, StoreConnection, StoreLiteral, TlsConfig,
    TlsDomainsSource, TlsMode, WebSocketHandlers, WebSocketJsonStatement, WebSocketRoute,
    WebSocketSendJsonStatement, WebSocketSseBridgeStatement, normalize_http_header_name,
};
use crate::parser::source_ast::{
    SourceFile, SourceNode, SourceObjectEntry, SourceProp, SourceValue,
};
use crate::parser::source_config::{parse_desktop_cors_config, parse_server_cors_config};
use crate::parser::source_db::{
    database_action_endpoint_behavior, database_endpoint_behavior, parse_database_entity,
    parse_database_seeder, parse_database_statement, store_literal,
};
use crate::parser::source_imports::resolve_import;
use crate::parser::source_kv::{
    infer_kv_statement, kv_action_endpoint_behavior, parse_kv_statement, validate_kv_handles,
    validate_kv_statement_references,
};
use crate::parser::source_parser::parse_source_file;
use crate::parser::source_queue::{
    infer_queue_statement, parse_queue_statement, queue_action_endpoint_behavior,
    validate_queue_handles, validate_queue_statement_references,
};
use crate::parser::source_stdlib::{dowe_type_from_stdlib_return, parse_stdlib_call};
use crate::parser::source_types::{
    TypeRegistry, is_shared_type_path, type_from_store_literal, validate_reference_path,
};
use crate::parser::source_vector::{
    infer_vector_statement, parse_vector_statement, validate_vector_handles,
    validate_vector_statement_references, vector_action_endpoint_behavior,
};
use dowe_stdlib::StdlibSurface;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;

include!("source_server/entry.rs");
include!("source_server/module_imports.rs");
include!("source_server/config_bindings.rs");
include!("source_server/declarations.rs");
include!("source_server/server_config.rs");
include!("source_server/endpoint_groups.rs");
include!("source_server/routes.rs");
include!("source_server/transports.rs");
include!("source_server/actions.rs");
include!("source_server/function_actions.rs");
include!("source_server/middleware.rs");
include!("source_server/inference.rs");
include!("source_server/request_actions.rs");
include!("source_server/capability_actions.rs");
include!("source_server/background_jobs.rs");
include!("source_server/http_and_crypto.rs");
include!("source_server/file_actions.rs");
include!("source_server/password_actions.rs");
include!("source_server/validation.rs");
include!("source_server/action_helpers.rs");
include!("source_server/behavior.rs");
include!("source_server/responses.rs");
include!("source_server/props.rs");
include!("source_server/diagnostics.rs");

#[cfg(test)]
mod tests;
