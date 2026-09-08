use crate::{AgentError, AgentResult};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

pub(super) struct DataLock {
    _file: File,
}

impl DataLock {
    pub fn acquire(path: &Path) -> AgentResult<Self> {
        Self::acquire_with_retry(path, true)
    }

    pub fn acquire_nowait(path: &Path) -> AgentResult<Self> {
        Self::acquire_with_retry(path, false)
    }

    fn acquire_with_retry(path: &Path, retry: bool) -> AgentResult<Self> {
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
        let deadline = Instant::now() + Duration::from_millis(100);
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Self { _file: file }),
                Err(std::fs::TryLockError::WouldBlock) if retry && Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(_) => {
                    return Err(AgentError::new(
                        "agent data is busy; retry without replaying operations",
                    ));
                }
            }
        }
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
