use crate::error::{QueueError, QueueResult};
use crate::model::QueueMessage;
use dowe_id::generate_ulid;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FORMAT_VERSION: u8 = 1;

#[derive(Serialize, Deserialize)]
pub(crate) struct QueueState {
    pub(crate) version: u8,
    pub(crate) name: String,
    pub(crate) queues: BTreeMap<String, PersistedQueue>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PersistedQueue {
    pub(crate) bindings: BTreeSet<String>,
    pub(crate) ready: VecDeque<QueueMessage>,
    pub(crate) in_flight: BTreeMap<String, InFlight>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct InFlight {
    pub(crate) session: String,
    pub(crate) message: QueueMessage,
}

pub fn queue_root(project_root: &Path) -> PathBuf {
    project_root.join(".dowe").join("queue")
}

pub(crate) fn read_state(path: &Path, name: &str) -> QueueResult<QueueState> {
    let state_path = path.join("state.json");
    if !state_path.exists() {
        return Ok(QueueState {
            version: FORMAT_VERSION,
            name: name.to_string(),
            queues: BTreeMap::new(),
        });
    }
    let bytes = fs::read(state_path)?;
    let state = serde_json::from_slice::<QueueState>(&bytes)
        .map_err(|_| QueueError::Corruption("Queue state cannot be read".to_string()))?;
    if state.version != FORMAT_VERSION || state.name != name {
        return Err(QueueError::Corruption(
            "Queue state format is incompatible".to_string(),
        ));
    }
    Ok(state)
}

pub(crate) fn requeue_recovered(state: &mut QueueState) -> bool {
    let mut recovered = false;
    for queue in state.queues.values_mut() {
        if queue.in_flight.is_empty() {
            continue;
        }
        let messages = std::mem::take(&mut queue.in_flight)
            .into_values()
            .map(|mut delivery| {
                delivery.message.redelivered = true;
                delivery.message
            })
            .collect::<Vec<_>>();
        for message in messages.into_iter().rev() {
            queue.ready.push_front(message);
        }
        recovered = true;
    }
    recovered
}

pub(crate) fn persist(path: &Path, state: &QueueState) -> QueueResult<()> {
    let bytes = serde_json::to_vec(state)
        .map_err(|_| QueueError::DurabilityError("Queue state cannot be encoded".to_string()))?;
    let target = path.join("state.json");
    let temporary = path.join(format!(".state-{}.tmp", generate_ulid()));
    let mut file = File::create(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all().map_err(|_| {
        QueueError::DurabilityError("Queue state cannot be synchronized".to_string())
    })?;
    fs::rename(&temporary, &target)?;
    Ok(())
}

pub(crate) fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}
