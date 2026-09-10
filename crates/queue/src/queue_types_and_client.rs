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

