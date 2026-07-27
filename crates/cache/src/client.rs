use crate::cloudflare::{CloudflareKvClient, CloudflareKvConfig};
use crate::dowe::{DoweCacheClient, DoweCacheConfig};
use crate::error::{KvError, KvResult};
use crate::redis_provider::{RedisCacheClient, RedisCacheConfig};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheProviderKind {
    CloudflareKv,
    Redis,
    Dowe,
}

pub fn close_remote_connections() {
    crate::dowe::clear_connections();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RemoteCacheConfig {
    pub provider: CacheProviderKind,
    pub host: String,
    pub port: u16,
    pub account: String,
    pub secret: String,
    pub name: String,
}

#[derive(Clone)]
pub enum RemoteCacheClient {
    CloudflareKv(CloudflareKvClient),
    Redis(RedisCacheClient),
    Dowe(DoweCacheClient),
}

impl RemoteCacheClient {
    pub async fn new(config: RemoteCacheConfig) -> KvResult<Self> {
        validate_config(&config)?;
        match config.provider {
            CacheProviderKind::CloudflareKv => CloudflareKvClient::new(CloudflareKvConfig {
                host: config.host,
                port: config.port,
                account: config.account,
                secret: config.secret,
                namespace: config.name,
            })
            .map(Self::CloudflareKv),
            CacheProviderKind::Redis => RedisCacheClient::connect(RedisCacheConfig {
                host: config.host,
                port: config.port,
                account: config.account,
                secret: config.secret,
                namespace: config.name,
            })
            .await
            .map(Self::Redis),
            CacheProviderKind::Dowe => DoweCacheClient::new(DoweCacheConfig {
                host: config.host,
                port: config.port,
                account: config.account,
                secret: config.secret,
                name: config.name,
            })
            .map(Self::Dowe),
        }
    }

    pub async fn get(&self, key: &str, required: bool) -> KvResult<Value> {
        match self {
            Self::CloudflareKv(client) => client.get(key, required).await,
            Self::Redis(client) => client.get(key, required).await,
            Self::Dowe(client) => client.get(key, required).await,
        }
    }

    pub async fn set(&self, key: &str, value: Value) -> KvResult<Value> {
        match self {
            Self::CloudflareKv(client) => client.set(key, value).await,
            Self::Redis(client) => client.set(key, value).await,
            Self::Dowe(client) => client.set(key, value).await,
        }
    }

    pub async fn delete(&self, key: &str) -> KvResult<Value> {
        match self {
            Self::CloudflareKv(client) => client.delete(key).await,
            Self::Redis(client) => client.delete(key).await,
            Self::Dowe(client) => client.delete(key).await,
        }
    }

    pub async fn keys(&self, prefix: Option<&str>) -> KvResult<Value> {
        match self {
            Self::CloudflareKv(client) => client.keys(prefix).await,
            Self::Redis(client) => client.keys(prefix).await,
            Self::Dowe(client) => client.keys(prefix).await,
        }
    }

    pub async fn clear(&self) -> KvResult<Value> {
        match self {
            Self::CloudflareKv(client) => client.clear().await,
            Self::Redis(client) => client.clear().await,
            Self::Dowe(client) => client.clear().await,
        }
    }
}

fn validate_config(config: &RemoteCacheConfig) -> KvResult<()> {
    for (name, value) in [
        ("host", config.host.as_str()),
        ("account", config.account.as_str()),
        ("secret", config.secret.as_str()),
        ("name", config.name.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(KvError::InvalidRequest(format!(
                "Cache connection `{name}` is empty"
            )));
        }
    }
    if config.port == 0 {
        return Err(KvError::InvalidRequest(
            "Cache connection port must be greater than zero".to_string(),
        ));
    }
    Ok(())
}
