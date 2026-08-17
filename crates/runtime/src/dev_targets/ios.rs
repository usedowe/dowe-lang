use super::{
    cleanup_command, ensure_dir, ensure_file,
    ios_cache::{cached_ios_app, ios_app_cache_key, prune_ios_app_cache, publish_ios_app},
    ios_incremental::{IOS_INCREMENTAL_MODULE_NAME, IosHotModuleSnapshot, IosIncrementalWorkspace},
    print_target_started, print_target_starting, quiet_command_options, run_required,
};
use crate::dev::{
    DevTarget, ExternalTargetStartup, HostOs, IosSimulatorOption, IosSimulatorSelection,
};
use crate::dev_modules::{
    DevModuleRevision, PublishedDevModule, publish_dev_module, publish_dev_module_if_current,
};
use crate::error::{RuntimeError, RuntimeResult};
use dowe_compiler::{CompiledProject, GeneratedFile};
use dowe_spawn::{SpawnConfig, StreamMode};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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
    let cleanup_configs = ios_cleanup_commands(&simulator.udid, quit_simulators_on_exit);
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
    run_required(
        DevTarget::Ios,
        ios_install_config(&simulator.udid, &app_bundle)
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    let launch_result = run_required(
        DevTarget::Ios,
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
    run_required(
        DevTarget::Ios,
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
    compile_ios_asset_catalog(ios_root, &bundle, &build_root)?;
    let cached_bundle = publish_ios_app(project_root, &cache_key, &bundle)?;
    fs::remove_dir_all(build_root)?;
    Ok(cached_bundle)
}

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
    let result = run_required(
        DevTarget::Ios,
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
    run_required(
        DevTarget::Ios,
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

fn compile_ios_asset_catalog(
    ios_root: &Path,
    bundle: &Path,
    build_root: &Path,
) -> RuntimeResult<()> {
    let catalog = ios_root.join("Assets.xcassets");
    if !catalog.is_dir() {
        return Ok(());
    }
    run_required(
        DevTarget::Ios,
        SpawnConfig::new(
            "xcrun",
            ios_asset_catalog_args(
                &catalog,
                bundle,
                &build_root.join("asset-catalog-info.plist"),
            ),
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    Ok(())
}

fn ios_asset_catalog_args(catalog: &Path, bundle: &Path, partial_plist: &Path) -> Vec<String> {
    vec![
        "actool".to_string(),
        catalog.to_string_lossy().to_string(),
        "--compile".to_string(),
        bundle.to_string_lossy().to_string(),
        "--platform".to_string(),
        "iphonesimulator".to_string(),
        "--minimum-deployment-target".to_string(),
        "17.0".to_string(),
        "--target-device".to_string(),
        "iphone".to_string(),
        "--target-device".to_string(),
        "ipad".to_string(),
        "--app-icon".to_string(),
        "AppIcon".to_string(),
        "--output-partial-info-plist".to_string(),
        partial_plist.to_string_lossy().to_string(),
    ]
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

fn prepare_ios_simulator(selection: Option<IosSimulatorSelection>) -> RuntimeResult<IosSimulator> {
    if let Some(selection) = selection {
        return prepare_selected_ios_simulator(&selection);
    }

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

fn prepare_selected_ios_simulator(
    selection: &IosSimulatorSelection,
) -> RuntimeResult<IosSimulator> {
    let option = simulator_options()?
        .into_iter()
        .find(|option| option.udid() == selection.udid())
        .ok_or_else(|| {
            RuntimeError::new(format!(
                "iOS app target failed: selected simulator `{}` is not available",
                selection.udid()
            ))
        })?;

    let boot_requested = !option.is_booted();
    if option.state() != "Booted" && option.state() != "Booting" {
        run_required(
            DevTarget::Ios,
            SpawnConfig::new("xcrun", ["simctl", "boot", option.udid()])
                .with_options(quiet_command_options(None, StreamMode::Ignore)),
        )?;
    }

    Ok(IosSimulator {
        udid: option.udid().to_string(),
        boot_requested,
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

fn ios_cleanup_commands(udid: &str, quit_simulators_on_exit: bool) -> Vec<SpawnConfig> {
    if !quit_simulators_on_exit {
        return Vec::new();
    }
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

pub(super) fn simulator_options() -> RuntimeResult<Vec<IosSimulatorOption>> {
    if HostOs::current() != HostOs::Macos {
        return Ok(Vec::new());
    }
    let output = run_required(
        DevTarget::Ios,
        SpawnConfig::new("xcrun", ["simctl", "list", "devices", "available", "-j"])
            .with_options(quiet_command_options(None, StreamMode::Pipe)),
    )?;
    parse_ios_simulator_options(&output.stdout_bytes)
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

fn parse_ios_simulator_options(contents: &[u8]) -> RuntimeResult<Vec<IosSimulatorOption>> {
    let value = serde_json::from_slice::<Value>(contents)
        .map_err(|error| RuntimeError::new(format!("iOS app target failed: {error}")))?;
    let Some(runtimes) = value.get("devices").and_then(Value::as_object) else {
        return Ok(Vec::new());
    };
    let mut options = Vec::new();

    for (runtime, devices) in runtimes {
        let Some(devices) = devices.as_array() else {
            continue;
        };
        for device in devices {
            let available = device
                .get("isAvailable")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if !available {
                continue;
            }
            let Some(name) = device.get("name").and_then(Value::as_str) else {
                continue;
            };
            let Some(udid) = device.get("udid").and_then(Value::as_str) else {
                continue;
            };
            let state = device
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or("Unknown");
            options.push(IosSimulatorOption::new(
                name,
                udid,
                ios_runtime_label(runtime),
                state,
            ));
        }
    }

    options.sort_by(|left, right| {
        (!left.is_booted())
            .cmp(&!right.is_booted())
            .then_with(|| left.runtime().cmp(right.runtime()))
            .then_with(|| left.name().cmp(right.name()))
            .then_with(|| left.udid().cmp(right.udid()))
    });
    Ok(options)
}

fn ios_runtime_label(runtime: &str) -> String {
    let suffix = runtime.rsplit('.').next().unwrap_or(runtime);
    if let Some(version) = suffix.strip_prefix("iOS-") {
        return format!("iOS {}", version.replace('-', "."));
    }
    suffix.replace('-', " ")
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
    static SIGNATURE: OnceLock<Vec<u8>> = OnceLock::new();
    if let Some(signature) = SIGNATURE.get() {
        return Ok(signature.clone());
    }
    let mut signature = b"dowe-ios-cache-v1".to_vec();
    for args in ios_toolchain_signature_commands() {
        let output = run_required(
            DevTarget::Ios,
            SpawnConfig::new("xcrun", args.clone())
                .with_options(quiet_command_options(None, StreamMode::Pipe)),
        )?;
        let command = args.join("\0");
        signature.extend(command.len().to_le_bytes());
        signature.extend(command.as_bytes());
        signature.extend(output.stdout_bytes.len().to_le_bytes());
        signature.extend(output.stdout_bytes);
    }
    let _ = SIGNATURE.set(signature.clone());
    Ok(signature)
}

fn ios_toolchain_signature_commands() -> Vec<Vec<&'static str>> {
    vec![
        vec!["swiftc", "--version"],
        vec!["xcodebuild", "-version"],
        vec!["--sdk", "iphonesimulator", "--show-sdk-path"],
        vec!["--sdk", "iphonesimulator", "--show-sdk-version"],
    ]
}

fn ios_swift_job_count() -> usize {
    std::thread::available_parallelism()
        .map(|value| bounded_ios_swift_job_count(value.get()))
        .unwrap_or(2)
}

fn bounded_ios_swift_job_count(parallelism: usize) -> usize {
    parallelism.clamp(1, 2)
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
        "-driver-batch-size-limit".to_string(),
        "1".to_string(),
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
        IOS_INCREMENTAL_MODULE_NAME, bounded_ios_swift_job_count, ios_asset_catalog_args,
        ios_build_root, ios_cleanup_commands, ios_hot_module_compile_args,
        ios_hot_module_link_args, ios_install_config, ios_launch_config, ios_open_simulator_config,
        ios_runtime_label, ios_simulator_target, ios_swift_compile_args, ios_swift_job_count,
        ios_swift_link_args, ios_swift_object_files, ios_swift_output_map,
        ios_toolchain_signature_commands, parse_ios_simulator_options, run_ios_hot_module_pipeline,
    };
    use crate::dev_modules::DevModuleRevision;
    use crate::error::RuntimeError;
    use dowe_spawn::StreamMode;
    use std::cell::{Cell, RefCell};
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    #[test]
    fn builds_ios_simulator_target_for_host_arch() {
        let target = ios_simulator_target();

        assert!(target.ends_with("-apple-ios17.0-simulator"));
    }

    #[test]
    fn parses_ios_simulator_options() {
        let options = parse_ios_simulator_options(
            br#"{
              "devices": {
                "com.apple.CoreSimulator.SimRuntime.iOS-17-5": [
                  {"name":"iPhone 15","udid":"BOOTED","state":"Booted","isAvailable":true},
                  {"name":"iPad Pro","udid":"UNAVAILABLE","state":"Shutdown","isAvailable":false}
                ],
                "com.apple.CoreSimulator.SimRuntime.iOS-18-0": [
                  {"name":"iPhone 16","udid":"SHUTDOWN","state":"Shutdown","isAvailable":true}
                ]
              }
            }"#,
        )
        .expect("options");

        assert_eq!(options.len(), 2);
        assert_eq!(options[0].udid(), "BOOTED");
        assert_eq!(options[0].label(), "iPhone 15 (iOS 17.5, Booted)");
        assert_eq!(options[1].udid(), "SHUTDOWN");
        assert_eq!(options[1].label(), "iPhone 16 (iOS 18.0, Shutdown)");
    }

    #[test]
    fn formats_ios_runtime_labels() {
        assert_eq!(
            ios_runtime_label("com.apple.CoreSimulator.SimRuntime.iOS-18-2"),
            "iOS 18.2"
        );
        assert_eq!(ios_runtime_label("custom-runtime"), "custom runtime");
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
        let launch = ios_launch_config("TEST-UDID", "app.test", Some("http://127.0.0.1:5000"));
        let open = ios_open_simulator_config();

        assert_eq!(install.command, "xcrun");
        assert_eq!(
            install.args,
            ["simctl", "install", "TEST-UDID", "/project/DoweIosApp.app"]
        );
        assert_eq!(launch.command, "xcrun");
        assert_eq!(
            launch.args,
            [
                "simctl",
                "launch",
                "TEST-UDID",
                "app.test",
                "--dowe-dev-server",
                "http://127.0.0.1:5000"
            ]
        );
        assert_eq!(open.command, "open");
        assert_eq!(open.args, ["-a", "Simulator"]);
    }

    #[test]
    fn limits_ios_swift_parallel_jobs_to_leave_host_headroom() {
        assert_eq!(bounded_ios_swift_job_count(1), 1);
        assert_eq!(bounded_ios_swift_job_count(2), 2);
        assert_eq!(bounded_ios_swift_job_count(10), 2);
        assert!((1..=2).contains(&ios_swift_job_count()));
    }

    #[test]
    fn captures_swift_xcode_sdk_path_and_sdk_version_for_toolchain_identity() {
        assert_eq!(
            ios_toolchain_signature_commands(),
            [
                ["swiftc", "--version"].as_slice(),
                ["xcodebuild", "-version"].as_slice(),
                ["--sdk", "iphonesimulator", "--show-sdk-path"].as_slice(),
                ["--sdk", "iphonesimulator", "--show-sdk-version"].as_slice(),
            ]
        );
    }

    #[test]
    fn builds_ios_swift_args_for_unbatched_compile() {
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
        assert!(args.contains(&"-driver-batch-size-limit".to_string()));
        assert_eq!(arg_after(&args, "-driver-batch-size-limit"), Some("1"));
        assert!(!args.contains(&"-disable-batch-mode".to_string()));
        assert!(!args.contains(&"-driver-batch-count".to_string()));
        assert!(args.contains(&"-j".to_string()));
        assert_eq!(arg_after(&args, "-j"), Some("2"));
        assert!(!args.contains(&"-whole-module-optimization".to_string()));
        assert!(!args.contains(&"-num-threads".to_string()));
        assert!(args.contains(&"-c".to_string()));
        assert!(args.contains(&"-output-file-map".to_string()));
        assert!(!args.contains(&"-o".to_string()));
        assert!(args.contains(&"DoweIosApp.swift".to_string()));
        assert!(args.contains(&"GeneratedViews.swift".to_string()));
    }

    #[test]
    fn builds_ios_hot_module_without_app_install_inputs() {
        let sources = vec![
            "/project/.dowe/dev/ios/incremental/toolchain/sources/GeneratedViews.swift".to_string(),
            "/project/.dowe/dev/ios/incremental/toolchain/sources/dev/DoweIosViewModule.swift"
                .to_string(),
        ];
        let objects = ios_swift_object_files(
            &sources,
            Path::new("/project/.dowe/dev/ios/incremental/toolchain/objects"),
        );
        let args = ios_hot_module_compile_args(
            &sources,
            Path::new("/project/.dowe/dev/ios/incremental/toolchain/output-file-map.json"),
            "arm64-apple-ios17.0-simulator".to_string(),
            8,
        );
        let link = ios_hot_module_link_args(
            &objects,
            Path::new("/project/.dowe/dev/ios/incremental/toolchain/links/abc123.dylib"),
            "arm64-apple-ios17.0-simulator".to_string(),
        );

        assert!(args.contains(&"-c".to_string()));
        assert!(args.contains(&"-output-file-map".to_string()));
        assert!(!args.contains(&"-emit-library".to_string()));
        assert_eq!(
            arg_after(&args, "-module-name"),
            Some(IOS_INCREMENTAL_MODULE_NAME)
        );
        assert!(args.contains(&"-incremental".to_string()));
        assert!(args.contains(&"-enable-incremental-file-hashing".to_string()));
        assert!(args.contains(&"-enable-batch-mode".to_string()));
        assert_eq!(arg_after(&args, "-driver-batch-size-limit"), Some("1"));
        assert!(!args.contains(&"-driver-batch-count".to_string()));
        assert_eq!(arg_after(&args, "-j"), Some("8"));
        assert!(!args.contains(&"-Xfrontend".to_string()));
        assert!(!args.contains(&"-disable-availability-checking".to_string()));
        assert!(!args.contains(&"-typecheck".to_string()));
        assert!(link.contains(&"-emit-library".to_string()));
        assert!(link.contains(
            &"/project/.dowe/dev/ios/incremental/toolchain/objects/GeneratedViews.o".to_string()
        ));
        assert!(!args.contains(&"simctl".to_string()));
        assert!(!args.contains(&"install".to_string()));
    }

    #[test]
    fn retries_one_full_compile_and_link_after_incremental_link_failure() {
        let operations = RefCell::new(Vec::new());
        let link_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || true,
            || {
                operations.borrow_mut().push("compile");
                Ok(true)
            },
            || {
                operations.borrow_mut().push("link");
                let attempt = link_count.get();
                link_count.set(attempt + 1);
                if attempt == 0 {
                    Err(RuntimeError::new("stale object"))
                } else {
                    Ok(())
                }
            },
            || {
                operations.borrow_mut().push("reset");
                Ok(())
            },
        )
        .expect("recovery");

        assert!(built);
        assert_eq!(
            operations.into_inner(),
            ["compile", "link", "reset", "compile", "link"]
        );
    }

    #[test]
    fn reports_swift_compile_failure_without_repeating_the_full_build() {
        let operations = RefCell::new(Vec::new());

        let error = run_ios_hot_module_pipeline(
            || true,
            || {
                operations.borrow_mut().push("compile");
                Err(RuntimeError::new("generated Swift is invalid"))
            },
            || {
                operations.borrow_mut().push("link");
                Ok(())
            },
            || {
                operations.borrow_mut().push("reset");
                Ok(())
            },
        )
        .expect_err("compile error");

        assert!(error.to_string().contains("generated Swift is invalid"));
        assert_eq!(operations.into_inner(), ["compile"]);
    }

    #[test]
    fn retries_one_full_compile_after_successful_incomplete_output() {
        let operations = RefCell::new(Vec::new());
        let compile_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || true,
            || {
                operations.borrow_mut().push("compile");
                let attempt = compile_count.get();
                compile_count.set(attempt + 1);
                Ok(attempt > 0)
            },
            || {
                operations.borrow_mut().push("link");
                Ok(())
            },
            || {
                operations.borrow_mut().push("reset");
                Ok(())
            },
        )
        .expect("cache recovery");

        assert!(built);
        assert_eq!(
            operations.into_inner(),
            ["compile", "reset", "compile", "link"]
        );
    }

    #[test]
    fn performs_at_most_one_full_recovery_attempt() {
        let compile_count = Cell::new(0);
        let link_count = Cell::new(0);
        let reset_count = Cell::new(0);

        let result = run_ios_hot_module_pipeline(
            || true,
            || {
                compile_count.set(compile_count.get() + 1);
                Ok(true)
            },
            || {
                link_count.set(link_count.get() + 1);
                Err(RuntimeError::new("link failed"))
            },
            || {
                reset_count.set(reset_count.get() + 1);
                Ok(())
            },
        );

        assert!(result.is_err());
        assert_eq!(compile_count.get(), 2);
        assert_eq!(link_count.get(), 2);
        assert_eq!(reset_count.get(), 1);
    }

    #[test]
    fn superseded_revision_stops_before_link_and_recovery_commands() {
        let latest = Arc::new(Mutex::new(1));
        let revision = DevModuleRevision::new(1, Arc::clone(&latest));
        let compile_count = Cell::new(0);
        let link_count = Cell::new(0);
        let reset_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || revision.is_current(),
            || {
                compile_count.set(compile_count.get() + 1);
                *latest.lock().expect("latest") = 2;
                Ok(true)
            },
            || {
                link_count.set(link_count.get() + 1);
                Err(RuntimeError::new("must not link"))
            },
            || {
                reset_count.set(reset_count.get() + 1);
                Ok(())
            },
        )
        .expect("obsolete build");

        assert!(!built);
        assert_eq!(compile_count.get(), 1);
        assert_eq!(link_count.get(), 0);
        assert_eq!(reset_count.get(), 0);
    }

    #[test]
    fn superseded_revision_does_not_start_link_recovery() {
        let latest = Arc::new(Mutex::new(1));
        let revision = DevModuleRevision::new(1, Arc::clone(&latest));
        let compile_count = Cell::new(0);
        let link_count = Cell::new(0);
        let reset_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || revision.is_current(),
            || {
                compile_count.set(compile_count.get() + 1);
                Ok(true)
            },
            || {
                link_count.set(link_count.get() + 1);
                *latest.lock().expect("latest") = 2;
                Err(RuntimeError::new("stale object"))
            },
            || {
                reset_count.set(reset_count.get() + 1);
                Ok(())
            },
        )
        .expect("obsolete recovery");

        assert!(!built);
        assert_eq!(compile_count.get(), 1);
        assert_eq!(link_count.get(), 1);
        assert_eq!(reset_count.get(), 0);
    }

    #[test]
    fn obsolete_revision_does_not_start_initial_compile() {
        let latest = Arc::new(Mutex::new(2));
        let revision = DevModuleRevision::new(1, latest);
        let compile_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || revision.is_current(),
            || {
                compile_count.set(compile_count.get() + 1);
                Ok(true)
            },
            || Ok(()),
            || Ok(()),
        )
        .expect("obsolete build");

        assert!(!built);
        assert_eq!(compile_count.get(), 0);
    }

    #[test]
    fn superseded_recovery_compile_does_not_start_second_link() {
        let latest = Arc::new(Mutex::new(1));
        let revision = DevModuleRevision::new(1, Arc::clone(&latest));
        let compile_count = Cell::new(0);
        let link_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || revision.is_current(),
            || {
                let attempt = compile_count.get();
                compile_count.set(attempt + 1);
                if attempt == 1 {
                    *latest.lock().expect("latest") = 2;
                }
                Ok(true)
            },
            || {
                link_count.set(link_count.get() + 1);
                Err(RuntimeError::new("stale object"))
            },
            || Ok(()),
        )
        .expect("obsolete recovery link");

        assert!(!built);
        assert_eq!(compile_count.get(), 2);
        assert_eq!(link_count.get(), 1);
    }

    #[test]
    fn superseded_incomplete_recovery_is_not_reported_as_cache_failure() {
        let latest = Arc::new(Mutex::new(1));
        let revision = DevModuleRevision::new(1, Arc::clone(&latest));
        let compile_count = Cell::new(0);
        let link_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || revision.is_current(),
            || {
                let attempt = compile_count.get();
                compile_count.set(attempt + 1);
                if attempt == 1 {
                    *latest.lock().expect("latest") = 2;
                    Ok(false)
                } else {
                    Ok(false)
                }
            },
            || {
                link_count.set(link_count.get() + 1);
                Ok(())
            },
            || Ok(()),
        )
        .expect("obsolete recovery");

        assert!(!built);
        assert_eq!(compile_count.get(), 2);
        assert_eq!(link_count.get(), 0);
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
    fn builds_ios_asset_catalog_arguments() {
        let args = ios_asset_catalog_args(
            Path::new("/project/.dowe/apps/ios/Assets.xcassets"),
            Path::new("/project/.dowe/dev/ios/build/1/DoweIosApp.app"),
            Path::new("/project/.dowe/dev/ios/build/1/asset-info.plist"),
        );

        assert_eq!(args[0], "actool");
        assert_eq!(
            arg_after(&args, "--compile"),
            Some("/project/.dowe/dev/ios/build/1/DoweIosApp.app")
        );
        assert_eq!(arg_after(&args, "--app-icon"), Some("AppIcon"));
        assert!(
            args.windows(2)
                .any(|values| values == ["--target-device", "iphone"])
        );
        assert!(
            args.windows(2)
                .any(|values| values == ["--target-device", "ipad"])
        );
    }

    #[test]
    fn builds_ios_cleanup_commands_for_simulator_session() {
        let commands = ios_cleanup_commands("TEST-UDID", true);

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

    #[test]
    fn skips_ios_cleanup_commands_when_simulator_quit_is_disabled() {
        assert!(ios_cleanup_commands("TEST-UDID", false).is_empty());
    }
}
