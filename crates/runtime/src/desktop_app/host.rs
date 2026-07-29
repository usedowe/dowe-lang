use crate::RuntimeResult;
use std::path::Path;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

pub(super) fn run(name: &str, entry: &Path) -> RuntimeResult<()> {
    #[cfg(target_os = "linux")]
    {
        return linux::run(name, entry);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::run(name, entry);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = (name, entry);
        Err(crate::RuntimeError::new(
            "embedded Dowe desktop applications support Windows and Linux",
        ))
    }
}

pub(super) fn run_uri(name: &str, uri: &str) -> RuntimeResult<()> {
    #[cfg(target_os = "linux")]
    {
        return linux::run_uri(name, uri);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::run_uri(name, uri);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = (name, uri);
        Err(crate::RuntimeError::new(
            "the internal Dowe desktop host supports Windows and Linux",
        ))
    }
}
