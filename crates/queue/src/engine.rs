use crate::error::{QueueError, QueueResult};
use crate::model::{
    BindReport, DeclareReport, DeliveryReceipt, DirectPublishReport, PublishReport, PurgeReport,
    QueueDelivery, QueueInspection, QueueInspectionEntry, QueueMessage, delivery,
};
use crate::names::{validate_consumer_name, validate_namespace, validate_queue_name};
use crate::protocol::QueueDeliveryFrame;
use crate::storage::{
    InFlight, PersistedQueue, QueueState, persist, read_state, requeue_recovered, timestamp,
};
use crate::topic::{topic_matches, validate_pattern, validate_topic};
use dowe_id::generate_ulid;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use tokio::sync::Notify;

static ENGINES: OnceLock<Mutex<HashMap<PathBuf, Weak<SharedEngine>>>> = OnceLock::new();

#[derive(Clone)]
pub struct DoweQueue {
    shared: Arc<SharedEngine>,
}

pub struct DoweSubscription {
    engine: DoweQueue,
    queue: String,
    session: String,
    closed: bool,
}

struct SharedEngine {
    path: PathBuf,
    _lock: File,
    state: Mutex<QueueState>,
    notify: Notify,
}

struct LocalDelivery {
    message: QueueMessage,
    receipt: String,
}

struct LocalReceipt {
    engine: DoweQueue,
    queue: String,
    session: String,
    receipt: String,
    resolved: bool,
}

impl DoweQueue {
    pub fn open(project_root: &Path, name: &str) -> QueueResult<Self> {
        open_namespace(project_root, name)
    }

    pub fn init(project_root: &Path, name: &str) -> QueueResult<()> {
        init_namespace(project_root, name)
    }

    pub fn name(&self) -> String {
        self.shared
            .state
            .lock()
            .map(|state| state.name.clone())
            .unwrap_or_default()
    }

    pub fn declare(&self, queue: &str) -> QueueResult<DeclareReport> {
        validate_queue_name(queue)?;
        let mut state = self.lock_state()?;
        let created = if state.queues.contains_key(queue) {
            false
        } else {
            state.queues.insert(
                queue.to_string(),
                PersistedQueue {
                    bindings: BTreeSet::new(),
                    ready: VecDeque::new(),
                    in_flight: BTreeMap::new(),
                },
            );
            persist(&self.shared.path, &state)?;
            true
        };
        Ok(DeclareReport {
            queue: queue.to_string(),
            created: Some(created),
        })
    }

    pub fn bind(&self, queue: &str, pattern: &str) -> QueueResult<BindReport> {
        validate_queue_name(queue)?;
        validate_pattern(pattern)?;
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let created = queue_state.bindings.insert(pattern.to_string());
        if created {
            persist(&self.shared.path, &state)?;
        }
        Ok(BindReport {
            queue: queue.to_string(),
            pattern: pattern.to_string(),
            created: Some(created),
        })
    }

    pub fn publish(&self, topic: &str, value: Value) -> QueueResult<PublishReport> {
        validate_topic(topic)?;
        let message = QueueMessage {
            id: generate_ulid(),
            topic: topic.to_string(),
            value,
            published_at: timestamp(),
            redelivered: false,
        };
        let mut state = self.lock_state()?;
        let mut destinations = Vec::new();
        for (queue, queue_state) in &mut state.queues {
            if queue_state
                .bindings
                .iter()
                .any(|pattern| topic_matches(pattern, topic))
            {
                queue_state.ready.push_back(message.clone());
                destinations.push(queue.clone());
            }
        }
        persist(&self.shared.path, &state)?;
        drop(state);
        if !destinations.is_empty() {
            self.shared.notify.notify_waiters();
        }
        Ok(PublishReport {
            id: message.id,
            destinations: Some(destinations),
            confirmed: true,
        })
    }

    pub fn publish_direct(&self, queue: &str, value: Value) -> QueueResult<DirectPublishReport> {
        validate_queue_name(queue)?;
        let message = QueueMessage {
            id: generate_ulid(),
            topic: queue.to_string(),
            value,
            published_at: timestamp(),
            redelivered: false,
        };
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        queue_state.ready.push_back(message.clone());
        persist(&self.shared.path, &state)?;
        drop(state);
        self.shared.notify.notify_waiters();
        Ok(DirectPublishReport {
            id: message.id,
            confirmed: true,
        })
    }

    pub fn inspect(&self) -> QueueResult<QueueInspection> {
        let state = self.lock_state()?;
        let queues = state
            .queues
            .iter()
            .map(|(name, queue)| QueueInspectionEntry {
                queue: name.clone(),
                bindings: queue.bindings.iter().cloned().collect(),
                ready: queue.ready.len(),
                in_flight: queue.in_flight.len(),
            })
            .collect();
        Ok(QueueInspection {
            name: state.name.clone(),
            queues: Some(queues),
        })
    }

    pub fn inspect_messages(&self, queue: &str, limit: usize) -> QueueResult<Vec<QueueMessage>> {
        validate_queue_name(queue)?;
        let state = self.lock_state()?;
        let queue_state = state
            .queues
            .get(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let limit = limit.clamp(1, 100);
        let mut messages = queue_state
            .ready
            .iter()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        if messages.len() < limit {
            messages.extend(
                queue_state
                    .in_flight
                    .values()
                    .take(limit.saturating_sub(messages.len()))
                    .map(|delivery| delivery.message.clone()),
            );
        }
        Ok(messages)
    }

    pub fn purge(&self, queue: &str) -> QueueResult<PurgeReport> {
        validate_queue_name(queue)?;
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let removed = queue_state.ready.len();
        queue_state.ready.clear();
        persist(&self.shared.path, &state)?;
        Ok(PurgeReport {
            queue: queue.to_string(),
            removed,
        })
    }

    pub fn subscribe(&self, queue: &str, consumer: &str) -> QueueResult<DoweSubscription> {
        validate_queue_name(queue)?;
        validate_consumer_name(consumer)?;
        let state = self.lock_state()?;
        if !state.queues.contains_key(queue) {
            return Err(QueueError::QueueNotFound(
                "Queue does not exist".to_string(),
            ));
        }
        drop(state);
        Ok(DoweSubscription {
            engine: self.clone(),
            queue: queue.to_string(),
            session: generate_ulid(),
            closed: false,
        })
    }

    fn take(&self, queue: &str, session: &str) -> QueueResult<Option<LocalDelivery>> {
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let Some(message) = queue_state.ready.pop_front() else {
            return Ok(None);
        };
        let receipt = generate_ulid();
        queue_state.in_flight.insert(
            receipt.clone(),
            InFlight {
                session: session.to_string(),
                message: message.clone(),
            },
        );
        persist(&self.shared.path, &state)?;
        Ok(Some(LocalDelivery { message, receipt }))
    }

    fn settle(&self, queue: &str, session: &str, receipt: &str, requeue: bool) -> QueueResult<()> {
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let in_flight = queue_state.in_flight.remove(receipt).ok_or_else(|| {
            QueueError::InvalidReceipt("Queue delivery receipt is invalid or expired".to_string())
        })?;
        if in_flight.session != session {
            queue_state.in_flight.insert(receipt.to_string(), in_flight);
            return Err(QueueError::InvalidReceipt(
                "Queue delivery receipt does not belong to this subscription".to_string(),
            ));
        }
        if requeue {
            let mut message = in_flight.message;
            message.redelivered = true;
            queue_state.ready.push_front(message);
        }
        persist(&self.shared.path, &state)?;
        drop(state);
        if requeue {
            self.shared.notify.notify_waiters();
        }
        Ok(())
    }

    fn requeue_session(&self, queue: &str, session: &str) -> QueueResult<()> {
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let receipts = queue_state
            .in_flight
            .iter()
            .filter(|(_, delivery)| delivery.session == session)
            .map(|(receipt, _)| receipt.clone())
            .collect::<Vec<_>>();
        if receipts.is_empty() {
            return Ok(());
        }
        let mut messages = Vec::with_capacity(receipts.len());
        for receipt in receipts {
            if let Some(mut in_flight) = queue_state.in_flight.remove(&receipt) {
                in_flight.message.redelivered = true;
                messages.push(in_flight.message);
            }
        }
        for message in messages.into_iter().rev() {
            queue_state.ready.push_front(message);
        }
        persist(&self.shared.path, &state)?;
        drop(state);
        self.shared.notify.notify_waiters();
        Ok(())
    }

    fn lock_state(&self) -> QueueResult<std::sync::MutexGuard<'_, QueueState>> {
        self.shared
            .state
            .lock()
            .map_err(|_| QueueError::DurabilityError("Queue engine lock failed".to_string()))
    }
}

impl DoweSubscription {
    pub async fn next(&mut self) -> QueueResult<Option<QueueDelivery>> {
        let Some(local_delivery) = self.next_raw().await? else {
            return Ok(None);
        };
        Ok(Some(delivery(
            local_delivery.message,
            LocalReceipt {
                engine: self.engine.clone(),
                queue: self.queue.clone(),
                session: self.session.clone(),
                receipt: local_delivery.receipt,
                resolved: false,
            },
        )))
    }

    pub async fn close(&mut self) -> QueueResult<()> {
        self.close_sync()
    }

    pub(crate) async fn next_frame(&mut self) -> QueueResult<Option<QueueDeliveryFrame>> {
        Ok(self.next_raw().await?.map(|delivery| QueueDeliveryFrame {
            message: delivery.message,
            receipt: delivery.receipt,
        }))
    }

    pub(crate) fn ack_token(&self, receipt: &str) -> QueueResult<()> {
        self.engine
            .settle(&self.queue, &self.session, receipt, false)
    }

    pub(crate) fn nack_token(&self, receipt: &str, requeue: bool) -> QueueResult<()> {
        self.engine
            .settle(&self.queue, &self.session, receipt, requeue)
    }

    async fn next_raw(&mut self) -> QueueResult<Option<LocalDelivery>> {
        if self.closed {
            return Ok(None);
        }
        loop {
            let notified = self.engine.shared.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if let Some(delivery) = self.engine.take(&self.queue, &self.session)? {
                return Ok(Some(delivery));
            }
            notified.await;
            if self.closed {
                return Ok(None);
            }
        }
    }

    fn close_sync(&mut self) -> QueueResult<()> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        self.engine.requeue_session(&self.queue, &self.session)
    }
}

impl Drop for DoweSubscription {
    fn drop(&mut self) {
        let _ = self.close_sync();
    }
}

impl LocalReceipt {
    fn settle(&mut self, requeue: bool) -> QueueResult<()> {
        if self.resolved {
            return Err(QueueError::InvalidReceipt(
                "Queue delivery receipt is already resolved".to_string(),
            ));
        }
        self.resolved = true;
        self.engine
            .settle(&self.queue, &self.session, &self.receipt, requeue)
    }
}

impl DeliveryReceipt for LocalReceipt {
    fn ack<'a>(
        &'a mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = QueueResult<()>> + Send + 'a>> {
        Box::pin(async move { self.settle(false) })
    }

    fn nack<'a>(
        &'a mut self,
        requeue: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = QueueResult<()>> + Send + 'a>> {
        Box::pin(async move { self.settle(requeue) })
    }
}

pub use crate::storage::queue_root;

pub fn init_namespace(project_root: &Path, name: &str) -> QueueResult<()> {
    open_namespace(project_root, name).map(|_| ())
}

pub fn open_namespace(project_root: &Path, name: &str) -> QueueResult<DoweQueue> {
    validate_namespace(name)?;
    let path = queue_root(project_root).join(name);
    fs::create_dir_all(&path)?;
    let registry = ENGINES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut registry = registry
        .lock()
        .map_err(|_| QueueError::DurabilityError("Queue registry lock failed".to_string()))?;
    if let Some(shared) = registry.get(&path).and_then(Weak::upgrade) {
        return Ok(DoweQueue { shared });
    }
    let lock = acquire_namespace_lock(&path)?;
    let mut state = read_state(&path, name)?;
    if requeue_recovered(&mut state) {
        persist(&path, &state)?;
    } else if !path.join("state.json").exists() {
        persist(&path, &state)?;
    }
    let shared = Arc::new(SharedEngine {
        path: path.clone(),
        _lock: lock,
        state: Mutex::new(state),
        notify: Notify::new(),
    });
    registry.insert(path, Arc::downgrade(&shared));
    Ok(DoweQueue { shared })
}

fn acquire_namespace_lock(path: &Path) -> QueueResult<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path.join(".lock"))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(QueueError::DurabilityError(
            "Queue namespace is already in use".to_string(),
        )),
        Err(TryLockError::Error(_)) => Err(QueueError::DurabilityError(
            "Queue namespace lock cannot be acquired".to_string(),
        )),
    }
}

pub fn list_namespaces(project_root: &Path) -> QueueResult<Vec<String>> {
    let root = queue_root(project_root);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() || entry.file_name() == "_auth" {
            continue;
        }
        if !entry.path().join("state.json").exists() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        validate_namespace(&name)?;
        names.push(name);
    }
    names.sort();
    Ok(names)
}
