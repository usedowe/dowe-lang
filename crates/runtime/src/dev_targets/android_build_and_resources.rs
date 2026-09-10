struct AndroidTools {
    emulator: PathBuf,
    adb: PathBuf,
    aapt2: PathBuf,
    d8: PathBuf,
    apksigner: PathBuf,
    zipalign: PathBuf,
    android_jar: PathBuf,
}

fn build_android_apk(
    project_root: &Path,
    android_root: &Path,
    tools: &AndroidTools,
) -> RuntimeResult<PathBuf> {
    let manifest = ensure_file(
        android_root.join("dev/AndroidManifest.xml"),
        DevTarget::Android,
    )?;
    let source = ensure_file(
        android_root.join("dev/src/dev/dowe/generated/DoweDevHostActivity.java"),
        DevTarget::Android,
    )?;
    let build = project_root.join(".dowe/dev/android/host");
    if build.exists() {
        fs::remove_dir_all(&build)?;
    }
    let generated = build.join("gen");
    let classes = build.join("classes");
    let dex = build.join("dex");
    let resources = android_root.join("app/src/main/res");
    let assets = android_root.join("app/src/main/assets");
    fs::create_dir_all(&assets)?;
    fs::create_dir_all(&generated)?;
    fs::create_dir_all(&classes)?;
    fs::create_dir_all(&dex)?;
    let base_apk = build.join("base.apk");
    let compiled_resources = build.join("resources.zip");
    let unsigned_apk = build.join("unsigned.apk");
    let aligned_apk = build.join("aligned.apk");
    let signed_apk = build.join("DoweDev.apk");
    let keystore = android_root.join("build/debug.keystore");
    if let Some(parent) = keystore.parent() {
        fs::create_dir_all(parent)?;
    }

    let resource_inputs = compile_android_resources(&resources, &compiled_resources, tools)?;
    run_required(
        DevTarget::Android,
        SpawnConfig::new(tools.aapt2.to_string_lossy().to_string(), {
            let mut args = vec![
                "link".to_string(),
                "-o".to_string(),
                base_apk.to_string_lossy().to_string(),
                "-I".to_string(),
                tools.android_jar.to_string_lossy().to_string(),
                "--manifest".to_string(),
                manifest.to_string_lossy().to_string(),
                "-A".to_string(),
                assets.to_string_lossy().to_string(),
                "--java".to_string(),
                generated.to_string_lossy().to_string(),
            ];
            args.extend(resource_inputs);
            args
        })
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    run_required(
        DevTarget::Android,
        SpawnConfig::new(
            "javac",
            android_javac_args(&tools.android_jar, &classes, &source, &generated)?,
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    run_required(
        DevTarget::Android,
        SpawnConfig::new(tools.d8.to_string_lossy().to_string(), {
            let mut args = vec![
                "--lib".to_string(),
                tools.android_jar.to_string_lossy().to_string(),
                "--output".to_string(),
                dex.to_string_lossy().to_string(),
            ];
            args.extend(compiled_activity_classes(&classes)?);
            args
        })
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    fs::copy(&base_apk, &unsigned_apk)?;
    run_required(
        DevTarget::Android,
        SpawnConfig::new(
            "jar",
            [
                "uf".to_string(),
                unsigned_apk.to_string_lossy().to_string(),
                "-C".to_string(),
                dex.to_string_lossy().to_string(),
                "classes.dex".to_string(),
            ],
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    create_debug_keystore(&keystore)?;
    run_required(
        DevTarget::Android,
        SpawnConfig::new(
            tools.zipalign.to_string_lossy().to_string(),
            [
                "-f".to_string(),
                "4".to_string(),
                unsigned_apk.to_string_lossy().to_string(),
                aligned_apk.to_string_lossy().to_string(),
            ],
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    run_required(
        DevTarget::Android,
        SpawnConfig::new(
            tools.apksigner.to_string_lossy().to_string(),
            [
                "sign".to_string(),
                "--ks".to_string(),
                keystore.to_string_lossy().to_string(),
                "--ks-pass".to_string(),
                "pass:android".to_string(),
                "--key-pass".to_string(),
                "pass:android".to_string(),
                "--min-sdk-version".to_string(),
                "26".to_string(),
                "--out".to_string(),
                signed_apk.to_string_lossy().to_string(),
                aligned_apk.to_string_lossy().to_string(),
            ],
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    Ok(signed_apk)
}

pub(super) fn build_hot_module_if_current(
    root: &Path,
    files: &[GeneratedFile],
    revision: &DevModuleRevision,
) -> RuntimeResult<Option<PublishedDevModule>> {
    let sdk = android_sdk_root()?;
    let tools = android_tools(&sdk)?;
    build_hot_module_with_tools_and_revision(root, files, &tools, Some(revision))
}

fn build_hot_module_with_tools(
    project: &CompiledProject,
    tools: &AndroidTools,
) -> RuntimeResult<PublishedDevModule> {
    build_hot_module_with_tools_and_revision(&project.root, &project.apps.files, tools, None)?
        .ok_or_else(|| RuntimeError::new("unconditional Android module publication was skipped"))
}

fn build_hot_module_with_tools_and_revision(
    root: &Path,
    files: &[GeneratedFile],
    tools: &AndroidTools,
    revision: Option<&DevModuleRevision>,
) -> RuntimeResult<Option<PublishedDevModule>> {
    if revision.is_some_and(|revision| !revision.is_current()) {
        return Ok(None);
    }
    let sources = android_hot_module_sources(files)?;
    let base_classes = ensure_dir(
        root.join(".dowe/dev/android/host/classes"),
        DevTarget::Android,
    )?;
    let toolchain = android_toolchain_fingerprint(&tools.d8, &tools.android_jar, &base_classes)?;
    if revision.is_some_and(|revision| !revision.is_current()) {
        return Ok(None);
    }
    let version = android_hot_module_version(&sources, &toolchain);
    let published = root
        .join(".dowe/dev/modules/android")
        .join(format!("{version}.dex"));
    if published.is_file() {
        return publish_android_module(root, &version, &published, revision);
    }
    let module = build_android_incremental_dex(
        root,
        &sources,
        &toolchain,
        &version,
        "javac",
        &tools.d8,
        &tools.android_jar,
        &base_classes,
        revision,
    )?;
    let Some(module) = module else {
        return Ok(None);
    };
    publish_android_module(root, &version, &module, revision)
}

fn android_hot_module_sources(
    files: &[GeneratedFile],
) -> RuntimeResult<Vec<AndroidHotModuleSource>> {
    let root = Path::new("apps/android/dev/src/dev/dowe/generated");
    let mut sources = files
        .iter()
        .filter(|file| file.target == "android")
        .filter_map(|file| {
            let relative = file.relative_path.strip_prefix(root).ok()?;
            (relative.extension().and_then(|value| value.to_str()) == Some("java")
                && relative.file_name().and_then(|value| value.to_str())
                    != Some("DoweDevHostActivity.java"))
            .then(|| AndroidHotModuleSource {
                relative_path: relative.to_path_buf(),
                content: file.content.clone(),
            })
        })
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    if sources.is_empty() {
        return Err(RuntimeError::new(
            "Android module failed: missing generated hot module sources",
        ));
    }
    Ok(sources)
}

fn publish_android_module(
    root: &Path,
    version: &str,
    module: &Path,
    revision: Option<&DevModuleRevision>,
) -> RuntimeResult<Option<PublishedDevModule>> {
    match revision {
        Some(revision) => {
            publish_dev_module_if_current(root, "android", version, "dex", module, revision)
        }
        None => publish_dev_module(root, "android", version, "dex", module).map(Some),
    }
}

fn compile_android_resources(
    resources: &Path,
    output: &Path,
    tools: &AndroidTools,
) -> RuntimeResult<Vec<String>> {
    if !resources.is_dir() {
        return Ok(Vec::new());
    }
    run_required(
        DevTarget::Android,
        SpawnConfig::new(
            tools.aapt2.to_string_lossy().to_string(),
            [
                "compile".to_string(),
                "--dir".to_string(),
                resources.to_string_lossy().to_string(),
                "-o".to_string(),
                output.to_string_lossy().to_string(),
            ],
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    Ok(vec![output.to_string_lossy().to_string()])
}

fn android_java_sources(activity: &Path, generated: &Path) -> RuntimeResult<Vec<String>> {
    let mut sources = Vec::new();
    collect_java_sources(generated, &mut sources)?;
    sources.push(activity.to_path_buf());
    sources.sort();
    Ok(sources
        .into_iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect())
}

fn android_javac_args(
    android_jar: &Path,
    classes: &Path,
    activity: &Path,
    generated: &Path,
) -> RuntimeResult<Vec<String>> {
    let mut args = vec![
        "-g:none".to_string(),
        "-proc:none".to_string(),
        "-classpath".to_string(),
        android_jar.to_string_lossy().to_string(),
        "-d".to_string(),
        classes.to_string_lossy().to_string(),
    ];
    args.extend(android_java_sources(activity, generated)?);
    Ok(args)
}

fn collect_java_sources(root: &Path, sources: &mut Vec<PathBuf>) -> RuntimeResult<()> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_java_sources(&path, sources)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("java") {
            sources.push(path);
        }
    }
    Ok(())
}

fn compiled_activity_classes(classes: &Path) -> RuntimeResult<Vec<String>> {
    let mut paths = Vec::new();
    collect_class_files(classes, &mut paths)?;
    let mut paths = paths
        .into_iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    paths.sort();
    if paths.is_empty() {
        return Err(RuntimeError::new(format!(
            "Android app target failed: missing compiled classes under {}",
            classes.display()
        )));
    }
    Ok(paths)
}

fn collect_class_files(root: &Path, paths: &mut Vec<PathBuf>) -> RuntimeResult<()> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_class_files(&path, paths)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("class") {
            paths.push(path);
        }
    }
    Ok(())
}

fn create_debug_keystore(path: &Path) -> RuntimeResult<()> {
    if path.exists() {
        return Ok(());
    }

    run_required(
        DevTarget::Android,
        SpawnConfig::new(
            "keytool",
            [
                "-genkeypair".to_string(),
                "-keystore".to_string(),
                path.to_string_lossy().to_string(),
                "-storepass".to_string(),
                "android".to_string(),
                "-keypass".to_string(),
                "android".to_string(),
                "-alias".to_string(),
                "androiddebugkey".to_string(),
                "-keyalg".to_string(),
                "RSA".to_string(),
                "-keysize".to_string(),
                "2048".to_string(),
                "-validity".to_string(),
                "10000".to_string(),
                "-dname".to_string(),
                "CN=Android Debug,O=Android,C=US".to_string(),
            ],
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    Ok(())
}


