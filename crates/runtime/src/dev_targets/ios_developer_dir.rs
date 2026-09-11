use super::{quiet_command_options, run_required};
use crate::dev::DevTarget;
use crate::error::{RuntimeError, RuntimeResult};
use crate::logging::log_dev_info;
use dowe_spawn::{SpawnConfig, StreamMode};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static DIRECTORY: Mutex<Option<PathBuf>> = Mutex::new(None);

pub(super) fn prepare_command(config: SpawnConfig) -> RuntimeResult<SpawnConfig> {
    let mut cached = DIRECTORY
        .lock()
        .map_err(|_| RuntimeError::new("iOS setup lock failed"))?;
    if cached.is_none() {
        let explicit = env::var_os("DEVELOPER_DIR").map(PathBuf::from);
        let active = run_required(
            DevTarget::Ios,
            probe_config("xcode-select", ["--print-path"]),
        )
        .ok()
        .and_then(|output| String::from_utf8(output.stdout_bytes).ok())
        .map(|path| PathBuf::from(path.trim()));
        let selected =
            select_developer_directory(explicit.clone(), active.clone(), installed_xcodes())?;
        if explicit.is_none() && active.as_ref() != Some(&selected) {
            log_dev_info(format!(
                "iOS setup: using Xcode at {} for this session",
                selected.display()
            ));
        }
        prepare_xcode(&selected)?;
        *cached = Some(selected);
    }
    Ok(apply_developer_directory(
        config,
        cached.as_ref().expect("prepared Xcode"),
    ))
}

fn probe_config(command: &str, args: impl IntoIterator<Item = impl Into<String>>) -> SpawnConfig {
    let mut config =
        SpawnConfig::new(command, args).with_options(quiet_command_options(None, StreamMode::Pipe));
    config.options.timeout_ms = Some(30_000);
    config
}

fn prepare_xcode(directory: &Path) -> RuntimeResult<()> {
    prepare_xcode_with(directory, |config| {
        run_required(DevTarget::Ios, config).map(|_| ())
    })
}

fn prepare_xcode_with(
    directory: &Path,
    mut execute: impl FnMut(SpawnConfig) -> RuntimeResult<()>,
) -> RuntimeResult<()> {
    let mut run = |config| execute(apply_developer_directory(config, directory));
    let xcodebuild = directory
        .join("usr/bin/xcodebuild")
        .to_string_lossy()
        .into_owned();
    if run(probe_config(&xcodebuild, ["-checkFirstLaunchStatus"])).is_err() {
        log_dev_info("iOS setup: installing Xcode first-launch components");
        run(setup_config(&xcodebuild, ["-runFirstLaunch"]))
            .map_err(|error| RuntimeError::new(format!("iOS setup: Xcode initialization failed; complete any license or administrator request in Xcode and retry. {error}")))?;
    }
    run(probe_config("xcrun", ["--find", "simctl"]))
        .map_err(|error| RuntimeError::new(format!("iOS setup: Xcode is missing simctl after initialization. Repair this Xcode installation. {error}")))?;
    if run(probe_config(
        "xcrun",
        ["--sdk", "iphonesimulator", "--show-sdk-path"],
    ))
    .is_err()
    {
        log_dev_info("iOS setup: downloading iOS platform support");
        run(setup_config(
            "xcrun",
            ["xcodebuild", "-downloadPlatform", "iOS"],
        ))?;
        run(probe_config(
            "xcrun",
            ["--sdk", "iphonesimulator", "--show-sdk-path"],
        ))?;
    }
    Ok(())
}

pub(super) fn setup_config(
    command: &str,
    args: impl IntoIterator<Item = impl Into<String>>,
) -> SpawnConfig {
    let mut config = SpawnConfig::new(command, args)
        .with_options(quiet_command_options(None, StreamMode::Inherit));
    config.options.stdin = StreamMode::Inherit;
    config.options.stderr = StreamMode::Inherit;
    config.options.timeout_ms = Some(3_600_000);
    config
}

fn installed_xcodes() -> Vec<PathBuf> {
    let mut roots = vec![PathBuf::from("/Applications")];
    if let Some(home) = env::var_os("HOME") {
        roots.push(PathBuf::from(home).join("Applications"));
    }
    let mut candidates = vec![PathBuf::from("/Applications/Xcode.app/Contents/Developer")];
    for root in roots {
        if let Ok(entries) = fs::read_dir(root) {
            let mut paths = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path().join("Contents/Developer"))
                .filter(|path| path.join("usr/bin/xcodebuild").is_file())
                .collect::<Vec<_>>();
            paths.sort();
            candidates.extend(paths);
        }
    }
    candidates
}

fn select_developer_directory(
    explicit: Option<PathBuf>,
    active: Option<PathBuf>,
    candidates: Vec<PathBuf>,
) -> RuntimeResult<PathBuf> {
    let normalize = |path: PathBuf| {
        if path.extension().is_some_and(|extension| extension == "app") {
            path.join("Contents/Developer")
        } else {
            path
        }
    };
    if let Some(path) = explicit {
        let path = normalize(path);
        return if path.join("usr/bin/xcodebuild").is_file() {
            Ok(path)
        } else {
            Err(RuntimeError::new(
                "iOS setup: DEVELOPER_DIR does not point to a full Xcode installation; select an installed Xcode or remove this override",
            ))
        };
    }
    active.into_iter().chain(candidates).map(normalize)
        .find(|path| path.join("usr/bin/xcodebuild").is_file())
        .ok_or_else(|| RuntimeError::new("iOS setup: full Xcode is required. Install Xcode from the Mac App Store, then run dowe dev again; Dowe will prepare its iOS components and simulator automatically"))
}

pub(super) fn apply_developer_directory(mut config: SpawnConfig, directory: &Path) -> SpawnConfig {
    config.options.env.insert(
        "DEVELOPER_DIR".to_string(),
        directory.to_string_lossy().into_owned(),
    );
    if config.command == "open" && config.args == ["-a", "Simulator"] {
        config.args[1] = directory
            .join("Applications/Simulator.app")
            .to_string_lossy()
            .into_owned();
    }
    config
}

#[cfg(test)]
#[path = "ios_developer_dir_tests.rs"]
mod tests;
