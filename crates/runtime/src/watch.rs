use crate::error::{RuntimeError, RuntimeResult};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug)]
pub struct SourceWatcher {
    root: PathBuf,
    snapshot: SourceSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourceSnapshot {
    files: BTreeMap<PathBuf, FileStamp>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileStamp {
    len: u64,
    modified: Option<SystemTime>,
}

impl SourceWatcher {
    pub fn new(root: impl AsRef<Path>) -> RuntimeResult<Self> {
        let root = root.as_ref().to_path_buf();
        let snapshot = SourceSnapshot::scan(&root)?;
        Ok(Self { root, snapshot })
    }

    pub fn poll(&mut self) -> RuntimeResult<Vec<String>> {
        let next = SourceSnapshot::scan(&self.root)?;
        let paths = self.snapshot.changed_paths(&next);
        self.snapshot = next;
        Ok(paths)
    }
}

impl SourceSnapshot {
    fn scan(root: &Path) -> RuntimeResult<Self> {
        let mut files = BTreeMap::new();
        scan_dir(root, root, &mut files)?;
        Ok(Self { files })
    }

    fn changed_paths(&self, next: &Self) -> Vec<String> {
        let mut paths = BTreeSet::new();

        for (path, stamp) in &self.files {
            match next.files.get(path) {
                Some(next_stamp) if next_stamp == stamp => {}
                _ => {
                    paths.insert(path.clone());
                }
            }
        }

        for path in next.files.keys() {
            if !self.files.contains_key(path) {
                paths.insert(path.clone());
            }
        }

        paths
            .into_iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect()
    }
}

fn scan_dir(
    root: &Path,
    dir: &Path,
    files: &mut BTreeMap<PathBuf, FileStamp>,
) -> RuntimeResult<()> {
    for entry in fs::read_dir(dir).map_err(|error| RuntimeError::new(error.to_string()))? {
        let entry = entry.map_err(|error| RuntimeError::new(error.to_string()))?;
        if entry
            .file_type()
            .map_err(|error| RuntimeError::new(error.to_string()))?
            .is_symlink()
        {
            continue;
        }
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|error| RuntimeError::new(error.to_string()))?;

        if metadata.is_dir() {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if name.starts_with('.') || name == "target" {
                continue;
            }
            scan_dir(root, &path, files)?;
        } else if metadata.is_file()
            && (path.extension().and_then(|value| value.to_str()) == Some("dowe")
                || path.parent() == Some(root)
                    && matches!(
                        path.file_name().and_then(|value| value.to_str()),
                        Some(".env" | ".env.example")
                    )
                || path.starts_with(root.join("icons")))
        {
            insert_file(root, &path, files)?;
        }
    }

    Ok(())
}

fn insert_file(
    root: &Path,
    path: &Path,
    files: &mut BTreeMap<PathBuf, FileStamp>,
) -> RuntimeResult<()> {
    let metadata = fs::metadata(path).map_err(|error| RuntimeError::new(error.to_string()))?;
    let relative = path.strip_prefix(root).unwrap_or(path).to_path_buf();
    files.insert(
        relative,
        FileStamp {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SourceWatcher;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detects_root_entry_files_and_source_modules() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("pages")).expect("src");
        let mut watcher = SourceWatcher::new(temp.path()).expect("watcher");

        fs::write(temp.path().join("main.dowe"), "one").expect("create");
        let created = watcher.poll().expect("created");
        assert!(created.contains(&"main.dowe".to_string()));

        fs::write(temp.path().join("main.dowe"), "two").expect("modify");
        let modified = watcher.poll().expect("modified");
        assert!(modified.contains(&"main.dowe".to_string()));

        fs::write(temp.path().join("theme.dowe"), "theme").expect("theme");
        fs::write(temp.path().join(".env.example"), "BACKEND_URL=").expect("env example");
        fs::write(temp.path().join(".env"), "BACKEND_URL=").expect("env");
        let configured = watcher.poll().expect("configured");
        assert!(configured.contains(&"theme.dowe".to_string()));
        assert!(configured.contains(&".env.example".to_string()));
        assert!(configured.contains(&".env".to_string()));

        fs::rename(
            temp.path().join("main.dowe"),
            temp.path().join("pages/server.dowe"),
        )
        .expect("rename");
        let renamed = watcher.poll().expect("renamed");
        assert!(renamed.contains(&"main.dowe".to_string()));
        assert!(renamed.contains(&"pages/server.dowe".to_string()));

        fs::remove_file(temp.path().join("pages/server.dowe")).expect("delete");
        let deleted = watcher.poll().expect("deleted");
        assert!(deleted.contains(&"pages/server.dowe".to_string()));
    }

    #[test]
    fn ignores_generated_and_build_files() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join(".dowe")).expect("dowe");
        fs::create_dir_all(temp.path().join("target")).expect("target");
        let mut watcher = SourceWatcher::new(temp.path()).expect("watcher");

        fs::write(temp.path().join(".dowe/generated.js"), "one").expect("dowe");
        fs::write(temp.path().join("target/output"), "one").expect("target");

        assert!(watcher.poll().expect("poll").is_empty());
    }

    #[test]
    fn detects_project_icons_without_watching_unrelated_assets() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("icons/web")).expect("icons");
        fs::create_dir_all(temp.path().join("assets/photos")).expect("photos");
        let mut watcher = SourceWatcher::new(temp.path()).expect("watcher");

        fs::write(temp.path().join("icons/web/favicon-32x32.png"), "icon").expect("icon");
        fs::write(temp.path().join("assets/photos/hero.png"), "photo").expect("photo");
        let changes = watcher.poll().expect("poll");

        assert_eq!(changes, ["icons/web/favicon-32x32.png"]);
    }
}
