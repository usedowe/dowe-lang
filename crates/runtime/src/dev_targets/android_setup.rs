fn prepare_android_tools(sdk: &Path) -> RuntimeResult<AndroidTools> {
    static PREPARATION: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = PREPARATION
        .lock()
        .map_err(|_| RuntimeError::new("Android setup lock failed"))?;
    let packages = missing_android_packages(sdk);
    if !packages.is_empty() {
        let manager = android_command_tool(sdk, "sdkmanager")?;
        crate::logging::log_dev_info(format!("Android setup: installing {}", packages.join(", ")));
        run_required(DevTarget::Android, android_install_config(sdk, &manager, packages))
            .map_err(|error| RuntimeError::new(format!("Android setup: SDK package installation failed. Complete any SDK license request and retry. {error}")))?;
    }
    android_tools(sdk)
}

fn missing_android_packages(sdk: &Path) -> Vec<String> {
    let mut packages = Vec::new();
    if !executable_path(sdk.join("platform-tools/adb")).is_file() {
        packages.push("platform-tools".to_string());
    }
    if !executable_path(sdk.join("emulator/emulator")).is_file() {
        packages.push("emulator".to_string());
    }
    if android_build_tools(sdk).is_err() {
        packages.push("build-tools;36.0.0".to_string());
    }
    if latest_android_jar(sdk).is_err() {
        packages.push("platforms;android-36".to_string());
    }
    packages
}

fn android_command_tool(sdk: &Path, name: &str) -> RuntimeResult<PathBuf> {
    let root = sdk.join("cmdline-tools");
    let mut candidates = vec![root.join("latest"), root.join("dowe")];
    if let Ok(directory) = latest_child(root) {
        candidates.push(directory);
    }
    for directory in candidates {
        let path = android_script_path(directory.join("bin").join(name));
        if path.is_file() {
            return Ok(path);
        }
    }
    super::android_command_tools::install(sdk)?;
    ensure_file(
        android_script_path(sdk.join("cmdline-tools/dowe/bin").join(name)),
        DevTarget::Android,
    )
}

fn android_script_path(path: PathBuf) -> PathBuf {
    if cfg!(target_os = "windows") {
        path.with_extension("bat")
    } else {
        path
    }
}

fn android_install_config(sdk: &Path, manager: &Path, packages: Vec<String>) -> SpawnConfig {
    let mut args = vec![
        format!("--sdk_root={}", sdk.display()),
        "--install".to_string(),
    ];
    args.extend(packages);
    let mut config = SpawnConfig::new(manager.to_string_lossy(), args)
        .with_options(quiet_command_options(None, StreamMode::Inherit));
    config.options.stdin = StreamMode::Inherit;
    config.options.stderr = StreamMode::Inherit;
    config.options.timeout_ms = Some(3_600_000);
    config
}

fn ensure_android_avd(sdk: &Path, tools: &AndroidTools) -> RuntimeResult<()> {
    static PREPARATION: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = PREPARATION
        .lock()
        .map_err(|_| RuntimeError::new("Android device setup lock failed"))?;
    if !android_device_serials(&tools.adb)?.is_empty() || !android_avds(&tools.emulator)?.is_empty()
    {
        return Ok(());
    }
    let architecture = android_image_architecture(env::consts::ARCH)?;
    let sdkmanager = android_command_tool(sdk, "sdkmanager")?;
    let avdmanager = android_command_tool(sdk, "avdmanager")?;
    let image = installed_android_image(sdk, architecture);
    let name = create_android_avd_with(
        sdk,
        &sdkmanager,
        &avdmanager,
        image,
        architecture,
        |config| run_required(DevTarget::Android, config).map(|output| output.stdout_bytes),
    )?;
    if !android_avds(&tools.emulator)?.contains(&name) {
        return Err(RuntimeError::new(
            "Android setup: the created emulator is not available; check ANDROID_AVD_HOME and SDK permissions, then retry",
        ));
    }
    Ok(())
}

fn android_image_architecture(architecture: &str) -> RuntimeResult<&'static str> {
    match architecture {
        "aarch64" => Ok("arm64-v8a"),
        "x86_64" => Ok("x86_64"),
        _ => Err(RuntimeError::new(format!(
            "Android setup: emulator images are not supported for host architecture {architecture}"
        ))),
    }
}

fn installed_android_image(sdk: &Path, architecture: &str) -> Option<String> {
    let mut images = Vec::new();
    for platform in fs::read_dir(sdk.join("system-images"))
        .ok()?
        .filter_map(Result::ok)
    {
        let name = platform.file_name().to_string_lossy().into_owned();
        let Some(api) = name
            .strip_prefix("android-")
            .and_then(|version| version.parse::<u32>().ok())
            .filter(|api| *api >= 26)
        else {
            continue;
        };
        for tag in ["google_apis", "google_apis_playstore", "default"] {
            let root = platform.path().join(tag).join(architecture);
            if root.join("package.xml").is_file() && root.join("system.img").is_file() {
                images.push((api, format!("system-images;{name};{tag};{architecture}")));
            }
        }
    }
    images.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    images.into_iter().next().map(|(_, image)| image)
}

fn create_android_avd_with(
    sdk: &Path,
    sdkmanager: &Path,
    avdmanager: &Path,
    image: Option<String>,
    architecture: &str,
    mut run: impl FnMut(SpawnConfig) -> RuntimeResult<Vec<u8>>,
) -> RuntimeResult<String> {
    let image = match image {
        Some(image) => image,
        None => {
            let image = format!("system-images;android-36;google_apis;{architecture}");
            crate::logging::log_dev_info(format!("Android setup: downloading {image}"));
            run(android_install_config(sdk, sdkmanager, vec![image.clone()]))?;
            image
        }
    };
    let api = image
        .split(';')
        .nth(1)
        .unwrap_or("android")
        .replace('-', "_");
    let name = format!("Dowe_{api}_{}", architecture.replace('-', "_"));
    crate::logging::log_dev_info(format!("Android setup: creating {name}"));
    let mut config = SpawnConfig::new(
        avdmanager.to_string_lossy(),
        [
            "create",
            "avd",
            "--name",
            &name,
            "--package",
            &image,
            "--device",
            "pixel",
        ],
    )
    .with_options(quiet_command_options(None, StreamMode::Pipe));
    config.options.timeout_ms = Some(120_000);
    config.options.env.insert(
        "ANDROID_HOME".to_string(),
        sdk.to_string_lossy().into_owned(),
    );
    config.options.env.insert(
        "ANDROID_SDK_ROOT".to_string(),
        sdk.to_string_lossy().into_owned(),
    );
    run(config).map_err(|error| {
        RuntimeError::new(format!("Android setup: emulator creation failed. {error}"))
    })?;
    Ok(name)
}

#[cfg(test)]
#[path = "android_setup_tests.rs"]
mod setup_tests;
