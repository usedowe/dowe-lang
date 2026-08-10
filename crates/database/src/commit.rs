use crate::engine::{Database, StoreRecord};
use crate::error::{StoreError, StoreResult};
use crate::names::validate_table_name;
use crate::state::{
    apply_transaction, ensure_available, latest_version, prepare_insert, record_id, value_at,
};
use crate::storage;
use crate::wal::{self, WalOperation, WalTransaction};
use dowe_id::generate_ulid;
use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

const GROUP_WINDOW: Duration = Duration::from_micros(750);
const MAX_GROUP_SIZE: usize = 256;

type PreparedInsert = (String, String, StoreRecord);

#[derive(Debug, Default)]
pub(crate) struct CommitGroup {
    queue: Mutex<CommitQueue>,
}

#[derive(Debug, Default)]
struct CommitQueue {
    active: bool,
    requests: VecDeque<Arc<CommitRequest>>,
}

#[derive(Debug)]
struct CommitRequest {
    base_version: u64,
    inserts: Vec<PreparedInsert>,
    result: Mutex<Option<StoreResult<Vec<StoreRecord>>>>,
    ready: Condvar,
}

impl Database {
    pub(crate) fn commit_inserts(
        &self,
        base_version: u64,
        inserts: &[(String, StoreRecord)],
    ) -> StoreResult<Vec<StoreRecord>> {
        if inserts.is_empty() {
            return Ok(Vec::new());
        }
        let mut prepared = Vec::with_capacity(inserts.len());
        let mut keys = BTreeSet::new();
        for (table, record) in inserts {
            validate_table_name(table)?;
            storage::ensure_table(&self.shared.root, table)?;
            let record = prepare_insert(record.clone())?;
            let id = record_id(&record)?.to_string();
            if !keys.insert((table.clone(), id.clone())) {
                return Err(StoreError::TransactionConflict(
                    "transaction inserts the same record more than once".to_string(),
                ));
            }
            prepared.push((table.clone(), id, record));
        }
        let request = Arc::new(CommitRequest {
            base_version,
            inserts: prepared,
            result: Mutex::new(None),
            ready: Condvar::new(),
        });
        let leader = self.enqueue_commit(request.clone())?;
        if leader {
            std::thread::sleep(GROUP_WINDOW);
            self.process_commit_queue()?;
        }
        request.wait()
    }

    pub(crate) fn commit_operations(
        &self,
        state: &mut crate::state::DatabaseState,
        operations: Vec<WalOperation>,
    ) -> StoreResult<()> {
        if operations.is_empty() {
            return Ok(());
        }
        let version = state.version.checked_add(1).ok_or_else(|| {
            StoreError::DurabilityError("Database version overflowed".to_string())
        })?;
        let transaction = WalTransaction {
            id: generate_ulid(),
            base_version: state.version,
            version,
            operations,
        };
        let mut file = self
            .shared
            .wal
            .lock()
            .map_err(|_| StoreError::DurabilityError("Database WAL lock failed".to_string()))?;
        if let Err(error) = wal::append(&mut file, &transaction) {
            state.poisoned = true;
            return Err(error);
        }
        apply_transaction(state, &transaction);
        Ok(())
    }

    fn enqueue_commit(&self, request: Arc<CommitRequest>) -> StoreResult<bool> {
        let mut queue = self.shared.commits.queue.lock().map_err(|_| {
            StoreError::DurabilityError("Database commit queue lock failed".to_string())
        })?;
        queue.requests.push_back(request);
        let leader = !queue.active;
        if leader {
            queue.active = true;
        }
        Ok(leader)
    }

    fn process_commit_queue(&self) -> StoreResult<()> {
        loop {
            let requests = {
                let mut queue = self.shared.commits.queue.lock().map_err(|_| {
                    StoreError::DurabilityError("Database commit queue lock failed".to_string())
                })?;
                let request_count = queue.requests.len().min(MAX_GROUP_SIZE);
                let requests = queue.requests.drain(..request_count).collect::<Vec<_>>();
                if requests.is_empty() {
                    queue.active = false;
                    return Ok(());
                }
                requests
            };
            self.process_commit_group(&requests);
        }
    }

    fn process_commit_group(&self, requests: &[Arc<CommitRequest>]) {
        let Ok(mut state) = self.write_state() else {
            complete_all(requests, lock_error());
            return;
        };
        if let Err(error) = ensure_available(&state) {
            complete_all(requests, error);
            return;
        }
        let mut pending = BTreeSet::new();
        let mut accepted = Vec::new();
        for request in requests {
            match validate_request(request, &state, &pending) {
                Ok(()) => {
                    for (table, id, _) in &request.inserts {
                        pending.insert((table.clone(), id.clone()));
                    }
                    let Some(version) = state.version.checked_add(accepted.len() as u64 + 1) else {
                        request.complete(Err(StoreError::DurabilityError(
                            "Database version overflowed".to_string(),
                        )));
                        continue;
                    };
                    accepted.push((
                        request.clone(),
                        WalTransaction {
                            id: generate_ulid(),
                            base_version: request.base_version,
                            version,
                            operations: request
                                .inserts
                                .iter()
                                .map(|(table, id, record)| WalOperation::Upsert {
                                    table: table.clone(),
                                    id: id.clone(),
                                    record: record.clone(),
                                })
                                .collect(),
                        },
                    ));
                }
                Err(error) => request.complete(Err(error)),
            }
        }
        if accepted.is_empty() {
            return;
        }
        let write_result = self.write_commit_group(&accepted);
        if let Err(error) = write_result {
            state.poisoned = true;
            complete_accepted(&accepted, error);
            return;
        }
        for (request, transaction) in accepted {
            apply_transaction(&mut state, &transaction);
            request.complete(Ok(request
                .inserts
                .iter()
                .map(|(_, _, record)| record.clone())
                .collect()));
        }
    }

    fn write_commit_group(
        &self,
        accepted: &[(Arc<CommitRequest>, WalTransaction)],
    ) -> StoreResult<()> {
        let mut file = self
            .shared
            .wal
            .lock()
            .map_err(|_| StoreError::DurabilityError("Database WAL lock failed".to_string()))?;
        for (_, transaction) in accepted {
            wal::append_unsynced(&mut file, transaction)?;
        }
        wal::sync(&mut file)
    }
}

impl CommitRequest {
    fn wait(&self) -> StoreResult<Vec<StoreRecord>> {
        let mut result = self.result.lock().map_err(|_| lock_error())?;
        while result.is_none() {
            result = self.ready.wait(result).map_err(|_| lock_error())?;
        }
        result.clone().expect("commit result")
    }

    fn complete(&self, value: StoreResult<Vec<StoreRecord>>) {
        if let Ok(mut result) = self.result.lock() {
            *result = Some(value);
            self.ready.notify_one();
        }
    }
}

fn validate_request(
    request: &CommitRequest,
    state: &crate::state::DatabaseState,
    pending: &BTreeSet<(String, String)>,
) -> StoreResult<()> {
    if request.base_version > state.version {
        return Err(StoreError::TransactionConflict(
            "transaction snapshot is invalid".to_string(),
        ));
    }
    for (table, id, _) in &request.inserts {
        if pending.contains(&(table.clone(), id.clone())) {
            return Err(StoreError::AlreadyExists(format!(
                "record `{id}` already exists"
            )));
        }
        let history = state
            .tables
            .get(table)
            .and_then(|table| table.records.get(id));
        if latest_version(history.map(Vec::as_slice)) > request.base_version {
            return Err(StoreError::TransactionConflict(
                "record changed after the transaction began".to_string(),
            ));
        }
        if value_at(history.map(Vec::as_slice), state.version).is_some() {
            return Err(StoreError::AlreadyExists(format!(
                "record `{id}` already exists"
            )));
        }
    }
    Ok(())
}

fn complete_all(requests: &[Arc<CommitRequest>], error: StoreError) {
    for request in requests {
        request.complete(Err(error.clone()));
    }
}

fn complete_accepted(accepted: &[(Arc<CommitRequest>, WalTransaction)], error: StoreError) {
    for (request, _) in accepted {
        request.complete(Err(error.clone()));
    }
}

fn lock_error() -> StoreError {
    StoreError::DurabilityError("Database transaction synchronization failed".to_string())
}
