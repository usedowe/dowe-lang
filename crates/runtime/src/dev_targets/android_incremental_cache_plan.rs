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

