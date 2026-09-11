fn android_device_serial(adb: &Path) -> RuntimeResult<Option<String>> {
    Ok(android_device_serials(adb)?.into_iter().next())
}

fn android_device_serials(adb: &Path) -> RuntimeResult<Vec<String>> {
    let output = run_required(
        DevTarget::Android,
        android_device_query(adb, ["devices"]),
    )?;
    let text = String::from_utf8_lossy(&output.stdout_bytes);
    let mut devices = text
        .lines()
        .skip(1)
        .filter_map(parse_adb_device)
        .collect::<Vec<_>>();
    devices.sort_by_key(|serial| !serial.starts_with("emulator-"));
    Ok(devices)
}

fn first_android_avd(emulator: &Path) -> RuntimeResult<String> {
    android_avds(emulator)?.into_iter().next().ok_or_else(|| {
        RuntimeError::new("Android app target failed: no Android virtual devices found")
    })
}

fn android_avds(emulator: &Path) -> RuntimeResult<Vec<String>> {
    let output = run_required(
        DevTarget::Android,
        SpawnConfig::new(emulator.to_string_lossy().to_string(), ["-list-avds"])
            .with_options(quiet_command_options(None, StreamMode::Pipe)),
    )?;
    Ok(parse_android_avds(&String::from_utf8_lossy(
        &output.stdout_bytes,
    )))
}

fn parse_android_avds(contents: &str) -> Vec<String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn wait_for_android_boot(adb: &Path, serial: &str) -> RuntimeResult<()> {
    for _ in 0..120 {
        let output = run_required(
            DevTarget::Android,
            adb_config(adb, serial, ["shell", "getprop", "sys.boot_completed"])
                .with_options(quiet_command_options(None, StreamMode::Pipe)),
        )?;
        if String::from_utf8_lossy(&output.stdout_bytes).trim() == "1" {
            return Ok(());
        }
        thread::sleep(Duration::from_secs(1));
    }
    Err(RuntimeError::new(
        "Android app target failed: emulator did not finish booting",
    ))
}

fn adb_config(
    adb: &Path,
    serial: &str,
    args: impl IntoIterator<Item = impl AsRef<str>>,
) -> SpawnConfig {
    let mut values = vec!["-s".to_string(), serial.to_string()];
    values.extend(args.into_iter().map(|value| value.as_ref().to_string()));
    SpawnConfig::new(adb.to_string_lossy().to_string(), values)
}

fn android_emulator_cleanup_config(
    adb: &Path,
    serial: &str,
    quit_simulators_on_exit: bool,
) -> Option<SpawnConfig> {
    if !quit_simulators_on_exit || !serial.starts_with("emulator-") {
        return None;
    }
    Some(
        adb_config(adb, serial, ["emu", "kill"])
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )
}

fn parse_adb_device(line: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    let serial = parts.next()?;
    let state = parts.next()?;
    if state == "device" {
        Some(serial.to_string())
    } else {
        None
    }
}

fn android_tools(sdk: &Path) -> RuntimeResult<AndroidTools> {
    let build_tools = android_build_tools(sdk)?;
    Ok(AndroidTools {
        emulator: ensure_file(
            executable_path(sdk.join("emulator/emulator")),
            DevTarget::Android,
        )?,
        adb: ensure_file(
            executable_path(sdk.join("platform-tools/adb")),
            DevTarget::Android,
        )?,
        aapt2: ensure_file(
            executable_path(build_tools.join("aapt2")),
            DevTarget::Android,
        )?,
        d8: ensure_file(
            android_script_path(build_tools.join("d8")),
            DevTarget::Android,
        )?,
        apksigner: ensure_file(
            android_script_path(build_tools.join("apksigner")),
            DevTarget::Android,
        )?,
        zipalign: ensure_file(
            executable_path(build_tools.join("zipalign")),
            DevTarget::Android,
        )?,
        android_jar: latest_android_jar(sdk)?,
    })
}

fn android_sdk_root() -> RuntimeResult<PathBuf> {
    for key in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Ok(value) = env::var(key) {
            let path = PathBuf::from(value);
            if !path.as_os_str().is_empty() {
                return Ok(path);
            }
        }
    }

    let home = env::var("HOME").ok().map(PathBuf::from);
    let candidates = match (HostOs::current(), home) {
        (HostOs::Macos, Some(home)) => vec![home.join("Library/Android/sdk")],
        (HostOs::Linux, Some(home)) => vec![home.join("Android/Sdk")],
        (HostOs::Windows, _) => env::var("LOCALAPPDATA")
            .ok()
            .map(PathBuf::from)
            .map(|path| vec![path.join("Android/Sdk")])
            .unwrap_or_default(),
        _ => Vec::new(),
    };

    candidates.into_iter().next().ok_or_else(|| {
        RuntimeError::new(
            "Android setup: cannot locate the SDK; set ANDROID_HOME to a writable SDK directory",
        )
    })
}

fn latest_android_jar(sdk: &Path) -> RuntimeResult<PathBuf> {
    let platform = fs::read_dir(sdk.join("platforms"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.join("android.jar").is_file())
        .max_by_key(|path| super::child_version_key(path))
        .ok_or_else(|| RuntimeError::new("Android setup: no complete SDK platform is installed"))?;
    Ok(platform.join("android.jar"))
}

fn android_build_tools(sdk: &Path) -> RuntimeResult<PathBuf> {
    fs::read_dir(sdk.join("build-tools"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            ["aapt2", "zipalign"]
                .iter()
                .all(|name| executable_path(path.join(name)).is_file())
                && ["d8", "apksigner"]
                    .iter()
                    .all(|name| android_script_path(path.join(name)).is_file())
        })
        .max_by_key(|path| super::child_version_key(path))
        .ok_or_else(|| {
            RuntimeError::new("Android setup: no complete SDK build-tools are installed")
        })
}
