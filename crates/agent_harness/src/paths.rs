use crate::error::{HarnessError, HarnessResult};
use crate::model::FileRecord;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteMode {
    Preserve,
    Overwrite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WriteOutcome {
    Created(FileRecord),
    Preserved(FileRecord),
    Overwritten(FileRecord),
}

pub(crate) fn write_agent_file(
    root: &Path,
    relative_path: &Path,
    content: &str,
    mode: WriteMode,
) -> HarnessResult<WriteOutcome> {
    let path = safe_agent_path(root, relative_path)?;
    let logical = format!(".agents/{}", slash_path(relative_path));

    if path.exists() {
        reject_symlink(&path, ".agents")?;
        if mode == WriteMode::Preserve {
            return Ok(WriteOutcome::Preserved(FileRecord { path: logical }));
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| HarnessError::at_path(parent, error.to_string()))?;
    }

    let existed = path.exists();
    fs::write(&path, content).map_err(|error| HarnessError::at_path(&path, error.to_string()))?;

    if existed {
        Ok(WriteOutcome::Overwritten(FileRecord { path: logical }))
    } else {
        Ok(WriteOutcome::Created(FileRecord { path: logical }))
    }
}

pub(crate) fn create_agent_dir(root: &Path, relative_path: &Path) -> HarnessResult<WriteOutcome> {
    let path = safe_agent_path(root, relative_path)?;
    let logical = format!(".agents/{}", slash_path(relative_path));

    if path.exists() {
        reject_symlink(&path, ".agents")?;
        return Ok(WriteOutcome::Preserved(FileRecord { path: logical }));
    }

    fs::create_dir_all(&path).map_err(|error| HarnessError::at_path(&path, error.to_string()))?;
    Ok(WriteOutcome::Created(FileRecord { path: logical }))
}

pub(crate) fn write_dowe_evidence(
    root: &Path,
    relative_path: &Path,
    content: &str,
) -> HarnessResult<String> {
    let path = safe_dowe_evidence_path(root, relative_path)?;
    let logical = format!(".dowe/agent-harnesses/{}", slash_path(relative_path));

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| HarnessError::at_path(parent, error.to_string()))?;
    }

    fs::write(&path, content).map_err(|error| HarnessError::at_path(&path, error.to_string()))?;
    Ok(logical)
}

pub(crate) fn safe_project_relative_path(root: &Path, path: &Path) -> HarnessResult<String> {
    if path.is_absolute() {
        let canonical = path
            .canonicalize()
            .map_err(|error| HarnessError::at_path(path, error.to_string()))?;
        let root = root
            .canonicalize()
            .map_err(|error| HarnessError::at_path(root, error.to_string()))?;
        let relative = canonical
            .strip_prefix(&root)
            .map_err(|_| HarnessError::new("path must stay under project root"))?;
        return Ok(slash_path(relative));
    }

    ensure_safe_relative(path, "path must stay under project root")?;
    Ok(slash_path(path))
}

pub(crate) fn safe_agent_path(root: &Path, relative_path: &Path) -> HarnessResult<PathBuf> {
    ensure_safe_relative(relative_path, "path must stay under .agents")?;
    let agent_root = root.join(".agents");
    ensure_base_dir(root, &agent_root, ".agents")?;
    let path = agent_root.join(relative_path);
    reject_symlink_ancestors(&path, &agent_root, ".agents")?;
    Ok(path)
}

fn safe_dowe_evidence_path(root: &Path, relative_path: &Path) -> HarnessResult<PathBuf> {
    ensure_safe_relative(relative_path, "path must stay under .dowe/agent-harnesses")?;
    let evidence_root = root.join(".dowe/agent-harnesses");
    ensure_base_dir(root, &evidence_root, ".dowe/agent-harnesses")?;
    let path = evidence_root.join(relative_path);
    reject_symlink_ancestors(&path, &evidence_root, ".dowe/agent-harnesses")?;
    Ok(path)
}

fn ensure_safe_relative(path: &Path, message: &str) -> HarnessResult<()> {
    let invalid = path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        });

    if invalid {
        Err(HarnessError::new(message))
    } else {
        Ok(())
    }
}

fn ensure_base_dir(root: &Path, base: &Path, label: &str) -> HarnessResult<()> {
    if root.exists() {
        reject_symlink(root, "project root")?;
    }
    if base.exists() {
        reject_symlink(base, label)?;
        return Ok(());
    }
    fs::create_dir_all(base).map_err(|error| HarnessError::at_path(base, error.to_string()))
}

fn reject_symlink_ancestors(path: &Path, base: &Path, label: &str) -> HarnessResult<()> {
    let mut current = path.parent();
    while let Some(parent) = current {
        if parent == base {
            return Ok(());
        }
        if parent.exists() {
            reject_symlink(parent, label)?;
        }
        current = parent.parent();
    }
    Ok(())
}

fn reject_symlink(path: &Path, label: &str) -> HarnessResult<()> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| HarnessError::at_path(path, error.to_string()))?;
    if metadata.file_type().is_symlink() {
        Err(HarnessError::at_path(
            path,
            format!("{label} must not be a symlink"),
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}
