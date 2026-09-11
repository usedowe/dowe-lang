pub(super) fn start(
    project: &CompiledProject,
    selection: Option<AndroidDeviceSelection>,
    quit_simulators_on_exit: bool,
    dev_origin: Option<&str>,
) -> RuntimeResult<ExternalTargetStartup> {
    let android_root = ensure_dir(project.root.join(".dowe/apps/android"), DevTarget::Android)?;
    let sdk = android_sdk_root()?;
    let tools = prepare_android_tools(&sdk)?;
    let mut processes = Vec::new();

    print_target_starting(DevTarget::Android);
    let selected_serial = match selection {
        Some(AndroidDeviceSelection::Connected { serial }) => {
            let serials = android_device_serials(&tools.adb)?;
            if serials.iter().any(|value| value == &serial) {
                serial
            } else {
                return Err(RuntimeError::new(format!(
                    "Android app target failed: selected Android device `{serial}` is not connected"
                )));
            }
        }
        Some(AndroidDeviceSelection::Avd { name }) => {
            ensure_android_emulator(
                &tools.adb,
                &tools.emulator,
                &name,
                &mut processes,
            )?
        }
        None => {
            ensure_android_avd(&sdk, &tools)?;
            if let Some(serial) = android_device_serial(&tools.adb)? {
                serial
            } else {
                let avd = first_android_avd(&tools.emulator)?;
                ensure_android_emulator(&tools.adb, &tools.emulator, &avd, &mut processes)?
            }
        }
    };

    let apk = cleanup_on_error(
        build_android_apk(&project.root, &android_root, &tools),
        &mut processes,
    )?;
    cleanup_on_error(build_hot_module_with_tools(project, &tools), &mut processes)?;

    cleanup_on_error(
        run_required(
            DevTarget::Android,
            adb_config(&tools.adb, &selected_serial, ["wait-for-device"])
                .with_options(quiet_command_options(None, StreamMode::Ignore)),
        ),
        &mut processes,
    )?;
    cleanup_on_error(
        wait_for_android_boot(&tools.adb, &selected_serial),
        &mut processes,
    )?;
    if let Some(port) = android_loopback_backend_port(project) {
        let port = format!("tcp:{port}");
        cleanup_on_error(
            run_required(
                DevTarget::Android,
                adb_config(&tools.adb, &selected_serial, ["reverse", &port, &port])
                    .with_options(quiet_command_options(None, StreamMode::Ignore)),
            ),
            &mut processes,
        )?;
    }
    let dev_origin = dev_origin.ok_or_else(|| {
        RuntimeError::new("Android app target failed: missing Dowe development module server")
    })?;
    if let Some(port) = loopback_url_port(dev_origin) {
        let port = format!("tcp:{port}");
        cleanup_on_error(
            run_required(
                DevTarget::Android,
                adb_config(&tools.adb, &selected_serial, ["reverse", &port, &port])
                    .with_options(quiet_command_options(None, StreamMode::Ignore)),
            ),
            &mut processes,
        )?;
    }
    cleanup_on_error(
        uninstall_existing_app(&tools.adb, &selected_serial, &project.app_config.bundle),
        &mut processes,
    )?;
    cleanup_on_error(
        run_required(
            DevTarget::Android,
            adb_config(
                &tools.adb,
                &selected_serial,
                ["install", "-r", &apk.to_string_lossy()],
            )
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
        ),
        &mut processes,
    )?;
    let launch_component = format!(
        "{}/dev.dowe.generated.DoweDevHostActivity",
        project.app_config.bundle
    );
    cleanup_on_error(
        run_required(
            DevTarget::Android,
            adb_config(
                &tools.adb,
                &selected_serial,
                [
                    "shell",
                    "am",
                    "start",
                    "-n",
                    launch_component.as_str(),
                    "--es",
                    "doweDevServer",
                    dev_origin,
                ],
            )
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
        ),
        &mut processes,
    )?;
    print_target_started(DevTarget::Android);

    let mut startup = if quit_simulators_on_exit {
        ExternalTargetStartup::from_processes(processes)
    } else {
        ExternalTargetStartup::default()
    };
    if let Some(config) =
        android_emulator_cleanup_config(&tools.adb, &selected_serial, quit_simulators_on_exit)
    {
        startup
            .cleanups
            .push(cleanup_command(DevTarget::Android, config));
    }
    Ok(startup)
}

pub(super) fn device_options() -> RuntimeResult<Vec<AndroidDeviceOption>> {
    let sdk = android_sdk_root()?;
    let tools = prepare_android_tools(&sdk)?;
    ensure_android_avd(&sdk, &tools)?;
    let mut options = Vec::new();

    for serial in android_device_serials(&tools.adb)? {
        options.push(AndroidDeviceOption::new(
            format!("Connected device {serial}"),
            AndroidDeviceSelection::Connected { serial },
        ));
    }

    for name in android_avds(&tools.emulator)? {
        options.push(AndroidDeviceOption::new(
            format!("Android emulator {name}"),
            AndroidDeviceSelection::Avd { name },
        ));
    }

    Ok(options)
}

fn cleanup_on_error<T>(
    result: RuntimeResult<T>,
    processes: &mut Vec<RunningExternalProcess>,
) -> RuntimeResult<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            stop_android_processes(processes);
            Err(error)
        }
    }
}

fn stop_android_processes(processes: &mut Vec<RunningExternalProcess>) {
    for process in processes.drain(..) {
        let _ = process.child.cancel();
        let _ = process.child.wait();
    }
}

fn android_loopback_backend_port(project: &CompiledProject) -> Option<u16> {
    let value = project
        .environment_config
        .variable("BACKEND_URL")?
        .resolved_value
        .as_deref()?;
    loopback_url_port(value)
}

fn loopback_url_port(value: &str) -> Option<u16> {
    let (authority, default_port) = value
        .strip_prefix("http://")
        .map(|value| (value, 80))
        .or_else(|| value.strip_prefix("https://").map(|value| (value, 443)))?;
    let authority = authority.split('/').next()?;
    if authority == "localhost" || authority == "127.0.0.1" || authority == "[::1]" {
        return Some(default_port);
    }
    if let Some(port) = authority
        .strip_prefix("localhost:")
        .or_else(|| authority.strip_prefix("127.0.0.1:"))
        .or_else(|| authority.strip_prefix("[::1]:"))
    {
        return port.parse().ok();
    }
    None
}

fn uninstall_existing_app(adb: &Path, serial: &str, app_bundle: &str) -> RuntimeResult<()> {
    let _ = run_allow_failure(
        DevTarget::Android,
        adb_config(adb, serial, ["uninstall", app_bundle])
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    Ok(())
}

