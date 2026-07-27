use crate::error::{CodeGraphError, CodeGraphResult};
use crate::model::CodeGraphMode;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(crate) fn discover_files(root: &Path, mode: CodeGraphMode) -> CodeGraphResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    visit(root, root, mode, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit(
    root: &Path,
    dir: &Path,
    mode: CodeGraphMode,
    files: &mut Vec<PathBuf>,
) -> CodeGraphResult<()> {
    if !dir.exists() {
        return Ok(());
    }

    let entries = sorted_dir_entries(dir)?;
    for entry in entries {
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(&path);
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| CodeGraphError::at_path(&path, error.to_string()))?;

        if metadata.file_type().is_symlink() || should_skip(relative, &name, metadata.is_dir()) {
            continue;
        }

        if metadata.is_dir() {
            visit(root, &path, mode, files)?;
        } else if should_include(relative, mode) {
            files.push(path);
        }
    }

    Ok(())
}

fn should_skip(relative: &Path, name: &str, is_dir: bool) -> bool {
    if name == ".DS_Store" || name == ".git" || name == "target" || name == "node_modules" {
        return true;
    }
    if relative.starts_with(".dowe/codegraph") {
        return true;
    }
    is_dir && name.starts_with('.') && name != ".agents" && name != ".dowe"
}

fn should_include(relative: &Path, mode: CodeGraphMode) -> bool {
    match mode {
        CodeGraphMode::Dowe => {
            relative == Path::new("Cargo.toml")
                || relative == Path::new("AGENTS.md")
                || relative.starts_with("specs")
                || relative.starts_with("agents")
                || relative == Path::new("dowe-lang/Cargo.toml")
                || relative.starts_with("dowe-lang/crates")
                || relative.starts_with("docs")
                || relative.starts_with("dowe-lang/src")
                || relative == Path::new("dowe-lsp/Cargo.toml")
                || relative == Path::new("dowe-lsp/README.md")
                || relative.starts_with("dowe-lsp/crates")
                || relative == Path::new("dowe-zed/Cargo.toml")
                || relative == Path::new("dowe-zed/README.md")
                || relative.starts_with("dowe-zed/src")
                || relative.starts_with("dowe-zed/languages")
                || relative.starts_with("dowe-zed/tree-sitter-dowe")
                || relative == Path::new("dowe-llm/README.md")
                || relative.starts_with("dowe-llm/contracts")
        }
        CodeGraphMode::Project => {
            relative == Path::new("Cargo.toml")
                || relative.starts_with("crates")
                || relative.starts_with("src")
                || relative.starts_with("specs")
                || relative.starts_with("docs")
                || relative.starts_with(".agents")
                || relative.starts_with(".dowe")
        }
    }
}

pub(crate) fn safe_generated_path(root: &Path, relative: &Path) -> CodeGraphResult<PathBuf> {
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
        || !relative.starts_with(".dowe/codegraph")
    {
        return Err(CodeGraphError::new(
            "CodeGraph output must stay under .dowe/codegraph",
        ));
    }

    let base = root.join(".dowe/codegraph");
    if base.exists() {
        reject_symlink(&base, ".dowe/codegraph")?;
    }
    let path = root.join(relative);
    reject_symlink_ancestors(&path, &base)?;
    Ok(path)
}

fn reject_symlink_ancestors(path: &Path, base: &Path) -> CodeGraphResult<()> {
    let mut current = path.parent();
    while let Some(parent) = current {
        if parent == base {
            return Ok(());
        }
        if parent.exists() {
            reject_symlink(parent, ".dowe/codegraph")?;
        }
        current = parent.parent();
    }
    Ok(())
}

fn reject_symlink(path: &Path, label: &str) -> CodeGraphResult<()> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| CodeGraphError::at_path(path, error.to_string()))?;
    if metadata.file_type().is_symlink() {
        Err(CodeGraphError::at_path(
            path,
            format!("{label} must not be a symlink"),
        ))
    } else {
        Ok(())
    }
}

fn sorted_dir_entries(path: &Path) -> CodeGraphResult<Vec<std::fs::DirEntry>> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| CodeGraphError::at_path(path, error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| CodeGraphError::at_path(path, error.to_string()))?;
    entries.sort_by_key(|entry| entry.path());
    Ok(entries)
}

pub(crate) fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}
