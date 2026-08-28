use super::{quiet_command_options, run_required};
use crate::dev::DevTarget;
use crate::dev_modules::DevModuleRevision;
use crate::error::{RuntimeError, RuntimeResult};
use dowe_spawn::{SpawnConfig, StreamMode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

const CACHE_SCHEMA: u8 = 1;
const CACHE_FINGERPRINT: &[u8] = b"dowe-android-dex-cache-v1";
const RETAINED_INACTIVE_TOOLCHAINS: usize = 1;
const GENERATED_PACKAGE: &str = "dev/dowe/generated";
const CORE_SOURCE: &str = "DoweDevActivity.java";
const ROUTE_PREFIX: &str = "DoweDevRoute";
const LAYOUT_PREFIX: &str = "DoweDevLayout";
const JAVAC_FLAGS: &[&str] = &["-g:none", "-proc:none", "-implicit:none"];
const D8_INTERMEDIATE_FLAGS: &[&str] =
    &["--min-api", "26", "--intermediate", "--file-per-class-file"];
const D8_MERGE_FLAGS: &[&str] = &["--min-api", "26"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AndroidHotModuleSource {
    pub relative_path: PathBuf,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourceFingerprint {
    digest: String,
    shard: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CacheManifest {
    schema: u8,
    toolchain: String,
    sources: BTreeMap<String, CachedSource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CachedSource {
    digest: String,
    classes: Vec<String>,
    dex_key: String,
    dex_files: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IncrementalPlan {
    full_rebuild: bool,
    compile: Vec<String>,
    remove: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CacheUse {
    key: String,
    last_used: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BuildStatus {
    Complete,
    Superseded,
}

#[derive(Debug)]
struct StagedBuildFailure {
    error: RuntimeError,
    retry_full: bool,
}

impl From<RuntimeError> for StagedBuildFailure {
    fn from(error: RuntimeError) -> Self {
        Self {
            error,
            retry_full: false,
        }
    }
}

impl From<std::io::Error> for StagedBuildFailure {
    fn from(error: std::io::Error) -> Self {
        RuntimeError::from(error).into()
    }
}

pub(super) fn android_toolchain_fingerprint(
    d8: &Path,
    android_jar: &Path,
    base_classes: &Path,
) -> RuntimeResult<String> {
    let javac = run_required(
        DevTarget::Android,
        SpawnConfig::new("javac", ["-version"])
            .with_options(quiet_command_options(None, StreamMode::Pipe)),
    )?;
    let mut hash = Sha256::new();
    update_digest(&mut hash, CACHE_FINGERPRINT);
    update_digest(&mut hash, JAVAC_FLAGS.join("\0").as_bytes());
    update_digest(&mut hash, D8_INTERMEDIATE_FLAGS.join("\0").as_bytes());
    update_digest(&mut hash, D8_MERGE_FLAGS.join("\0").as_bytes());
    update_digest(&mut hash, &javac.stdout_bytes);
    update_digest(&mut hash, &javac.stderr_bytes);
    update_path_digest(&mut hash, d8)?;
    for file in [d8.parent().map(|root| root.join("lib/d8.jar"))]
        .into_iter()
        .flatten()
        .filter(|path| path.is_file())
    {
        update_path_digest(&mut hash, &file)?;
    }
    update_path_digest(&mut hash, android_jar)?;
    update_path_digest(&mut hash, base_classes)?;
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub(super) fn android_hot_module_version(
    sources: &[AndroidHotModuleSource],
    toolchain: &str,
) -> String {
    let mut hash = Sha256::new();
    update_digest(
        &mut hash,
        &dowe_components::VIEW_IR_SCHEMA_VERSION.to_le_bytes(),
    );
    update_digest(&mut hash, toolchain.as_bytes());
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

#[allow(clippy::too_many_arguments)]
pub(super) fn build_android_incremental_dex(
    project_root: &Path,
    sources: &[AndroidHotModuleSource],
    toolchain: &str,
    version: &str,
    javac: &str,
    d8: &Path,
    android_jar: &Path,
    base_classes: &Path,
    revision: Option<&DevModuleRevision>,
) -> RuntimeResult<Option<PathBuf>> {
    if !revision_is_current(revision) {
        return Ok(None);
    }
    let fingerprints = source_fingerprints(sources)?;
    let cache_root = project_root.join(".dowe/dev/android/dex-cache");
    fs::create_dir_all(&cache_root)?;
    recover_cache_entry(&cache_root, toolchain)?;
    let entry = cache_root.join(toolchain);
    let cached = load_manifest(&entry);
    let complete = cached
        .as_ref()
        .is_some_and(|manifest| cache_is_complete(&entry, manifest, toolchain));
    let mut plan = plan_incremental(&fingerprints, cached.as_ref(), complete);
    let merged = entry.join("merged").join(version).join("classes.dex");
    if plan.compile.is_empty() && plan.remove.is_empty() && merged.is_file() {
        if !revision_is_current(revision) {
            return Ok(None);
        }
        touch_cache_entry(&entry, version)?;
        prune_toolchain_caches(&cache_root, toolchain)?;
        return Ok(Some(merged));
    }

    let staging = loop {
        let staging = staging_path(&cache_root, toolchain);
        let prepared = if !plan.full_rebuild && entry.is_dir() {
            copy_cache_entry(&entry, &staging)
        } else {
            fs::create_dir_all(&staging).map_err(RuntimeError::from)
        };
        if let Err(error) = prepared {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
        let result = build_staged_cache(
            &staging,
            sources,
            &fingerprints,
            cached.as_ref(),
            &plan,
            toolchain,
            version,
            javac,
            d8,
            android_jar,
            base_classes,
            revision,
        );
        match result {
            Ok(BuildStatus::Complete) => break staging,
            Ok(BuildStatus::Superseded) => {
                let _ = fs::remove_dir_all(&staging);
                return Ok(None);
            }
            Err(failure) if should_retry_full(&plan, &failure) => {
                let _ = fs::remove_dir_all(&staging);
                if !revision_is_current(revision) {
                    return Ok(None);
                }
                plan = full_plan(&fingerprints);
            }
            Err(failure) => {
                let _ = fs::remove_dir_all(&staging);
                return Err(failure.error);
            }
        }
    };
    if !publish_cache_entry_if_current(&cache_root, toolchain, &staging, revision)? {
        let _ = fs::remove_dir_all(&staging);
        return Ok(None);
    }
    prune_toolchain_caches(&cache_root, toolchain)?;
    let merged = entry.join("merged").join(version).join("classes.dex");
    if merged.is_file() {
        Ok(Some(merged))
    } else {
        Err(RuntimeError::new(
            "Android module failed: incremental DEX cache published without merged output",
        ))
    }
}

#[allow(clippy::too_many_arguments)]
fn build_staged_cache(
    staging: &Path,
    sources: &[AndroidHotModuleSource],
    fingerprints: &BTreeMap<String, SourceFingerprint>,
    cached: Option<&CacheManifest>,
    plan: &IncrementalPlan,
    toolchain: &str,
    version: &str,
    javac: &str,
    d8: &Path,
    android_jar: &Path,
    base_classes: &Path,
    revision: Option<&DevModuleRevision>,
) -> Result<BuildStatus, StagedBuildFailure> {
    let sources_root = staging.join("sources");
    let classes_root = staging.join("classes");
    if sources_root.exists() {
        fs::remove_dir_all(&sources_root)?;
    }
    if plan.full_rebuild && classes_root.exists() {
        fs::remove_dir_all(&classes_root)?;
    }
    fs::create_dir_all(&sources_root)?;
    fs::create_dir_all(&classes_root)?;
    let source_paths = materialize_sources(&sources_root, sources)?;
    let mut states = if plan.full_rebuild {
        BTreeMap::new()
    } else {
        cached
            .map(|manifest| manifest.sources.clone())
            .unwrap_or_default()
    };
    for path in plan.remove.iter().chain(&plan.compile) {
        if let Some(previous) = states.remove(path) {
            remove_cached_classes(&classes_root, &previous)?;
        }
    }

    if !plan.compile.is_empty() {
        if !revision_is_current(revision) {
            return Ok(BuildStatus::Superseded);
        }
        let selected = source_paths
            .iter()
            .filter(|(path, _)| plan.compile.contains(path))
            .map(|(_, path)| path.clone())
            .collect::<Vec<_>>();
        let compiled_root = staging.join("new-classes");
        if compiled_root.exists() {
            fs::remove_dir_all(&compiled_root)?;
        }
        fs::create_dir_all(&compiled_root)?;
        retryable_if_incremental(
            run_javac(
                javac,
                &selected,
                &compiled_root,
                &classes_root,
                android_jar,
                base_classes,
            ),
            plan,
        )?;
        for path in &plan.compile {
            let fingerprint = fingerprints
                .get(path)
                .expect("planned Android source fingerprint");
            let classes = discover_compiled_classes(&compiled_root, Path::new(path))?;
            copy_relative_files(&compiled_root, &classes_root, &classes)?;
            states.insert(
                path.clone(),
                CachedSource {
                    digest: fingerprint.digest.clone(),
                    classes,
                    dex_key: String::new(),
                    dex_files: Vec::new(),
                },
            );
        }
        fs::remove_dir_all(compiled_root)?;
    }

    if !revision_is_current(revision) {
        return Ok(BuildStatus::Superseded);
    }
    retryable_dex(build_missing_intermediate_dex(
        staging,
        &classes_root,
        &mut states,
        &plan.compile,
        d8,
        android_jar,
        base_classes,
        toolchain,
    ))?;
    if !revision_is_current(revision) {
        return Ok(BuildStatus::Superseded);
    }
    let merged = staging.join("merged").join(version);
    if merged.exists() {
        fs::remove_dir_all(&merged)?;
    }
    fs::create_dir_all(&merged)?;
    retryable_dex(run_d8_merge(d8, android_jar, &states, staging, &merged))?;
    if !revision_is_current(revision) {
        return Ok(BuildStatus::Superseded);
    }
    if !merged.join("classes.dex").is_file() {
        return Err(StagedBuildFailure {
            error: RuntimeError::new("Android module failed: D8 merge did not produce classes.dex"),
            retry_full: true,
        });
    }
    prune_staged_outputs(staging, &states, version)?;
    let manifest = CacheManifest {
        schema: CACHE_SCHEMA,
        toolchain: toolchain.to_string(),
        sources: states,
    };
    fs::write(
        staging.join("manifest.json"),
        serde_json::to_vec(&manifest)
            .map_err(|error| RuntimeError::new(format!("Android cache failed: {error}")))?,
    )?;
    touch_cache_entry(staging, version)?;
    Ok(BuildStatus::Complete)
}

fn retryable_dex<T>(result: RuntimeResult<T>) -> Result<T, StagedBuildFailure> {
    result.map_err(|error| StagedBuildFailure {
        error,
        retry_full: true,
    })
}

fn retryable_if_incremental<T>(
    result: RuntimeResult<T>,
    plan: &IncrementalPlan,
) -> Result<T, StagedBuildFailure> {
    result.map_err(|error| StagedBuildFailure {
        error,
        retry_full: !plan.full_rebuild,
    })
}

fn should_retry_full(plan: &IncrementalPlan, failure: &StagedBuildFailure) -> bool {
    !plan.full_rebuild && failure.retry_full
}

fn run_javac(
    javac: &str,
    sources: &[PathBuf],
    output: &Path,
    cached_classes: &Path,
    android_jar: &Path,
    base_classes: &Path,
) -> RuntimeResult<()> {
    let classpath = env::join_paths([android_jar, base_classes, cached_classes])
        .map_err(|error| RuntimeError::new(format!("Android module classpath failed: {error}")))?
        .to_string_lossy()
        .to_string();
    let mut args = JAVAC_FLAGS
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    args.extend([
        "-classpath".to_string(),
        classpath,
        "-d".to_string(),
        output.to_string_lossy().to_string(),
    ]);
    args.extend(
        sources
            .iter()
            .map(|source| source.to_string_lossy().to_string()),
    );
    run_required(
        DevTarget::Android,
        SpawnConfig::new(javac, args).with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_missing_intermediate_dex(
    staging: &Path,
    classes_root: &Path,
    states: &mut BTreeMap<String, CachedSource>,
    changed: &[String],
    d8: &Path,
    android_jar: &Path,
    base_classes: &Path,
    toolchain: &str,
) -> RuntimeResult<()> {
    let mut missing = Vec::new();
    let mut program_classes = BTreeSet::new();
    for path in changed {
        let state = states.get_mut(path).expect("compiled Android source state");
        state.dex_key = dex_key(toolchain, classes_root, &state.classes)?;
        let dex_root = staging.join("dex").join(&state.dex_key);
        let cached_files = collect_relative_files(&dex_root, "dex")?;
        if cached_files.is_empty() {
            program_classes.extend(state.classes.iter().cloned());
            missing.push(path.clone());
        } else {
            state.dex_files = cached_files
                .into_iter()
                .map(|path| {
                    Path::new("dex")
                        .join(&state.dex_key)
                        .join(path)
                        .to_string_lossy()
                        .to_string()
                })
                .collect();
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    let classpath_root = staging.join("d8-classpath");
    let output_root = staging.join("d8-output");
    if classpath_root.exists() {
        fs::remove_dir_all(&classpath_root)?;
    }
    if output_root.exists() {
        fs::remove_dir_all(&output_root)?;
    }
    copy_tree_excluding(classes_root, &classpath_root, &program_classes)?;
    fs::create_dir_all(&output_root)?;
    let mut args = D8_INTERMEDIATE_FLAGS
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    args.extend([
        "--lib".to_string(),
        android_jar.to_string_lossy().to_string(),
        "--classpath".to_string(),
        base_classes.to_string_lossy().to_string(),
        "--classpath".to_string(),
        classpath_root.to_string_lossy().to_string(),
        "--output".to_string(),
        output_root.to_string_lossy().to_string(),
    ]);
    args.extend(
        program_classes
            .iter()
            .map(|path| classes_root.join(path).to_string_lossy().to_string()),
    );
    run_required(
        DevTarget::Android,
        SpawnConfig::new(d8.to_string_lossy().to_string(), args)
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    for path in missing {
        let state = states
            .get_mut(&path)
            .expect("compiled Android source state");
        let dex_files = discover_intermediate_dex(&output_root, &state.classes)?;
        let dex_root = staging.join("dex").join(&state.dex_key);
        fs::create_dir_all(&dex_root)?;
        copy_relative_files(&output_root, &dex_root, &dex_files)?;
        state.dex_files = dex_files
            .into_iter()
            .map(|path| {
                Path::new("dex")
                    .join(&state.dex_key)
                    .join(path)
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
    }
    fs::remove_dir_all(classpath_root)?;
    fs::remove_dir_all(output_root)?;
    Ok(())
}

fn run_d8_merge(
    d8: &Path,
    android_jar: &Path,
    states: &BTreeMap<String, CachedSource>,
    staging: &Path,
    output: &Path,
) -> RuntimeResult<()> {
    let mut inputs = states
        .values()
        .flat_map(|state| state.dex_files.iter())
        .map(|path| staging.join(path))
        .collect::<Vec<_>>();
    inputs.sort();
    if inputs.is_empty() || inputs.iter().any(|path| !path.is_file()) {
        return Err(RuntimeError::new(
            "Android module failed: incomplete intermediate DEX cache",
        ));
    }
    let mut args = D8_MERGE_FLAGS
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    args.extend([
        "--lib".to_string(),
        android_jar.to_string_lossy().to_string(),
        "--output".to_string(),
        output.to_string_lossy().to_string(),
    ]);
    args.extend(inputs.iter().map(|path| path.to_string_lossy().to_string()));
    run_required(
        DevTarget::Android,
        SpawnConfig::new(d8.to_string_lossy().to_string(), args)
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    Ok(())
}

fn source_fingerprints(
    sources: &[AndroidHotModuleSource],
) -> RuntimeResult<BTreeMap<String, SourceFingerprint>> {
    let mut fingerprints = BTreeMap::new();
    for source in sources {
        validate_relative_path(&source.relative_path, "Android generated source")?;
        let path = source.relative_path.to_string_lossy().replace('\\', "/");
        let name = source
            .relative_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let fingerprint = SourceFingerprint {
            digest: digest_bytes(source.content.as_bytes()),
            shard: is_incremental_shard(name),
        };
        if fingerprints.insert(path, fingerprint).is_some() {
            return Err(RuntimeError::new(
                "Android module failed: duplicate generated source",
            ));
        }
    }
    if !fingerprints.contains_key(CORE_SOURCE) {
        return Err(RuntimeError::new(
            "Android module failed: missing DoweDevActivity.java",
        ));
    }
    Ok(fingerprints)
}

fn plan_incremental(
    current: &BTreeMap<String, SourceFingerprint>,
    cached: Option<&CacheManifest>,
    cache_complete: bool,
) -> IncrementalPlan {
    let Some(cached) = cached.filter(|_| cache_complete) else {
        return full_plan(current);
    };
    let mut compile = current
        .iter()
        .filter(|(path, source)| {
            cached
                .sources
                .get(*path)
                .is_none_or(|previous| previous.digest != source.digest)
        })
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    let mut remove = cached
        .sources
        .keys()
        .filter(|path| !current.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    compile.sort();
    remove.sort();
    let shared_changed = compile
        .iter()
        .any(|path| current.get(path).is_some_and(|source| !source.shard))
        || remove
            .iter()
            .any(|path| !path.rsplit('/').next().is_some_and(is_incremental_shard));
    if shared_changed {
        full_plan(current)
    } else {
        IncrementalPlan {
            full_rebuild: false,
            compile,
            remove,
        }
    }
}

fn is_incremental_shard(name: &str) -> bool {
    name.ends_with(".java") && (name.starts_with(ROUTE_PREFIX) || name.starts_with(LAYOUT_PREFIX))
}

fn full_plan(current: &BTreeMap<String, SourceFingerprint>) -> IncrementalPlan {
    IncrementalPlan {
        full_rebuild: true,
        compile: current.keys().cloned().collect(),
        remove: Vec::new(),
    }
}

fn cache_is_complete(entry: &Path, manifest: &CacheManifest, toolchain: &str) -> bool {
    manifest.schema == CACHE_SCHEMA
        && manifest.toolchain == toolchain
        && !manifest.sources.is_empty()
        && manifest.sources.values().all(|source| {
            !source.classes.is_empty()
                && !source.dex_files.is_empty()
                && source
                    .classes
                    .iter()
                    .all(|path| safe_file(entry.join("classes"), path))
                && source
                    .dex_files
                    .iter()
                    .all(|path| safe_file(entry.to_path_buf(), path))
        })
}

fn safe_file(root: PathBuf, relative: &str) -> bool {
    let path = Path::new(relative);
    validate_relative_path(path, "Android cache path").is_ok() && root.join(path).is_file()
}

fn load_manifest(entry: &Path) -> Option<CacheManifest> {
    fs::read(entry.join("manifest.json"))
        .ok()
        .and_then(|contents| serde_json::from_slice(&contents).ok())
}

fn materialize_sources(
    root: &Path,
    sources: &[AndroidHotModuleSource],
) -> RuntimeResult<BTreeMap<String, PathBuf>> {
    let mut paths = BTreeMap::new();
    for source in sources {
        let relative = source.relative_path.to_string_lossy().replace('\\', "/");
        let path = root.join(&source.relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, &source.content)?;
        paths.insert(relative, path);
    }
    Ok(paths)
}

fn discover_compiled_classes(root: &Path, source: &Path) -> RuntimeResult<Vec<String>> {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| RuntimeError::new("Android module failed: invalid Java source name"))?;
    let package = root.join(GENERATED_PACKAGE);
    let mut classes = fs::read_dir(&package)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            (name == format!("{stem}.class")
                || name.starts_with(&format!("{stem}$")) && name.ends_with(".class"))
            .then(|| {
                Path::new(GENERATED_PACKAGE)
                    .join(name)
                    .to_string_lossy()
                    .to_string()
            })
        })
        .collect::<Vec<_>>();
    classes.sort();
    if classes.is_empty() {
        return Err(RuntimeError::new(format!(
            "Android module failed: javac produced no classes for {}",
            source.display()
        )));
    }
    Ok(classes)
}

fn discover_intermediate_dex(root: &Path, classes: &[String]) -> RuntimeResult<Vec<String>> {
    let stems = classes
        .iter()
        .filter_map(|path| Path::new(path).file_stem())
        .filter_map(|value| value.to_str())
        .collect::<BTreeSet<_>>();
    let mut files = collect_relative_files(root, "dex")?
        .into_iter()
        .filter(|path| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .is_some_and(|stem| stems.contains(stem))
        })
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    files.sort();
    if files.is_empty() {
        return Err(RuntimeError::new(
            "Android module failed: D8 produced no intermediate shard",
        ));
    }
    Ok(files)
}

fn remove_cached_classes(root: &Path, source: &CachedSource) -> RuntimeResult<()> {
    for relative in &source.classes {
        validate_relative_path(Path::new(relative), "Android cached class")?;
        let path = root.join(relative);
        if path.is_file() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn dex_key(toolchain: &str, classes_root: &Path, classes: &[String]) -> RuntimeResult<String> {
    let mut hash = Sha256::new();
    update_digest(&mut hash, toolchain.as_bytes());
    for relative in classes {
        update_digest(&mut hash, relative.as_bytes());
        update_digest(&mut hash, &fs::read(classes_root.join(relative))?);
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn prune_staged_outputs(
    staging: &Path,
    states: &BTreeMap<String, CachedSource>,
    version: &str,
) -> RuntimeResult<()> {
    let active_dex = states
        .values()
        .map(|source| source.dex_key.as_str())
        .collect::<BTreeSet<_>>();
    let dex_root = staging.join("dex");
    if dex_root.is_dir() {
        for entry in fs::read_dir(&dex_root)? {
            let entry = entry?;
            if entry.path().is_dir()
                && !active_dex.contains(entry.file_name().to_string_lossy().as_ref())
            {
                fs::remove_dir_all(entry.path())?;
            }
        }
    }
    let merged_root = staging.join("merged");
    if merged_root.is_dir() {
        for entry in fs::read_dir(&merged_root)? {
            let entry = entry?;
            if entry.path().is_dir() && entry.file_name().to_string_lossy() != version {
                fs::remove_dir_all(entry.path())?;
            }
        }
    }
    Ok(())
}

fn recover_cache_entry(root: &Path, key: &str) -> RuntimeResult<()> {
    let entry = root.join(key);
    let previous = root.join(format!(".{key}.previous"));
    if previous.exists() {
        if entry.exists() {
            fs::remove_dir_all(&previous)?;
        } else {
            fs::rename(&previous, &entry)?;
        }
    }
    for item in fs::read_dir(root)? {
        let item = item?;
        let name = item.file_name().to_string_lossy().to_string();
        if item.path().is_dir() && name.starts_with(&format!(".{key}.")) && name.ends_with(".tmp") {
            fs::remove_dir_all(item.path())?;
        }
    }
    Ok(())
}

fn publish_cache_entry(root: &Path, key: &str, staging: &Path) -> RuntimeResult<()> {
    let entry = root.join(key);
    let previous = root.join(format!(".{key}.previous"));
    if previous.exists() {
        fs::remove_dir_all(&previous)?;
    }
    if entry.exists() {
        fs::rename(&entry, &previous)?;
    }
    if let Err(error) = fs::rename(staging, &entry) {
        if previous.exists() && !entry.exists() {
            let _ = fs::rename(&previous, &entry);
        }
        return Err(error.into());
    }
    if previous.exists() {
        fs::remove_dir_all(previous)?;
    }
    Ok(())
}

fn publish_cache_entry_if_current(
    root: &Path,
    key: &str,
    staging: &Path,
    revision: Option<&DevModuleRevision>,
) -> RuntimeResult<bool> {
    let Some(revision) = revision else {
        publish_cache_entry(root, key, staging)?;
        return Ok(true);
    };
    let Some(result) = revision.run_if_current(|| publish_cache_entry(root, key, staging)) else {
        return Ok(false);
    };
    result?;
    Ok(true)
}

fn revision_is_current(revision: Option<&DevModuleRevision>) -> bool {
    revision.is_none_or(DevModuleRevision::is_current)
}

fn prune_toolchain_caches(root: &Path, active: &str) -> RuntimeResult<()> {
    let entries = fs::read_dir(root)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let key = entry.file_name().to_string_lossy().to_string();
            is_cache_key(&key).then(|| CacheUse {
                last_used: fs::metadata(entry.path().join("last-used"))
                    .and_then(|metadata| metadata.modified())
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_nanos())
                    .unwrap_or_default(),
                key,
            })
        })
        .collect::<Vec<_>>();
    for key in cache_keys_to_prune(&entries, active, RETAINED_INACTIVE_TOOLCHAINS) {
        fs::remove_dir_all(root.join(key))?;
    }
    Ok(())
}

fn cache_keys_to_prune(entries: &[CacheUse], active: &str, retained: usize) -> Vec<String> {
    let mut inactive = entries
        .iter()
        .filter(|entry| entry.key != active)
        .cloned()
        .collect::<Vec<_>>();
    inactive.sort_by(|left, right| {
        right
            .last_used
            .cmp(&left.last_used)
            .then_with(|| right.key.cmp(&left.key))
    });
    inactive
        .into_iter()
        .skip(retained)
        .map(|entry| entry.key)
        .collect()
}

fn touch_cache_entry(entry: &Path, version: &str) -> RuntimeResult<()> {
    fs::write(entry.join("last-used"), version.as_bytes())?;
    Ok(())
}

fn staging_path(root: &Path, key: &str) -> PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    root.join(format!(".{key}.{}.{}.tmp", std::process::id(), id))
}

fn is_cache_key(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_relative_path(path: &Path, label: &str) -> RuntimeResult<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(RuntimeError::new(format!(
            "{label} must use a safe relative path: {}",
            path.display()
        )));
    }
    Ok(())
}

fn copy_relative_files(source: &Path, target: &Path, files: &[String]) -> RuntimeResult<()> {
    for relative in files {
        validate_relative_path(Path::new(relative), "Android cache file")?;
        let destination = target.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source.join(relative), destination)?;
    }
    Ok(())
}

fn copy_tree(source: &Path, target: &Path) -> RuntimeResult<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let destination = target.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &destination)?;
        } else if fs::hard_link(entry.path(), &destination).is_err() {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

fn copy_cache_entry(source: &Path, target: &Path) -> RuntimeResult<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "manifest.json" || name == "last-used" {
            continue;
        }
        let destination = target.join(&name);
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &destination)?;
        } else if fs::hard_link(entry.path(), &destination).is_err() {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

fn copy_tree_excluding(
    source: &Path,
    target: &Path,
    excluded: &BTreeSet<String>,
) -> RuntimeResult<()> {
    fs::create_dir_all(target)?;
    for relative in collect_relative_files(source, "class")? {
        let relative_string = relative.to_string_lossy().to_string();
        if excluded.contains(&relative_string) {
            continue;
        }
        let destination = target.join(&relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        if fs::hard_link(source.join(&relative), &destination).is_err() {
            fs::copy(source.join(&relative), destination)?;
        }
    }
    Ok(())
}

fn collect_relative_files(root: &Path, extension: &str) -> RuntimeResult<Vec<PathBuf>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_relative_files_inner(root, root, extension, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_relative_files_inner(
    root: &Path,
    current: &Path,
    extension: &str,
    files: &mut Vec<PathBuf>,
) -> RuntimeResult<()> {
    for entry in fs::read_dir(current)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_relative_files_inner(root, &path, extension, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            files.push(
                path.strip_prefix(root)
                    .expect("collected Android cache file")
                    .to_path_buf(),
            );
        }
    }
    Ok(())
}

fn update_path_digest(hash: &mut Sha256, path: &Path) -> RuntimeResult<()> {
    let canonical = path.canonicalize()?;
    update_digest(hash, canonical.to_string_lossy().as_bytes());
    if canonical.is_dir() {
        for relative in collect_relative_files(&canonical, "class")? {
            update_digest(hash, relative.to_string_lossy().as_bytes());
            update_digest(hash, &fs::read(canonical.join(relative))?);
        }
    } else {
        update_digest(hash, &fs::read(canonical)?);
    }
    Ok(())
}

fn digest_bytes(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn update_digest(hash: &mut Sha256, value: &[u8]) {
    hash.update(value.len().to_le_bytes());
    hash.update(value);
}

#[cfg(test)]
#[path = "android_incremental_tests.rs"]
mod tests;
