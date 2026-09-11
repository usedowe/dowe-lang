pub(super) fn start(
    project: &CompiledProject,
    selection: Option<IosSimulatorSelection>,
    quit_simulators_on_exit: bool,
    dev_origin: Option<&str>,
) -> RuntimeResult<ExternalTargetStartup> {
    if HostOs::current() != HostOs::Macos {
        return Err(RuntimeError::new("target `ios` is only available on macOS"));
    }

    let ios_root = ensure_dir(project.root.join(".dowe/apps/ios"), DevTarget::Ios)?;
    print_target_starting(DevTarget::Ios);
    let simulator = prepare_ios_simulator(selection)?;
    let cleanup_configs = ios_cleanup_commands(&simulator.udid, quit_simulators_on_exit)
        .into_iter()
        .map(super::ios_developer_dir::prepare_command)
        .collect::<RuntimeResult<Vec<_>>>()?;
    if let Err(error) = launch_ios_app(project, &ios_root, &simulator, dev_origin) {
        run_ios_cleanup_configs(&cleanup_configs);
        return Err(error);
    }
    print_target_started(DevTarget::Ios);
    let mut startup = ExternalTargetStartup::default();
    for config in cleanup_configs {
        startup
            .cleanups
            .push(cleanup_command(DevTarget::Ios, config));
    }
    Ok(startup)
}

fn launch_ios_app(
    project: &CompiledProject,
    ios_root: &Path,
    simulator: &IosSimulator,
    dev_origin: Option<&str>,
) -> RuntimeResult<()> {
    let app_bundle = build_ios_app(&project.root, ios_root)?;
    if simulator.boot_requested {
        wait_ios_simulator_boot(&simulator.udid)?;
    }
    run_ios_required(
        ios_install_config(&simulator.udid, &app_bundle)
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    let launch_result = run_ios_required(
        ios_launch_config(&simulator.udid, &project.app_config.bundle, dev_origin)
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    );
    if launch_result.is_ok() {
        open_ios_simulator()?;
    }
    launch_result.map(|_| ())
}

fn ios_install_config(udid: &str, app_bundle: &Path) -> SpawnConfig {
    SpawnConfig::new(
        "xcrun",
        [
            "simctl".to_string(),
            "install".to_string(),
            udid.to_string(),
            app_bundle.to_string_lossy().to_string(),
        ],
    )
}

fn ios_launch_config(udid: &str, bundle: &str, dev_origin: Option<&str>) -> SpawnConfig {
    let mut args = vec![
        "simctl".to_string(),
        "launch".to_string(),
        udid.to_string(),
        bundle.to_string(),
    ];
    if let Some(dev_origin) = dev_origin {
        args.push("--dowe-dev-server".to_string());
        args.push(dev_origin.to_string());
    }
    SpawnConfig::new("xcrun", args)
}

fn ios_open_simulator_config() -> SpawnConfig {
    SpawnConfig::new("open", ["-a", "Simulator"])
}

fn open_ios_simulator() -> RuntimeResult<()> {
    run_ios_required(
        ios_open_simulator_config().with_options(quiet_command_options(None, StreamMode::Ignore)),
    )
    .map(|_| ())
}

fn build_ios_app(project_root: &Path, ios_root: &Path) -> RuntimeResult<PathBuf> {
    let host_source = ensure_file(ios_root.join("dev/DoweIosDevHost.swift"), DevTarget::Ios)?;
    let plist = ensure_file(ios_root.join("Info.plist"), DevTarget::Ios)?;
    let target = ios_simulator_target();
    let cache_key = ios_app_cache_key(ios_root, &target, &ios_toolchain_signature()?)?;
    if let Some(bundle) = cached_ios_app(project_root, &cache_key) {
        prune_ios_app_cache(project_root, &cache_key)?;
        return Ok(bundle);
    }
    let build_root = ios_build_root(project_root);
    let source_root = build_root.join("src");
    let objects_root = build_root.join("objects");
    let bundle = build_root.join("DoweIosApp.app");
    if build_root.exists() {
        fs::remove_dir_all(&build_root)?;
    }
    fs::create_dir_all(&source_root)?;
    fs::create_dir_all(&objects_root)?;
    fs::create_dir_all(&bundle)?;
    let host_file = "DoweIosDevHost.swift".to_string();
    fs::copy(host_source, source_root.join(&host_file))?;
    let swift_files = vec![host_file];
    let object_files = ios_swift_object_files(&swift_files, &objects_root);
    let output_map = build_root.join("output-file-map.json");
    let output_map_content = ios_swift_output_map(&swift_files, &object_files);
    let swift_jobs = ios_swift_job_count();
    fs::write(
        &output_map,
        serde_json::to_vec(&output_map_content)
            .map_err(|error| RuntimeError::new(format!("iOS app target failed: {error}")))?,
    )?;
    run_ios_required(
        SpawnConfig::new(
            "xcrun",
            ios_swift_compile_args(&swift_files, &output_map, target.clone(), swift_jobs),
        )
        .with_options(quiet_command_options(Some(source_root), StreamMode::Ignore)),
    )?;
    run_ios_required(
        SpawnConfig::new("xcrun", ios_swift_link_args(&object_files, &bundle, target))
            .with_options(quiet_command_options(
                Some(build_root.clone()),
                StreamMode::Ignore,
            )),
    )?;
    ensure_file(bundle.join("DoweIosApp"), DevTarget::Ios)?;
    fs::copy(plist, bundle.join("Info.plist"))?;
    copy_ios_resources(ios_root, &bundle)?;
    compile_ios_asset_catalog(ios_root, &bundle, &build_root)?;
    let cached_bundle = publish_ios_app(project_root, &cache_key, &bundle)?;
    fs::remove_dir_all(build_root)?;
    Ok(cached_bundle)
}
