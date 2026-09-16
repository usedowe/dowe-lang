use super::*;
use std::fs::{File, OpenOptions, TryLockError};
use std::time::{Duration, Instant};

pub(super) fn acquire(root: &Path, write: bool) -> CodeGraphResult<Option<File>> {
    let path = root.join(".dowe/codegraph.lock");
    reject_path(root, &path)?;
    if fs::symlink_metadata(&path).is_ok_and(|metadata| !metadata.is_file()) {
        return Err(CodeGraphError::new("CodeGraph lock is not a regular file"));
    }
    let file = match OpenOptions::new()
        .read(true)
        .write(write)
        .create(write)
        .truncate(false)
        .open(&path)
    {
        Ok(file) => file,
        Err(error) if !write && error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !file.metadata()?.is_file() {
        return Err(CodeGraphError::new("CodeGraph lock is not a regular file"));
    }
    let started = Instant::now();
    loop {
        let result = if write {
            file.try_lock()
        } else {
            file.try_lock_shared()
        };
        match result {
            Ok(()) => return Ok(Some(file)),
            Err(TryLockError::WouldBlock) if started.elapsed() < Duration::from_secs(2) => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(TryLockError::WouldBlock) => {
                return Err(CodeGraphError::new(
                    "CodeGraph store is busy; retry after the active operation finishes",
                ));
            }
            Err(TryLockError::Error(error)) => return Err(error.into()),
        }
    }
}
