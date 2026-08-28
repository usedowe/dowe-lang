use dowe_compiler::CompiledProject;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::error::{RuntimeError, RuntimeResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PublishedDevModule {
    pub target: String,
    pub version: String,
    pub path: String,
    pub file: PathBuf,
}

#[derive(Clone)]
pub(crate) struct DevModuleRevision {
    revision: u64,
    latest: Arc<Mutex<u64>>,
}

impl DevModuleRevision {
    pub(crate) fn new(revision: u64, latest: Arc<Mutex<u64>>) -> Self {
        Self { revision, latest }
    }

    pub(crate) fn is_current(&self) -> bool {
        *self.latest.lock().expect("module revision lock") == self.revision
    }

    pub(crate) fn run_if_current<T>(&self, action: impl FnOnce() -> T) -> Option<T> {
        let latest = self.latest.lock().expect("module revision lock");
        if *latest != self.revision {
            return None;
        }
        Some(action())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct DevModuleManifest {
    version: u8,
    ir_schema_version: u32,
    targets: BTreeMap<String, DevModuleManifestEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DevModuleManifestEntry {
    version: String,
    path: String,
}

pub(crate) fn web_module_version(project: &CompiledProject) -> String {
    let mut hash = Sha256::new();
    hash.update(dowe_components::VIEW_IR_SCHEMA_VERSION.to_le_bytes());
    hash.update(
        project
            .web
            .pages
            .first()
            .map(|page| page.router_file_name.as_str())
            .unwrap_or("router.js")
            .as_bytes(),
    );
    hash.update(project.web.design_file_name().as_bytes());
    for chunk in &project.web.chunks {
        hash.update(chunk.id.as_bytes());
        hash.update(chunk.css_file_name.as_bytes());
    }
    for page in &project.web.pages {
        hash.update(page.route_path.as_bytes());
        hash.update(page.page_chunk_id.as_bytes());
        for layout in &page.layout_chunk_ids {
            hash.update(layout.as_bytes());
        }
        for runtime in &page.runtime_chunks {
            hash.update(runtime.as_bytes());
        }
    }
    for chunk in &project.web.translation_chunks {
        hash.update(chunk.id.as_bytes());
        hash.update(chunk.locale.as_bytes());
    }
    let digest = hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    digest[..16].to_string()
}

pub(crate) fn publish_dev_module(
    root: &Path,
    target: &str,
    version: &str,
    extension: &str,
    source: &Path,
) -> RuntimeResult<PublishedDevModule> {
    publish_dev_module_with_revision(root, target, version, extension, source, None)?
        .ok_or_else(|| RuntimeError::new("unconditional module publication was skipped"))
}

pub(crate) fn publish_dev_module_if_current(
    root: &Path,
    target: &str,
    version: &str,
    extension: &str,
    source: &Path,
    revision: &DevModuleRevision,
) -> RuntimeResult<Option<PublishedDevModule>> {
    publish_dev_module_with_revision(root, target, version, extension, source, Some(revision))
}

fn publish_dev_module_with_revision(
    root: &Path,
    target: &str,
    version: &str,
    extension: &str,
    source: &Path,
    revision: Option<&DevModuleRevision>,
) -> RuntimeResult<Option<PublishedDevModule>> {
    let modules_root = root.join(".dowe/dev/modules");
    let target_root = modules_root.join(target);
    fs::create_dir_all(&target_root)?;
    let file_name = format!("{version}.{extension}");
    let file = target_root.join(&file_name);
    let staged = if file.is_file() {
        None
    } else {
        let staged = staged_module_path(&target_root, &file_name);
        fs::copy(source, &staged)?;
        Some(staged)
    };
    let revision_guard = revision.map(|revision| {
        revision
            .latest
            .lock()
            .expect("module revision publication lock")
    });
    if let (Some(revision), Some(latest)) = (revision, revision_guard.as_deref())
        && *latest != revision.revision
    {
        if let Some(staged) = staged {
            let _ = fs::remove_file(staged);
        }
        return Ok(None);
    }
    let _guard = module_publish_lock().lock().expect("module publish lock");
    if !file.is_file() {
        if let Some(staged) = &staged {
            fs::rename(staged, &file)?;
        }
    } else if let Some(staged) = &staged {
        fs::remove_file(staged)?;
    }
    let path = format!("/_dowe/dev/modules/{target}/{file_name}");
    let manifest_path = modules_root.join("manifest.json");
    let mut manifest = fs::read(&manifest_path)
        .ok()
        .and_then(|contents| serde_json::from_slice::<DevModuleManifest>(&contents).ok())
        .filter(|manifest| manifest.ir_schema_version == dowe_components::VIEW_IR_SCHEMA_VERSION)
        .unwrap_or_else(|| DevModuleManifest {
            version: 1,
            ir_schema_version: dowe_components::VIEW_IR_SCHEMA_VERSION,
            targets: BTreeMap::new(),
        });
    let previous_path = manifest.targets.get(target).map(|entry| entry.path.clone());
    manifest.targets.insert(
        target.to_string(),
        DevModuleManifestEntry {
            version: version.to_string(),
            path: path.clone(),
        },
    );
    let contents = serde_json::to_vec(&manifest)
        .map_err(|error| RuntimeError::new(format!("module manifest failed: {error}")))?;
    let staged_manifest = modules_root.join(".manifest.json.tmp");
    fs::write(&staged_manifest, contents)?;
    fs::rename(staged_manifest, manifest_path)?;
    prune_target_modules(&target_root, &path, previous_path.as_deref())?;

    Ok(Some(PublishedDevModule {
        target: target.to_string(),
        version: version.to_string(),
        path,
        file,
    }))
}

fn staged_module_path(target_root: &Path, file_name: &str) -> PathBuf {
    static NEXT_STAGING_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_STAGING_ID.fetch_add(1, Ordering::Relaxed);
    target_root.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), id))
}

fn prune_target_modules(
    target_root: &Path,
    active_path: &str,
    previous_path: Option<&str>,
) -> RuntimeResult<()> {
    let active = active_path.rsplit('/').next();
    let previous = previous_path.and_then(|path| path.rsplit('/').next());
    for entry in fs::read_dir(target_root)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().and_then(|value| value.to_str());
        if name != active && name != previous && !name.is_some_and(|value| value.starts_with('.')) {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn module_publish_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use super::{DevModuleRevision, web_module_version};
    use dowe_compiler::compile_dev;
    use std::fs;
    use std::sync::{Arc, Barrier, Mutex};
    use tempfile::TempDir;

    #[test]
    fn web_module_version_changes_with_generated_page_content() {
        let temp = TempDir::new().expect("tempdir");
        write_project(temp.path(), "First");
        let first = compile_dev(temp.path()).expect("first");
        let first_version = web_module_version(&first);
        write_project(temp.path(), "Second");
        let second = compile_dev(temp.path()).expect("second");
        let second_version = web_module_version(&second);

        assert_ne!(first_version, second_version);
        assert_eq!(first_version.len(), 16);
        assert_eq!(second_version.len(), 16);
    }

    #[test]
    fn publishes_module_before_atomically_updating_manifest() {
        let temp = TempDir::new().expect("tempdir");
        let source = temp.path().join("module.dex");
        fs::write(&source, b"dex contents").expect("source");

        let published = super::publish_dev_module(temp.path(), "android", "abc123", "dex", &source)
            .expect("publish");

        assert_eq!(published.target, "android");
        assert_eq!(published.path, "/_dowe/dev/modules/android/abc123.dex");
        assert_eq!(fs::read(&published.file).expect("module"), b"dex contents");
        let manifest = fs::read_to_string(temp.path().join(".dowe/dev/modules/manifest.json"))
            .expect("manifest");
        assert!(manifest.contains(r#""android":{"version":"abc123""#));
        assert!(
            !temp
                .path()
                .join(".dowe/dev/modules/.manifest.json.tmp")
                .exists()
        );
    }

    #[test]
    fn replaces_a_module_manifest_with_an_old_ir_schema() {
        let temp = TempDir::new().expect("tempdir");
        let source = temp.path().join("module.dex");
        fs::write(&source, b"new dex").expect("source");
        let manifest_path = temp.path().join(".dowe/dev/modules/manifest.json");
        fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("directory");
        fs::write(
            &manifest_path,
            r#"{"version":1,"ir_schema_version":0,"targets":{"android":{"version":"old","path":"/_dowe/dev/modules/android/old.dex"}}}"#,
        )
        .expect("old manifest");

        super::publish_dev_module(temp.path(), "ios", "new", "dylib", &source).expect("publish");

        let manifest = fs::read_to_string(manifest_path).expect("manifest");
        assert!(manifest.contains(r#""ir_schema_version":1"#));
        assert!(manifest.contains(r#""ios":{"version":"new""#));
        assert!(!manifest.contains("old.dex"));
    }

    #[test]
    fn retains_only_active_and_previous_target_modules() {
        let temp = TempDir::new().expect("tempdir");
        for version in ["first", "second", "third"] {
            let source = temp.path().join(format!("{version}.dex"));
            fs::write(&source, version).expect("source");
            super::publish_dev_module(temp.path(), "android", version, "dex", &source)
                .expect("publish");
        }

        let target = temp.path().join(".dowe/dev/modules/android");
        assert!(!target.join("first.dex").exists());
        assert!(target.join("second.dex").is_file());
        assert!(target.join("third.dex").is_file());
    }

    #[test]
    fn obsolete_revision_cannot_publish_or_replace_manifest() {
        let temp = TempDir::new().expect("tempdir");
        let current = temp.path().join("current.dex");
        let obsolete = temp.path().join("obsolete.dex");
        fs::write(&current, "current").expect("current");
        fs::write(&obsolete, "obsolete").expect("obsolete");
        super::publish_dev_module(temp.path(), "android", "current", "dex", &current)
            .expect("publish current");
        let latest = Arc::new(Mutex::new(2));
        let revision = DevModuleRevision::new(1, latest);

        let published = super::publish_dev_module_if_current(
            temp.path(),
            "android",
            "obsolete",
            "dex",
            &obsolete,
            &revision,
        )
        .expect("skip obsolete");

        assert!(published.is_none());
        assert!(
            !temp
                .path()
                .join(".dowe/dev/modules/android/obsolete.dex")
                .exists()
        );
        let manifest = fs::read_to_string(temp.path().join(".dowe/dev/modules/manifest.json"))
            .expect("manifest");
        assert!(manifest.contains(r#""version":"current""#));
        assert!(!manifest.contains("obsolete"));
    }

    #[test]
    fn concurrent_targets_preserve_both_manifest_entries() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path().to_path_buf();
        let android = root.join("android.dex");
        let ios = root.join("ios.dylib");
        fs::write(&android, "android").expect("android");
        fs::write(&ios, "ios").expect("ios");
        let barrier = Arc::new(Barrier::new(3));
        let handles = [("android", "dex", android), ("ios", "dylib", ios)]
            .into_iter()
            .map(|(target, extension, source)| {
                let root = root.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    super::publish_dev_module(
                        &root,
                        target,
                        &format!("{target}-version"),
                        extension,
                        &source,
                    )
                    .expect("publish");
                })
            })
            .collect::<Vec<_>>();

        barrier.wait();
        for handle in handles {
            handle.join().expect("publication thread");
        }

        let manifest =
            fs::read_to_string(root.join(".dowe/dev/modules/manifest.json")).expect("manifest");
        assert!(manifest.contains(r#""android":{"version":"android-version""#));
        assert!(manifest.contains(r#""ios":{"version":"ios-version""#));
    }

    fn write_project(root: &std::path::Path, text: &str) {
        fs::create_dir_all(root.join("pages")).expect("pages");
        fs::create_dir_all(root.join("routes")).expect("routes");
        fs::write(
            root.join("main.dowe"),
            "import routes from \"@/routes/view\"\n\nmain\n  views:routes\n  server port:0\n",
        )
        .expect("main");
        fs::write(
            root.join("routes/view.dowe"),
            "import indexPage from \"../pages/index\"\n\nviews routes\n  route path:\"/\" page:indexPage\n",
        )
        .expect("routes");
        fs::write(
            root.join("pages/index.dowe"),
            format!("page indexPage\n  Text\n    \"{text}\"\n"),
        )
        .expect("page");
    }
}
