use crate::error::{RuntimeError, RuntimeResult};
use dowe_cache::open_database as open_kv_database;
use dowe_compiler::{TlsConfig, TlsDomainsSource, TlsMode};
use dowe_database::{StoreValue, open_database};
use serde_json::Value;
use std::collections::BTreeSet;
use std::net::IpAddr;
use std::path::Path;

pub(crate) fn effective_domains(root: &Path, config: &TlsConfig) -> RuntimeResult<Vec<String>> {
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
        None => {}
    }
    validate_domains(config.mode, domains.into_iter().collect())
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
    use super::effective_domains;
    use dowe_cache::open_database;
    use dowe_compiler::{TlsConfig, TlsDomainsSource, TlsMode};
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn combines_static_and_managed_kv_domains() {
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
        };

        assert_eq!(
            effective_domains(root.path(), &config).expect("effective domains"),
            ["api.example.com", "example.com"]
        );
    }
}
