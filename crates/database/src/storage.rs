use crate::codec::{Reader, Writer, decode_record, encode_record};
use crate::engine::StoreRecord;
use crate::error::{StoreError, StoreResult};
use crate::names::{validate_field_name, validate_table_name};
use crate::security::{create_private_directory, secure_file};
use dowe_id::generate_ulid;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const RECORD_MAGIC: &[u8] = b"DOWE_DB_RECORD_V1\n";
const INDEX_MAGIC: &[u8] = b"DOWE_DB_INDEX_V1\n";
const TABLE_MAGIC: &[u8] = b"DOWE_DB_TABLE_V1\n";
const OP_UPSERT: u8 = 1;
const OP_DELETE: u8 = 2;

pub(crate) fn ensure_table(root: &Path, table: &str) -> StoreResult<()> {
    validate_table_name(table)?;
    let table_root = root.join(table);
    create_private_directory(&table_root)?;
    create_private_directory(&table_root.join("wal"))?;
    create_private_directory(&table_root.join("segments"))?;
    create_private_directory(&table_root.join("indexes"))?;
    create_private_directory(&table_root.join("snapshots"))?;
    create_private_directory(&table_root.join("cache"))?;
    let metadata = table_root.join("metadata.bin");
    if !metadata.exists() {
        let mut writer = Writer::new();
        writer.bytes(TABLE_MAGIC);
        writer.string(table);
        atomic_write(&metadata, &writer.into_bytes())?;
    }
    Ok(())
}

pub(crate) fn load_tables(
    root: &Path,
) -> StoreResult<BTreeMap<String, BTreeMap<String, StoreRecord>>> {
    let mut tables = BTreeMap::new();
    for table in table_names(root)? {
        tables.insert(table.clone(), load_table(root, &table)?);
    }
    Ok(tables)
}

pub(crate) fn table_names(root: &Path) -> StoreResult<Vec<String>> {
    let mut tables = Vec::new();
    if !root.exists() {
        return Ok(tables);
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && entry.path().join("metadata.bin").exists()
            && let Some(name) = entry.file_name().to_str()
        {
            validate_table_name(name)?;
            tables.push(name.to_string());
        }
    }
    tables.sort();
    Ok(tables)
}

pub(crate) fn index_fields(root: &Path, table: &str) -> StoreResult<Vec<String>> {
    let index_root = root.join(table).join("indexes");
    if !index_root.exists() {
        return Ok(Vec::new());
    }
    let mut indexes = Vec::new();
    for entry in fs::read_dir(index_root)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && let Some(name) = entry.path().file_stem().and_then(|name| name.to_str())
        {
            validate_field_name(name)?;
            indexes.push(name.to_string());
        }
    }
    indexes.sort();
    Ok(indexes)
}

pub(crate) fn create_index(
    root: &Path,
    table: &str,
    field: &str,
    records: &BTreeMap<String, StoreRecord>,
) -> StoreResult<PathBuf> {
    validate_table_name(table)?;
    validate_field_name(field)?;
    ensure_table(root, table)?;
    let path = root
        .join(table)
        .join("indexes")
        .join(format!("{field}.idx"));
    write_index(&path, table, field, records)?;
    Ok(path)
}

pub(crate) fn rewrite_indexes(
    root: &Path,
    table: &str,
    records: &BTreeMap<String, StoreRecord>,
) -> StoreResult<()> {
    for field in index_fields(root, table)? {
        let path = root
            .join(table)
            .join("indexes")
            .join(format!("{field}.idx"));
        write_index(&path, table, &field, records)?;
    }
    Ok(())
}

pub(crate) fn rewrite_table(
    root: &Path,
    table: &str,
    records: &BTreeMap<String, StoreRecord>,
) -> StoreResult<()> {
    ensure_table(root, table)?;
    let path = root.join(table).join("segments").join("data.bin");
    let mut writer = Writer::new();
    for (id, record) in records {
        writer.bytes(RECORD_MAGIC);
        writer.u8(OP_UPSERT);
        writer.string(id);
        writer.raw(&encode_record(record)?);
    }
    atomic_write(&path, &writer.into_bytes())
}

fn load_table(root: &Path, table: &str) -> StoreResult<BTreeMap<String, StoreRecord>> {
    validate_table_name(table)?;
    let path = root.join(table).join("segments").join("data.bin");
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let mut reader = Reader::new(&bytes);
    let mut records = BTreeMap::new();
    while !reader.is_done() {
        reader.magic(RECORD_MAGIC)?;
        let operation = reader.u8()?;
        let id = reader.string()?;
        match operation {
            OP_UPSERT => {
                records.insert(id, decode_record(&reader.raw()?)?);
            }
            OP_DELETE => {
                reader.raw()?;
                records.remove(&id);
            }
            value => {
                return Err(StoreError::Corruption(format!(
                    "unknown Database operation tag {value}"
                )));
            }
        }
    }
    Ok(records)
}

fn write_index(
    path: &Path,
    table: &str,
    field: &str,
    records: &BTreeMap<String, StoreRecord>,
) -> StoreResult<()> {
    let mut writer = Writer::new();
    writer.bytes(INDEX_MAGIC);
    writer.string(table);
    writer.string(field);
    let mut values = records
        .iter()
        .filter_map(|(id, record)| {
            record
                .get(field)
                .map(|value| (value.comparable_text(), id.clone()))
        })
        .collect::<Vec<_>>();
    values.sort();
    writer.u32(values.len() as u32);
    for (value, id) in values {
        writer.string(&value);
        writer.string(&id);
    }
    atomic_write(path, &writer.into_bytes())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> StoreResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| StoreError::DurabilityError("Database path has no parent".to_string()))?;
    create_private_directory(parent)?;
    let temporary = parent.join(format!(".database-{}.tmp", generate_ulid()));
    let mut file = File::create(&temporary)?;
    secure_file(&file)?;
    file.write_all(bytes)?;
    file.sync_all()
        .map_err(|error| StoreError::DurabilityError(error.to_string()))?;
    fs::rename(&temporary, path)?;
    sync_directory(parent)
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
