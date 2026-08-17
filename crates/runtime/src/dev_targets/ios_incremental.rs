use crate::error::{RuntimeError, RuntimeResult};
use dowe_compiler::GeneratedFile;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) const IOS_INCREMENTAL_MODULE_NAME: &str = "DoweIosViewModule";

const IOS_INCREMENTAL_CACHE_SCHEMA: &[u8] = b"dowe-ios-incremental-v2";
const IOS_HOT_MODULE_VERSION_SCHEMA: &[u8] = b"dowe-ios-hot-module-v2";
const IOS_HOST_ABI_SCHEMA: &[u8] = b"dowe-ios-dev-host-abi-v2";
const IOS_SOURCE_REVISION_PLACEHOLDER: &str = "__DOWE_IOS_SOURCE_REVISION__";
const RETAINED_INACTIVE_TOOLCHAINS: usize = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct IosHotModuleSource {
    relative_path: PathBuf,
    content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct IosHotModuleSnapshot {
    pub version: String,
    pub cache_key: String,
    sources: Vec<IosHotModuleSource>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct IosIncrementalSource {
    pub source: PathBuf,
    pub object: PathBuf,
    pub swift_dependencies: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct IosIncrementalWorkspace {
    pub sources: Vec<IosIncrementalSource>,
    pub output_map: PathBuf,
    pub master_dependencies: PathBuf,
    pub dependency_graph: PathBuf,
    pub linked_module: PathBuf,
    objects_root: PathBuf,
}

impl IosHotModuleSnapshot {
    pub(super) fn from_generated_files(
        files: &[GeneratedFile],
        target: &str,
        toolchain_signature: &[u8],
    ) -> RuntimeResult<Self> {
        let host = files
            .iter()
            .find(|file| {
                file.target == "ios"
                    && file.relative_path == Path::new("apps/ios/dev/DoweIosDevHost.swift")
            })
            .ok_or_else(|| RuntimeError::new("iOS module failed: missing generated host ABI"))?;
        let mut sources = files
            .iter()
            .filter_map(ios_hot_module_source)
            .collect::<Vec<_>>();
        sources.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        let mut unique = BTreeSet::new();
        if sources
            .iter()
            .any(|source| !unique.insert(source.relative_path.clone()))
        {
            return Err(RuntimeError::new(
                "iOS module failed: duplicate generated Swift source",
            ));
        }
        if sources.is_empty() {
            return Err(RuntimeError::new(
                "iOS module failed: missing generated Swift sources",
            ));
        }
        let factory = sources
            .iter()
            .find(|source| source.relative_path == Path::new("dev/DoweIosViewModule.swift"))
            .ok_or_else(|| RuntimeError::new("iOS module failed: missing generated factory"))?;
        if !factory.content.contains(IOS_SOURCE_REVISION_PLACEHOLDER) {
            return Err(RuntimeError::new(
                "iOS module failed: missing generated Objective-C revision placeholder",
            ));
        }
        let host_abi = ios_host_abi_signature(host);
        let version = ios_hot_module_version(&sources, target, toolchain_signature, &host_abi);
        for source in &mut sources {
            source.content = source
                .content
                .replace(IOS_SOURCE_REVISION_PLACEHOLDER, &version);
        }
        let cache_key = ios_incremental_cache_key(target, toolchain_signature, &host_abi);
        Ok(Self {
            version,
            cache_key,
            sources,
        })
    }
}

impl IosIncrementalWorkspace {
    pub fn prepare(project_root: &Path, snapshot: &IosHotModuleSnapshot) -> RuntimeResult<Self> {
        let incremental_root = project_root.join(".dowe/dev/ios/incremental");
        let workspace_root = incremental_root.join(&snapshot.cache_key);
        let sources_root = workspace_root.join("sources");
        let objects_root = workspace_root.join("objects");
        let links_root = workspace_root.join("links");
        fs::create_dir_all(&sources_root)?;
        fs::create_dir_all(&objects_root)?;
        fs::create_dir_all(&links_root)?;

        let mut expected_sources = BTreeSet::new();
        let mut expected_objects = BTreeSet::new();
        let mut sources = Vec::with_capacity(snapshot.sources.len());
        for source in &snapshot.sources {
            let source_path = sources_root.join(&source.relative_path);
            let object_path = objects_root.join(&source.relative_path).with_extension("o");
            let dependencies_path = objects_root
                .join(&source.relative_path)
                .with_extension("swiftdeps");
            write_if_changed(&source_path, source.content.as_bytes())?;
            expected_sources.insert(source_path.clone());
            expected_objects.insert(object_path.clone());
            expected_objects.insert(dependencies_path.clone());
            sources.push(IosIncrementalSource {
                source: source_path,
                object: object_path,
                swift_dependencies: dependencies_path,
            });
        }
        remove_obsolete_files(&sources_root, &expected_sources)?;
        remove_obsolete_files(&objects_root, &expected_objects)?;
        for source in &sources {
            if let Some(parent) = source.object.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        let output_map = workspace_root.join("output-file-map.json");
        let master_dependencies = workspace_root.join("master.swiftdeps");
        let dependency_graph = workspace_root.join("master.priors");
        let output_map_contents =
            serde_json::to_vec(&ios_incremental_output_map(&sources, &master_dependencies))
                .map_err(|error| RuntimeError::new(format!("iOS module failed: {error}")))?;
        write_if_changed(&output_map, &output_map_contents)?;

        let linked_module = links_root.join(format!("{}.dylib", snapshot.version));
        remove_obsolete_links(&links_root, &linked_module)?;
        fs::write(
            workspace_root.join("last-used"),
            snapshot.version.as_bytes(),
        )?;
        prune_incremental_toolchains(&incremental_root, &snapshot.cache_key)?;

        Ok(Self {
            sources,
            output_map,
            master_dependencies,
            dependency_graph,
            linked_module,
            objects_root,
        })
    }

    pub fn source_files(&self) -> Vec<String> {
        self.sources
            .iter()
            .map(|source| source.source.to_string_lossy().to_string())
            .collect()
    }

    pub fn object_files(&self) -> Vec<PathBuf> {
        self.sources
            .iter()
            .map(|source| source.object.clone())
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        self.sources
            .iter()
            .all(|source| source.object.is_file() && source.swift_dependencies.is_file())
            && (self.master_dependencies.is_file() || self.dependency_graph.is_file())
    }

    pub fn reset_outputs(&self) -> RuntimeResult<()> {
        if self.objects_root.exists() {
            fs::remove_dir_all(&self.objects_root)?;
        }
        fs::create_dir_all(&self.objects_root)?;
        for source in &self.sources {
            if let Some(parent) = source.object.parent() {
                fs::create_dir_all(parent)?;
            }
        }
        if self.master_dependencies.is_file() {
            fs::remove_file(&self.master_dependencies)?;
        }
        if self.dependency_graph.is_file() {
            fs::remove_file(&self.dependency_graph)?;
        }
        Ok(())
    }

    pub fn remove_linked_module(&self) {
        let _ = fs::remove_file(&self.linked_module);
    }
}

fn ios_hot_module_source(file: &GeneratedFile) -> Option<IosHotModuleSource> {
    if file.target != "ios" {
        return None;
    }
    let relative_path = file.relative_path.strip_prefix("apps/ios").ok()?;
    if relative_path.extension().and_then(|value| value.to_str()) != Some("swift")
        || relative_path == Path::new("DoweIosApp.swift")
        || relative_path == Path::new("dev/DoweIosDevHost.swift")
    {
        return None;
    }
    Some(IosHotModuleSource {
        relative_path: relative_path.to_path_buf(),
        content: file.content.clone(),
    })
}

fn ios_host_abi_signature(host: &GeneratedFile) -> Vec<u8> {
    let mut hash = Sha256::new();
    update_digest(&mut hash, IOS_HOST_ABI_SCHEMA);
    update_digest(&mut hash, host.relative_path.to_string_lossy().as_bytes());
    update_digest(&mut hash, host.content.as_bytes());
    hash.finalize().to_vec()
}

fn ios_hot_module_version(
    sources: &[IosHotModuleSource],
    target: &str,
    toolchain_signature: &[u8],
    host_abi: &[u8],
) -> String {
    let mut hash = Sha256::new();
    update_digest(&mut hash, IOS_HOT_MODULE_VERSION_SCHEMA);
    update_digest(&mut hash, IOS_INCREMENTAL_MODULE_NAME.as_bytes());
    update_digest(&mut hash, target.as_bytes());
    update_digest(&mut hash, toolchain_signature);
    update_digest(&mut hash, host_abi);
    for source in sources {
        update_digest(&mut hash, source.relative_path.to_string_lossy().as_bytes());
        update_digest(&mut hash, source.content.as_bytes());
    }
    format!("{:x}", hash.finalize())[..16].to_string()
}

fn ios_incremental_cache_key(target: &str, toolchain_signature: &[u8], host_abi: &[u8]) -> String {
    let mut hash = Sha256::new();
    update_digest(&mut hash, IOS_INCREMENTAL_CACHE_SCHEMA);
    update_digest(&mut hash, IOS_INCREMENTAL_MODULE_NAME.as_bytes());
    update_digest(&mut hash, target.as_bytes());
    update_digest(&mut hash, toolchain_signature);
    update_digest(&mut hash, host_abi);
    format!("{:x}", hash.finalize())
}

fn ios_incremental_output_map(
    sources: &[IosIncrementalSource],
    master_dependencies: &Path,
) -> Value {
    let mut entries = Map::new();
    entries.insert(
        String::new(),
        Value::Object(Map::from_iter([(
            "swift-dependencies".to_string(),
            Value::String(master_dependencies.to_string_lossy().to_string()),
        )])),
    );
    for source in sources {
        entries.insert(
            source.source.to_string_lossy().to_string(),
            Value::Object(Map::from_iter([
                (
                    "object".to_string(),
                    Value::String(source.object.to_string_lossy().to_string()),
                ),
                (
                    "swift-dependencies".to_string(),
                    Value::String(source.swift_dependencies.to_string_lossy().to_string()),
                ),
            ])),
        );
    }
    Value::Object(entries)
}

fn write_if_changed(path: &Path, contents: &[u8]) -> RuntimeResult<()> {
    if fs::read(path).ok().as_deref() == Some(contents) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

fn remove_obsolete_files(root: &Path, expected: &BTreeSet<PathBuf>) -> RuntimeResult<bool> {
    let mut empty = true;
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            if remove_obsolete_files(&path, expected)? {
                fs::remove_dir(&path)?;
            } else {
                empty = false;
            }
        } else if expected.contains(&path) {
            empty = false;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(empty)
}

fn remove_obsolete_links(root: &Path, active: &Path) -> RuntimeResult<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_file() && path != active {
            fs::remove_file(path)?;
        }
    }
    if active.is_file() {
        fs::remove_file(active)?;
    }
    Ok(())
}

fn prune_incremental_toolchains(root: &Path, active_key: &str) -> RuntimeResult<()> {
    let mut inactive = fs::read_dir(root)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter(|entry| entry.file_name().to_string_lossy() != active_key)
        .collect::<Vec<_>>();
    inactive.sort_by(|left, right| {
        let left_time = fs::metadata(left.path().join("last-used"))
            .and_then(|metadata| metadata.modified())
            .ok();
        let right_time = fs::metadata(right.path().join("last-used"))
            .and_then(|metadata| metadata.modified())
            .ok();
        right_time
            .cmp(&left_time)
            .then_with(|| right.file_name().cmp(&left.file_name()))
    });
    for entry in inactive.into_iter().skip(RETAINED_INACTIVE_TOOLCHAINS) {
        fs::remove_dir_all(entry.path())?;
    }
    Ok(())
}

fn update_digest(hash: &mut Sha256, value: &[u8]) {
    hash.update(value.len().to_le_bytes());
    hash.update(value);
}

#[cfg(test)]
mod tests {
    use super::{IOS_INCREMENTAL_MODULE_NAME, IosHotModuleSnapshot, IosIncrementalWorkspace};
    use dowe_compiler::GeneratedFile;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[test]
    fn captures_ios_hot_sources_and_materializes_objc_revision() {
        let first_files = hot_module_files("pages", "route-a", "host");
        let second_files = hot_module_files("pages", "route-b", "host");
        let first = IosHotModuleSnapshot::from_generated_files(
            &first_files,
            "arm64-apple-ios17.0-simulator",
            b"swift-a",
        )
        .expect("snapshot");
        let second = IosHotModuleSnapshot::from_generated_files(
            &second_files,
            "arm64-apple-ios17.0-simulator",
            b"swift-a",
        )
        .expect("snapshot");

        assert_eq!(first.sources.len(), 3);
        assert!(
            first
                .sources
                .iter()
                .any(|source| source.relative_path == Path::new("DowePages.swift"))
        );
        assert_eq!(
            first.sources[2].relative_path,
            Path::new("dev/DoweIosViewModule.swift")
        );
        assert_ne!(first.version, second.version);
        let first_factory = factory_content(&first);
        let second_factory = factory_content(&second);
        assert!(first_factory.contains(&format!(
            "@objc(DoweIosDevModuleController_{})",
            first.version
        )));
        assert!(second_factory.contains(&format!(
            "@objc(DoweIosDevModuleController_{})",
            second.version
        )));
        assert!(!first_factory.contains("__DOWE_IOS_SOURCE_REVISION__"));
        assert!(first_factory.contains("@_cdecl(\"dowe_create_root_view_controller\")"));
        let first_pages = first
            .sources
            .iter()
            .find(|source| source.relative_path == Path::new("DowePages.swift"))
            .expect("first pages");
        let second_pages = second
            .sources
            .iter()
            .find(|source| source.relative_path == Path::new("DowePages.swift"))
            .expect("second pages");
        assert_eq!(first_pages.content, second_pages.content);
    }

    #[test]
    fn versions_module_and_cache_with_target_toolchain_sdk_and_host_abi() {
        let files = hot_module_files("pages", "route", "host-a");
        let source_change = hot_module_files("pages", "changed", "host-a");
        let host_change = hot_module_files("pages", "route", "host-b");
        let baseline = IosHotModuleSnapshot::from_generated_files(
            &files,
            "arm64-apple-ios17.0-simulator",
            b"swift-xcode-sdk-a",
        )
        .expect("baseline");
        let changed_source = IosHotModuleSnapshot::from_generated_files(
            &source_change,
            "arm64-apple-ios17.0-simulator",
            b"swift-xcode-sdk-a",
        )
        .expect("source");
        let changed_target = IosHotModuleSnapshot::from_generated_files(
            &files,
            "x86_64-apple-ios17.0-simulator",
            b"swift-xcode-sdk-a",
        )
        .expect("target");
        let changed_toolchain = IosHotModuleSnapshot::from_generated_files(
            &files,
            "arm64-apple-ios17.0-simulator",
            b"swift-xcode-sdk-b",
        )
        .expect("toolchain");
        let changed_host = IosHotModuleSnapshot::from_generated_files(
            &host_change,
            "arm64-apple-ios17.0-simulator",
            b"swift-xcode-sdk-a",
        )
        .expect("host");

        assert_ne!(baseline.version, changed_source.version);
        assert_eq!(baseline.cache_key, changed_source.cache_key);
        assert_ne!(baseline.version, changed_target.version);
        assert_ne!(baseline.cache_key, changed_target.cache_key);
        assert_ne!(baseline.version, changed_toolchain.version);
        assert_ne!(baseline.cache_key, changed_toolchain.cache_key);
        assert_ne!(baseline.version, changed_host.version);
        assert_ne!(baseline.cache_key, changed_host.cache_key);
    }

    #[test]
    fn prepares_incremental_sources_objects_and_dependency_map() {
        let temp = tempdir().expect("tempdir");
        let files = hot_module_files("pages", "route", "host");
        let snapshot = IosHotModuleSnapshot::from_generated_files(
            &files,
            "arm64-apple-ios17.0-simulator",
            b"swift-a",
        )
        .expect("snapshot");

        let workspace =
            IosIncrementalWorkspace::prepare(temp.path(), &snapshot).expect("workspace");
        let output_map: serde_json::Value =
            serde_json::from_slice(&fs::read(&workspace.output_map).expect("output map"))
                .expect("json");
        let first = workspace
            .sources
            .iter()
            .find(|source| source.source.ends_with("DowePages.swift"))
            .expect("pages source");

        assert_eq!(IOS_INCREMENTAL_MODULE_NAME, "DoweIosViewModule");
        assert_eq!(fs::read_to_string(&first.source).expect("source"), "pages");
        assert_eq!(
            output_map[""]["swift-dependencies"],
            workspace.master_dependencies.to_string_lossy().as_ref()
        );
        assert_eq!(
            output_map[first.source.to_string_lossy().as_ref()]["object"],
            first.object.to_string_lossy().as_ref()
        );
        assert_eq!(
            output_map[first.source.to_string_lossy().as_ref()]["swift-dependencies"],
            first.swift_dependencies.to_string_lossy().as_ref()
        );
    }

    #[test]
    fn preserves_current_objects_and_removes_obsolete_units() {
        let temp = tempdir().expect("tempdir");
        let mut first_files = hot_module_files("first", "route", "host");
        first_files.push(generated("apps/ios/Second.swift", "second", "ios"));
        let first = IosHotModuleSnapshot::from_generated_files(
            &first_files,
            "arm64-apple-ios17.0-simulator",
            b"swift-a",
        )
        .expect("snapshot");
        let workspace = IosIncrementalWorkspace::prepare(temp.path(), &first).expect("workspace");
        for source in &workspace.sources {
            fs::write(&source.object, source.source.to_string_lossy().as_bytes()).expect("object");
            fs::write(&source.swift_dependencies, b"deps").expect("dependencies");
        }
        fs::write(&workspace.dependency_graph, b"priors").expect("priors");
        assert!(workspace.is_complete());
        let first_source = workspace
            .sources
            .iter()
            .find(|source| source.source.ends_with("DowePages.swift"))
            .expect("first source");
        let obsolete = workspace
            .sources
            .iter()
            .find(|source| source.source.ends_with("Second.swift"))
            .expect("obsolete source");
        let first_object = first_source.object.clone();
        let obsolete_object = obsolete.object.clone();
        let obsolete_source = obsolete.source.clone();
        let second_files = hot_module_files("changed", "route", "host");
        let second = IosHotModuleSnapshot::from_generated_files(
            &second_files,
            "arm64-apple-ios17.0-simulator",
            b"swift-a",
        )
        .expect("snapshot");

        let next = IosIncrementalWorkspace::prepare(temp.path(), &second).expect("workspace");

        assert!(first_object.is_file());
        assert!(!obsolete_object.exists());
        assert!(!obsolete_source.exists());
        let next_pages = next
            .sources
            .iter()
            .find(|source| source.source.ends_with("DowePages.swift"))
            .expect("pages");
        assert_eq!(
            fs::read_to_string(&next_pages.source).expect("source"),
            "changed"
        );
    }

    #[test]
    fn invalidates_and_bounds_incremental_toolchain_caches() {
        let temp = tempdir().expect("tempdir");
        let files = hot_module_files("pages", "route", "host");
        let target = "arm64-apple-ios17.0-simulator";
        let first =
            IosHotModuleSnapshot::from_generated_files(&files, target, b"swift-a").expect("first");
        let second =
            IosHotModuleSnapshot::from_generated_files(&files, target, b"swift-b").expect("second");
        let third =
            IosHotModuleSnapshot::from_generated_files(&files, target, b"swift-c").expect("third");
        let first_key = first.cache_key.clone();
        let second_key = second.cache_key.clone();
        let third_key = third.cache_key.clone();

        IosIncrementalWorkspace::prepare(temp.path(), &first).expect("first workspace");
        IosIncrementalWorkspace::prepare(temp.path(), &second).expect("second workspace");
        IosIncrementalWorkspace::prepare(temp.path(), &third).expect("third workspace");

        let root = temp.path().join(".dowe/dev/ios/incremental");
        let retained = fs::read_dir(&root)
            .expect("caches")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert_eq!(retained.len(), 2);
        assert!(retained.contains(&third_key));
        assert!(retained.contains(&first_key) || retained.contains(&second_key));
        assert_ne!(first_key, second_key);
        assert_ne!(second_key, third_key);
    }

    fn hot_module_files(pages: &str, route: &str, host: &str) -> Vec<GeneratedFile> {
        vec![
            generated("apps/ios/DoweIosApp.swift", "app", "ios"),
            generated("apps/ios/DowePages.swift", pages, "ios"),
            generated("apps/ios/DowePageIndexView.swift", route, "ios"),
            generated("apps/ios/dev/DoweIosDevHost.swift", host, "ios"),
            generated(
                "apps/ios/dev/DoweIosViewModule.swift",
                "@objc(DoweIosDevModuleController___DOWE_IOS_SOURCE_REVISION__)\n@_cdecl(\"dowe_create_root_view_controller\")",
                "ios",
            ),
            generated("apps/android/App.java", "android", "android-dev"),
        ]
    }

    fn factory_content(snapshot: &IosHotModuleSnapshot) -> &str {
        &snapshot
            .sources
            .iter()
            .find(|source| source.relative_path == Path::new("dev/DoweIosViewModule.swift"))
            .expect("factory")
            .content
    }

    fn generated(path: &str, content: &str, target: &str) -> GeneratedFile {
        GeneratedFile {
            relative_path: PathBuf::from(path),
            content: content.to_string(),
            kind: "test".to_string(),
            target: target.to_string(),
        }
    }
}
