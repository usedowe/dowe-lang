use crate::codec::{Reader, Writer};
use crate::error::{StoreError, StoreResult};
use crate::security::{create_private_directory, secure_file};
use dowe_id::generate_ulid;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::fs::File;

const METADATA_MAGIC: &[u8] = b"DOWE_DB_METADATA_V1\n";
const FORMAT_VERSION: u32 = 1;
const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseMetadata {
    pub database_id: String,
    pub name: String,
    pub format_version: u32,
    pub created_at: u64,
    pub updated_at: u64,
    pub engine_version: String,
    pub sharding: String,
    pub replication: String,
}

impl DatabaseMetadata {
    pub fn new(name: &str) -> Self {
        let now = unix_seconds();
        Self {
            database_id: generate_ulid(),
            name: name.to_string(),
            format_version: FORMAT_VERSION,
            created_at: now,
            updated_at: now,
            engine_version: ENGINE_VERSION.to_string(),
            sharding: "disabled".to_string(),
            replication: "disabled".to_string(),
        }
    }
}

pub fn write_metadata(path: &Path, metadata: &DatabaseMetadata) -> StoreResult<()> {
    let mut writer = Writer::new();
    writer.bytes(METADATA_MAGIC);
    writer.string(&metadata.database_id);
    writer.string(&metadata.name);
    writer.u32(metadata.format_version);
    writer.u64(metadata.created_at);
    writer.u64(metadata.updated_at);
    writer.string(&metadata.engine_version);
    writer.string(&metadata.sharding);
    writer.string(&metadata.replication);
    let parent = path.parent().ok_or_else(|| {
        StoreError::DurabilityError("Database metadata path has no parent".to_string())
    })?;
    create_private_directory(parent)?;
    let temporary = parent.join(format!("metadata-{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)?;
    secure_file(&file)?;
    file.write_all(&writer.into_bytes())?;
    file.sync_all()
        .map_err(|error| StoreError::DurabilityError(error.to_string()))?;
    fs::rename(temporary, path)?;
    sync_directory(parent)?;
    Ok(())
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

pub fn read_metadata(path: &Path) -> StoreResult<DatabaseMetadata> {
    let bytes = fs::read(path)?;
    let mut reader = Reader::new(&bytes);
    reader.magic(METADATA_MAGIC)?;
    let metadata = DatabaseMetadata {
        database_id: reader.string()?,
        name: reader.string()?,
        format_version: reader.u32()?,
        created_at: reader.u64()?,
        updated_at: reader.u64()?,
        engine_version: reader.string()?,
        sharding: reader.string()?,
        replication: reader.string()?,
    };
    if metadata.format_version != FORMAT_VERSION {
        return Err(StoreError::UnsupportedFormat(format!(
            "Database format version {} is not supported",
            metadata.format_version
        )));
    }
    Ok(metadata)
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
