use super::{
    cleanup_command, ensure_dir, ensure_file,
    ios_cache::{cached_ios_app, ios_app_cache_key, publish_ios_app},
    print_target_started, print_target_starting, quiet_command_options, run_required,
};
use crate::dev::{DevTarget, ExternalTargetStartup, HostOs};
use crate::error::{RuntimeError, RuntimeResult};
use dowe_compiler::CompiledProject;
use dowe_spawn::{SpawnConfig, StreamMode};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn start(project: &CompiledProject) -> RuntimeResult<ExternalTargetStartup> {
    if HostOs::current() != HostOs::Macos {
        return Err(RuntimeError::new("target `ios` is only available on macOS"));
    }

    let ios_root = ensure_dir(project.root.join(".dowe/apps/ios"), DevTarget::Ios)?;
    print_target_starting(DevTarget::Ios);
    let simulator = prepare_ios_simulator()?;
    let cleanup_configs = ios_cleanup_commands(&simulator.udid);
    if let Err(error) = launch_ios_app(project, &ios_root, &simulator) {
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
) -> RuntimeResult<()> {
    let app_bundle = build_ios_app(&project.root, ios_root, simulator.boot_requested)?;
    if simulator.boot_requested {
        wait_ios_simulator_boot(&simulator.udid)?;
    }
    run_required(
        DevTarget::Ios,
        ios_install_config(&simulator.udid, &app_bundle)
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    let launch_result = run_required(
        DevTarget::Ios,
        ios_launch_config(&simulator.udid, &project.app_config.bundle)
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

fn ios_launch_config(udid: &str, bundle: &str) -> SpawnConfig {
    SpawnConfig::new(
        "xcrun",
        [
            "simctl".to_string(),
            "launch".to_string(),
            udid.to_string(),
            bundle.to_string(),
        ],
    )
}

fn ios_open_simulator_config() -> SpawnConfig {
    SpawnConfig::new("open", ["-a", "Simulator"])
}

fn open_ios_simulator() -> RuntimeResult<()> {
    run_required(
        DevTarget::Ios,
        ios_open_simulator_config().with_options(quiet_command_options(None, StreamMode::Ignore)),
    )
    .map(|_| ())
}

fn build_ios_app(
    project_root: &Path,
    ios_root: &Path,
    simulator_booting: bool,
) -> RuntimeResult<PathBuf> {
    ensure_file(ios_root.join("DoweIosApp.swift"), DevTarget::Ios)?;
    ensure_file(ios_root.join("GeneratedViews.swift"), DevTarget::Ios)?;
    let plist = ensure_file(ios_root.join("Info.plist"), DevTarget::Ios)?;
    let target = ios_simulator_target();
    let cache_key = ios_app_cache_key(ios_root, &target, &ios_toolchain_signature()?)?;
    if let Some(bundle) = cached_ios_app(project_root, &cache_key) {
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
    let mut swift_files = Vec::new();
    for entry in fs::read_dir(ios_root)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("swift") {
            continue;
        }
        let Some(file_name) = path
            .file_name()
            .and_then(|value| value.to_str())
            .map(ToOwned::to_owned)
        else {
            continue;
        };
        fs::copy(&path, source_root.join(&file_name))?;
        swift_files.push(file_name);
    }
    swift_files.sort();
    let object_files = ios_swift_object_files(&swift_files, &objects_root);
    let output_map = build_root.join("output-file-map.json");
    let output_map_content = ios_swift_output_map(&swift_files, &object_files);
    let swift_jobs = ios_swift_job_count(simulator_booting);
    fs::write(
        &output_map,
        serde_json::to_vec(&output_map_content)
            .map_err(|error| RuntimeError::new(format!("iOS app target failed: {error}")))?,
    )?;
    run_required(
        DevTarget::Ios,
        SpawnConfig::new(
            "xcrun",
            ios_swift_compile_args(&swift_files, &output_map, target.clone(), swift_jobs),
        )
        .with_options(quiet_command_options(Some(source_root), StreamMode::Ignore)),
    )?;
    run_required(
        DevTarget::Ios,
        SpawnConfig::new("xcrun", ios_swift_link_args(&object_files, &bundle, target))
            .with_options(quiet_command_options(
                Some(build_root.clone()),
                StreamMode::Ignore,
            )),
    )?;
    ensure_file(bundle.join("DoweIosApp"), DevTarget::Ios)?;
    fs::copy(plist, bundle.join("Info.plist"))?;
    copy_ios_resources(ios_root, &bundle)?;
    let cached_bundle = publish_ios_app(project_root, &cache_key, &bundle)?;
    fs::remove_dir_all(build_root)?;
    Ok(cached_bundle)
}

fn ios_build_root(project_root: &Path) -> PathBuf {
    project_root
        .join(".dowe/dev/ios/build")
        .join(std::process::id().to_string())
}

fn copy_ios_resources(ios_root: &Path, bundle: &Path) -> RuntimeResult<()> {
    let fonts = ios_root.join("Fonts");
    if fonts.is_dir() {
        copy_dir(&fonts, &bundle.join("Fonts"))?;
    }
    for entry in fs::read_dir(ios_root)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) == Some("lproj")
            && let Some(name) = path.file_name()
        {
            copy_dir(&path, &bundle.join(name))?;
        }
    }
    Ok(())
}

fn copy_dir(source: &Path, destination: &Path) -> RuntimeResult<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        let target = destination.join(path.file_name().expect("directory entry has a file name"));
        if path.is_dir() {
            copy_dir(&path, &target)?;
        } else {
            fs::copy(path, target)?;
        }
    }
    Ok(())
}

struct IosSimulator {
    udid: String,
    boot_requested: bool,
}

fn prepare_ios_simulator() -> RuntimeResult<IosSimulator> {
    if let Some(udid) = find_ios_device("booted")? {
        return Ok(IosSimulator {
            udid,
            boot_requested: false,
        });
    }

    let udid = find_ios_device("available")?.ok_or_else(|| {
        RuntimeError::new("iOS app target failed: no available iOS simulator found")
    })?;
    run_required(
        DevTarget::Ios,
        SpawnConfig::new("xcrun", ["simctl", "boot", &udid])
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    Ok(IosSimulator {
        udid,
        boot_requested: true,
    })
}

fn wait_ios_simulator_boot(udid: &str) -> RuntimeResult<()> {
    run_required(
        DevTarget::Ios,
        SpawnConfig::new("xcrun", ["simctl", "bootstatus", udid, "-b"])
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )
    .map(|_| ())
}

fn ios_cleanup_commands(udid: &str) -> Vec<SpawnConfig> {
    vec![
        SpawnConfig::new(
            "xcrun",
            [
                "simctl".to_string(),
                "shutdown".to_string(),
                udid.to_string(),
            ],
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
        SpawnConfig::new(
            "osascript",
            [
                "-e".to_string(),
                "tell application \"Simulator\" to quit".to_string(),
            ],
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    ]
}

fn run_ios_cleanup_configs(configs: &[SpawnConfig]) {
    for config in configs {
        let _ = run_required(DevTarget::Ios, config.clone());
    }
}

fn find_ios_device(mode: &str) -> RuntimeResult<Option<String>> {
    let output = run_required(
        DevTarget::Ios,
        SpawnConfig::new("xcrun", ["simctl", "list", "devices", mode, "-j"])
            .with_options(quiet_command_options(None, StreamMode::Pipe)),
    )?;
    let value = serde_json::from_slice::<Value>(&output.stdout_bytes)
        .map_err(|error| RuntimeError::new(format!("iOS app target failed: {error}")))?;
    let Some(runtimes) = value.get("devices").and_then(Value::as_object) else {
        return Ok(None);
    };

    for devices in runtimes.values() {
        let Some(devices) = devices.as_array() else {
            continue;
        };
        for device in devices {
            let available = device
                .get("isAvailable")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let Some(udid) = device.get("udid").and_then(Value::as_str) else {
                continue;
            };
            if available {
                return Ok(Some(udid.to_string()));
            }
        }
    }

    Ok(None)
}

fn ios_simulator_target() -> String {
    let arch = match env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x86_64",
        other => other,
    };
    format!("{arch}-apple-ios17.0-simulator")
}

fn ios_toolchain_signature() -> RuntimeResult<Vec<u8>> {
    let mut signature = b"dowe-ios-cache-v1".to_vec();
    for args in [
        vec!["swiftc", "--version"],
        vec!["--sdk", "iphonesimulator", "--show-sdk-path"],
    ] {
        let output = run_required(
            DevTarget::Ios,
            SpawnConfig::new("xcrun", args)
                .with_options(quiet_command_options(None, StreamMode::Pipe)),
        )?;
        signature.extend(output.stdout_bytes.len().to_le_bytes());
        signature.extend(output.stdout_bytes);
    }
    Ok(signature)
}

fn ios_swift_job_count(simulator_booting: bool) -> usize {
    let limit = if simulator_booting { 2 } else { 8 };
    std::thread::available_parallelism()
        .map(|value| value.get().clamp(1, limit))
        .unwrap_or(4)
        .min(limit)
}

fn ios_swift_object_files(swift_files: &[String], objects_root: &Path) -> Vec<PathBuf> {
    swift_files
        .iter()
        .map(|file| {
            let stem = Path::new(file)
                .file_stem()
                .and_then(|value| value.to_str())
                .expect("Swift source has a file stem");
            objects_root.join(format!("{stem}.o"))
        })
        .collect()
}

fn ios_swift_output_map(swift_files: &[String], object_files: &[PathBuf]) -> Value {
    let entries = swift_files
        .iter()
        .zip(object_files)
        .map(|(source, object)| {
            let mut outputs = serde_json::Map::new();
            outputs.insert(
                "object".to_string(),
                Value::String(object.to_string_lossy().to_string()),
            );
            (source.clone(), Value::Object(outputs))
        })
        .collect();
    Value::Object(entries)
}

fn ios_swift_compile_args(
    swift_files: &[String],
    output_map: &Path,
    target: String,
    jobs: usize,
) -> Vec<String> {
    let jobs = jobs.to_string();
    let mut args = vec![
        "--sdk".to_string(),
        "iphonesimulator".to_string(),
        "swiftc".to_string(),
        "-parse-as-library".to_string(),
        "-enable-batch-mode".to_string(),
        "-driver-batch-count".to_string(),
        jobs.clone(),
        "-target".to_string(),
        target,
        "-j".to_string(),
        jobs,
        "-c".to_string(),
    ];
    args.extend(swift_files.iter().cloned());
    args.extend([
        "-output-file-map".to_string(),
        output_map.to_string_lossy().to_string(),
    ]);
    args
}

fn ios_swift_link_args(object_files: &[PathBuf], bundle: &Path, target: String) -> Vec<String> {
    let mut args = vec![
        "--sdk".to_string(),
        "iphonesimulator".to_string(),
        "swiftc".to_string(),
        "-target".to_string(),
        target,
    ];
    args.extend(
        object_files
            .iter()
            .map(|path| path.to_string_lossy().to_string()),
    );
    args.extend([
        "-o".to_string(),
        bundle.join("DoweIosApp").to_string_lossy().to_string(),
    ]);
    args
}

#[cfg(test)]
mod tests {
    use super::{
        ios_build_root, ios_cleanup_commands, ios_install_config, ios_launch_config,
        ios_open_simulator_config, ios_simulator_target, ios_swift_compile_args,
        ios_swift_job_count, ios_swift_link_args, ios_swift_object_files, ios_swift_output_map,
    };
    use dowe_spawn::StreamMode;
    use std::path::Path;

    #[test]
    fn builds_ios_simulator_target_for_host_arch() {
        let target = ios_simulator_target();

        assert!(target.ends_with("-apple-ios17.0-simulator"));
    }

    #[test]
    fn builds_ios_app_outside_generated_apps_root() {
        let root = Path::new("/project");
        let build_root = ios_build_root(root);

        assert_eq!(
            build_root,
            root.join(".dowe/dev/ios/build")
                .join(std::process::id().to_string())
        );
        assert!(!build_root.starts_with(root.join(".dowe/apps")));
    }

    #[test]
    fn builds_ios_install_launch_and_open_commands() {
        let install = ios_install_config("TEST-UDID", Path::new("/project/DoweIosApp.app"));
        let launch = ios_launch_config("TEST-UDID", "app.test");
        let open = ios_open_simulator_config();

        assert_eq!(install.command, "xcrun");
        assert_eq!(
            install.args,
            ["simctl", "install", "TEST-UDID", "/project/DoweIosApp.app"]
        );
        assert_eq!(launch.command, "xcrun");
        assert_eq!(launch.args, ["simctl", "launch", "TEST-UDID", "app.test"]);
        assert_eq!(open.command, "open");
        assert_eq!(open.args, ["-a", "Simulator"]);
    }

    #[test]
    fn bounds_ios_swift_parallel_jobs_for_ready_simulator() {
        assert!((1..=8).contains(&ios_swift_job_count(false)));
    }

    #[test]
    fn limits_ios_swift_parallel_jobs_during_simulator_boot() {
        assert!((1..=2).contains(&ios_swift_job_count(true)));
    }

    #[test]
    fn builds_ios_swift_args_for_batched_compile() {
        let args = ios_swift_compile_args(
            &[
                "DoweIosApp.swift".to_string(),
                "GeneratedViews.swift".to_string(),
            ],
            Path::new("/project/.dowe/dev/ios/build/1/output-file-map.json"),
            "arm64-apple-ios17.0-simulator".to_string(),
            2,
        );

        assert!(args.contains(&"-enable-batch-mode".to_string()));
        assert!(args.contains(&"-driver-batch-count".to_string()));
        assert!(args.contains(&"-j".to_string()));
        assert_eq!(arg_after(&args, "-driver-batch-count"), Some("2"));
        assert_eq!(arg_after(&args, "-j"), Some("2"));
        assert!(!args.contains(&"-whole-module-optimization".to_string()));
        assert!(!args.contains(&"-num-threads".to_string()));
        assert!(args.contains(&"-c".to_string()));
        assert!(args.contains(&"-output-file-map".to_string()));
        assert!(!args.contains(&"-o".to_string()));
        assert!(args.contains(&"DoweIosApp.swift".to_string()));
        assert!(args.contains(&"GeneratedViews.swift".to_string()));
    }

    fn arg_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
        args.windows(2)
            .find(|window| window[0] == flag)
            .map(|window| window[1].as_str())
    }

    #[test]
    fn builds_ios_swift_output_map_and_link_args() {
        let swift_files = vec![
            "DoweIosApp.swift".to_string(),
            "GeneratedViews.swift".to_string(),
        ];
        let objects = ios_swift_object_files(
            &swift_files,
            Path::new("/project/.dowe/dev/ios/build/1/objects"),
        );
        let output_map = ios_swift_output_map(&swift_files, &objects);
        let link_args = ios_swift_link_args(
            &objects,
            Path::new("/project/.dowe/dev/ios/build/1/DoweIosApp.app"),
            "arm64-apple-ios17.0-simulator".to_string(),
        );

        assert_eq!(
            output_map["DoweIosApp.swift"]["object"],
            "/project/.dowe/dev/ios/build/1/objects/DoweIosApp.o"
        );
        assert_eq!(
            output_map["GeneratedViews.swift"]["object"],
            "/project/.dowe/dev/ios/build/1/objects/GeneratedViews.o"
        );
        assert!(
            link_args.contains(&"/project/.dowe/dev/ios/build/1/objects/DoweIosApp.o".to_string())
        );
        assert!(link_args.contains(&"-o".to_string()));
    }

    #[test]
    fn builds_ios_cleanup_commands_for_simulator_session() {
        let commands = ios_cleanup_commands("TEST-UDID");

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "xcrun");
        assert_eq!(commands[0].args, ["simctl", "shutdown", "TEST-UDID"]);
        assert_eq!(commands[0].options.stdout, StreamMode::Ignore);
        assert_eq!(commands[0].options.stderr, StreamMode::Pipe);
        assert_eq!(commands[1].command, "osascript");
        assert_eq!(
            commands[1].args,
            ["-e", "tell application \"Simulator\" to quit"]
        );
    }
}
