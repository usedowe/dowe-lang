use crate::{HarnessError, HarnessResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

mod git;
mod integration;
mod recovery;
mod results;
mod safety;
mod snapshot;
use git::*;
pub use integration::{
    IntegrationFaultPoint, PreparedIntegration, apply_isolated_worktrees,
    prepare_isolated_worktrees,
};
pub use recovery::{IntegrationRecovery, recover_pending_integration};
pub use results::{WorkerResult, capture_worker_result, create_worktree_from_results};
use safety::*;

const WORKTREE_ROOT: &str = ".dowe/agent-worktrees";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsolatedWorktree {
    pub id: String,
    pub path: String,
    pub base_revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationApplyReport {
    pub files: Vec<String>,
    pub new_files: Vec<String>,
    pub patch_bytes: u64,
    pub new_file_bytes: u64,
}

pub fn create_isolated_worktree(root: &Path, id: &str) -> HarnessResult<IsolatedWorktree> {
    validate_id(id)?;
    let root = root
        .canonicalize()
        .map_err(|error| HarnessError::at_path(root, error.to_string()))?;
    ensure_git_root(&root)?;
    ensure_clean_host(&root)?;
    let revision = git(&root, ["rev-parse", "HEAD"])?.stdout.trim().to_owned();
    let path = root.join(WORKTREE_ROOT).join(id);
    ensure_path_is_safe(&root, &path)?;
    if path.exists() {
        return Err(HarnessError::at_path(&path, "worktree already exists"));
    }
    std::fs::create_dir_all(path.parent().expect("worktree root has parent"))?;
    git(
        &root,
        [
            "worktree",
            "add",
            "--detach",
            &path.to_string_lossy(),
            "HEAD",
        ],
    )?;
    Ok(IsolatedWorktree {
        id: id.into(),
        path: path.to_string_lossy().into_owned(),
        base_revision: revision,
    })
}

pub fn remove_isolated_worktree(root: &Path, id: &str) -> HarnessResult<()> {
    validate_id(id)?;
    let root = root
        .canonicalize()
        .map_err(|error| HarnessError::at_path(root, error.to_string()))?;
    ensure_git_root(&root)?;
    let path = root.join(WORKTREE_ROOT).join(id);
    ensure_path_is_safe(&root, &path)?;
    if !path.exists() {
        return Err(HarnessError::at_path(&path, "worktree does not exist"));
    }
    ensure_removable(&path)?;
    git(&root, ["worktree", "remove", &path.to_string_lossy()])?;
    Ok(())
}

pub fn validate_isolated_worktree(root: &Path, worker: &IsolatedWorktree) -> HarnessResult<()> {
    let root = root.canonicalize()?;
    ensure_git_root(&root)?;
    ensure_worktree_identity(&root, worker)?;
    if git(&root, ["rev-parse", "HEAD"])?.stdout.trim() != worker.base_revision {
        return Err(HarnessError::new(
            "worker base no longer matches the host commit",
        ));
    }
    Ok(())
}

pub fn cleanup_isolated_worktrees(
    root: &Path,
    worktrees: &[IsolatedWorktree],
) -> HarnessResult<()> {
    let root = root
        .canonicalize()
        .map_err(|error| HarnessError::at_path(root, error.to_string()))?;
    ensure_git_root(&root)?;
    for worktree in worktrees {
        ensure_worktree_identity(&root, worktree)?;
        ensure_removable(Path::new(&worktree.path))?;
    }
    for worktree in worktrees {
        remove_isolated_worktree(&root, &worktree.id)?;
    }
    Ok(())
}
