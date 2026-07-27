mod auth;
mod client;
mod cloudflare;
mod codec;
mod dowe;
mod engine;
mod error;
mod names;
mod protocol;
mod redis_provider;
mod service;

pub use auth::{
    CreatedCacheAccount, CreatedKvUser, create_account, create_user, verify_account, verify_user,
};
pub use client::{
    CacheProviderKind, RemoteCacheClient, RemoteCacheConfig, close_remote_connections,
};
pub use engine::{
    KvDatabase, KvInspection, KvSetReport, clear_memory, init_database, kv_root, list_databases,
    open_database,
};
pub use error::{KvError, KvResult};
pub use protocol::{CacheRequest, CacheResponse};
pub use service::{
    CacheServerConfig, RunningCacheServer, cache_service_upgrade, serve_cache_server,
    start_cache_server,
};

#[cfg(test)]
mod tests;
