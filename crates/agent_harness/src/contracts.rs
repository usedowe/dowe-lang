use crate::{HarnessError, HarnessResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::{Component, Path};

const MAX_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersistentContract {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub fingerprint: String,
    #[serde(default)]
    pub request_schema: Option<Value>,
    #[serde(default)]
    pub response_schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractRegistry {
    pub schema: u32,
    pub contracts: Vec<PersistentContract>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContractRegistryStatus {
    pub registry: ContractRegistry,
    pub verified: usize,
}

pub fn load_contract_registry(root: &Path) -> HarnessResult<Option<ContractRegistryStatus>> {
    let path = root.join(".agent/contracts.json");
    let metadata = match std::fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    reject_symlink_ancestors(root, &path)?;
    if !metadata.is_file() || metadata.len() > MAX_BYTES {
        return Err(HarnessError::new(
            "contract registry must be a regular file up to 2 MiB",
        ));
    }
    let registry: ContractRegistry = serde_json::from_slice(&std::fs::read(&path)?)?;
    if registry.schema != 1 || registry.contracts.is_empty() || registry.contracts.len() > 512 {
        return Err(HarnessError::new(
            "contract registry schema or bounds are invalid",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut verified = 0;
    for contract in &registry.contracts {
        if contract.id.is_empty()
            || contract.id.len() > 128
            || !ids.insert(contract.id.clone())
            || contract.kind.is_empty()
            || contract.kind.len() > 64
            || !is_hash(&contract.fingerprint)
        {
            return Err(HarnessError::new(
                "contract registry contains an invalid or duplicate contract",
            ));
        }
        let file = safe_project_file(root, &contract.path)?;
        let bytes = std::fs::read(&file).map_err(|_| {
            HarnessError::new(format!("contract source is missing: {}", contract.path))
        })?;
        let actual = digest(&bytes);
        if actual != contract.fingerprint {
            return Err(HarnessError::new(format!(
                "contract `{}` is stale; expected {}, found {}",
                contract.id, contract.fingerprint, actual
            )));
        }
        verified += 1;
    }
    Ok(Some(ContractRegistryStatus { registry, verified }))
}

fn safe_project_file(root: &Path, value: &str) -> HarnessResult<std::path::PathBuf> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
        || value.starts_with('.')
    {
        return Err(HarnessError::new(
            "contract source must be a visible project-relative file",
        ));
    }
    let full = root.join(path);
    if !full.is_file() {
        return Err(HarnessError::new("contract source must be a regular file"));
    }
    Ok(full)
}

fn reject_symlink_ancestors(root: &Path, path: &Path) -> HarnessResult<()> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| HarnessError::new("contract registry path escaped project root"))?
        .to_path_buf();
    let root = root
        .canonicalize()
        .map_err(|_| HarnessError::new("contract project root is unavailable"))?;
    let mut current = root;
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(HarnessError::new(
                "contract registry path is not project-relative",
            ));
        }
        current.push(component.as_os_str());
        if std::fs::symlink_metadata(&current)
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err(HarnessError::new(
                "contract registry must not traverse symlinks",
            ));
        }
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn is_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_registry_is_explicitly_optional() {
        let root = tempfile::tempdir().expect("tempdir");
        assert!(load_contract_registry(root.path()).unwrap().is_none());
    }

    #[test]
    fn registry_requires_current_source_fingerprint() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("server")).expect("server");
        let source = b"route /health";
        std::fs::write(root.path().join("server/api.dowe"), source).expect("source");
        let registry = ContractRegistry {
            schema: 1,
            contracts: vec![PersistentContract {
                id: "health".into(),
                kind: "http".into(),
                path: "server/api.dowe".into(),
                fingerprint: digest(source),
                request_schema: None,
                response_schema: None,
            }],
        };
        std::fs::create_dir_all(root.path().join(".agent")).expect("agent");
        std::fs::write(
            root.path().join(".agent/contracts.json"),
            serde_json::to_vec(&registry).expect("json"),
        )
        .expect("registry");
        assert_eq!(
            load_contract_registry(root.path())
                .unwrap()
                .unwrap()
                .verified,
            1
        );
        std::fs::write(root.path().join("server/api.dowe"), b"route /changed")
            .expect("changed source");
        assert!(load_contract_registry(root.path()).is_err());
    }
}
