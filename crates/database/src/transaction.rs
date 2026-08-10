use crate::engine::{Database, StoreRecord};
use crate::error::{StoreError, StoreResult};
use crate::state::prepare_insert;

pub struct Transaction {
    database: Database,
    operations: Vec<PendingOperation>,
    base_version: StoreResult<u64>,
    finished: bool,
}

#[derive(Clone)]
enum PendingOperation {
    Insert { table: String, record: StoreRecord },
}

impl Transaction {
    pub fn new(database: Database) -> Self {
        let base_version = database.current_version();
        Self {
            database,
            operations: Vec::new(),
            base_version,
            finished: false,
        }
    }

    pub fn insert(&mut self, table: &str, record: StoreRecord) -> StoreResult<StoreRecord> {
        if self.finished {
            return Err(StoreError::TransactionConflict(
                "transaction is already finished".to_string(),
            ));
        }
        let staged = prepare_insert(record)?;
        self.operations.push(PendingOperation::Insert {
            table: table.to_string(),
            record: staged.clone(),
        });
        Ok(staged)
    }

    pub fn commit(mut self) -> StoreResult<Vec<StoreRecord>> {
        let base_version = self.base_version.clone()?;
        let inserts = self
            .operations
            .iter()
            .map(|operation| match operation {
                PendingOperation::Insert { table, record } => (table.clone(), record.clone()),
            })
            .collect::<Vec<_>>();
        let inserted = self.database.commit_inserts(base_version, &inserts)?;
        self.finished = true;
        Ok(inserted)
    }

    pub fn records(&self, table: &str) -> StoreResult<Vec<StoreRecord>> {
        if self.finished {
            return Err(StoreError::TransactionConflict(
                "transaction is already finished".to_string(),
            ));
        }
        let mut records = self
            .database
            .records_at(table, self.base_version.clone()?)?;
        records.extend(
            self.operations
                .iter()
                .filter_map(|operation| match operation {
                    PendingOperation::Insert {
                        table: operation_table,
                        record,
                    } if operation_table == table => Some(record.clone()),
                    _ => None,
                }),
        );
        Ok(records)
    }

    pub fn rollback(mut self) {
        self.operations.clear();
        self.finished = true;
    }
}
