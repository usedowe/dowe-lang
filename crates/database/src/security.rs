use crate::error::StoreResult;
use std::fs::{self, File};
use std::path::Path;

pub(crate) fn create_private_directory(path: &Path) -> StoreResult<()> {
    fs::create_dir_all(path)?;
    secure_directory(path)
}

#[cfg(unix)]
pub(crate) fn secure_directory(path: &Path) -> StoreResult<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(not(unix))]
pub(crate) fn secure_directory(_path: &Path) -> StoreResult<()> {
    Ok(())
}

#[cfg(unix)]
pub(crate) fn secure_file(file: &File) -> StoreResult<()> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
pub(crate) fn secure_file(_file: &File) -> StoreResult<()> {
    Ok(())
}
