use crate::error::{DoweError, DoweResult};
use std::fs;
use std::path::{Path, PathBuf};

pub fn discover_dowe_sources(root: &Path) -> DoweResult<Vec<PathBuf>> {
    let src = root.join("src");
    let mut files = Vec::new();

    if !src.exists() {
        return Ok(files);
    }

    scan_dir(root, &src, &mut files)?;
    files.sort();
    Ok(files)
}

fn scan_dir(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) -> DoweResult<()> {
    let mut entries = fs::read_dir(dir)
        .map_err(|error| DoweError::at_path(dir, error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| DoweError::at_path(dir, error.to_string()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|error| DoweError::at_path(&path, error.to_string()))?;
        if metadata.is_dir() {
            scan_dir(root, &path, files)?;
        } else if metadata.is_file() {
            classify_source(root, &path, files)?;
        }
    }

    Ok(())
}

fn classify_source(root: &Path, path: &Path, files: &mut Vec<PathBuf>) -> DoweResult<()> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    match extension {
        "dowe" => {
            files.push(path.to_path_buf());
            Ok(())
        }
        "ts" | "tsx" => Err(unsupported_source(
            root,
            path,
            "TypeScript and TSX are no longer supported Dowe source formats; use `.dowe` files under `/src`",
        )),
        "js" | "jsx" | "mjs" | "cjs" => Err(unsupported_source(
            root,
            path,
            "JavaScript is not a supported Dowe source format; use `.dowe` files under `/src`",
        )),
        _ => Ok(()),
    }
}

fn unsupported_source(root: &Path, path: &Path, message: &str) -> DoweError {
    let relative = path.strip_prefix(root).unwrap_or(path);
    DoweError::at_path(
        path,
        format!("unsupported source `{}`: {message}", relative.display()),
    )
}
