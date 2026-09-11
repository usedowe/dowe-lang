pub(super) fn build_hot_module_if_current(
    root: &Path,
    files: &[GeneratedFile],
    revision: &DevModuleRevision,
) -> RuntimeResult<Option<PublishedDevModule>> {
    build_hot_module_with_revision(root, files, Some(revision))
}

fn build_hot_module_with_revision(
    root: &Path,
    files: &[GeneratedFile],
    revision: Option<&DevModuleRevision>,
) -> RuntimeResult<Option<PublishedDevModule>> {
    if HostOs::current() != HostOs::Macos {
        return Err(RuntimeError::new("target `ios` is only available on macOS"));
    }
    if !ios_revision_is_current(revision) {
        return Ok(None);
    }
    let target = ios_simulator_target();
    let toolchain_signature = ios_toolchain_signature()?;
    if !ios_revision_is_current(revision) {
        return Ok(None);
    }
    let snapshot =
        IosHotModuleSnapshot::from_generated_files(files, &target, &toolchain_signature)?;
    let version = snapshot.version.clone();
    let published = root
        .join(".dowe/dev/modules/ios")
        .join(format!("{version}.dylib"));
    if published.is_file() {
        return publish_ios_module(root, &version, &published, revision);
    }
    let workspace = IosIncrementalWorkspace::prepare(root, &snapshot)?;
    let build_result = build_hot_module_artifact(&workspace, &target, revision);
    let built = match build_result {
        Ok(built) => built,
        Err(error) => {
            workspace.remove_linked_module();
            return Err(error);
        }
    };
    if !built {
        workspace.remove_linked_module();
        return Ok(None);
    }
    if let Err(error) = ensure_file(workspace.linked_module.clone(), DevTarget::Ios) {
        workspace.remove_linked_module();
        return Err(error);
    }
    let result = publish_ios_module(root, &version, &workspace.linked_module, revision);
    workspace.remove_linked_module();
    result
}

fn ios_revision_is_current(revision: Option<&DevModuleRevision>) -> bool {
    revision.map(DevModuleRevision::is_current).unwrap_or(true)
}

fn publish_ios_module(
    root: &Path,
    version: &str,
    module: &Path,
    revision: Option<&DevModuleRevision>,
) -> RuntimeResult<Option<PublishedDevModule>> {
    match revision {
        Some(revision) => {
            publish_dev_module_if_current(root, "ios", version, "dylib", module, revision)
        }
        None => publish_dev_module(root, "ios", version, "dylib", module).map(Some),
    }
}

fn build_hot_module_artifact(
    workspace: &IosIncrementalWorkspace,
    target: &str,
    revision: Option<&DevModuleRevision>,
) -> RuntimeResult<bool> {
    run_ios_hot_module_pipeline(
        || ios_revision_is_current(revision),
        || compile_hot_module_once(workspace, target),
        || link_hot_module_once(workspace, target),
        || {
            workspace.remove_linked_module();
            workspace.reset_outputs()
        },
    )
}

fn compile_hot_module_once(
    workspace: &IosIncrementalWorkspace,
    target: &str,
) -> RuntimeResult<bool> {
    let result = run_ios_required(
        SpawnConfig::new(
            "xcrun",
            ios_hot_module_compile_args(
                &workspace.source_files(),
                &workspace.output_map,
                target.to_string(),
                ios_swift_job_count(),
            ),
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    );
    if let Err(error) = result {
        return Err(error);
    }
    Ok(workspace.is_complete())
}

fn link_hot_module_once(workspace: &IosIncrementalWorkspace, target: &str) -> RuntimeResult<()> {
    run_ios_required(
        SpawnConfig::new(
            "xcrun",
            ios_hot_module_link_args(
                &workspace.object_files(),
                &workspace.linked_module,
                target.to_string(),
            ),
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )
    .map(|_| ())
}

fn run_ios_hot_module_pipeline<Current, Compile, Link, Reset>(
    is_current: Current,
    mut compile: Compile,
    mut link: Link,
    mut reset: Reset,
) -> RuntimeResult<bool>
where
    Current: Fn() -> bool,
    Compile: FnMut() -> RuntimeResult<bool>,
    Link: FnMut() -> RuntimeResult<()>,
    Reset: FnMut() -> RuntimeResult<()>,
{
    if !is_current() {
        return Ok(false);
    }
    let recovery_required = match compile()? {
        true => {
            if !is_current() {
                return Ok(false);
            }
            link().is_err()
        }
        false => true,
    };
    if !recovery_required {
        return Ok(true);
    }
    if !is_current() {
        return Ok(false);
    }
    reset()?;
    if !is_current() {
        return Ok(false);
    }
    let recovery_complete = compile()?;
    if !is_current() {
        return Ok(false);
    }
    if !recovery_complete {
        return Err(RuntimeError::new(
            "iOS module failed: full compiler recovery did not produce every object and dependency file",
        ));
    }
    link()?;
    Ok(true)
}

fn ios_hot_module_compile_args(
    sources: &[String],
    output_map: &Path,
    target: String,
    jobs: usize,
) -> Vec<String> {
    let mut args = vec![
        "--sdk".to_string(),
        "iphonesimulator".to_string(),
        "swiftc".to_string(),
        "-parse-as-library".to_string(),
        "-incremental".to_string(),
        "-enable-incremental-file-hashing".to_string(),
        "-enable-batch-mode".to_string(),
        "-driver-batch-size-limit".to_string(),
        "1".to_string(),
        "-j".to_string(),
        jobs.to_string(),
        "-target".to_string(),
        target,
        "-module-name".to_string(),
        IOS_INCREMENTAL_MODULE_NAME.to_string(),
        "-c".to_string(),
    ];
    args.extend(sources.iter().cloned());
    args.extend([
        "-output-file-map".to_string(),
        output_map.to_string_lossy().to_string(),
    ]);
    args
}

fn ios_hot_module_link_args(
    object_files: &[PathBuf],
    output: &Path,
    target: String,
) -> Vec<String> {
    let mut args = vec![
        "--sdk".to_string(),
        "iphonesimulator".to_string(),
        "swiftc".to_string(),
        "-emit-library".to_string(),
        "-target".to_string(),
        target,
    ];
    args.extend(
        object_files
            .iter()
            .map(|path| path.to_string_lossy().to_string()),
    );
    args.extend(["-o".to_string(), output.to_string_lossy().to_string()]);
    args
}
