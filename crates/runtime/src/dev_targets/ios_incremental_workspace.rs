use crate::error::{RuntimeError, RuntimeResult};
use dowe_compiler::GeneratedFile;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) const IOS_INCREMENTAL_MODULE_NAME: &str = "DoweIosViewModule";

const IOS_INCREMENTAL_CACHE_SCHEMA: &[u8] = b"dowe-ios-incremental-v3";
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
    update_digest(
        &mut hash,
        &dowe_components::VIEW_IR_SCHEMA_VERSION.to_le_bytes(),
    );
    update_digest(&mut hash, IOS_HOT_MODULE_VERSION_SCHEMA);
    update_digest(&mut hash, IOS_INCREMENTAL_MODULE_NAME.as_bytes());
    update_digest(&mut hash, target.as_bytes());
    update_digest(&mut hash, toolchain_signature);
    update_digest(&mut hash, host_abi);
    for source in sources {
        update_digest(&mut hash, source.relative_path.to_string_lossy().as_bytes());
        update_digest(&mut hash, source.content.as_bytes());
    }
    let digest = hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    digest[..16].to_string()
}

fn ios_incremental_cache_key(target: &str, toolchain_signature: &[u8], host_abi: &[u8]) -> String {
    let mut hash = Sha256::new();
    update_digest(
        &mut hash,
        &dowe_components::VIEW_IR_SCHEMA_VERSION.to_le_bytes(),
    );
    update_digest(&mut hash, IOS_INCREMENTAL_CACHE_SCHEMA);
    update_digest(&mut hash, IOS_INCREMENTAL_MODULE_NAME.as_bytes());
    update_digest(&mut hash, target.as_bytes());
    update_digest(&mut hash, toolchain_signature);
    update_digest(&mut hash, host_abi);
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
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

