use crate::{AgentError, AgentResult};
use std::fs::{self, File, OpenOptions};
use std::path::Path;

pub(super) struct DataLock {
    _file: File,
}

impl DataLock {
    pub fn acquire(path: &Path) -> AgentResult<Self> {
        if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(AgentError::new("agent lock cannot be a symlink"));
        }
        let parent = path
            .parent()
            .ok_or_else(|| AgentError::new("agent lock has no parent"))?;
        fs::create_dir_all(parent)?;
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(path)?;
        file.try_lock().map_err(|_| {
            AgentError::new("agent data is busy; retry without replaying operations")
        })?;
        Ok(Self { _file: file })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_files_can_remain_without_blocking_recovery() {
        let home = tempfile::tempdir().unwrap();
        let path = home.path().join("session.lock");
        let first = DataLock::acquire(&path).unwrap();
        assert!(DataLock::acquire(&path).is_err());
        drop(first);
        assert!(path.is_file());
        assert!(DataLock::acquire(&path).is_ok());
    }
}
