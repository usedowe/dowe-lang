use crate::codec::{Reader, Writer, decode_record, encode_record};
use crate::engine::StoreRecord;
use crate::error::{StoreError, StoreResult};
use crate::names::validate_table_name;
use crate::security::{create_private_directory, secure_file};
use dowe_id::validate_ulid;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const FRAME_MAGIC: &[u8] = b"DOWE_DB_TX_V2\n";
const COMMIT_MAGIC: &[u8] = b"DOWE_DB_COMMIT_V2\n";
const CHECKSUM_LENGTH: usize = 32;
const MAX_PAYLOAD_LENGTH: usize = 64 * 1024 * 1024;
const OP_UPSERT: u8 = 1;
const OP_DELETE: u8 = 2;

#[derive(Debug, Clone)]
pub(crate) struct WalTransaction {
    pub(crate) id: String,
    pub(crate) base_version: u64,
    pub(crate) version: u64,
    pub(crate) operations: Vec<WalOperation>,
}

#[derive(Debug, Clone)]
pub(crate) enum WalOperation {
    Upsert {
        table: String,
        id: String,
        record: StoreRecord,
    },
    Delete {
        table: String,
        id: String,
    },
}

pub(crate) struct RecoveredWal {
    pub(crate) file: File,
    pub(crate) transactions: Vec<WalTransaction>,
}

pub(crate) fn wal_path(database_root: &Path) -> PathBuf {
    database_root.join("wal").join("transactions-v2.bin")
}

pub(crate) fn recover(database_root: &Path) -> StoreResult<RecoveredWal> {
    let path = wal_path(database_root);
    if let Some(parent) = path.parent() {
        create_private_directory(parent)?;
    }
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)?;
    secure_file(&file)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let mut transactions = Vec::new();
    let mut offset = 0usize;
    while offset < bytes.len() {
        let frame_start = offset;
        let minimum = FRAME_MAGIC.len() + 4 + CHECKSUM_LENGTH + COMMIT_MAGIC.len();
        if bytes.len() - offset < minimum {
            truncate_tail(&mut file, frame_start)?;
            break;
        }
        if &bytes[offset..offset + FRAME_MAGIC.len()] != FRAME_MAGIC {
            return Err(StoreError::Corruption(
                "Database WAL frame magic is invalid".to_string(),
            ));
        }
        offset += FRAME_MAGIC.len();
        let payload_length = read_u32(&bytes, offset)? as usize;
        offset += 4;
        if payload_length > MAX_PAYLOAD_LENGTH {
            return Err(StoreError::Corruption(
                "Database WAL frame exceeds the maximum size".to_string(),
            ));
        }
        let frame_length = payload_length
            .checked_add(CHECKSUM_LENGTH)
            .and_then(|length| length.checked_add(COMMIT_MAGIC.len()))
            .ok_or_else(|| {
                StoreError::Corruption("Database WAL frame length overflowed".to_string())
            })?;
        if bytes.len() - offset < frame_length {
            truncate_tail(&mut file, frame_start)?;
            break;
        }
        let payload = &bytes[offset..offset + payload_length];
        offset += payload_length;
        let checksum = &bytes[offset..offset + CHECKSUM_LENGTH];
        offset += CHECKSUM_LENGTH;
        if &bytes[offset..offset + COMMIT_MAGIC.len()] != COMMIT_MAGIC {
            return Err(StoreError::Corruption(
                "Database WAL commit marker is invalid".to_string(),
            ));
        }
        offset += COMMIT_MAGIC.len();
        if Sha256::digest(payload).as_slice() != checksum {
            return Err(StoreError::Corruption(
                "Database WAL checksum is invalid".to_string(),
            ));
        }
        transactions.push(decode_transaction(payload)?);
    }
    file.seek(SeekFrom::End(0))?;
    Ok(RecoveredWal { file, transactions })
}

pub(crate) fn append(file: &mut File, transaction: &WalTransaction) -> StoreResult<()> {
    append_unsynced(file, transaction)?;
    sync(file)
}

pub(crate) fn append_unsynced(file: &mut File, transaction: &WalTransaction) -> StoreResult<()> {
    let payload = encode_transaction(transaction)?;
    if payload.len() > MAX_PAYLOAD_LENGTH {
        return Err(StoreError::DurabilityError(
            "Database transaction exceeds the maximum WAL frame size".to_string(),
        ));
    }
    let checksum = Sha256::digest(&payload);
    let mut frame = Writer::new();
    frame.bytes(FRAME_MAGIC);
    frame.u32(payload.len() as u32);
    frame.bytes(&payload);
    frame.bytes(&checksum);
    frame.bytes(COMMIT_MAGIC);
    file.write_all(&frame.into_bytes())
        .map_err(|error| StoreError::DurabilityError(error.to_string()))
}

pub(crate) fn sync(file: &mut File) -> StoreResult<()> {
    file.sync_data()
        .map_err(|error| StoreError::DurabilityError(error.to_string()))
}

pub(crate) fn reset(file: &mut File) -> StoreResult<()> {
    file.set_len(0)
        .map_err(|error| StoreError::DurabilityError(error.to_string()))?;
    file.sync_data()
        .map_err(|error| StoreError::DurabilityError(error.to_string()))?;
    file.seek(SeekFrom::Start(0))?;
    Ok(())
}

fn encode_transaction(transaction: &WalTransaction) -> StoreResult<Vec<u8>> {
    validate_ulid(&transaction.id)?;
    if transaction.operations.is_empty() {
        return Err(StoreError::DurabilityError(
            "Database transaction has no WAL operations".to_string(),
        ));
    }
    let mut writer = Writer::new();
    writer.string(&transaction.id);
    writer.u64(transaction.base_version);
    writer.u64(transaction.version);
    writer.u32(transaction.operations.len() as u32);
    for operation in &transaction.operations {
        match operation {
            WalOperation::Upsert { table, id, record } => {
                validate_table_name(table)?;
                validate_ulid(id)?;
                validate_record_id(record, id).map_err(|_| {
                    StoreError::DurabilityError(
                        "Database WAL record id does not match its operation".to_string(),
                    )
                })?;
                writer.u8(OP_UPSERT);
                writer.string(table);
                writer.string(id);
                writer.raw(&encode_record(record)?);
            }
            WalOperation::Delete { table, id } => {
                validate_table_name(table)?;
                validate_ulid(id)?;
                writer.u8(OP_DELETE);
                writer.string(table);
                writer.string(id);
            }
        }
    }
    Ok(writer.into_bytes())
}

fn decode_transaction(payload: &[u8]) -> StoreResult<WalTransaction> {
    let mut reader = Reader::new(payload);
    let id = reader.string()?;
    validate_ulid(&id)?;
    let base_version = reader.u64()?;
    let version = reader.u64()?;
    let operation_count = reader.u32()?;
    if operation_count == 0 {
        return Err(StoreError::Corruption(
            "Database WAL transaction has no operations".to_string(),
        ));
    }
    let mut operations = Vec::with_capacity(operation_count as usize);
    let mut keys = BTreeSet::new();
    for _ in 0..operation_count {
        let tag = reader.u8()?;
        let table = reader.string()?;
        validate_table_name(&table)?;
        let id = reader.string()?;
        validate_ulid(&id)?;
        if !keys.insert((table.clone(), id.clone())) {
            return Err(StoreError::Corruption(
                "Database WAL transaction repeats a record".to_string(),
            ));
        }
        operations.push(match tag {
            OP_UPSERT => {
                let record = decode_record(&reader.raw()?)?;
                validate_record_id(&record, &id)?;
                WalOperation::Upsert { table, id, record }
            }
            OP_DELETE => WalOperation::Delete { table, id },
            value => {
                return Err(StoreError::Corruption(format!(
                    "unknown Database WAL operation tag {value}"
                )));
            }
        });
    }
    if !reader.is_done() {
        return Err(StoreError::Corruption(
            "Database WAL transaction contains trailing bytes".to_string(),
        ));
    }
    Ok(WalTransaction {
        id,
        base_version,
        version,
        operations,
    })
}

fn validate_record_id(record: &StoreRecord, id: &str) -> StoreResult<()> {
    match record.get("id") {
        Some(crate::value::StoreValue::Ulid(record_id)) if record_id == id => Ok(()),
        _ => Err(StoreError::Corruption(
            "Database WAL record id does not match its operation".to_string(),
        )),
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> StoreResult<u32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| StoreError::Corruption("Database WAL length overflowed".to_string()))?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| StoreError::Corruption("Database WAL length is incomplete".to_string()))?;
    let mut value = [0u8; 4];
    value.copy_from_slice(slice);
    Ok(u32::from_le_bytes(value))
}

fn truncate_tail(file: &mut File, length: usize) -> StoreResult<()> {
    file.set_len(length as u64)
        .map_err(|error| StoreError::DurabilityError(error.to_string()))?;
    file.sync_data()
        .map_err(|error| StoreError::DurabilityError(error.to_string()))?;
    file.seek(SeekFrom::End(0))?;
    Ok(())
}
