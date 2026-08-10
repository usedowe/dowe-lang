use crate::engine::StoreRecord;
use crate::error::{StoreError, StoreResult};
use crate::names::validate_field_name;
use crate::storage;
use crate::value::StoreValue;
use crate::wal::{WalOperation, WalTransaction};
use dowe_id::{generate_ulid, validate_ulid};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Default)]
pub(crate) struct DatabaseState {
    pub(crate) version: u64,
    pub(crate) poisoned: bool,
    pub(crate) tables: BTreeMap<String, TableState>,
}

#[derive(Debug, Default)]
pub(crate) struct TableState {
    pub(crate) records: BTreeMap<String, Vec<RecordVersion>>,
    indexes: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct RecordVersion {
    version: u64,
    value: Option<StoreRecord>,
}

impl DatabaseState {
    pub(crate) fn current_records(
        &self,
        table: &str,
        version: u64,
    ) -> BTreeMap<String, StoreRecord> {
        self.tables
            .get(table)
            .map(|table| {
                table
                    .records
                    .iter()
                    .filter_map(|(id, history)| {
                        value_at(Some(history), version)
                            .cloned()
                            .map(|record| (id.clone(), record))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn checkpoint(&mut self) {
        let version = self.version;
        for table in self.tables.values_mut() {
            table.records.retain(|_, history| {
                let value = value_at(Some(history), version).cloned();
                history.clear();
                if let Some(value) = value {
                    history.push(RecordVersion {
                        version: 0,
                        value: Some(value),
                    });
                    true
                } else {
                    false
                }
            });
        }
        self.version = 0;
    }

    pub(crate) fn create_index(&mut self, table: &str, field: &str) {
        let records = self.current_records(table, self.version);
        let mut values = BTreeMap::<String, BTreeSet<String>>::new();
        for (id, record) in records {
            if let Some(value) = record.get(field) {
                values
                    .entry(value.comparable_text())
                    .or_default()
                    .insert(id);
            }
        }
        self.tables
            .entry(table.to_string())
            .or_default()
            .indexes
            .insert(field.to_string(), values);
    }

    pub(crate) fn has_index(&self, table: &str, field: &str) -> bool {
        field == "id"
            || self
                .tables
                .get(table)
                .is_some_and(|table| table.indexes.contains_key(field))
    }

    pub(crate) fn indexed_records(
        &self,
        table: &str,
        field: &str,
        expected: &StoreValue,
        version: u64,
    ) -> Option<Vec<StoreRecord>> {
        if version != self.version {
            return None;
        }
        let table = self.tables.get(table)?;
        let ids = if field == "id" {
            let id = expected.comparable_text();
            if table.records.contains_key(&id) {
                let mut ids = BTreeSet::new();
                ids.insert(id);
                ids
            } else {
                BTreeSet::new()
            }
        } else {
            table
                .indexes
                .get(field)?
                .get(&expected.comparable_text())
                .cloned()
                .unwrap_or_default()
        };
        Some(
            ids.iter()
                .filter_map(|id| {
                    value_at(table.records.get(id).map(Vec::as_slice), version).cloned()
                })
                .collect(),
        )
    }
}

pub(crate) fn legacy_state(root: &Path) -> StoreResult<DatabaseState> {
    let mut state = DatabaseState::default();
    for (table, records) in storage::load_tables(root)? {
        let table_state = state.tables.entry(table.clone()).or_default();
        for (id, record) in records {
            table_state.records.insert(
                id,
                vec![RecordVersion {
                    version: 0,
                    value: Some(record),
                }],
            );
        }
        for field in storage::index_fields(root, &table)? {
            state.create_index(&table, &field);
        }
    }
    Ok(state)
}

pub(crate) fn apply_recovered_transaction(
    state: &mut DatabaseState,
    transaction: &WalTransaction,
) -> StoreResult<()> {
    if transaction.version
        != state
            .version
            .checked_add(1)
            .ok_or_else(|| StoreError::Corruption("Database WAL version overflowed".to_string()))?
        || transaction.base_version > state.version
    {
        return Err(StoreError::Corruption(
            "Database WAL transaction versions are not contiguous".to_string(),
        ));
    }
    apply_transaction(state, transaction);
    Ok(())
}

pub(crate) fn apply_transaction(state: &mut DatabaseState, transaction: &WalTransaction) {
    let current_version = state.version;
    for operation in &transaction.operations {
        match operation {
            WalOperation::Upsert { table, id, record } => {
                let table = state.tables.entry(table.clone()).or_default();
                let previous = table
                    .records
                    .get(id)
                    .and_then(|history| value_at(Some(history), current_version))
                    .cloned();
                update_indexes(table, id, previous.as_ref(), Some(record));
                table
                    .records
                    .entry(id.clone())
                    .or_default()
                    .push(RecordVersion {
                        version: transaction.version,
                        value: Some(record.clone()),
                    });
            }
            WalOperation::Delete { table, id } => {
                let table = state.tables.entry(table.clone()).or_default();
                let previous = table
                    .records
                    .get(id)
                    .and_then(|history| value_at(Some(history), current_version))
                    .cloned();
                update_indexes(table, id, previous.as_ref(), None);
                table
                    .records
                    .entry(id.clone())
                    .or_default()
                    .push(RecordVersion {
                        version: transaction.version,
                        value: None,
                    });
            }
        }
    }
    state.version = transaction.version;
}

fn update_indexes(
    table: &mut TableState,
    id: &str,
    previous: Option<&StoreRecord>,
    next: Option<&StoreRecord>,
) {
    for (field, values) in &mut table.indexes {
        if let Some(value) = previous.and_then(|record| record.get(field)) {
            let key = value.comparable_text();
            let remove_key = if let Some(ids) = values.get_mut(&key) {
                ids.remove(id);
                ids.is_empty()
            } else {
                false
            };
            if remove_key {
                values.remove(&key);
            }
        }
        if let Some(value) = next.and_then(|record| record.get(field)) {
            values
                .entry(value.comparable_text())
                .or_default()
                .insert(id.to_string());
        }
    }
}

pub(crate) fn value_at(history: Option<&[RecordVersion]>, version: u64) -> Option<&StoreRecord> {
    history?
        .iter()
        .rev()
        .find(|record| record.version <= version)
        .and_then(|record| record.value.as_ref())
}

pub(crate) fn latest_version(history: Option<&[RecordVersion]>) -> u64 {
    history
        .and_then(|history| history.last())
        .map(|record| record.version)
        .unwrap_or(0)
}

pub(crate) fn record_id(record: &StoreRecord) -> StoreResult<&str> {
    match record.get("id") {
        Some(StoreValue::Ulid(value)) => Ok(value),
        _ => Err(StoreError::InvalidUlid(
            "record id must be a ULID".to_string(),
        )),
    }
}

pub(crate) fn ensure_available(state: &DatabaseState) -> StoreResult<()> {
    if state.poisoned {
        return Err(StoreError::DurabilityError(
            "Database requires restart after an ambiguous WAL failure".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn values_equal(left: &StoreValue, right: &StoreValue) -> bool {
    left.comparable_text() == right.comparable_text()
}

pub(crate) fn prepare_insert(mut record: StoreRecord) -> StoreResult<StoreRecord> {
    for field in record.keys() {
        validate_field_name(field)?;
    }
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
    record.insert("id".to_string(), StoreValue::Ulid(id));
    Ok(record)
}

pub(crate) fn validate_patch(patch: &StoreRecord) -> StoreResult<()> {
    for field in patch.keys() {
        validate_field_name(field)?;
        if field == "id" {
            return Err(StoreError::TypeError(
                "record id cannot be updated".to_string(),
            ));
        }
    }
    Ok(())
}
