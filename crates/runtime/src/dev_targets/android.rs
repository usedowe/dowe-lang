use super::{
    ensure_dir, ensure_file, executable_path, latest_child, print_target_started,
    print_target_starting, quiet_command_options, run_allow_failure, run_required,
    spawn_background,
};
use crate::dev::{DevTarget, ExternalTargetStartup, HostOs, RunningExternalProcess};
use crate::error::{RuntimeError, RuntimeResult};
use dowe_compiler::CompiledProject;
use dowe_spawn::{SpawnConfig, StreamMode};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

pub(super) fn start(project: &CompiledProject) -> RuntimeResult<ExternalTargetStartup> {
    let android_root = ensure_dir(project.root.join(".dowe/apps/android"), DevTarget::Android)?;
    let sdk = android_sdk_root()?;
    let tools = android_tools(&sdk)?;
    let mut processes = Vec::new();

    print_target_starting(DevTarget::Android);
    let existing_serial = android_device_serial(&tools.adb)?;
    if existing_serial.is_none() {
        let avd = first_android_avd(&tools.emulator)?;
        processes.push(spawn_background(
            DevTarget::Android,
            SpawnConfig::new(
                tools.emulator.to_string_lossy().to_string(),
                ["-avd".to_string(), avd],
            )
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
        )?);
    }

    let apk = cleanup_on_error(build_android_apk(&android_root, &tools), &mut processes)?;
    let serial = match existing_serial {
        Some(serial) => serial,
        None => cleanup_on_error(wait_for_android_device(&tools.adb), &mut processes)?,
    };

    cleanup_on_error(
        run_required(
            DevTarget::Android,
            adb_config(&tools.adb, &serial, ["wait-for-device"])
                .with_options(quiet_command_options(None, StreamMode::Ignore)),
        ),
        &mut processes,
    )?;
    cleanup_on_error(wait_for_android_boot(&tools.adb, &serial), &mut processes)?;
    if let Some(port) = android_loopback_backend_port(project) {
        let port = format!("tcp:{port}");
        cleanup_on_error(
            run_required(
                DevTarget::Android,
                adb_config(&tools.adb, &serial, ["reverse", &port, &port])
                    .with_options(quiet_command_options(None, StreamMode::Ignore)),
            ),
            &mut processes,
        )?;
    }
    cleanup_on_error(
        uninstall_existing_app(&tools.adb, &serial, &project.app_config.bundle),
        &mut processes,
    )?;
    cleanup_on_error(
        run_required(
            DevTarget::Android,
            adb_config(
                &tools.adb,
                &serial,
                ["install", "-r", &apk.to_string_lossy()],
            )
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
        ),
        &mut processes,
    )?;
    let launch_component = format!(
        "{}/dev.dowe.generated.DoweDevActivity",
        project.app_config.bundle
    );
    cleanup_on_error(
        run_required(
            DevTarget::Android,
            adb_config(
                &tools.adb,
                &serial,
                ["shell", "am", "start", "-n", launch_component.as_str()],
            )
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
        ),
        &mut processes,
    )?;
    print_target_started(DevTarget::Android);

    Ok(ExternalTargetStartup::from_processes(processes))
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

struct AndroidTools {
    emulator: PathBuf,
    adb: PathBuf,
    aapt2: PathBuf,
    d8: PathBuf,
    apksigner: PathBuf,
    zipalign: PathBuf,
    android_jar: PathBuf,
}

fn build_android_apk(android_root: &Path, tools: &AndroidTools) -> RuntimeResult<PathBuf> {
    let manifest = ensure_file(
        android_root.join("dev/AndroidManifest.xml"),
        DevTarget::Android,
    )?;
    let source = ensure_file(
        android_root.join("dev/src/dev/dowe/generated/DoweDevActivity.java"),
        DevTarget::Android,
    )?;
    let build = android_root.join("build/dev-apk");
    if build.exists() {
        fs::remove_dir_all(&build)?;
    }
    let generated = build.join("gen");
    let classes = build.join("classes");
    let dex = build.join("dex");
    let resources = android_root.join("app/src/main/res");
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

fn android_device_serial(adb: &Path) -> RuntimeResult<Option<String>> {
    let output = run_required(
        DevTarget::Android,
        SpawnConfig::new(adb.to_string_lossy().to_string(), ["devices"])
            .with_options(quiet_command_options(None, StreamMode::Pipe)),
    )?;
    let text = String::from_utf8_lossy(&output.stdout_bytes);
    let mut devices = text
        .lines()
        .skip(1)
        .filter_map(parse_adb_device)
        .collect::<Vec<_>>();
    devices.sort_by_key(|serial| !serial.starts_with("emulator-"));
    Ok(devices.into_iter().next())
}

fn first_android_avd(emulator: &Path) -> RuntimeResult<String> {
    let output = run_required(
        DevTarget::Android,
        SpawnConfig::new(emulator.to_string_lossy().to_string(), ["-list-avds"])
            .with_options(quiet_command_options(None, StreamMode::Pipe)),
    )?;
    String::from_utf8_lossy(&output.stdout_bytes)
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            RuntimeError::new("Android app target failed: no Android virtual devices found")
        })
}

fn wait_for_android_device(adb: &Path) -> RuntimeResult<String> {
    for _ in 0..120 {
        if let Some(serial) = android_device_serial(adb)? {
            return Ok(serial);
        }
        thread::sleep(Duration::from_secs(1));
    }
    Err(RuntimeError::new(
        "Android app target failed: emulator did not become available",
    ))
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
    let build_tools = latest_child(sdk.join("build-tools"))?;
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
        d8: ensure_file(executable_path(build_tools.join("d8")), DevTarget::Android)?,
        apksigner: ensure_file(
            executable_path(build_tools.join("apksigner")),
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
            if path.is_dir() {
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

    candidates
        .into_iter()
        .find(|path| path.is_dir())
        .ok_or_else(|| RuntimeError::new("Android app target failed: Android SDK not found"))
}

fn latest_android_jar(sdk: &Path) -> RuntimeResult<PathBuf> {
    let platform = latest_child(sdk.join("platforms"))?;
    ensure_file(platform.join("android.jar"), DevTarget::Android)
}

#[cfg(test)]
mod tests {
    use super::{
        android_java_sources, android_javac_args, compiled_activity_classes, loopback_url_port,
        parse_adb_device,
    };
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn parses_only_ready_adb_devices() {
        assert_eq!(
            parse_adb_device("emulator-5554\tdevice"),
            Some("emulator-5554".to_string())
        );
        assert_eq!(parse_adb_device("ZT322K45WX\tunauthorized"), None);
    }

    #[test]
    fn maps_loopback_backend_urls_to_adb_reverse_ports() {
        assert_eq!(loopback_url_port("http://127.0.0.1:8080"), Some(8080));
        assert_eq!(loopback_url_port("http://localhost/api"), Some(80));
        assert_eq!(loopback_url_port("https://[::1]:9443/api"), Some(9443));
        assert_eq!(loopback_url_port("https://api.example.com"), None);
    }

    #[test]
    fn packages_activity_and_generated_nested_classes() {
        let root = TempDir::new().expect("tempdir");
        let package = root.path().join("dev/dowe/generated");
        fs::create_dir_all(&package).expect("package");
        fs::write(package.join("DoweDevActivity.class"), "").expect("activity");
        fs::write(package.join("DoweDevActivity$DoweAction.class"), "").expect("nested");
        fs::write(package.join("R.class"), "").expect("r");
        fs::write(package.join("R$string.class"), "").expect("r string");

        let classes = compiled_activity_classes(root.path()).expect("classes");

        assert_eq!(classes.len(), 4);
        assert!(classes[0].ends_with("DoweDevActivity$DoweAction.class"));
        assert!(classes[1].ends_with("DoweDevActivity.class"));
        assert!(classes[2].ends_with("R$string.class"));
        assert!(classes[3].ends_with("R.class"));
    }

    #[test]
    fn compiles_activity_with_generated_android_java_sources() {
        let root = TempDir::new().expect("tempdir");
        let activity = root.path().join("DoweDevActivity.java");
        let package = root.path().join("gen/dev/dowe/generated");
        fs::create_dir_all(&package).expect("package");
        fs::write(&activity, "").expect("activity");
        fs::write(package.join("R.java"), "").expect("r");
        fs::write(package.join("BuildConfig.java"), "").expect("build config");

        let sources = android_java_sources(&activity, &root.path().join("gen")).expect("sources");

        assert_eq!(sources.len(), 3);
        assert!(sources[0].ends_with("DoweDevActivity.java"));
        assert!(sources[1].ends_with("BuildConfig.java"));
        assert!(sources[2].ends_with("R.java"));
    }

    #[test]
    fn builds_javac_args_with_lightweight_dev_flags() {
        let root = TempDir::new().expect("tempdir");
        let activity = root.path().join("DoweDevActivity.java");
        let generated = root.path().join("gen");
        fs::create_dir_all(generated.join("dev/dowe/generated")).expect("generated");
        fs::write(&activity, "").expect("activity");
        fs::write(generated.join("dev/dowe/generated/R.java"), "").expect("r");

        let args = android_javac_args(
            &root.path().join("android.jar"),
            &root.path().join("classes"),
            &activity,
            &generated,
        )
        .expect("args");

        assert_eq!(args[0], "-g:none");
        assert_eq!(args[1], "-proc:none");
        assert!(args.iter().any(|arg| arg.ends_with("DoweDevActivity.java")));
        assert!(args.iter().any(|arg| arg.ends_with("R.java")));
    }
}
