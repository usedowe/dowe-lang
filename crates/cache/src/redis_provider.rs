use crate::error::{KvError, KvResult};
use crate::names::{validate_database_name, validate_key};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use reqwest::Url;
use serde_json::{Map, Value};
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedisCacheConfig {
    pub host: String,
    pub port: u16,
    pub account: String,
    pub secret: String,
    pub namespace: String,
}

#[derive(Clone)]
pub struct RedisCacheClient {
    connection: ConnectionManager,
    prefix: String,
}

impl RedisCacheClient {
    pub async fn connect(config: RedisCacheConfig) -> KvResult<Self> {
        validate_database_name(&config.namespace)?;
        let url = redis_url(&config)?;
        let client = redis::Client::open(url.as_str())
            .map_err(|_| KvError::InvalidRequest("Redis connection is invalid".to_string()))?;
        let connection = client.get_connection_manager().await.map_err(redis_error)?;
        Ok(Self {
            connection,
            prefix: format!("dowe:{}:", config.namespace),
        })
    }

    pub async fn get(&self, key: &str, required: bool) -> KvResult<Value> {
        validate_key(key)?;
        let mut connection = self.connection.clone();
        let value = connection
            .get::<_, Option<Vec<u8>>>(self.physical_key(key))
            .await
            .map_err(redis_error)?;
        let Some(value) = value else {
            if required {
                return Err(KvError::NotFound("Cache key was not found".to_string()));
            }
            return Ok(Value::Null);
        };
        serde_json::from_slice(&value)
            .map_err(|_| KvError::Remote("Redis Cache value is not valid JSON".to_string()))
    }

    pub async fn set(&self, key: &str, value: Value) -> KvResult<Value> {
        validate_key(key)?;
        let mut connection = self.connection.clone();
        connection
            .set::<_, _, ()>(self.physical_key(key), serde_json::to_vec(&value)?)
            .await
            .map_err(redis_error)?;
        Ok(set_json(key))
    }

    pub async fn delete(&self, key: &str) -> KvResult<Value> {
        validate_key(key)?;
        let mut connection = self.connection.clone();
        let deleted = connection
            .del::<_, u64>(self.physical_key(key))
            .await
            .map_err(redis_error)?;
        Ok(delete_json(deleted > 0))
    }

    pub async fn keys(&self, prefix: Option<&str>) -> KvResult<Value> {
        let physical_prefix = format!(
            "{}{}",
            escape_pattern(&self.prefix),
            prefix.map(escape_pattern).unwrap_or_default()
        );
        let physical = self.scan(format!("{physical_prefix}*")).await?;
        let mut keys = physical
            .into_iter()
            .filter_map(|key| key.strip_prefix(&self.prefix).map(str::to_string))
            .collect::<Vec<_>>();
        keys.sort();
        keys.dedup();
        Ok(Value::Array(keys.into_iter().map(Value::String).collect()))
    }

    pub async fn clear(&self) -> KvResult<Value> {
        let keys = self
            .scan(format!("{}*", escape_pattern(&self.prefix)))
            .await?;
        let mut connection = self.connection.clone();
        for chunk in keys.chunks(512) {
            redis::cmd("DEL")
                .arg(chunk)
                .query_async::<()>(&mut connection)
                .await
                .map_err(redis_error)?;
        }
        Ok(clear_json(keys.len()))
    }

    async fn scan(&self, pattern: String) -> KvResult<Vec<String>> {
        let mut connection = self.connection.clone();
        let mut cursor = 0u64;
        let mut keys = Vec::new();
        loop {
            let (next, batch): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(500)
                .query_async(&mut connection)
                .await
                .map_err(redis_error)?;
            keys.extend(batch);
            cursor = next;
            if cursor == 0 {
                break;
            }
        }
        Ok(keys)
    }

    fn physical_key(&self, key: &str) -> String {
        format!("{}{key}", self.prefix)
    }
}

fn redis_url(config: &RedisCacheConfig) -> KvResult<Url> {
    let (scheme, host) = if let Some(host) = config.host.strip_prefix("rediss://") {
        ("rediss", host)
    } else if let Some(host) = config.host.strip_prefix("redis://") {
        ("redis", host)
    } else if is_loopback(&config.host) {
        ("redis", config.host.as_str())
    } else {
        ("rediss", config.host.as_str())
    };
    if scheme == "redis" && !is_loopback(host) {
        return Err(KvError::InvalidRequest(
            "Redis requires TLS outside loopback".to_string(),
        ));
    }
    let mut url = Url::parse(&format!("{scheme}://localhost"))
        .map_err(|_| KvError::InvalidRequest("Redis URL is invalid".to_string()))?;
    url.set_host(Some(host))
        .map_err(|_| KvError::InvalidRequest("Redis host is invalid".to_string()))?;
    url.set_port(Some(config.port))
        .map_err(|_| KvError::InvalidRequest("Redis port is invalid".to_string()))?;
    url.set_username(&config.account)
        .map_err(|_| KvError::InvalidRequest("Redis account is invalid".to_string()))?;
    url.set_password(Some(&config.secret))
        .map_err(|_| KvError::InvalidRequest("Redis secret is invalid".to_string()))?;
    url.set_path("/0");
    Ok(url)
}

fn redis_error(error: redis::RedisError) -> KvError {
    if error.kind() == redis::ErrorKind::AuthenticationFailed {
        KvError::Authentication("Redis authentication failed".to_string())
    } else {
        KvError::Remote("Redis Cache operation failed".to_string())
    }
}

fn is_loopback(host: &str) -> bool {
    let host = host
        .strip_prefix("redis://")
        .or_else(|| host.strip_prefix("rediss://"))
        .unwrap_or(host);
    host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

fn escape_pattern(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        if matches!(character, '*' | '?' | '[' | ']' | '\\') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn set_json(key: &str) -> Value {
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(true));
    output.insert("key".to_string(), Value::String(key.to_string()));
    Value::Object(output)
}

fn delete_json(deleted: bool) -> Value {
    let mut output = Map::new();
    output.insert("deleted".to_string(), Value::Bool(deleted));
    Value::Object(output)
}

fn clear_json(cleared: usize) -> Value {
    let mut output = Map::new();
    output.insert("cleared".to_string(), Value::from(cleared as u64));
    Value::Object(output)
}

#[cfg(test)]
mod tests {
    use super::{RedisCacheConfig, escape_pattern, redis_url};

    #[test]
    fn redis_remote_defaults_to_tls() {
        let url = redis_url(&RedisCacheConfig {
            host: "cache.example.com".to_string(),
            port: 6380,
            account: "app".to_string(),
            secret: "secret".to_string(),
            namespace: "clinic".to_string(),
        })
        .expect("url");
        assert_eq!(url.scheme(), "rediss");
    }

    #[test]
    fn redis_patterns_escape_user_globs() {
        assert_eq!(escape_pattern("app:*?[x]"), "app:\\*\\?\\[x\\]");
    }
}
