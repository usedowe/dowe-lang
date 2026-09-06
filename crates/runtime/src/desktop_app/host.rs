use crate::RuntimeResult;
use std::path::Path;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DesktopHostMode {
    Bundled,
    Development,
}

impl DesktopHostMode {
    fn developer_tools_enabled(self) -> bool {
        matches!(self, Self::Development)
    }
}

pub(super) fn run(name: &str, entry: &Path) -> RuntimeResult<()> {
    let mode = DesktopHostMode::Bundled;
    #[cfg(target_os = "linux")]
    {
        return linux::run(name, entry, mode.developer_tools_enabled());
    }
    #[cfg(target_os = "windows")]
    {
        return windows::run(name, entry, mode.developer_tools_enabled());
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = (name, entry, mode.developer_tools_enabled());
        Err(crate::RuntimeError::new(
            "embedded Dowe desktop applications support Windows and Linux",
        ))
    }
}

pub(super) fn run_uri(name: &str, uri: &str) -> RuntimeResult<()> {
    let mode = DesktopHostMode::Development;
    #[cfg(target_os = "linux")]
    {
        return linux::run_uri(name, uri, mode.developer_tools_enabled());
    }
    #[cfg(target_os = "windows")]
    {
        return windows::run_uri(name, uri, mode.developer_tools_enabled());
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = (name, uri, mode.developer_tools_enabled());
        Err(crate::RuntimeError::new(
            "the internal Dowe desktop host supports Windows and Linux",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopHostMode;

    #[test]
    fn developer_tools_are_enabled_only_for_development_hosts() {
        assert!(!DesktopHostMode::Bundled.developer_tools_enabled());
        assert!(DesktopHostMode::Development.developer_tools_enabled());
    }
}
