use crate::error::{DeployError, DeployResult};
use std::fs;
use std::path::{Path, PathBuf};

pub fn reset_dir(path: &Path) -> DeployResult<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn write_file(path: &Path, content: impl AsRef<[u8]>) -> DeployResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

pub fn copy_file(source: &Path, destination: &Path) -> DeployResult<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination)?;
    Ok(())
}

pub fn copy_tree(source: &Path, destination: &Path) -> DeployResult<()> {
    if !source.exists() {
        return Ok(());
    }
    fs::create_dir_all(destination)?;
    let mut entries = fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let source = entry.path();
        let destination = destination.join(entry.file_name());
        if source.is_dir() {
            copy_tree(&source, &destination)?;
        } else if source.is_file() {
            copy_file(&source, &destination)?;
        }
    }
    Ok(())
}

pub fn collect_files(root: &Path) -> DeployResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_into(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_into(root: &Path, path: &Path, files: &mut Vec<PathBuf>) -> DeployResult<()> {
    let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_files_into(root, &path, files)?;
        } else if path.is_file() {
            files.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
    Ok(())
}

pub fn target_dir(root: &Path, target: &str) -> DeployResult<PathBuf> {
    if target.is_empty()
        || target
            .chars()
            .any(|value| !(value.is_ascii_lowercase() || value == '-'))
    {
        return Err(DeployError::new("invalid deploy target directory"));
    }
    Ok(root.join(".dowe/dist").join(target))
}
