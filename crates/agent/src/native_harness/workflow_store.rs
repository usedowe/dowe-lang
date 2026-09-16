use super::{HarnessStore, lock::DataLock};
use crate::{AgentError, AgentResult};
use dowe_agent_harness::coordinator::Coordinator;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCheckpoint {
    pub schema: u32,
    pub sequence: u64,
    pub project: String,
    pub coordinator: Coordinator,
    #[serde(default)]
    pub(crate) continuation: Option<super::workflow::WorkflowContinuation>,
}

impl WorkflowCheckpoint {
    pub fn can_resume(&self) -> bool {
        self.continuation
            .as_ref()
            .is_some_and(|state| state.resumable)
    }
}

impl HarnessStore {
    pub(super) fn lease_workflow(&self, id: &str) -> AgentResult<DataLock> {
        DataLock::acquire_nowait(&self.workflow_path(id)?.with_extension("active.lock"))
    }
    pub(super) fn workflow_path(&self, id: &str) -> AgentResult<PathBuf> {
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
        {
            return Err(AgentError::new(
                "workflow id must contain only letters, digits, hyphens or underscores",
            ));
        }
        let path = self.root().join(".agent/tasks").join(format!("{id}.json"));
        for ancestor in path.ancestors() {
            if fs::symlink_metadata(ancestor).is_ok_and(|m| m.file_type().is_symlink()) {
                return Err(AgentError::new("workflow paths must not traverse symlinks"));
            }
        }
        Ok(path)
    }

    pub fn load_workflow(&self, id: &str) -> AgentResult<WorkflowCheckpoint> {
        let path = self.workflow_path(id)?;
        let _lock = DataLock::acquire(&path.with_extension("lock"))?;
        self.read_workflow(id, &path)
    }

    fn read_workflow(&self, id: &str, path: &std::path::Path) -> AgentResult<WorkflowCheckpoint> {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_file() || metadata.len() > 2 * 1024 * 1024 {
            return Err(AgentError::new(
                "workflow checkpoint is not a bounded regular file",
            ));
        }
        let checkpoint: WorkflowCheckpoint = serde_json::from_slice(&fs::read(path)?)?;
        let project = super::digest(self.root().as_os_str().as_encoded_bytes());
        if checkpoint.schema != 1
            || checkpoint.coordinator.id() != id
            || checkpoint.project != project
        {
            return Err(AgentError::new("workflow schema or ownership mismatch"));
        }
        Ok(checkpoint)
    }

    pub fn save_workflow(
        &self,
        coordinator: &Coordinator,
        expected_sequence: Option<u64>,
    ) -> AgentResult<u64> {
        let _lease = self.lease_workflow(coordinator.id())?;
        self.save_workflow_state(coordinator, expected_sequence, None)
    }

    pub(super) fn save_workflow_state(
        &self,
        coordinator: &Coordinator,
        expected_sequence: Option<u64>,
        continuation: Option<super::workflow::WorkflowContinuation>,
    ) -> AgentResult<u64> {
        let path = self.workflow_path(coordinator.id())?;
        let _lock = DataLock::acquire(&path.with_extension("lock"))?;
        let previous = if path.exists() {
            Some(self.read_workflow(coordinator.id(), &path)?.sequence)
        } else {
            None
        };
        if previous != expected_sequence {
            return Err(AgentError::new("workflow changed; reload before saving"));
        }
        let sequence = previous
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| AgentError::new("workflow sequence exhausted"))?;
        let checkpoint = WorkflowCheckpoint {
            schema: 1,
            sequence,
            project: super::digest(self.root().as_os_str().as_encoded_bytes()),
            coordinator: coordinator.clone(),
            continuation,
        };
        if serde_json::to_vec(&checkpoint)?.len() > 2 * 1024 * 1024 {
            return Err(AgentError::new("workflow exceeds 2 MiB"));
        }
        crate::auth::write_private_json(&path, &checkpoint)?;
        Ok(sequence)
    }

    pub fn recover_workflow(
        &self,
        id: &str,
        expected_sequence: u64,
    ) -> AgentResult<WorkflowCheckpoint> {
        let _lease = self.lease_workflow(id)?;
        let mut checkpoint = self.load_workflow(id)?;
        if checkpoint.sequence != expected_sequence {
            return Err(AgentError::new("workflow changed before recovery"));
        }
        checkpoint.coordinator.recover();
        checkpoint.continuation = None;
        checkpoint.sequence =
            self.save_workflow_state(&checkpoint.coordinator, Some(expected_sequence), None)?;
        Ok(checkpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dowe_agent_harness::coordinator::{Intent, Phase, ScheduledTask};

    #[test]
    fn checkpoints_reject_stale_writers_and_recovery_revokes_approval() {
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let store = HarnessStore::new(home.path(), root.path()).unwrap();
        let mut coordinator = Coordinator::new("login", Intent::Build, "Build login").unwrap();
        coordinator
            .propose(
                vec![ScheduledTask::new("code", vec![], vec!["src".into()]).unwrap()],
                vec!["works".into()],
            )
            .unwrap();
        coordinator.approve(coordinator.revision()).unwrap();
        let sequence = store.save_workflow(&coordinator, None).unwrap();
        assert!(store.save_workflow(&coordinator, None).is_err());
        let checkpoint = store.recover_workflow("login", sequence).unwrap();
        assert_eq!(checkpoint.coordinator.phase(), Phase::Planning);
        assert!(store.save_workflow(&coordinator, Some(sequence)).is_err());
        assert!(root.path().join(".agent/tasks/login.json").is_file());
    }

    #[test]
    fn active_workflow_excludes_another_runner_or_recovery_writer() {
        let root = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let store = HarnessStore::new(home.path(), root.path()).unwrap();
        let coordinator = Coordinator::new("active", Intent::Build, "Build changes").unwrap();
        let sequence = store.save_workflow(&coordinator, None).unwrap();
        let lease = store.lease_workflow("active").unwrap();
        assert!(store.lease_workflow("active").is_err());
        assert!(store.save_workflow(&coordinator, Some(sequence)).is_err());
        assert!(store.recover_workflow("active", sequence).is_err());
        assert_eq!(store.load_workflow("active").unwrap().sequence, sequence);
        drop(lease);
        assert!(store.lease_workflow("active").is_ok());
    }
}
