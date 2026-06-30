use crate::server::DevRuntimeState;
use crate::server_actions::{
    execute_resolved_log, execute_server_action, execute_server_action_with_resolver,
};
use axum::body::{Body, Bytes};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::header::{
    ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, CACHE_CONTROL, CONTENT_TYPE,
    ORIGIN, VARY,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use dowe_compiler::{
    AgentResponseEndpoint, CompiledProject, CorsConfig, DoweType, EndpointBehavior,
    HttpActionJsonEndpoint, HttpConnectionValue, HttpHeaderValue, HttpMethod, HttpProxyEndpoint,
    HttpRedirectPolicy, HttpResponseMode, KvActionJsonEndpoint, KvConnectionValue, KvCredential,
    KvRemoteConnection, OutboundHttpRequest, ServerConfig, ServerKvStatement, ServerMiddleware,
    ServerMiddlewareResponseBody, ServerMiddlewareStatement, ServerSecret, ServerStatement,
    ServerStoreStatement, StoreActionJsonEndpoint, StoreConnection, StoreConnectionValue,
    StoreCredential, StoreFilter, StoreLiteral, StoreRemoteConnection, StoreTransactionEndpoint,
    StoreTransactionOperation, ViewPage, WebOutput, WebSocketHandlers, WebSocketSendJsonStatement,
    WebSocketSseBridgeStatement, normalize_cors_method, normalize_http_header_name,
};
use dowe_crypto::{
    JwtValidationOptions, decrypt_jwe_dir_a256gcm, encrypt_jwe_dir_a256gcm, sign_jws_hs256,
    verify_jws_hs256,
};
use dowe_kv::{KvDatabase, RemoteKvClient, RemoteKvConfig, open_database as open_kv_database};
use dowe_store::{
    Database, RemoteStoreClient, RemoteStoreConfig, StoreRecord, StoreValue, init_database,
    open_database,
};
use futures_util::StreamExt;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

include!("handlers/entrypoints.rs");
include!("handlers/websocket.rs");
include!("handlers/middleware.rs");
include!("handlers/cors.rs");
include!("handlers/store_endpoints.rs");
include!("handlers/http_actions.rs");
include!("handlers/store_types.rs");
include!("handlers/store_helpers.rs");
include!("handlers/web_assets.rs");
include!("handlers/store_context_execute.rs");
include!("handlers/store_context_store.rs");
include!("handlers/store_context_kv.rs");
include!("handlers/store_context_resolve.rs");
