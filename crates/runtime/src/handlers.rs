use crate::database_runtime::{ConfiguredDatabaseClient, configured_database_client};
use crate::server::DevRuntimeState;
use crate::server_actions::{
    execute_resolved_log, execute_server_action, execute_server_action_with_resolver,
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::body::{Body, Bytes};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::header::{
    ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, CACHE_CONTROL, CONTENT_TYPE,
    LOCATION, ORIGIN, VARY,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use dowe_cache::{
    CacheProviderKind, KvDatabase, RemoteCacheClient, RemoteCacheConfig,
    open_database as open_kv_database,
};
use dowe_compiler::{
    AgentResponseEndpoint, CacheConnection, CacheConnectionValue, CacheProvider, CompiledProject,
    CorsConfig, DoweType, EndpointBehavior, HttpActionJsonEndpoint, HttpBytesEndpoint,
    HttpConnectionValue, HttpHeaderValue, HttpMethod, HttpProxyEndpoint, HttpRedirectPolicy,
    HttpResponseMode, HttpReverseProxyEndpoint, KvActionJsonEndpoint, OutboundHttpRequest,
    QueueActionJsonEndpoint, QueueConnection, QueueConnectionValue,
    QueueProvider as CompilerQueueProvider, ResponseCookie, ReverseProxyStrategy, ServerConfig,
    ServerCryptoAesCtrStatement, ServerCryptoCencAesCtrStatement, ServerFileStatement,
    ServerFunctionAction, ServerJwtStatement, ServerKvStatement, ServerMiddleware,
    ServerMiddlewareResponseBody, ServerMiddlewareStatement, ServerPasswordStatement,
    ServerQueueStatement, ServerSecret, ServerSpawnStatement, ServerStatement,
    ServerStoreStatement, ServerVectorStatement, StoreActionJsonEndpoint, StoreConnection,
    StoreFilter, StoreLiteral, StoreTransactionEndpoint, StoreTransactionOperation,
    VectorActionJsonEndpoint, VectorConnection, VectorConnectionValue, ViewPage, WebOutput,
    WebSocketHandlers, WebSocketSendJsonStatement, WebSocketSseBridgeStatement,
    normalize_cors_method, normalize_http_header_name,
};
use dowe_crypto::{
    JwtValidationOptions, aes_128_ctr, cenc_aes_128_ctr, decrypt_jwe_dir_a256gcm,
    encrypt_jwe_dir_a256gcm, sign_jws_hs256, verify_jws_hs256,
};
use dowe_database::{
    D1Client, Database, DatabaseTransactionInsert, DoweDatabaseClient, PostgresClient, StoreRecord,
    StoreValue, bind_query_params, init_database, open_database,
};
use dowe_queue::{
    DoweQueue, QueueClient, QueueConfig, QueueError, QueueProvider as RuntimeQueueProvider,
    open_namespace as open_queue_namespace,
};
use dowe_vector::{
    DoweVectorClient, DoweVectorConfig, VectorDatabase, open_database as open_vector_database,
};
use futures_util::StreamExt;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
include!("handlers/store_context_session.rs");
include!("handlers/store_context_vector.rs");
include!("handlers/store_context_queue.rs");
include!("handlers/store_context_file.rs");
include!("handlers/store_context_resolve.rs");

pub(crate) async fn execute_background_action(
    project: &CompiledProject,
    action: &ServerFunctionAction,
    args: Value,
    cache_mode: CacheRuntimeMode,
) -> crate::RuntimeResult<()> {
    let params = HashMap::new();
    let body = Bytes::new();
    execute_reusable_action(
        project,
        &project.root,
        &params,
        &body,
        None,
        None,
        action,
        args,
        cache_mode,
    )
    .await
    .map(|_| ())
    .map_err(|error| crate::RuntimeError::new(format!("{}: {}", error.code, error.message)))
}
