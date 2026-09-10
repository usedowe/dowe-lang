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

