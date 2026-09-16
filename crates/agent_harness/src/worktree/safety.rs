use super::*;
use std::path::PathBuf;

pub(super) fn ensure_clean_host(root: &Path) -> HarnessResult<()> {
    let status = git_bytes(
        root,
        ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    )?;
    for entry in nul_paths(&status.stdout)? {
        if !entry.starts_with("?? ") || !generated_path(&entry[3..]) {
            return Err(HarnessError::new(
                "worktree operation requires a clean host checkout; preserve user changes first",
            ));
        }
    }
    Ok(())
}

pub(super) fn generated_path(path: &str) -> bool {
    path == ".dowe-agent-write.lock"
        || path.starts_with(".agents/capabilities/")
        || path.starts_with(".agent/tasks/")
        || path == ".dowe"
        || path.starts_with(".dowe/")
}

pub(super) fn nul_paths(bytes: &[u8]) -> HarnessResult<Vec<String>> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            String::from_utf8(path.to_vec())
                .map_err(|_| HarnessError::new("worktree path is not valid UTF-8"))
        })
        .collect()
}

pub(super) fn validate_relative_path(path: &str) -> HarnessResult<()> {
    let value = Path::new(path);
    if path.is_empty()
        || path.len() > 512
        || path.contains('\\')
        || path.chars().any(char::is_control)
        || value
            .components()
            .any(|component| component.as_os_str() == ".git")
        || value.is_absolute()
        || value.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(HarnessError::new(
            "new worktree path must be project-relative",
        ));
    }
    Ok(())
}

pub(super) fn validate_id(id: &str) -> HarnessResult<()> {
    if id.is_empty()
        || id.len() > 96
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(HarnessError::new("worktree ID is invalid"));
    }
    Ok(())
}

pub(super) fn ensure_git_root(root: &Path) -> HarnessResult<()> {
    let result = git(root, ["rev-parse", "--show-toplevel"])?;
    let actual = PathBuf::from(result.stdout.trim())
        .canonicalize()
        .map_err(|error| HarnessError::new(format!("git root is unavailable: {error}")))?;
    if actual != root {
        return Err(HarnessError::new(
            "worktree root must be the repository root",
        ));
    }
    Ok(())
}

pub(super) fn ensure_path_is_safe(root: &Path, path: &Path) -> HarnessResult<()> {
    let expected = root.join(WORKTREE_ROOT);
    if path.parent() != Some(expected.as_path()) || path.exists() && path.is_symlink() {
        return Err(HarnessError::new(
            "worktree path must stay under .dowe/agent-worktrees",
        ));
    }
    reject_symlinks(root, path)
}

pub(super) fn reject_symlinks(root: &Path, path: &Path) -> HarnessResult<()> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| HarnessError::new("path escapes project"))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        if !matches!(component, std::path::Component::Normal(_)) {
            return Err(HarnessError::new("path must be normalized"));
        }
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(HarnessError::at_path(&current, "symlink is not allowed"));
            }
            Ok(_) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

pub(super) fn ensure_removable(path: &Path) -> HarnessResult<()> {
    let status = git_bytes(
        path,
        [
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignored",
        ],
    )?;
    if !status.stdout.is_empty() {
        return Err(HarnessError::at_path(
            path,
            "worktree retained: modified, untracked or ignored data requires explicit recovery",
        ));
    }
    Ok(())
}

pub(super) fn ensure_worktree_identity(
    root: &Path,
    worktree: &IsolatedWorktree,
) -> HarnessResult<()> {
    validate_id(&worktree.id)?;
    let path = Path::new(&worktree.path);
    if path != root.join(WORKTREE_ROOT).join(&worktree.id) {
        return Err(HarnessError::new("worktree ID and path do not match"));
    }
    ensure_path_is_safe(root, path)?;
    ensure_git_root(path)?;
    let head = git(path, ["rev-parse", "HEAD"])?;
    if head.stdout.trim() != worktree.base_revision {
        return Err(HarnessError::new(
            "worktree HEAD differs from recorded base",
        ));
    }
    let common = |directory: &Path| -> HarnessResult<PathBuf> {
        let value = git(directory, ["rev-parse", "--git-common-dir"])?;
        Ok(directory.join(value.stdout.trim()).canonicalize()?)
    };
    if common(path)? != common(root)? {
        return Err(HarnessError::new("worktree belongs to another repository"));
    }
    Ok(())
}
