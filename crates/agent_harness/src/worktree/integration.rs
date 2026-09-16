use super::*;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct PreparedIntegration {
    root: PathBuf,
    worktrees: Vec<IsolatedWorktree>,
    patch: Vec<u8>,
    additions: BTreeMap<String, Vec<u8>>,
    executables: BTreeSet<String>,
    report: IntegrationApplyReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationFaultPoint {
    AfterPatch,
    TerminateProcessAfterPatch,
}

impl PreparedIntegration {
    pub fn approval_manifest(&self) -> serde_json::Value {
        let hash = |bytes: &[u8]| {
            Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };
        serde_json::json!({
            "report": self.report,
            "worktrees": self.worktrees,
            "patchSha256": hash(&self.patch),
            "trackedPatch": (self.patch.len() <= 1024 * 1024).then(|| String::from_utf8_lossy(&self.patch).into_owned()),
            "newFiles": self.additions.iter().map(|(path, bytes)| serde_json::json!({"path":path,"bytes":bytes.len(),"sha256":hash(bytes),"executable":self.executables.contains(path)})).collect::<Vec<_>>()
        })
    }

    pub fn apply(self) -> HarnessResult<IntegrationApplyReport> {
        self.apply_with_fault_injection(None)
    }

    /// Deliberately leaves the durable journal at an irreversible boundary.
    /// This is used by fault-injection tests to model an abrupt process stop.
    pub fn apply_with_fault_injection(
        self,
        fault: Option<IntegrationFaultPoint>,
    ) -> HarnessResult<IntegrationApplyReport> {
        let current = prepare_isolated_worktrees(&self.root, &self.worktrees)?;
        if current != self {
            return Err(HarnessError::new(
                "integration changed after preparation; a new approval is required",
            ));
        }
        materialize(self, fault)
    }
}

pub fn apply_isolated_worktrees(
    root: &Path,
    worktrees: &[IsolatedWorktree],
) -> HarnessResult<IntegrationApplyReport> {
    prepare_isolated_worktrees(root, worktrees)?.apply()
}

pub fn prepare_isolated_worktrees(
    root: &Path,
    worktrees: &[IsolatedWorktree],
) -> HarnessResult<PreparedIntegration> {
    let root = root
        .canonicalize()
        .map_err(|error| HarnessError::at_path(root, error.to_string()))?;
    ensure_git_root(&root)?;
    if worktrees.is_empty() {
        return Ok(PreparedIntegration {
            root,
            worktrees: vec![],
            patch: vec![],
            additions: BTreeMap::new(),
            executables: BTreeSet::new(),
            report: IntegrationApplyReport {
                files: Vec::new(),
                new_files: Vec::new(),
                patch_bytes: 0,
                new_file_bytes: 0,
            },
        });
    }
    ensure_clean_host(&root)?;
    let head = git(&root, ["rev-parse", "HEAD"])?.stdout.trim().to_owned();
    let mut patch = Vec::new();
    let mut files = Vec::new();
    let mut additions = std::collections::BTreeMap::<String, Vec<u8>>::new();
    let mut executables = BTreeSet::new();
    for worktree in worktrees {
        ensure_worktree_identity(&root, worktree)?;
        let path = Path::new(&worktree.path);
        ensure_path_is_safe(&root, path)?;
        if !path.is_dir() || worktree.base_revision != head {
            return Err(HarnessError::new(
                "worktree is missing or has a different integration base",
            ));
        }
        let untracked = git_bytes(path, ["ls-files", "--others", "--exclude-standard", "-z"])?;
        for name in nul_paths(&untracked.stdout)? {
            if generated_path(&name) {
                continue;
            }
            validate_relative_path(&name)?;
            let source = path.join(&name);
            reject_symlinks(path, &source)?;
            let metadata = std::fs::symlink_metadata(&source)?;
            if !metadata.is_file() || metadata.len() > 16 * 1024 * 1024 {
                return Err(HarnessError::new("new worktree file exceeds 16 MiB"));
            }
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(&source)?
                .take(16 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() > 16 * 1024 * 1024 {
                return Err(HarnessError::new("new worktree file grew beyond 16 MiB"));
            }
            #[cfg(unix)]
            let executable = {
                use std::os::unix::fs::PermissionsExt;
                metadata.permissions().mode() & 0o111 != 0
            };
            #[cfg(not(unix))]
            let executable = false;
            if let Some(previous) = additions.insert(name.clone(), bytes.clone()) {
                if previous != bytes || executables.contains(&name) != executable {
                    return Err(HarnessError::new(
                        "different worktrees produced the same new file",
                    ));
                }
            }
            if executable {
                executables.insert(name.clone());
            }
            files.push(name);
            if additions.values().map(Vec::len).sum::<usize>() > 64 * 1024 * 1024 {
                return Err(HarnessError::new("new worktree files exceed 64 MiB"));
            }
        }
        let names = git_bytes(path, ["diff", "--name-only", "-z", "HEAD", "--"])?;
        for name in nul_paths(&names.stdout)? {
            validate_relative_path(&name)?;
            reject_symlinks(path, &path.join(&name))?;
            reject_symlinks(&root, &root.join(&name))?;
            files.push(name);
        }
        let diff = git_bytes(
            path,
            [
                "diff",
                "--binary",
                "--no-ext-diff",
                "--no-textconv",
                "HEAD",
                "--",
            ],
        )?;
        if !diff.stdout.is_empty() {
            patch.extend_from_slice(&diff.stdout);
        }
    }
    files.sort();
    files.dedup();
    for name in additions.keys() {
        reject_symlinks(&root, &root.join(name))?;
        if root.join(name).try_exists()? {
            return Err(HarnessError::new(
                "new worktree file already exists in the host checkout",
            ));
        }
    }
    let new_files = additions.keys().cloned().collect::<Vec<_>>();
    let new_file_bytes = additions.values().map(|bytes| bytes.len() as u64).sum();
    let patch_bytes = patch.len() as u64;
    if patch_bytes + new_file_bytes > 64 * 1024 * 1024 {
        return Err(HarnessError::new("integration exceeds 64 MiB"));
    }
    if !patch.is_empty() {
        git_input(&root, ["apply", "--check", "--binary", "-"], &patch)?;
    }
    Ok(PreparedIntegration {
        root,
        worktrees: worktrees.to_vec(),
        patch,
        additions,
        executables,
        report: IntegrationApplyReport {
            files,
            new_files,
            patch_bytes,
            new_file_bytes,
        },
    })
}

fn materialize(
    prepared: PreparedIntegration,
    fault: Option<IntegrationFaultPoint>,
) -> HarnessResult<IntegrationApplyReport> {
    if prepared.report.files.is_empty() {
        return Ok(prepared.report);
    }
    let manifest = prepared.approval_manifest();
    let PreparedIntegration {
        root,
        patch,
        additions,
        executables,
        report,
        ..
    } = prepared;
    let staging = root.join(".dowe").join("integration-pending");
    reject_symlinks(&root, &staging)?;
    std::fs::create_dir(&staging).map_err(|error| HarnessError::new(format!("cannot acquire integration journal: {error}; inspect .dowe/integration-pending before retrying")))?;
    durable_write(
        &staging.join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    durable_write(&staging.join("tracked.patch"), &patch)?;
    let mut staged = Vec::new();
    for (name, bytes) in &additions {
        let temporary = staging.join("new").join(name);
        if let Some(parent) = temporary.parent() {
            std::fs::create_dir_all(parent)?;
        }
        durable_write(&temporary, bytes)?;
        staged.push((temporary, root.join(name), executables.contains(name)));
    }
    durable_write(&staging.join("state"), b"prepared: application may have started; inspect host and retained worktree before recovery\n")?;
    #[cfg(unix)]
    std::fs::File::open(&staging)?.sync_all()?;
    let apply_result = if patch.is_empty() {
        Ok(())
    } else {
        git_input(&root, ["apply", "--binary", "-"], &patch)
    };
    if let Err(error) = apply_result {
        return Err(HarnessError::new(format!(
            "{error}; integration journal retained at {}",
            staging.display()
        )));
    }
    if matches!(
        fault,
        Some(IntegrationFaultPoint::AfterPatch)
            | Some(IntegrationFaultPoint::TerminateProcessAfterPatch)
    ) {
        if matches!(
            fault,
            Some(IntegrationFaultPoint::TerminateProcessAfterPatch)
        ) {
            std::process::exit(75);
        }
        return Err(HarnessError::new(format!(
            "fault injection after patch; integration journal retained at {}",
            staging.display()
        )));
    }
    let mut moved = Vec::new();
    for (temporary, destination, executable) in staged {
        if let Err(error) = reject_symlinks(&root, &destination) {
            return Err(rollback(&root, &staging, &patch, &moved, error));
        }
        if let Some(parent) = destination.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                return Err(rollback(&root, &staging, &patch, &moved, error));
            }
        }
        let mut output = match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
        {
            Ok(file) => file,
            Err(error) => return Err(rollback(&root, &staging, &patch, &moved, error)),
        };
        moved.push(destination);
        let copied = std::fs::File::open(&temporary)
            .and_then(|mut input| std::io::copy(&mut input, &mut output))
            .and_then(|_| {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    output.set_permissions(std::fs::Permissions::from_mode(if executable {
                        0o755
                    } else {
                        0o644
                    }))?;
                }
                #[cfg(not(unix))]
                let _ = executable;
                Ok(())
            })
            .and_then(|_| output.sync_all());
        if let Err(error) = copied {
            return Err(rollback(&root, &staging, &patch, &moved, error));
        }
    }
    durable_write(&staging.join("state"), b"applied\n")?;
    let receipt_id = Sha256::digest(serde_json::to_vec(&manifest)?)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let receipt = root
        .join(".dowe")
        .join(format!("integration-applied-{receipt_id}"));
    if std::fs::symlink_metadata(&receipt).is_ok() {
        return Err(HarnessError::new(
            "integration applied but receipt already exists; pending journal retained",
        ));
    }
    std::fs::rename(&staging, &receipt)?;
    #[cfg(unix)]
    std::fs::File::open(root.join(".dowe"))?.sync_all()?;
    Ok(report)
}

fn durable_write(path: &Path, bytes: &[u8]) -> HarnessResult<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn rollback(
    root: &Path,
    staging: &Path,
    patch: &[u8],
    moved: &[PathBuf],
    cause: impl std::fmt::Display,
) -> HarnessError {
    let mut failures = Vec::new();
    for path in moved.iter().rev() {
        if let Err(error) = std::fs::remove_file(path) {
            failures.push(format!("{}: {error}", path.display()));
        }
    }
    if !patch.is_empty() {
        if let Err(error) = git_input(root, ["apply", "--reverse", "--binary", "-"], patch) {
            failures.push(error.to_string());
        }
    }
    if failures.is_empty() {
        let _ = std::fs::remove_dir_all(staging);
        HarnessError::new(format!(
            "integration failed: {cause}; file effects rolled back"
        ))
    } else {
        HarnessError::new(format!(
            "integration failed: {cause}; rollback incomplete: {}; recovery data retained at {}",
            failures.join("; "),
            staging.display()
        ))
    }
}
