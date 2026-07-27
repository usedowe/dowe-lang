use crate::engine::{init_database, vector_root};
use crate::error::{VectorError, VectorResult};
use crate::names::{validate_account_name, validate_database_name};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static AUTH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedVectorAccount {
    pub name: String,
    pub account: String,
    pub secret: String,
    pub generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VectorAccount {
    name: String,
    account: String,
    salt: String,
    secret_hash: String,
    created_at: String,
    updated_at: String,
}

pub fn create_account(
    project_root: &Path,
    name: &str,
    account: &str,
    secret: Option<&str>,
) -> VectorResult<CreatedVectorAccount> {
    validate_database_name(name)?;
    validate_account_name(account)?;
    init_database(project_root, name)?;
    let (secret, generated) = match secret {
        Some(value) if !value.is_empty() => (value.to_string(), false),
        Some(_) => {
            return Err(VectorError::Authentication(
                "Vector secret must not be empty".to_string(),
            ));
        }
        None => (generate_secret(), true),
    };
    let salt = generate_salt();
    let secret_hash = hash_secret(&salt, &secret);
    let now = timestamp();
    let _guard = auth_guard()?;
    let mut accounts = read_accounts(project_root)?;
    if let Some(existing) = accounts
        .iter_mut()
        .find(|entry| entry.name == name && entry.account == account)
    {
        existing.salt = salt;
        existing.secret_hash = secret_hash;
        existing.updated_at = now;
    } else {
        accounts.push(VectorAccount {
            name: name.to_string(),
            account: account.to_string(),
            salt,
            secret_hash,
            created_at: now.clone(),
            updated_at: now,
        });
    }
    accounts.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.account.cmp(&right.account))
    });
    write_accounts(project_root, &accounts)?;
    Ok(CreatedVectorAccount {
        name: name.to_string(),
        account: account.to_string(),
        secret,
        generated,
    })
}

pub fn verify_account(
    project_root: &Path,
    name: &str,
    account: &str,
    secret: &str,
) -> VectorResult<()> {
    validate_database_name(name)?;
    validate_account_name(account)?;
    if secret.is_empty() {
        return Err(VectorError::Authentication(
            "Vector secret is required".to_string(),
        ));
    }
    let accounts = read_accounts(project_root)?;
    let mut saw_account = false;
    for entry in accounts.iter().filter(|entry| entry.account == account) {
        saw_account = true;
        let expected = hash_secret(&entry.salt, secret);
        if constant_time_eq(expected.as_bytes(), entry.secret_hash.as_bytes()) {
            if entry.name == name {
                return Ok(());
            }
            return Err(VectorError::Authorization(format!(
                "Vector account `{account}` is not assigned to database `{name}`"
            )));
        }
    }
    let message = if saw_account {
        "Vector secret is invalid"
    } else {
        "Vector account is invalid"
    };
    Err(VectorError::Authentication(message.to_string()))
}

fn read_accounts(project_root: &Path) -> VectorResult<Vec<VectorAccount>> {
    let path = accounts_path(project_root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    serde_json::from_slice(&fs::read(path)?)
        .map_err(|error| VectorError::Corruption(error.to_string()))
}

fn write_accounts(project_root: &Path, accounts: &[VectorAccount]) -> VectorResult<()> {
    let root = auth_root(project_root);
    fs::create_dir_all(&root)?;
    let path = accounts_path(project_root);
    let temp = root.join(".accounts.tmp");
    let mut file = File::create(&temp)?;
    file.write_all(&serde_json::to_vec(accounts)?)?;
    file.sync_all()
        .map_err(|error| VectorError::DurabilityError(error.to_string()))?;
    fs::rename(temp, path)?;
    Ok(())
}

fn auth_root(project_root: &Path) -> PathBuf {
    vector_root(project_root).join("_auth")
}

fn accounts_path(project_root: &Path) -> PathBuf {
    auth_root(project_root).join("accounts.json")
}

fn auth_guard() -> VectorResult<std::sync::MutexGuard<'static, ()>> {
    AUTH_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| VectorError::DurabilityError("Vector auth lock failed".to_string()))
}

fn generate_secret() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn generate_salt() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hash_secret(salt: &str, secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update([0]);
    hasher.update(secret.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (left, right) in left.iter().zip(right) {
        diff |= left ^ right;
    }
    diff == 0
}

fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
