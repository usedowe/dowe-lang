use crate::error::{RuntimeError, RuntimeResult};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug)]
pub struct SourceWatcher {
    root: PathBuf,
    src: PathBuf,
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
        let src = root.join("src");
        let snapshot = SourceSnapshot::scan(&root, &src)?;
        Ok(Self {
            root,
            src,
            snapshot,
        })
    }

    pub fn poll(&mut self) -> RuntimeResult<Vec<String>> {
        let next = SourceSnapshot::scan(&self.root, &self.src)?;
        let paths = self.snapshot.changed_paths(&next);
        self.snapshot = next;
        Ok(paths)
    }
}

impl SourceSnapshot {
    fn scan(root: &Path, src: &Path) -> RuntimeResult<Self> {
        let mut files = BTreeMap::new();

        if !src.exists() {
            return Ok(Self { files });
        }

        scan_dir(root, src, &mut files)?;
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
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|error| RuntimeError::new(error.to_string()))?;

        if metadata.is_dir() {
            scan_dir(root, &path, files)?;
        } else if metadata.is_file() {
            let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            files.insert(
                relative,
                FileStamp {
                    len: metadata.len(),
                    modified: metadata.modified().ok(),
                },
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SourceWatcher;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detects_create_modify_delete_and_rename_under_src() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("src/pages")).expect("src");
        let mut watcher = SourceWatcher::new(temp.path()).expect("watcher");

        fs::write(temp.path().join("src/main.dowe"), "one").expect("create");
        let created = watcher.poll().expect("created");
        assert!(created.contains(&"src/main.dowe".to_string()));

        fs::write(temp.path().join("src/main.dowe"), "two").expect("modify");
        let modified = watcher.poll().expect("modified");
        assert!(modified.contains(&"src/main.dowe".to_string()));

        fs::rename(
            temp.path().join("src/main.dowe"),
            temp.path().join("src/pages/server.dowe"),
        )
        .expect("rename");
        let renamed = watcher.poll().expect("renamed");
        assert!(renamed.contains(&"src/main.dowe".to_string()));
        assert!(renamed.contains(&"src/pages/server.dowe".to_string()));

        fs::remove_file(temp.path().join("src/pages/server.dowe")).expect("delete");
        let deleted = watcher.poll().expect("deleted");
        assert!(deleted.contains(&"src/pages/server.dowe".to_string()));
    }

    #[test]
    fn ignores_files_outside_src() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("src")).expect("src");
        fs::create_dir_all(temp.path().join(".dowe")).expect("dowe");
        fs::create_dir_all(temp.path().join("target")).expect("target");
        let mut watcher = SourceWatcher::new(temp.path()).expect("watcher");

        fs::write(temp.path().join(".dowe/generated.js"), "one").expect("dowe");
        fs::write(temp.path().join("target/output"), "one").expect("target");

        assert!(watcher.poll().expect("poll").is_empty());
    }
}
