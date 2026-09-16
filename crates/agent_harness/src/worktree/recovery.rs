use super::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationRecovery {
    NothingPending,
    RolledBack { evidence: String },
    Finalized { receipt: String },
    PreservedForInspection { reason: String },
}

/// Recovers only effects that can be proven to belong to the pending journal.
/// Ambiguous files, patches, or states remain in place for explicit inspection.
pub fn recover_pending_integration(root: &Path) -> HarnessResult<IntegrationRecovery> {
    let root = root
        .canonicalize()
        .map_err(|error| HarnessError::at_path(root, error.to_string()))?;
    ensure_git_root(&root)?;
    let staging = root.join(".dowe").join("integration-pending");
    if !staging.exists() {
        return Ok(IntegrationRecovery::NothingPending);
    }
    reject_symlinks(&root, &staging)?;
    if !staging.is_dir() {
        return Ok(IntegrationRecovery::PreservedForInspection {
            reason: "pending integration path is not a directory".into(),
        });
    }
    let state = read_bounded(&staging.join("state"), 4096)?;
    let patch = read_bounded(&staging.join("tracked.patch"), 64 * 1024 * 1024)?;
    let manifest = read_bounded(&staging.join("manifest.json"), 2 * 1024 * 1024)?;
    let files = staged_files(&root, &staging)?;
    let reverse = if patch.is_empty() {
        true
    } else {
        git_input_check(
            &root,
            ["apply", "--reverse", "--check", "--binary", "-"],
            &patch,
        )?
    };
    let forward = if patch.is_empty() {
        true
    } else {
        git_input_check(&root, ["apply", "--check", "--binary", "-"], &patch)?
    };
    if state.starts_with(b"applied") {
        if !reverse || !new_files_match(&root, &files)? {
            return Ok(preserved("applied journal effects are not provable"));
        }
        let receipt_id = Sha256::digest(&manifest)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let receipt = root
            .join(".dowe")
            .join(format!("integration-applied-{receipt_id}"));
        if receipt.exists() {
            return Ok(preserved("integration receipt already exists"));
        }
        fs::rename(&staging, &receipt)?;
        return Ok(IntegrationRecovery::Finalized {
            receipt: receipt.to_string_lossy().into_owned(),
        });
    }
    if !state.starts_with(b"prepared") {
        return Ok(preserved("pending integration state is unknown"));
    }
    let new_files_absent = files.iter().all(|path| !path.exists());
    if reverse && (new_files_absent || new_files_match(&root, &files)?) {
        for path in files.iter().rev() {
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        if !patch.is_empty() {
            git_input(&root, ["apply", "--reverse", "--binary", "-"], &patch)?;
        }
        return Ok(IntegrationRecovery::RolledBack {
            evidence: quarantine(&root, &staging, "rolled-back")?,
        });
    }
    if forward && files.iter().all(|path| !path.exists()) {
        return Ok(IntegrationRecovery::PreservedForInspection {
            reason: "journal is prepared but no applied effect was found; evidence retained".into(),
        });
    }
    Ok(preserved(
        "journal is partially applied or conflicts with host data",
    ))
}

fn preserved(reason: &str) -> IntegrationRecovery {
    IntegrationRecovery::PreservedForInspection {
        reason: reason.into(),
    }
}

fn read_bounded(path: &Path, limit: u64) -> HarnessResult<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| HarnessError::at_path(path, error.to_string()))?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(HarnessError::at_path(
            path,
            "journal file is not bounded and regular",
        ));
    }
    Ok(fs::read(path)?)
}

fn staged_files(root: &Path, staging: &Path) -> HarnessResult<Vec<PathBuf>> {
    let directory = staging.join("new");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_files(root, &directory, &mut files)?;
    if files.len() > 4096 {
        return Err(HarnessError::new(
            "integration recovery has too many staged files",
        ));
    }
    Ok(files)
}

fn collect_files(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> HarnessResult<()> {
    reject_symlinks(root, directory)?;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.is_dir() {
            collect_files(root, &path, files)?;
        } else if metadata.is_file() {
            files.push(root.join(path.strip_prefix(directory).unwrap_or(&path)));
        } else {
            return Err(HarnessError::new(
                "journal contains a non-regular staged entry",
            ));
        }
    }
    Ok(())
}

fn new_files_match(root: &Path, staged: &[PathBuf]) -> HarnessResult<bool> {
    for destination in staged {
        let relative = destination
            .strip_prefix(root)
            .map_err(|_| HarnessError::new("staged path escaped journal"))?;
        let staged_path = root.join(".dowe/integration-pending/new").join(relative);
        if !destination.is_file() || fs::read(destination)? != fs::read(staged_path)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn quarantine(root: &Path, staging: &Path, label: &str) -> HarnessResult<String> {
    let digest = Sha256::digest(staging.to_string_lossy().as_bytes())
        .iter()
        .take(12)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let destination = root
        .join(".dowe")
        .join(format!("integration-recovery-{label}-{digest}"));
    if destination.exists() {
        return Err(HarnessError::new(
            "integration recovery evidence destination exists",
        ));
    }
    fs::rename(staging, &destination)?;
    Ok(destination.to_string_lossy().into_owned())
}
