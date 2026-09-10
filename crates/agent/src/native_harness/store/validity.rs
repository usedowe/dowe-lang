use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryValidity {
    pub catalog: Option<String>,
    pub source_summary_hash: Option<String>,
    pub invalidated: Option<String>,
}

pub(super) struct MemoryReview<'a> {
    store: &'a HarnessStore,
    catalog: String,
    summaries: BTreeMap<String, Option<String>>,
    files: BTreeMap<String, Option<String>>,
}
impl<'a> MemoryReview<'a> {
    pub(super) fn new(store: &'a HarnessStore) -> AgentResult<Self> {
        Ok(Self {
            store,
            catalog: super::super::catalog::catalog_fingerprint()?,
            summaries: BTreeMap::new(),
            files: BTreeMap::new(),
        })
    }
    pub(super) fn reason(&mut self, observation: &MemoryObservation) -> Option<&'static str> {
        if observation.validity.invalidated.is_some() {
            return Some("invalidated");
        }
        if observation
            .validity
            .catalog
            .as_ref()
            .is_some_and(|catalog| catalog != &self.catalog)
        {
            return Some("catalog_changed");
        }
        if let Some(id) = observation.source.strip_prefix("session:") {
            let Some(expected) = observation.validity.source_summary_hash.as_ref() else {
                return Some("source_proof_missing");
            };
            let current = self.summaries.entry(id.into()).or_insert_with(|| {
                self.store
                    .read_session(id, false)
                    .ok()?
                    .summary
                    .map(|summary| digest(summary.as_bytes()))
            });
            match current {
                None => return Some("source_unavailable"),
                Some(actual) if actual != expected => return Some("source_summary_changed"),
                _ => {}
            }
        } else if observation.validity.source_summary_hash.is_some() {
            return Some("source_proof_missing");
        }
        for (path, expected) in &observation.files {
            let current = self
                .files
                .entry(path.clone())
                .or_insert_with(|| self.store.file_fingerprint(path).ok());
            if current.as_ref() != Some(expected) {
                return Some("file_changed_or_unavailable");
            }
        }
        None
    }
}

impl HarnessStore {
    pub fn memory_status(&self) -> AgentResult<Vec<Value>> {
        let mut review = MemoryReview::new(self)?;
        Ok(self.observations()?.iter().map(|observation| {
            let reason = review.reason(observation);
            serde_json::json!({"id":observation.id,"confirmed":observation.confirmed,"eligible":observation.confirmed && reason.is_none(),"reason":reason.unwrap_or(if observation.confirmed {"current"} else {"candidate"}),"invalidated":observation.validity.invalidated})
        }).collect())
    }
    pub fn invalidate_memory(&self, id: &str, reason: &str) -> AgentResult<()> {
        if reason.trim().is_empty() || reason.len() > 512 {
            return Err(AgentError::new(
                "memory invalidation requires a reason of 1..512 bytes",
            ));
        }
        let path = self.memory_path()?;
        reject_symlink_ancestors(&path)?;
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let mut memories = self.memories()?;
        let observation = memories
            .observations
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| AgentError::new("memory id not found"))?;
        observation.validity.invalidated =
            Some(Redactor::for_project(&self.root).text(reason.trim()));
        observation.updated = now();
        write_private_json(&path, &memories)
    }
}

#[cfg(test)]
#[path = "validity_tests.rs"]
mod tests;
