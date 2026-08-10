use crate::error::{StoreError, StoreResult};
use crate::metadata::{DatabaseMetadata, read_metadata, write_metadata};
use crate::names::{validate_database_name, validate_field_name, validate_table_name};
use crate::query::{QueryOutcome, execute_sql};
use crate::security::{create_private_directory, secure_file};
use crate::state::{
    DatabaseState, apply_recovered_transaction, ensure_available, legacy_state, prepare_insert,
    validate_patch, values_equal,
};
use crate::storage;
use crate::transaction::Transaction;
use crate::value::{StoreValue, record_to_json};
use crate::wal::{self, WalOperation};
use dowe_id::validate_ulid;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock, Weak};

static DATABASES: OnceLock<Mutex<HashMap<PathBuf, Weak<SharedDatabase>>>> = OnceLock::new();
static INITIALIZATION: OnceLock<Mutex<()>> = OnceLock::new();

pub type StoreRecord = BTreeMap<String, StoreValue>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexInfo {
    pub table: String,
    pub field: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DatabaseInspection {
    pub database_id: String,
    pub name: String,
    pub format_version: u32,
    pub version: u64,
    pub tables: Vec<TableInspection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    pub(crate) shared: Arc<SharedDatabase>,
}

#[derive(Debug)]
pub(crate) struct SharedDatabase {
    pub(crate) root: PathBuf,
    pub(crate) name: String,
    pub(crate) metadata: DatabaseMetadata,
    pub(crate) _lock: File,
    pub(crate) state: RwLock<DatabaseState>,
    pub(crate) wal: Mutex<File>,
    pub(crate) commits: crate::commit::CommitGroup,
}

pub fn db_root(project_root: &Path) -> PathBuf {
    project_root.join(".dowe").join("db")
}

pub fn init_database(project_root: &Path, name: &str) -> StoreResult<DatabaseMetadata> {
    validate_database_name(name)?;
    let database_root = db_root(project_root).join(name);
    create_private_directory(&db_root(project_root))?;
    create_private_directory(&database_root)?;
    create_private_directory(&database_root.join("wal"))?;
    let metadata_path = database_root.join("metadata.bin");
    if metadata_path.exists() {
        return read_metadata(&metadata_path);
    }
    let _initialization = INITIALIZATION
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| StoreError::DurabilityError("Database init lock failed".to_string()))?;
    let _lock = acquire_file_lock(
        &database_root.join(".init.lock"),
        "Database is being initialized",
    )?;
    if metadata_path.exists() {
        return read_metadata(&metadata_path);
    }
    let metadata = DatabaseMetadata::new(name);
    write_metadata(&metadata_path, &metadata)?;
    Ok(metadata)
}

pub fn open_database(project_root: &Path, name: &str) -> StoreResult<Database> {
    validate_database_name(name)?;
    let database_root = db_root(project_root).join(name);
    let metadata_path = database_root.join("metadata.bin");
    if !metadata_path.exists() {
        return Err(StoreError::NotFound(format!(
            "database `{name}` does not exist"
        )));
    }
    let database_root = fs::canonicalize(database_root)?;
    let registry = DATABASES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut registry = registry
        .lock()
        .map_err(|_| StoreError::DurabilityError("Database registry lock failed".to_string()))?;
    if let Some(shared) = registry.get(&database_root).and_then(Weak::upgrade) {
        return Ok(Database { shared });
    }
    let lock = acquire_database_lock(&database_root)?;
    let metadata = read_metadata(&database_root.join("metadata.bin"))?;
    validate_ulid(&metadata.database_id)?;
    if metadata.name != name {
        return Err(StoreError::Corruption(
            "Database metadata name does not match its directory".to_string(),
        ));
    }
    let mut state = legacy_state(&database_root)?;
    let recovered = wal::recover(&database_root)?;
    for transaction in &recovered.transactions {
        apply_recovered_transaction(&mut state, transaction)?;
    }
    let shared = Arc::new(SharedDatabase {
        root: database_root.clone(),
        name: name.to_string(),
        metadata,
        _lock: lock,
        state: RwLock::new(state),
        wal: Mutex::new(recovered.file),
        commits: crate::commit::CommitGroup::default(),
    });
    registry.insert(database_root, Arc::downgrade(&shared));
    Ok(Database { shared })
}

pub fn list_databases(project_root: &Path) -> StoreResult<Vec<DatabaseMetadata>> {
    let root = db_root(project_root);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut databases = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() || entry.file_name() == "_auth" {
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
        &self.shared.metadata
    }

    pub fn insert(&self, table: &str, record: StoreRecord) -> StoreResult<StoreRecord> {
        let record = prepare_insert(record)?;
        let base_version = self.current_version()?;
        let mut inserted = self.commit_inserts(base_version, &[(table.to_string(), record)])?;
        inserted.pop().ok_or_else(|| {
            StoreError::DurabilityError("Database insert produced no record".to_string())
        })
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
        validate_patch(&patch)?;
        storage::ensure_table(&self.shared.root, table)?;
        let mut state = self.write_state()?;
        ensure_available(&state)?;
        let current = state.current_records(table, state.version);
        let mut operations = Vec::new();
        for (id, mut record) in current {
            if record
                .get(field)
                .is_some_and(|value| values_equal(value, expected))
            {
                for (key, value) in &patch {
                    record.insert(key.clone(), value.clone());
                }
                operations.push(WalOperation::Upsert {
                    table: table.to_string(),
                    id,
                    record,
                });
            }
        }
        let changed = operations.len();
        self.commit_operations(&mut state, operations)?;
        Ok(changed)
    }

    pub fn delete(&self, table: &str, field: &str, expected: &StoreValue) -> StoreResult<usize> {
        validate_table_name(table)?;
        validate_field_name(field)?;
        storage::ensure_table(&self.shared.root, table)?;
        let mut state = self.write_state()?;
        ensure_available(&state)?;
        let operations = state
            .current_records(table, state.version)
            .into_iter()
            .filter(|(_, record)| {
                record
                    .get(field)
                    .is_some_and(|value| values_equal(value, expected))
            })
            .map(|(id, _)| WalOperation::Delete {
                table: table.to_string(),
                id,
            })
            .collect::<Vec<_>>();
        let changed = operations.len();
        self.commit_operations(&mut state, operations)?;
        Ok(changed)
    }

    pub fn records(&self, table: &str) -> StoreResult<Vec<StoreRecord>> {
        let version = self.current_version()?;
        self.records_at(table, version)
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
        let mut state = self.write_state()?;
        ensure_available(&state)?;
        state.tables.entry(table.to_string()).or_default();
        let records = state.current_records(table, state.version);
        let path = storage::create_index(&self.shared.root, table, field, &records)?;
        state.create_index(table, field);
        Ok(IndexInfo {
            table: table.to_string(),
            field: field.to_string(),
            path,
        })
    }

    pub fn has_index(&self, table: &str, field: &str) -> bool {
        self.shared
            .state
            .read()
            .ok()
            .is_some_and(|state| state.has_index(table, field))
    }

    pub fn inspect(&self) -> StoreResult<DatabaseInspection> {
        let state = self.read_state()?;
        ensure_available(&state)?;
        let mut tables = Vec::new();
        for table in state.tables.keys() {
            tables.push(TableInspection {
                name: table.clone(),
                indexes: storage::index_fields(&self.shared.root, table)?,
                records: state.current_records(table, state.version).len(),
            });
        }
        Ok(DatabaseInspection {
            database_id: self.shared.metadata.database_id.clone(),
            name: self.shared.name.clone(),
            format_version: self.shared.metadata.format_version,
            version: state.version,
            tables,
        })
    }

    pub fn compact(&self) -> StoreResult<CompactReport> {
        let mut state = self.write_state()?;
        ensure_available(&state)?;
        let mut report = CompactReport {
            database: self.shared.name.clone(),
            tables: 0,
            records: 0,
        };
        for table in state.tables.keys() {
            let records = state.current_records(table, state.version);
            storage::rewrite_table(&self.shared.root, table, &records)?;
            storage::rewrite_indexes(&self.shared.root, table, &records)?;
            report.tables += 1;
            report.records += records.len();
        }
        let mut file = self
            .shared
            .wal
            .lock()
            .map_err(|_| StoreError::DurabilityError("Database WAL lock failed".to_string()))?;
        if let Err(error) = wal::reset(&mut file) {
            state.poisoned = true;
            return Err(error);
        }
        state.checkpoint();
        Ok(report)
    }

    pub fn transaction(&self) -> Transaction {
        Transaction::new(self.clone())
    }

    pub(crate) fn current_version(&self) -> StoreResult<u64> {
        let state = self.read_state()?;
        ensure_available(&state)?;
        Ok(state.version)
    }

    pub(crate) fn records_at(&self, table: &str, version: u64) -> StoreResult<Vec<StoreRecord>> {
        validate_table_name(table)?;
        let state = self.read_state()?;
        ensure_available(&state)?;
        if version > state.version {
            return Err(StoreError::TransactionConflict(
                "transaction snapshot is newer than Database state".to_string(),
            ));
        }
        Ok(state
            .current_records(table, version)
            .into_values()
            .collect())
    }

    pub(crate) fn indexed_records_at(
        &self,
        table: &str,
        field: &str,
        expected: &StoreValue,
        version: u64,
    ) -> StoreResult<Option<Vec<StoreRecord>>> {
        validate_table_name(table)?;
        validate_field_name(field)?;
        let state = self.read_state()?;
        ensure_available(&state)?;
        if version > state.version {
            return Err(StoreError::TransactionConflict(
                "transaction snapshot is newer than Database state".to_string(),
            ));
        }
        Ok(state.indexed_records(table, field, expected, version))
    }

    fn read_state(&self) -> StoreResult<std::sync::RwLockReadGuard<'_, DatabaseState>> {
        self.shared
            .state
            .read()
            .map_err(|_| StoreError::DurabilityError("Database read lock failed".to_string()))
    }

    pub(crate) fn write_state(
        &self,
    ) -> StoreResult<std::sync::RwLockWriteGuard<'_, DatabaseState>> {
        self.shared
            .state
            .write()
            .map_err(|_| StoreError::DurabilityError("Database write lock failed".to_string()))
    }
}

fn acquire_database_lock(path: &Path) -> StoreResult<File> {
    acquire_file_lock(&path.join(".lock"), "Database namespace is already in use")
}

fn acquire_file_lock(path: &Path, conflict: &str) -> StoreResult<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    secure_file(&file)?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(StoreError::TransactionConflict(conflict.to_string())),
        Err(TryLockError::Error(_)) => Err(StoreError::DurabilityError(
            "Database namespace lock cannot be acquired".to_string(),
        )),
    }
}
