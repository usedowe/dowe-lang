use crate::engine::{Database, StoreRecord};
use crate::error::{StoreError, StoreResult};
use crate::value::StoreValue;
use dowe_id::{generate_ulid, validate_ulid};

pub struct Transaction {
    database: Database,
    operations: Vec<PendingOperation>,
    finished: bool,
}

#[derive(Clone)]
enum PendingOperation {
    Insert { table: String, record: StoreRecord },
}

impl Transaction {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            operations: Vec::new(),
            finished: false,
        }
    }

    pub fn insert(&mut self, table: &str, record: StoreRecord) -> StoreResult<StoreRecord> {
        if self.finished {
            return Err(StoreError::TransactionConflict(
                "transaction is already finished".to_string(),
            ));
        }
        let mut staged = record;
        let id = match staged.get("id") {
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
        staged.insert("id".to_string(), StoreValue::Ulid(id));
        self.operations.push(PendingOperation::Insert {
            table: table.to_string(),
            record: staged.clone(),
        });
        Ok(staged)
    }

    pub fn commit(mut self) -> StoreResult<Vec<StoreRecord>> {
        let mut inserted = Vec::new();
        for operation in &self.operations {
            match operation {
                PendingOperation::Insert { table, record } => {
                    inserted.push(self.database.insert(table, record.clone())?);
                }
            }
        }
        self.finished = true;
        Ok(inserted)
    }

    pub fn rollback(mut self) {
        self.operations.clear();
        self.finished = true;
    }
}
