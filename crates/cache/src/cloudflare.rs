use crate::error::{KvError, KvResult};
use crate::names::validate_key;
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudflareKvConfig {
    pub host: String,
    pub port: u16,
    pub account: String,
    pub secret: String,
    pub namespace: String,
}

#[derive(Clone)]
pub struct CloudflareKvClient {
    config: CloudflareKvConfig,
    base: Url,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct Envelope<T> {
    success: bool,
    #[serde(default)]
    result: Option<T>,
    #[serde(default)]
    errors: Vec<EnvelopeError>,
    #[serde(default)]
    result_info: Option<ResultInfo>,
}

#[derive(Deserialize)]
struct EnvelopeError {
    #[serde(default)]
    message: String,
}

#[derive(Deserialize)]
struct ResultInfo {
    #[serde(default)]
    cursor: String,
}

#[derive(Deserialize)]
struct KeyRecord {
    name: String,
}

impl CloudflareKvClient {
    pub fn new(config: CloudflareKvConfig) -> KvResult<Self> {
        let base = cloudflare_base(&config)?;
        Ok(Self {
            config,
            base,
            client: reqwest::Client::new(),
        })
    }

    pub async fn get(&self, key: &str, required: bool) -> KvResult<Value> {
        validate_key(key)?;
        let response = self
            .client
            .get(self.value_url(key)?)
            .bearer_auth(&self.config.secret)
            .send()
            .await
            .map_err(remote_transport)?;
        if response.status() == StatusCode::NOT_FOUND {
            if required {
                return Err(KvError::NotFound("Cache key was not found".to_string()));
            }
            return Ok(Value::Null);
        }
        if !response.status().is_success() {
            return Err(remote_status(response.status()));
        }
        let bytes = response.bytes().await.map_err(remote_transport)?;
        serde_json::from_slice(&bytes)
            .map_err(|_| KvError::Remote("Cloudflare KV returned invalid JSON".to_string()))
    }

    pub async fn set(&self, key: &str, value: Value) -> KvResult<Value> {
        validate_key(key)?;
        let response = self
            .client
            .put(self.value_url(key)?)
            .bearer_auth(&self.config.secret)
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&value)?)
            .send()
            .await
            .map_err(remote_transport)?;
        validate_envelope(response).await?;
        Ok(set_json(key))
    }

    pub async fn delete(&self, key: &str) -> KvResult<Value> {
        validate_key(key)?;
        let response = self
            .client
            .delete(self.value_url(key)?)
            .bearer_auth(&self.config.secret)
            .send()
            .await
            .map_err(remote_transport)?;
        validate_envelope(response).await?;
        Ok(delete_json(true))
    }

    pub async fn keys(&self, prefix: Option<&str>) -> KvResult<Value> {
        let mut cursor = String::new();
        let mut keys = Vec::new();
        loop {
            let mut url = self.namespace_url(&["keys"])?;
            {
                let mut query = url.query_pairs_mut();
                query.append_pair("limit", "1000");
                if let Some(prefix) = prefix {
                    query.append_pair("prefix", prefix);
                }
                if !cursor.is_empty() {
                    query.append_pair("cursor", &cursor);
                }
            }
            let response = self
                .client
                .get(url)
                .bearer_auth(&self.config.secret)
                .send()
                .await
                .map_err(remote_transport)?;
            let envelope = parse_envelope::<Vec<KeyRecord>>(response).await?;
            keys.extend(
                envelope
                    .result
                    .unwrap_or_default()
                    .into_iter()
                    .map(|entry| entry.name),
            );
            cursor = envelope
                .result_info
                .map(|info| info.cursor)
                .unwrap_or_default();
            if cursor.is_empty() {
                break;
            }
        }
        keys.sort();
        keys.dedup();
        Ok(Value::Array(keys.into_iter().map(Value::String).collect()))
    }

    pub async fn clear(&self) -> KvResult<Value> {
        let keys = self
            .keys(None)
            .await?
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect::<Vec<_>>();
        for chunk in keys.chunks(10_000) {
            let response = self
                .client
                .delete(self.namespace_url(&["bulk"])?)
                .bearer_auth(&self.config.secret)
                .json(chunk)
                .send()
                .await
                .map_err(remote_transport)?;
            validate_envelope(response).await?;
        }
        Ok(clear_json(keys.len()))
    }

    fn value_url(&self, key: &str) -> KvResult<Url> {
        self.namespace_url(&["values", key])
    }

    fn namespace_url(&self, suffix: &[&str]) -> KvResult<Url> {
        let mut url = self.base.clone();
        let mut segments = url.path_segments_mut().map_err(|_| {
            KvError::InvalidRequest("Cloudflare KV host cannot be a base URL".to_string())
        })?;
        segments.extend([
            "client",
            "v4",
            "accounts",
            &self.config.account,
            "storage",
            "kv",
            "namespaces",
            &self.config.namespace,
        ]);
        segments.extend(suffix.iter().copied());
        drop(segments);
        Ok(url)
    }
}

fn cloudflare_base(config: &CloudflareKvConfig) -> KvResult<Url> {
    let raw = if config.host.contains("://") {
        config.host.clone()
    } else {
        format!("https://{}", config.host)
    };
    let mut url = Url::parse(&raw)
        .map_err(|_| KvError::InvalidRequest("Cloudflare KV host is invalid".to_string()))?;
    let host = url
        .host_str()
        .ok_or_else(|| KvError::InvalidRequest("Cloudflare KV host is missing".to_string()))?;
    if url.scheme() != "https" && !(url.scheme() == "http" && is_loopback(host)) {
        return Err(KvError::InvalidRequest(
            "Cloudflare KV requires HTTPS outside loopback".to_string(),
        ));
    }
    url.set_port(Some(config.port))
        .map_err(|_| KvError::InvalidRequest("Cloudflare KV port is invalid".to_string()))?;
    url.set_path("/");
    url.set_query(None);
    Ok(url)
}

async fn validate_envelope(response: reqwest::Response) -> KvResult<()> {
    parse_envelope::<Value>(response).await.map(|_| ())
}

async fn parse_envelope<T: for<'de> Deserialize<'de> + Default>(
    response: reqwest::Response,
) -> KvResult<Envelope<T>> {
    let status = response.status();
    let envelope = response
        .json::<Envelope<T>>()
        .await
        .map_err(|_| KvError::Remote("Cloudflare KV returned an invalid envelope".to_string()))?;
    if status.is_success() && envelope.success {
        return Ok(envelope);
    }
    let message = envelope
        .errors
        .into_iter()
        .find_map(|error| (!error.message.is_empty()).then_some(error.message))
        .unwrap_or_else(|| format!("Cloudflare KV request failed with HTTP {status}"));
    if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
        Err(KvError::Authentication(message))
    } else {
        Err(KvError::Remote(message))
    }
}

fn remote_transport(error: reqwest::Error) -> KvError {
    if error.is_timeout() {
        KvError::Remote("Cloudflare KV request timed out".to_string())
    } else {
        KvError::Remote("Cloudflare KV request failed".to_string())
    }
}

fn remote_status(status: StatusCode) -> KvError {
    if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
        KvError::Authentication("Cloudflare KV authentication failed".to_string())
    } else {
        KvError::Remote(format!("Cloudflare KV returned HTTP {status}"))
    }
}

fn is_loopback(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
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
    use super::{CloudflareKvConfig, cloudflare_base};

    #[test]
    fn cloudflare_base_requires_secure_remote_transport() {
        let config = CloudflareKvConfig {
            host: "http://api.cloudflare.com".to_string(),
            port: 80,
            account: "account".to_string(),
            secret: "secret".to_string(),
            namespace: "namespace".to_string(),
        };
        assert!(cloudflare_base(&config).is_err());
    }
}
