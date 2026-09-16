use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::sync::Arc;

pub(super) const MAX_BYTES: usize = 64 * 1024 * 1024;
pub(super) type Snapshot = BTreeMap<String, FileState>;

#[derive(Clone, PartialEq, Eq)]
pub(super) enum FileState {
    Unchanged,
    Deleted,
    Contents { bytes: Arc<[u8]>, executable: bool },
}

impl FileState {
    pub(super) fn len(&self) -> usize {
        match self {
            Self::Contents { bytes, .. } => bytes.len(),
            _ => 0,
        }
    }
}

pub(super) fn capture(root: &Path, worker: &IsolatedWorktree) -> HarnessResult<Snapshot> {
    ensure_worktree_identity(root, worker)?;
    let path = Path::new(&worker.path);
    let tracked = git_bytes(
        path,
        [
            "diff",
            "--name-only",
            "--no-ext-diff",
            "--no-textconv",
            "-z",
            "HEAD",
            "--",
        ],
    )?;
    let mut names = nul_paths(&tracked.stdout)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    if names.iter().any(|name| generated_path(name)) {
        return Err(HarnessError::new(
            "tracked runtime artifacts are not worker source results",
        ));
    }
    let untracked = git_bytes(path, ["ls-files", "--others", "--exclude-standard", "-z"])?;
    names.extend(
        nul_paths(&untracked.stdout)?
            .into_iter()
            .filter(|name| !generated_path(name)),
    );
    if names.len() > 4096 {
        return Err(HarnessError::new("worker result exceeds 4096 files"));
    }
    let mut snapshot = Snapshot::new();
    let mut total = 0;
    for name in names {
        validate_relative_path(&name)?;
        reject_symlinks(root, &root.join(&name))?;
        let source = path.join(&name);
        reject_symlinks(path, &source)?;
        let metadata = match std::fs::symlink_metadata(&source) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                snapshot.insert(name, FileState::Deleted);
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        if !metadata.is_file() || metadata.len() > 16 * 1024 * 1024 {
            return Err(HarnessError::at_path(
                &source,
                "worker file must be regular and at most 16 MiB",
            ));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(&source)?
            .take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        total += bytes.len();
        if bytes.len() > 16 * 1024 * 1024 || total > MAX_BYTES {
            return Err(HarnessError::new(
                "worker result exceeds source byte bounds",
            ));
        }
        #[cfg(unix)]
        let executable = {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode() & 0o111 != 0
        };
        #[cfg(not(unix))]
        let executable = false;
        snapshot.insert(
            name,
            FileState::Contents {
                bytes: bytes.into(),
                executable,
            },
        );
    }
    Ok(snapshot)
}

pub(super) fn populate(worker: &IsolatedWorktree, snapshot: &Snapshot) -> HarnessResult<()> {
    let root = Path::new(&worker.path);
    for (name, state) in snapshot {
        validate_relative_path(name)?;
        let path = root.join(name);
        reject_symlinks(root, &path)?;
        match state {
            FileState::Unchanged => (),
            FileState::Deleted => std::fs::remove_file(&path)?,
            FileState::Contents { bytes, executable } => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)?;
                file.write_all(bytes)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    file.set_permissions(std::fs::Permissions::from_mode(if *executable {
                        0o755
                    } else {
                        0o644
                    }))?;
                }
                #[cfg(not(unix))]
                let _ = executable;
                file.sync_all()?;
            }
        }
    }
    Ok(())
}
