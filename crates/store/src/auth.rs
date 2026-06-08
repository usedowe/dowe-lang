use crate::codec::{Reader, Writer};
use crate::engine::{init_database, store_root};
use crate::error::{StoreError, StoreResult};
use crate::names::{validate_database_name, validate_user_name};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const AUTH_MAGIC: &[u8] = b"DOWE_STORE_AUTH_V1\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedStoreUser {
    pub database: String,
    pub user: String,
    pub credential: String,
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

pub fn create_user(
    project_root: &Path,
    database: &str,
    user: &str,
    credential: Option<&str>,
) -> StoreResult<CreatedStoreUser> {
    validate_database_name(database)?;
    validate_user_name(user)?;
    init_database(project_root, database)?;
    let (credential, generated) = match credential {
        Some(value) if !value.is_empty() => (value.to_string(), false),
        Some(_) => {
            return Err(StoreError::Authentication(
                "Store credential must not be empty".to_string(),
            ));
        }
        None => (generate_credential(), true),
    };
    let salt = generate_salt();
    let token_hash = hash_credential(&salt, &credential);
    let now = timestamp();
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

pub fn verify_user(
    project_root: &Path,
    database: &str,
    user: &str,
    credential: &str,
) -> StoreResult<()> {
    validate_database_name(database)?;
    validate_user_name(user)?;
    if credential.is_empty() {
        return Err(StoreError::Authentication(
            "Store credential is required".to_string(),
        ));
    }
    let users = read_users(project_root)?;
    let mut saw_user = false;
    for entry in users.iter().filter(|entry| entry.user == user) {
        saw_user = true;
        let expected = hash_credential(&entry.salt, credential);
        if constant_time_eq(expected.as_bytes(), entry.token_hash.as_bytes()) {
            if entry.database == database {
                return Ok(());
            }
            return Err(StoreError::Authorization(format!(
                "Store user `{user}` is not assigned to database `{database}`"
            )));
        }
    }
    let message = if saw_user {
        "Store credential is invalid"
    } else {
        "Store user is invalid"
    };
    Err(StoreError::Authentication(message.to_string()))
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
            "Store auth catalog contains trailing bytes".to_string(),
        ));
    }
    Ok(users)
}

fn write_users(project_root: &Path, users: &[StoreUser]) -> StoreResult<()> {
    let root = auth_root(project_root);
    fs::create_dir_all(&root)?;
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
    let mut file = File::create(path)?;
    file.write_all(&writer.into_bytes())?;
    file.sync_all()
        .map_err(|error| StoreError::DurabilityError(error.to_string()))?;
    Ok(())
}

fn auth_root(project_root: &Path) -> PathBuf {
    store_root(project_root).join("_auth")
}

fn users_path(project_root: &Path) -> PathBuf {
    auth_root(project_root).join("users.bin")
}

fn generate_credential() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn generate_salt() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex(&bytes)
}

fn hash_credential(salt: &str, credential: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update([0]);
    hasher.update(credential.as_bytes());
    hex(&hasher.finalize())
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
