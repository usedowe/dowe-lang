use crate::codec::{Reader, Writer};
use crate::engine::{db_root, init_database};
use crate::error::{StoreError, StoreResult};
use crate::names::{validate_account_name, validate_database_name};
use crate::security::{create_private_directory, secure_file};
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Argon2, password_hash};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const AUTH_MAGIC: &[u8] = b"DOWE_DB_AUTH_V1\n";
static AUTH_CATALOG: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
struct CreatedStoreUser {
    pub database: String,
    pub user: String,
    pub credential: String,
    pub generated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedDatabaseAccount {
    pub database: String,
    pub account: String,
    pub secret: String,
    pub generated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoreUser {
    database: String,
    user: String,
    salt: String,
    token_hash: String,
    created_at: String,
    updated_at: String,
}

fn create_user(
    project_root: &Path,
    database: &str,
    user: &str,
    credential: Option<&str>,
) -> StoreResult<CreatedStoreUser> {
    validate_database_name(database)?;
    validate_account_name(user)?;
    init_database(project_root, database)?;
    let (credential, generated) = match credential {
        Some(value) if !value.is_empty() => (value.to_string(), false),
        Some(_) => {
            return Err(StoreError::Authentication(
                "Database secret must not be empty".to_string(),
            ));
        }
        None => (generate_credential(), true),
    };
    let salt = String::new();
    let token_hash = hash_credential(&credential)?;
    let now = timestamp();
    let _catalog = AUTH_CATALOG
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| StoreError::DurabilityError("Database auth lock failed".to_string()))?;
    let _file_lock = acquire_auth_lock(project_root)?;
    let mut users = read_users(project_root)?;
    if let Some(existing) = users
        .iter_mut()
        .find(|entry| entry.database == database && entry.user == user)
    {
        existing.salt = salt;
        existing.token_hash = token_hash;
        existing.updated_at = now;
    } else {
        users.push(StoreUser {
            database: database.to_string(),
            user: user.to_string(),
            salt,
            token_hash,
            created_at: now.clone(),
            updated_at: now,
        });
    }
    users.sort_by(|left, right| {
        left.database
            .cmp(&right.database)
            .then(left.user.cmp(&right.user))
    });
    write_users(project_root, &users)?;
    Ok(CreatedStoreUser {
        database: database.to_string(),
        user: user.to_string(),
        credential,
        generated,
    })
}

pub fn create_account(
    project_root: &Path,
    database: &str,
    account: &str,
    secret: Option<&str>,
) -> StoreResult<CreatedDatabaseAccount> {
    let created = create_user(project_root, database, account, secret)?;
    Ok(CreatedDatabaseAccount {
        database: created.database,
        account: created.user,
        secret: created.credential,
        generated: created.generated,
    })
}

fn verify_user(
    project_root: &Path,
    database: &str,
    user: &str,
    credential: &str,
) -> StoreResult<()> {
    validate_database_name(database)?;
    validate_account_name(user)?;
    if credential.is_empty() {
        return Err(StoreError::Authentication(
            "Database secret is required".to_string(),
        ));
    }
    let users = read_users(project_root)?;
    let mut credential_matches_other_database = false;
    for entry in users.iter().filter(|entry| entry.user == user) {
        if verify_credential(entry, credential) {
            if entry.database == database {
                return Ok(());
            }
            credential_matches_other_database = true;
        }
    }
    if credential_matches_other_database {
        return Err(StoreError::Authorization(format!(
            "Database account `{user}` is not assigned to database `{database}`"
        )));
    }
    Err(StoreError::Authentication(
        "Database account or secret is invalid".to_string(),
    ))
}

pub fn verify_account(
    project_root: &Path,
    database: &str,
    account: &str,
    secret: &str,
) -> StoreResult<()> {
    verify_user(project_root, database, account, secret)
}

fn read_users(project_root: &Path) -> StoreResult<Vec<StoreUser>> {
    let path = users_path(project_root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let mut reader = Reader::new(&bytes);
    reader.magic(AUTH_MAGIC)?;
    let count = reader.u32()?;
    let mut users = Vec::new();
    for _ in 0..count {
        users.push(StoreUser {
            database: reader.string()?,
            user: reader.string()?,
            salt: reader.string()?,
            token_hash: reader.string()?,
            created_at: reader.string()?,
            updated_at: reader.string()?,
        });
    }
    if !reader.is_done() {
        return Err(StoreError::Corruption(
            "Database auth catalog contains trailing bytes".to_string(),
        ));
    }
    Ok(users)
}

fn write_users(project_root: &Path, users: &[StoreUser]) -> StoreResult<()> {
    let root = auth_root(project_root);
    create_private_directory(&root)?;
    let path = root.join("users.bin");
    let mut writer = Writer::new();
    writer.bytes(AUTH_MAGIC);
    writer.u32(users.len() as u32);
    for user in users {
        writer.string(&user.database);
        writer.string(&user.user);
        writer.string(&user.salt);
        writer.string(&user.token_hash);
        writer.string(&user.created_at);
        writer.string(&user.updated_at);
    }
    let temporary = root.join(format!("users-{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)?;
    secure_file(&file)?;
    file.write_all(&writer.into_bytes())?;
    file.sync_all()
        .map_err(|error| StoreError::DurabilityError(error.to_string()))?;
    fs::rename(&temporary, path)?;
    sync_directory(&root)?;
    Ok(())
}

fn acquire_auth_lock(project_root: &Path) -> StoreResult<File> {
    let root = auth_root(project_root);
    create_private_directory(&root)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(root.join(".lock"))?;
    secure_file(&lock)?;
    match lock.try_lock() {
        Ok(()) => Ok(lock),
        Err(TryLockError::WouldBlock) => Err(StoreError::TransactionConflict(
            "Database auth catalog is being updated".to_string(),
        )),
        Err(TryLockError::Error(error)) => Err(StoreError::Io(error.to_string())),
    }
}

fn auth_root(project_root: &Path) -> PathBuf {
    db_root(project_root).join("_auth")
}

fn users_path(project_root: &Path) -> PathBuf {
    auth_root(project_root).join("users.bin")
}

fn generate_credential() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn hash_credential(credential: &str) -> StoreResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(credential.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(password_hash_error)
}

fn verify_credential(user: &StoreUser, credential: &str) -> bool {
    if user.token_hash.starts_with("$argon2id$") {
        return PasswordHash::new(&user.token_hash)
            .ok()
            .is_some_and(|hash| {
                Argon2::default()
                    .verify_password(credential.as_bytes(), &hash)
                    .is_ok()
            });
    }
    let expected = legacy_hash_credential(&user.salt, credential);
    constant_time_eq(expected.as_bytes(), user.token_hash.as_bytes())
}

fn legacy_hash_credential(salt: &str, credential: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update([0]);
    hasher.update(credential.as_bytes());
    hex(&hasher.finalize())
}

fn password_hash_error(error: password_hash::Error) -> StoreError {
    StoreError::Authentication(format!("Database secret hashing failed: {error}"))
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> StoreResult<()> {
    File::open(path)?
        .sync_all()
        .map_err(|error| StoreError::DurabilityError(error.to_string()))
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> StoreResult<()> {
    Ok(())
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

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
