use crate::codec::{Reader, Writer, decode_record, encode_record};
use crate::error::{StoreError, StoreResult};
use crate::metadata::{DatabaseMetadata, read_metadata, write_metadata};
use crate::names::{validate_database_name, validate_field_name, validate_table_name};
use crate::query::{QueryOutcome, execute_sql};
use crate::transaction::Transaction;
use crate::value::{StoreValue, record_to_json};
use dowe_id::{generate_ulid, validate_ulid};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const RECORD_MAGIC: &[u8] = b"DOWE_STORE_RECORD_V1\n";
const INDEX_MAGIC: &[u8] = b"DOWE_STORE_INDEX_V1\n";
const TABLE_MAGIC: &[u8] = b"DOWE_STORE_TABLE_V1\n";
const OP_UPSERT: u8 = 1;
const OP_DELETE: u8 = 2;

pub type StoreRecord = BTreeMap<String, StoreValue>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexInfo {
    pub table: String,
    pub field: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseInspection {
    pub database_id: String,
    pub name: String,
    pub format_version: u32,
    pub tables: Vec<TableInspection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableInspection {
    pub name: String,
    pub indexes: Vec<String>,
    pub records: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactReport {
    pub database: String,
    pub tables: usize,
    pub records: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryPlan {
    pub indexed: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct Database {
    root: PathBuf,
    name: String,
    metadata: DatabaseMetadata,
}

pub fn store_root(project_root: &Path) -> PathBuf {
    project_root.join(".dowe").join("store")
}

pub fn init_database(project_root: &Path, name: &str) -> StoreResult<DatabaseMetadata> {
    validate_database_name(name)?;
    let database_root = store_root(project_root).join(name);
    fs::create_dir_all(&database_root)?;
    let metadata_path = database_root.join("metadata.bin");
    if metadata_path.exists() {
        return read_metadata(&metadata_path);
    }
    let metadata = DatabaseMetadata::new(name);
    write_metadata(&metadata_path, &metadata)?;
    Ok(metadata)
}

pub fn open_database(project_root: &Path, name: &str) -> StoreResult<Database> {
    validate_database_name(name)?;
    let database_root = store_root(project_root).join(name);
    let metadata_path = database_root.join("metadata.bin");
    if !metadata_path.exists() {
        return Err(StoreError::NotFound(format!(
            "database `{name}` does not exist"
        )));
    }
    let metadata = read_metadata(&metadata_path)?;
    validate_ulid(&metadata.database_id)?;
    Ok(Database {
        root: database_root,
        name: name.to_string(),
        metadata,
    })
}

pub fn list_databases(project_root: &Path) -> StoreResult<Vec<DatabaseMetadata>> {
    let root = store_root(project_root);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut databases = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let metadata_path = entry.path().join("metadata.bin");
        if metadata_path.exists() {
            databases.push(read_metadata(&metadata_path)?);
        }
    }
    databases.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(databases)
}

impl Database {
    pub fn metadata(&self) -> &DatabaseMetadata {
        &self.metadata
    }

    pub fn insert(&self, table: &str, mut record: StoreRecord) -> StoreResult<StoreRecord> {
        validate_table_name(table)?;
        let id = match record.get("id") {
            Some(StoreValue::Ulid(value)) | Some(StoreValue::String(value)) => {
                validate_ulid(value)?;
                value.clone()
            }
            Some(_) => {
                return Err(StoreError::InvalidUlid(
                    "record id must be a ULID string".to_string(),
                ));
            }
            None => generate_ulid(),
        };
        record.insert("id".to_string(), StoreValue::Ulid(id.clone()));
        let mut records = self.load_table(table)?;
        if records.contains_key(&id) {
            return Err(StoreError::AlreadyExists(format!(
                "record `{id}` already exists"
            )));
        }
        self.append_operation(table, OP_UPSERT, &id, Some(&record))?;
        records.insert(id, record.clone());
        self.rewrite_indexes(table, &records)?;
        Ok(record)
    }

    pub fn update(
        &self,
        table: &str,
        field: &str,
        expected: &StoreValue,
        patch: StoreRecord,
    ) -> StoreResult<usize> {
        validate_table_name(table)?;
        validate_field_name(field)?;
        let mut records = self.load_table(table)?;
        let mut changed = 0usize;
        for (id, record) in records.iter_mut() {
            if record
                .get(field)
                .is_some_and(|value| values_equal(value, expected))
            {
                for (key, value) in &patch {
                    record.insert(key.clone(), value.clone());
                }
                self.append_operation(table, OP_UPSERT, id, Some(record))?;
                changed += 1;
            }
        }
        self.rewrite_indexes(table, &records)?;
        Ok(changed)
    }

    pub fn delete(&self, table: &str, field: &str, expected: &StoreValue) -> StoreResult<usize> {
        validate_table_name(table)?;
        validate_field_name(field)?;
        let mut records = self.load_table(table)?;
        let deleted = records
            .iter()
            .filter_map(|(id, record)| {
                record
                    .get(field)
                    .is_some_and(|value| values_equal(value, expected))
                    .then(|| id.clone())
            })
            .collect::<Vec<_>>();
        for id in &deleted {
            self.append_operation(table, OP_DELETE, id, None)?;
            records.remove(id);
        }
        self.rewrite_indexes(table, &records)?;
        Ok(deleted.len())
    }

    pub fn records(&self, table: &str) -> StoreResult<Vec<StoreRecord>> {
        validate_table_name(table)?;
        Ok(self.load_table(table)?.into_values().collect())
    }

    pub fn query(&self, sql: &str) -> StoreResult<Vec<StoreRecord>> {
        Ok(self.query_with_plan(sql)?.0)
    }

    pub fn query_json(&self, sql: &str) -> StoreResult<Value> {
        Ok(Value::Array(
            self.query(sql)?
                .iter()
                .map(record_to_json)
                .collect::<Vec<_>>(),
        ))
    }

    pub fn query_with_plan(&self, sql: &str) -> StoreResult<(Vec<StoreRecord>, QueryPlan)> {
        let outcome = execute_sql(self, sql)?;
        Ok(match outcome {
            QueryOutcome::Rows { rows, plan } => (rows, plan),
            QueryOutcome::Changed { count, detail } => {
                let mut row = StoreRecord::new();
                row.insert("changed".to_string(), StoreValue::UInt(count as u64));
                (
                    vec![row],
                    QueryPlan {
                        indexed: false,
                        detail,
                    },
                )
            }
        })
    }

    pub fn create_index(&self, table: &str, field: &str) -> StoreResult<IndexInfo> {
        validate_table_name(table)?;
        validate_field_name(field)?;
        self.ensure_table(table)?;
        let path = self
            .table_root(table)
            .join("indexes")
            .join(format!("{field}.idx"));
        let mut writer = Writer::new();
        writer.bytes(INDEX_MAGIC);
        writer.string(table);
        writer.string(field);
        fs::write(&path, writer.into_bytes())?;
        let records = self.load_table(table)?;
        self.rewrite_indexes(table, &records)?;
        Ok(IndexInfo {
            table: table.to_string(),
            field: field.to_string(),
            path,
        })
    }

    pub fn has_index(&self, table: &str, field: &str) -> bool {
        self.table_root(table)
            .join("indexes")
            .join(format!("{field}.idx"))
            .exists()
    }

    pub fn inspect(&self) -> StoreResult<DatabaseInspection> {
        let mut tables = Vec::new();
        for table in self.table_names()? {
            let records = self.load_table(&table)?.len();
            let mut indexes = Vec::new();
            let index_root = self.table_root(&table).join("indexes");
            if index_root.exists() {
                for entry in fs::read_dir(index_root)? {
                    let entry = entry?;
                    if entry.file_type()?.is_file()
                        && let Some(name) = entry.path().file_stem().and_then(|name| name.to_str())
                    {
                        indexes.push(name.to_string());
                    }
                }
            }
            indexes.sort();
            tables.push(TableInspection {
                name: table,
                indexes,
                records,
            });
        }
        tables.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(DatabaseInspection {
            database_id: self.metadata.database_id.clone(),
            name: self.name.clone(),
            format_version: self.metadata.format_version,
            tables,
        })
    }

    pub fn compact(&self) -> StoreResult<CompactReport> {
        let mut report = CompactReport {
            database: self.name.clone(),
            tables: 0,
            records: 0,
        };
        for table in self.table_names()? {
            let records = self.load_table(&table)?;
            self.rewrite_table(&table, &records)?;
            self.rewrite_indexes(&table, &records)?;
            report.tables += 1;
            report.records += records.len();
        }
        Ok(report)
    }

    pub fn transaction(&self) -> Transaction {
        Transaction::new(self.clone())
    }

    fn table_root(&self, table: &str) -> PathBuf {
        self.root.join(table)
    }

    fn ensure_table(&self, table: &str) -> StoreResult<()> {
        validate_table_name(table)?;
        let root = self.table_root(table);
        fs::create_dir_all(root.join("wal"))?;
        fs::create_dir_all(root.join("segments"))?;
        fs::create_dir_all(root.join("indexes"))?;
        fs::create_dir_all(root.join("snapshots"))?;
        fs::create_dir_all(root.join("cache"))?;
        let metadata = root.join("metadata.bin");
        if !metadata.exists() {
            let mut writer = Writer::new();
            writer.bytes(TABLE_MAGIC);
            writer.string(table);
            fs::write(metadata, writer.into_bytes())?;
        }
        Ok(())
    }

    fn load_table(&self, table: &str) -> StoreResult<BTreeMap<String, StoreRecord>> {
        validate_table_name(table)?;
        let path = self.table_root(table).join("segments").join("data.bin");
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
                    let record_bytes = reader.raw()?;
                    let record = decode_record(&record_bytes)?;
                    records.insert(id, record);
                }
                OP_DELETE => {
                    reader.raw()?;
                    records.remove(&id);
                }
                value => {
                    return Err(StoreError::Corruption(format!(
                        "unknown Store operation tag {value}"
                    )));
                }
            }
        }
        Ok(records)
    }

    fn append_operation(
        &self,
        table: &str,
        operation: u8,
        id: &str,
        record: Option<&StoreRecord>,
    ) -> StoreResult<()> {
        self.ensure_table(table)?;
        let bytes = record.map(encode_record).transpose()?.unwrap_or_default();
        let mut writer = Writer::new();
        writer.bytes(RECORD_MAGIC);
        writer.u8(operation);
        writer.string(id);
        writer.raw(&bytes);
        let payload = writer.into_bytes();
        append_synced(
            &self
                .table_root(table)
                .join("wal")
                .join("committed-transactions.bin"),
            &payload,
        )?;
        append_synced(
            &self.table_root(table).join("segments").join("data.bin"),
            &payload,
        )?;
        Ok(())
    }

    fn rewrite_table(
        &self,
        table: &str,
        records: &BTreeMap<String, StoreRecord>,
    ) -> StoreResult<()> {
        self.ensure_table(table)?;
        let path = self.table_root(table).join("segments").join("data.bin");
        let mut file = File::create(path)?;
        for (id, record) in records {
            let bytes = encode_record(record)?;
            let mut writer = Writer::new();
            writer.bytes(RECORD_MAGIC);
            writer.u8(OP_UPSERT);
            writer.string(id);
            writer.raw(&bytes);
            file.write_all(&writer.into_bytes())?;
        }
        file.sync_all()
            .map_err(|error| StoreError::DurabilityError(error.to_string()))?;
        Ok(())
    }

    fn rewrite_indexes(
        &self,
        table: &str,
        records: &BTreeMap<String, StoreRecord>,
    ) -> StoreResult<()> {
        let index_root = self.table_root(table).join("indexes");
        if !index_root.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(index_root)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let Some(field) = entry
                .path()
                .file_stem()
                .and_then(|name| name.to_str())
                .map(str::to_string)
            else {
                continue;
            };
            let path = entry.path();
            let mut writer = Writer::new();
            writer.bytes(INDEX_MAGIC);
            writer.string(table);
            writer.string(&field);
            let mut values = Vec::new();
            for (id, record) in records {
                if let Some(value) = record.get(&field) {
                    values.push((value.comparable_text(), id.clone()));
                }
            }
            values.sort();
            writer.u32(values.len() as u32);
            for (value, id) in values {
                writer.string(&value);
                writer.string(&id);
            }
            fs::write(path, writer.into_bytes())?;
        }
        Ok(())
    }

    fn table_names(&self) -> StoreResult<Vec<String>> {
        let mut tables = Vec::new();
        if !self.root.exists() {
            return Ok(tables);
        }
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && entry.path().join("metadata.bin").exists()
                && let Some(name) = entry.file_name().to_str()
            {
                tables.push(name.to_string());
            }
        }
        tables.sort();
        Ok(tables)
    }
}

fn append_synced(path: &Path, bytes: &[u8]) -> StoreResult<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
        .map_err(|error| StoreError::DurabilityError(error.to_string()))?;
    Ok(())
}

fn values_equal(left: &StoreValue, right: &StoreValue) -> bool {
    left.comparable_text() == right.comparable_text()
}
