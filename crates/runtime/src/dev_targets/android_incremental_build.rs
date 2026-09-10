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

