use crate::error::{RuntimeError, RuntimeResult};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

pub struct SourceWatcher {
    root: PathBuf,
    receiver: UnboundedReceiver<notify::Result<Event>>,
    _watcher: RecommendedWatcher,
}

impl SourceWatcher {
    pub fn new(root: impl AsRef<Path>) -> RuntimeResult<Self> {
        let root = root
            .as_ref()
            .canonicalize()
            .map_err(|error| RuntimeError::new(error.to_string()))?;
        let (sender, receiver) = unbounded_channel();
        let mut watcher = RecommendedWatcher::new(
            move |event| {
                let _ = sender.send(event);
            },
            Config::default().with_follow_symlinks(false),
        )
        .map_err(|error| RuntimeError::new(error.to_string()))?;
        watcher
            .watch(&root, RecursiveMode::Recursive)
            .map_err(|error| RuntimeError::new(error.to_string()))?;
        Ok(Self {
            root,
            receiver,
            _watcher: watcher,
        })
    }

    pub async fn receive(&mut self) -> RuntimeResult<Vec<String>> {
        loop {
            let event = self
                .receiver
                .recv()
                .await
                .ok_or_else(|| RuntimeError::new("source watcher stopped unexpectedly"))?
                .map_err(|error| RuntimeError::new(error.to_string()))?;
            let paths = self.paths_for_event(event);
            if !paths.is_empty() {
                return Ok(paths);
            }
        }
    }

    fn paths_for_event(&self, event: Event) -> Vec<String> {
        if !matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
        ) {
            return Vec::new();
        }
        event
            .paths
            .into_iter()
            .filter_map(|path| observed_relative_path(&self.root, &path))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

fn observed_relative_path(root: &Path, path: &Path) -> Option<String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let relative = absolute.strip_prefix(root).ok()?;
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| {
            !matches!(component, Component::Normal(_))
                || matches!(component.as_os_str().to_str(), Some(".dowe" | "target"))
        })
    {
        return None;
    }
    let observed = relative.extension().and_then(|value| value.to_str()) == Some("dowe")
        || relative
            .parent()
            .is_some_and(|parent| parent.as_os_str().is_empty())
            && matches!(
                relative.file_name().and_then(|value| value.to_str()),
                Some(".env" | ".env.example")
            )
        || relative.starts_with("icons");
    if !observed {
        return None;
    }
    if let Ok(metadata) = fs::symlink_metadata(&absolute)
        && (metadata.file_type().is_symlink() || !metadata.is_file())
    {
        return None;
    }
    Some(relative.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::SourceWatcher;
    use std::fs;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::time::timeout;

    #[tokio::test]
    async fn detects_root_entry_files_and_source_modules() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("pages")).expect("pages");
        let mut watcher = SourceWatcher::new(temp.path()).expect("watcher");

        fs::write(temp.path().join("main.dowe"), "one").expect("create");
        let created = receive_paths(&mut watcher).await;
        assert!(created.contains(&"main.dowe".to_string()));

        fs::write(temp.path().join("main.dowe"), "two").expect("modify");
        let modified = receive_paths(&mut watcher).await;
        assert!(modified.contains(&"main.dowe".to_string()));

        fs::rename(
            temp.path().join("main.dowe"),
            temp.path().join("pages/server.dowe"),
        )
        .expect("rename");
        let renamed = collect_paths(&mut watcher, 2).await;
        assert!(renamed.contains(&"main.dowe".to_string()));
        assert!(renamed.contains(&"pages/server.dowe".to_string()));

        fs::remove_file(temp.path().join("pages/server.dowe")).expect("delete");
        let deleted = receive_paths(&mut watcher).await;
        assert!(deleted.contains(&"pages/server.dowe".to_string()));
    }

    #[tokio::test]
    async fn detects_environment_and_icons_without_unrelated_assets() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("icons/web")).expect("icons");
        fs::create_dir_all(temp.path().join("assets/photos")).expect("photos");
        let mut watcher = SourceWatcher::new(temp.path()).expect("watcher");

        fs::write(temp.path().join(".env.example"), "BACKEND_URL=").expect("env example");
        fs::write(temp.path().join(".env"), "BACKEND_URL=").expect("env");
        fs::write(temp.path().join("icons/web/favicon.png"), "icon").expect("icon");
        fs::write(temp.path().join("assets/photos/hero.png"), "photo").expect("photo");
        let paths = collect_paths(&mut watcher, 3).await;

        assert!(paths.contains(&".env.example".to_string()));
        assert!(paths.contains(&".env".to_string()));
        assert!(paths.contains(&"icons/web/favicon.png".to_string()));
        assert!(!paths.contains(&"assets/photos/hero.png".to_string()));
    }

    #[tokio::test]
    async fn ignores_generated_and_build_files() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join(".dowe")).expect("dowe");
        fs::create_dir_all(temp.path().join("target")).expect("target");
        let mut watcher = SourceWatcher::new(temp.path()).expect("watcher");

        fs::write(temp.path().join(".dowe/generated.dowe"), "one").expect("dowe");
        fs::write(temp.path().join("target/output.dowe"), "one").expect("target");

        assert!(
            timeout(Duration::from_millis(150), watcher.receive())
                .await
                .is_err()
        );
    }

    async fn receive_paths(watcher: &mut SourceWatcher) -> Vec<String> {
        timeout(Duration::from_secs(3), watcher.receive())
            .await
            .expect("watch event timeout")
            .expect("watch event")
    }

    async fn collect_paths(watcher: &mut SourceWatcher, minimum: usize) -> Vec<String> {
        let mut paths = std::collections::BTreeSet::new();
        while paths.len() < minimum {
            paths.extend(receive_paths(watcher).await);
        }
        paths.into_iter().collect()
    }
}
