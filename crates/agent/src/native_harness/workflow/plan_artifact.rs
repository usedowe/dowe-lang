use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn destination(store: &HarnessStore, id: &str) -> AgentResult<PathBuf> {
    let execution = store.workflow_path(id)?;
    for record in [&execution, &execution.with_extension("requirements.json")] {
        if fs::symlink_metadata(record).is_ok() {
            return Err(AgentError::new(
                "workflow ID is already reserved; choose a new ID",
            ));
        }
    }
    let path = store.root().join(".agent/plans").join(format!("{id}.json"));
    check(&path)?;
    Ok(path)
}

fn check(path: &Path) -> AgentResult<()> {
    for ancestor in path.ancestors() {
        if fs::symlink_metadata(ancestor).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(AgentError::new("draft paths must not traverse symlinks"));
        }
    }
    match fs::symlink_metadata(path) {
        Ok(_) => Err(AgentError::new("draft destination already exists")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn publish(path: &Path, bytes: &[u8]) -> AgentResult<()> {
    use std::io::Write;
    check(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| AgentError::new("draft has no parent"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".draft-{}.tmp", identifier()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    check(path)?;
    if let Err(error) = fs::hard_link(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(AgentError::new(format!("draft was not published: {error}")));
    }
    fs::remove_file(&temporary)?;
    #[cfg(unix)]
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}
