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

