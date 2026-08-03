use crate::error::{RuntimeError, RuntimeResult};
use dowe_cache::open_database as open_kv_database;
use dowe_compiler::{
    EnvironmentConfig, HttpConnectionValue, ServerSecret, TlsConfig, TlsDomainsSource, TlsMode,
};
use dowe_database::{StoreValue, open_database};
use serde_json::Value;
use std::collections::BTreeSet;
use std::net::IpAddr;
use std::path::Path;

pub(crate) async fn effective_domains(
    root: &Path,
    config: &TlsConfig,
    environment: &EnvironmentConfig,
) -> RuntimeResult<Vec<String>> {
    let mut domains = config.domains.iter().cloned().collect::<BTreeSet<_>>();
    match &config.domains_from {
        Some(TlsDomainsSource::Kv { database, key }) => {
            let database = open_kv_database(root, database, true)
                .map_err(|error| RuntimeError::new(error.to_string()))?;
            if let Some(value) = database
                .get(key)
                .map_err(|error| RuntimeError::new(error.to_string()))?
            {
                extend_json_domains(&mut domains, value)?;
            }
        }
        Some(TlsDomainsSource::Database {
            database,
            table,
            field,
        }) => {
            let database = open_database(root, database)
                .map_err(|error| RuntimeError::new(error.to_string()))?;
            for record in database
                .records(table)
                .map_err(|error| RuntimeError::new(error.to_string()))?
            {
                if let Some(value) = record.get(field).and_then(store_domain) {
                    domains.insert(value);
                }
            }
        }
        Some(TlsDomainsSource::Endpoint {
            base,
            path,
            bearer,
            timeout_ms,
        }) => {
            let base = environment_value(environment, base)?;
            if !base.starts_with("https://") {
                return Err(RuntimeError::new("TLS domain endpoints must use HTTPS"));
            }
            let token = secret_value(environment, bearer)?;
            let response = reqwest::Client::builder()
                .timeout(std::time::Duration::from_millis(*timeout_ms))
                .build()
                .map_err(|error| RuntimeError::new(error.to_string()))?
                .get(format!("{}{}", base.trim_end_matches('/'), path))
                .bearer_auth(token)
                .send()
                .await
                .map_err(|error| {
                    RuntimeError::new(format!("TLS domain endpoint failed: {error}"))
                })?;
            if !response.status().is_success() {
                return Err(RuntimeError::new(format!(
                    "TLS domain endpoint returned {}",
                    response.status()
                )));
            }
            let value = response.json::<Value>().await.map_err(|error| {
                RuntimeError::new(format!("invalid TLS domain response: {error}"))
            })?;
            extend_endpoint_domains(&mut domains, value)?;
        }
        None => {}
    }
    validate_domains(config.mode, domains.into_iter().collect())
}

fn environment_value(
    environment: &EnvironmentConfig,
    value: &HttpConnectionValue,
) -> RuntimeResult<String> {
    match value {
        HttpConnectionValue::Static(value) => Ok(value.clone()),
        HttpConnectionValue::Environment(name) => environment
            .variable(name)
            .and_then(|variable| variable.resolved_value.clone())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| RuntimeError::new("TLS domain endpoint is not configured")),
    }
}

fn secret_value(environment: &EnvironmentConfig, secret: &ServerSecret) -> RuntimeResult<String> {
    match secret {
        ServerSecret::Environment(name) => environment
            .variable(name)
            .and_then(|variable| variable.resolved_value.clone())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| RuntimeError::new("TLS domain endpoint bearer is not configured")),
    }
}

pub(crate) fn validated_static_domains(config: &TlsConfig) -> RuntimeResult<Vec<String>> {
    validate_domains(
        config.mode,
        config
            .domains
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    )
}

fn validate_domains(mode: TlsMode, domains: Vec<String>) -> RuntimeResult<Vec<String>> {
    for domain in &domains {
        validate_effective_domain(mode, domain)?;
    }
    Ok(domains)
}

fn extend_json_domains(domains: &mut BTreeSet<String>, value: Value) -> RuntimeResult<()> {
    match value {
        Value::String(value) => {
            domains.insert(normalize_domain(value)?);
        }
        Value::Array(values) => {
            for value in values {
                let Value::String(value) = value else {
                    return Err(RuntimeError::new(
                        "TLS KV domain arrays must contain strings",
                    ));
                };
                domains.insert(normalize_domain(value)?);
            }
        }
        _ => {
            return Err(RuntimeError::new(
                "TLS KV domains must be a string or an array of strings",
            ));
        }
    }
    Ok(())
}

fn extend_endpoint_domains(domains: &mut BTreeSet<String>, value: Value) -> RuntimeResult<()> {
    match value {
        Value::Object(mut value) => {
            let value = value
                .remove("domains")
                .ok_or_else(|| RuntimeError::new("TLS domain response requires `domains`"))?;
            extend_endpoint_domains(domains, value)
        }
        Value::Array(values) => {
            for value in values {
                match value {
                    Value::String(value) => {
                        domains.insert(normalize_domain(value)?);
                    }
                    Value::Object(value) => {
                        let domain = value
                            .get("host")
                            .or_else(|| value.get("hostname"))
                            .and_then(Value::as_str)
                            .ok_or_else(|| {
                                RuntimeError::new(
                                    "TLS domain records require a string `host` or `hostname`",
                                )
                            })?;
                        domains.insert(normalize_domain(domain.to_string())?);
                    }
                    _ => {
                        return Err(RuntimeError::new(
                            "TLS domain arrays require strings or domain records",
                        ));
                    }
                }
            }
            Ok(())
        }
        _ => Err(RuntimeError::new(
            "TLS domain response must be an array or `{ domains:[...] }`",
        )),
    }
}

fn store_domain(value: &StoreValue) -> Option<String> {
    match value {
        StoreValue::String(value) | StoreValue::Dsf(value) => normalize_domain(value.clone()).ok(),
        _ => None,
    }
}

fn normalize_domain(value: String) -> RuntimeResult<String> {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() {
        Err(RuntimeError::new("TLS domains cannot be empty"))
    } else {
        Ok(value)
    }
}

fn validate_effective_domain(mode: TlsMode, domain: &str) -> RuntimeResult<()> {
    let local = domain == "localhost"
        || domain.ends_with(".localhost")
        || matches!(domain, "127.0.0.1" | "::1");
    let public_dns = domain.parse::<IpAddr>().is_err()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
        && domain.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        });
    match mode {
        TlsMode::Acme if local || !public_dns => Err(RuntimeError::new(format!(
            "invalid public ACME domain `{domain}`"
        ))),
        TlsMode::Local if !local => Err(RuntimeError::new(format!(
            "local TLS does not support public domain `{domain}`"
        ))),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::{effective_domains, extend_endpoint_domains};
    use dowe_cache::open_database;
    use dowe_compiler::{TlsConfig, TlsDomainsSource, TlsMode};
    use serde_json::json;
    use tempfile::tempdir;

    #[tokio::test]
    async fn combines_static_and_managed_kv_domains() {
        let root = tempdir().expect("root");
        let database = open_database(root.path(), "domains", true).expect("database");
        database
            .set("tls", json!(["api.example.com", "example.com"]))
            .expect("domains");
        let config = TlsConfig {
            mode: TlsMode::Acme,
            domains: vec!["example.com".to_string()],
            email: Some("admin@example.com".to_string()),
            staging: true,
            cache: ".dowe/tls".to_string(),
            domains_from: Some(TlsDomainsSource::Kv {
                database: "domains".to_string(),
                key: "tls".to_string(),
            }),
            refresh_seconds: 60,
            http_port: None,
        };

        assert_eq!(
            effective_domains(root.path(), &config, &Default::default())
                .await
                .expect("effective domains"),
            ["api.example.com", "example.com"]
        );
    }

    #[test]
    fn extracts_endpoint_domain_strings_and_records() {
        let mut domains = std::collections::BTreeSet::new();
        extend_endpoint_domains(
            &mut domains,
            serde_json::json!({
                "domains": [
                    "api.example.com",
                    { "host": "app.example.com" },
                    { "hostname": "www.example.com" }
                ]
            }),
        )
        .expect("domains");

        assert_eq!(
            domains.into_iter().collect::<Vec<_>>(),
            ["api.example.com", "app.example.com", "www.example.com"]
        );
    }
}
