use crate::engine::{init_namespace, queue_root};
use crate::error::{QueueError, QueueResult};
use crate::names::{validate_account_name, validate_namespace};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use dowe_id::generate_ulid;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{Debug, Formatter};
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static AUTH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Clone, PartialEq, Eq)]
pub struct CreatedQueueAccount {
    pub name: String,
    pub account: String,
    pub secret: String,
    pub generated: bool,
}

impl Debug for CreatedQueueAccount {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CreatedQueueAccount")
            .field("name", &self.name)
            .field("account", &self.account)
            .field("secret", &"[redacted]")
            .field("generated", &self.generated)
            .finish()
    }
}

#[derive(Serialize, Deserialize)]
struct AccountCatalog {
    version: u8,
    accounts: Vec<AccountRecord>,
}

#[derive(Serialize, Deserialize)]
struct AccountRecord {
    name: String,
    account: String,
    salt: String,
    secret_hash: String,
    created_at: u64,
    updated_at: u64,
}

pub fn create_account(
    project_root: &Path,
    name: &str,
    account: &str,
    secret: Option<&str>,
) -> QueueResult<CreatedQueueAccount> {
    validate_namespace(name)?;
    validate_account_name(account)?;
    let (secret, generated) = match secret {
        Some(value) if !value.is_empty() => (value.to_string(), false),
        Some(_) => {
            return Err(QueueError::Authentication(
                "Queue secret must not be empty".to_string(),
            ));
        }
        None => (generate_secret(), true),
    };
    let _process_lock = AUTH_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| QueueError::DurabilityError("Queue auth lock failed".to_string()))?;
    let root = auth_root(project_root);
    let _catalog_lock = acquire_catalog_writer_lock(&root)?;
    init_namespace(project_root, name)?;
    let mut catalog = read_catalog(project_root)?;
    let salt = generate_salt();
    let secret_hash = hash_secret(&salt, &secret);
    let now = timestamp();
    if let Some(existing) = catalog
        .accounts
        .iter_mut()
        .find(|entry| entry.name == name && entry.account == account)
    {
        existing.salt = salt;
        existing.secret_hash = secret_hash;
        existing.updated_at = now;
    } else {
        catalog.accounts.push(AccountRecord {
            name: name.to_string(),
            account: account.to_string(),
            salt,
            secret_hash,
            created_at: now,
            updated_at: now,
        });
    }
    catalog.accounts.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.account.cmp(&right.account))
    });
    write_catalog(&root, &catalog)?;
    Ok(CreatedQueueAccount {
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
) -> QueueResult<()> {
    validate_namespace(name)?;
    validate_account_name(account)?;
    if secret.is_empty() {
        return Err(QueueError::Authentication(
            "Queue secret is required".to_string(),
        ));
    }
    let catalog = read_catalog(project_root)?;
    let mut saw_account = false;
    let mut matched_pair = false;
    let mut matched_other_namespace = false;
    for record in catalog
        .accounts
        .iter()
        .filter(|record| record.account == account)
    {
        saw_account = true;
        let candidate = hash_secret(&record.salt, secret);
        if constant_time_eq(candidate.as_bytes(), record.secret_hash.as_bytes()) {
            if record.name == name {
                matched_pair = true;
            } else {
                matched_other_namespace = true;
            }
        }
    }
    if matched_pair {
        return Ok(());
    }
    if matched_other_namespace {
        return Err(QueueError::Authorization(
            "Queue account is not assigned to this namespace".to_string(),
        ));
    }
    Err(QueueError::Authentication(if saw_account {
        "Queue secret is invalid".to_string()
    } else {
        "Queue account is invalid".to_string()
    }))
}

fn read_catalog(project_root: &Path) -> QueueResult<AccountCatalog> {
    let path = catalog_path(project_root);
    if !path.exists() {
        return Ok(AccountCatalog {
            version: 1,
            accounts: Vec::new(),
        });
    }
    let bytes = fs::read(path)?;
    let catalog = serde_json::from_slice::<AccountCatalog>(&bytes)
        .map_err(|_| QueueError::Corruption("Queue auth catalog cannot be read".to_string()))?;
    if catalog.version != 1 {
        return Err(QueueError::Corruption(
            "Queue auth catalog format is incompatible".to_string(),
        ));
    }
    Ok(catalog)
}

fn write_catalog(root: &Path, catalog: &AccountCatalog) -> QueueResult<()> {
    let target = root.join("accounts.json");
    let temporary = root.join(format!(".accounts-{}.tmp", generate_ulid()));
    let bytes = serde_json::to_vec(catalog).map_err(|_| {
        QueueError::DurabilityError("Queue auth catalog cannot be encoded".to_string())
    })?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all().map_err(|_| {
        QueueError::DurabilityError("Queue auth catalog cannot be synchronized".to_string())
    })?;
    fs::rename(temporary, target)?;
    Ok(())
}

fn acquire_catalog_writer_lock(root: &Path) -> QueueResult<File> {
    fs::create_dir_all(root)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(root.join(".lock"))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(QueueError::DurabilityError(
            "Queue auth catalog is already in use".to_string(),
        )),
        Err(TryLockError::Error(_)) => Err(QueueError::DurabilityError(
            "Queue auth catalog lock cannot be acquired".to_string(),
        )),
    }
}

fn auth_root(project_root: &Path) -> PathBuf {
    queue_root(project_root).join("_auth")
}

fn catalog_path(project_root: &Path) -> PathBuf {
    auth_root(project_root).join("accounts.json")
}

fn generate_secret() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn generate_salt() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex(&bytes)
}

fn hash_secret(salt: &str, secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update([0]);
    hasher.update(secret.as_bytes());
    hex(&hasher.finalize())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
