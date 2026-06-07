mod android;
mod desktop;
mod ios;
mod ios_cache;

use crate::dev::{
    DevTarget, ExternalTargetStartup, RunningExternalCleanup, RunningExternalProcess,
};
use crate::error::{RuntimeError, RuntimeResult};
use crate::logging::log_info;
use dowe_compiler::CompiledProject;
use dowe_spawn::{KillTarget, ProcessControl, SpawnConfig, SpawnOptions, StreamMode, run, spawn};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub(crate) fn start_external_target(
    project: &CompiledProject,
    target: DevTarget,
    desktop_origin: Option<&str>,
) -> RuntimeResult<ExternalTargetStartup> {
    match target {
        DevTarget::Desktop => desktop::start(project, desktop_origin),
        DevTarget::Android => android::start(project),
        DevTarget::Ios => ios::start(project),
        DevTarget::Server | DevTarget::Web => Ok(ExternalTargetStartup::default()),
    }
}

pub(super) fn spawn_external(
    target: DevTarget,
    config: SpawnConfig,
) -> RuntimeResult<RunningExternalProcess> {
    print_target_starting(target);
    let child = spawn(config)
        .map_err(|error| RuntimeError::new(format!("{} target failed: {error}", target.label())))?;
    print_target_started(target);
    Ok(RunningExternalProcess { target, child })
}

pub(super) fn print_target_starting(target: DevTarget) {
    log_info(target_starting_message(target));
}

pub(super) fn print_target_started(target: DevTarget) {
    log_info(target_started_message(target));
}

fn target_starting_message(target: DevTarget) -> String {
    format!("{} starting", target.label())
}

fn target_started_message(target: DevTarget) -> String {
    format!("{} started", target.label())
}

pub(super) fn run_required(
    target: DevTarget,
    config: SpawnConfig,
) -> RuntimeResult<dowe_spawn::SpawnOutput> {
    let child = spawn(config)
        .map_err(|error| RuntimeError::new(format!("{} target failed: {error}", target.label())))?;
    let control = child.controller();
    register_active_external_command(control.clone());
    let output = child.wait();
    unregister_active_external_command(control.spawn_id);
    let output = output
        .map_err(|error| RuntimeError::new(format!("{} target failed: {error}", target.label())))?;

    if output.success {
        Ok(output)
    } else {
        let detail = command_failure_detail(&output);
        Err(RuntimeError::new(format!(
            "{} target failed with status {:?}{}",
            target.label(),
            output.exit_code,
            detail
                .map(|detail| format!(": {detail}"))
                .unwrap_or_default()
        )))
    }
}

fn command_failure_detail(output: &dowe_spawn::SpawnOutput) -> Option<String> {
    let bytes = if output.stderr_bytes.is_empty() {
        &output.stdout_bytes
    } else {
        &output.stderr_bytes
    };
    let detail = String::from_utf8_lossy(bytes).trim().to_string();
    (!detail.is_empty()).then_some(detail)
}

pub(crate) fn cancel_active_external_commands() {
    let controls = active_command_controls()
        .lock()
        .expect("active command lock")
        .clone();
    for control in controls {
        let _ = control.cancel();
    }
}

fn active_command_controls() -> &'static Mutex<Vec<ProcessControl>> {
    static CONTROLS: OnceLock<Mutex<Vec<ProcessControl>>> = OnceLock::new();
    CONTROLS.get_or_init(|| Mutex::new(Vec::new()))
}

fn register_active_external_command(control: ProcessControl) {
    active_command_controls()
        .lock()
        .expect("active command lock")
        .push(control);
}

fn unregister_active_external_command(spawn_id: u64) {
    active_command_controls()
        .lock()
        .expect("active command lock")
        .retain(|control| control.spawn_id != spawn_id);
}

pub(super) fn run_allow_failure(
    target: DevTarget,
    config: SpawnConfig,
) -> RuntimeResult<dowe_spawn::SpawnOutput> {
    run(config)
        .map_err(|error| RuntimeError::new(format!("{} target failed: {error}", target.label())))
}

pub(super) fn ensure_dir(path: PathBuf, target: DevTarget) -> RuntimeResult<PathBuf> {
    if path.is_dir() {
        Ok(path)
    } else {
        Err(RuntimeError::new(format!(
            "{} target failed: missing generated directory {}",
            target.label(),
            path.display()
        )))
    }
}

pub(super) fn ensure_file(path: PathBuf, target: DevTarget) -> RuntimeResult<PathBuf> {
    if path.is_file() {
        Ok(path)
    } else {
        Err(RuntimeError::new(format!(
            "{} target failed: missing generated file {}",
            target.label(),
            path.display()
        )))
    }
}

pub(super) fn command_options(cwd: Option<PathBuf>, stdout: StreamMode) -> SpawnOptions {
    SpawnOptions {
        cwd,
        stdin: StreamMode::Ignore,
        stdout,
        stderr: StreamMode::Inherit,
        kill_target: KillTarget::Group,
        ..SpawnOptions::default()
    }
}

pub(super) fn quiet_command_options(cwd: Option<PathBuf>, stdout: StreamMode) -> SpawnOptions {
    let mut options = command_options(cwd, stdout);
    options.stderr = StreamMode::Pipe;
    options
}

pub(super) fn spawn_background(
    target: DevTarget,
    config: SpawnConfig,
) -> RuntimeResult<RunningExternalProcess> {
    let child = spawn(config)
        .map_err(|error| RuntimeError::new(format!("{} target failed: {error}", target.label())))?;
    Ok(RunningExternalProcess { target, child })
}

pub(super) fn cleanup_command(target: DevTarget, config: SpawnConfig) -> RunningExternalCleanup {
    RunningExternalCleanup { target, config }
}

pub(super) fn latest_child(path: PathBuf) -> RuntimeResult<PathBuf> {
    let mut entries = fs::read_dir(&path)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|entry| entry.is_dir())
        .collect::<Vec<_>>();
    entries.sort();
    entries.pop().ok_or_else(|| {
        RuntimeError::new(format!("missing tool directory under {}", path.display()))
    })
}

pub(super) fn executable_path(path: PathBuf) -> PathBuf {
    if cfg!(windows) {
        path.with_extension("exe")
    } else {
        path
    }
}

#[cfg(test)]
mod tests {
    use super::{
        StreamMode, cancel_active_external_commands, quiet_command_options, run_required,
        target_started_message, target_starting_message,
    };
    use crate::dev::DevTarget;
    use dowe_spawn::SpawnConfig;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn quiet_command_options_suppresses_tool_stderr() {
        let options = quiet_command_options(None, StreamMode::Pipe);

        assert_eq!(options.stdout, StreamMode::Pipe);
        assert_eq!(options.stderr, StreamMode::Pipe);
    }

    #[test]
    fn target_status_messages_use_stable_labels() {
        assert_eq!(target_starting_message(DevTarget::Ios), "iOS app starting");
        assert_eq!(target_started_message(DevTarget::Ios), "iOS app started");
    }

    #[test]
    fn cancels_active_required_external_command() {
        let handle = thread::spawn(|| {
            run_required(
                DevTarget::Android,
                sleep_command().with_options(quiet_command_options(None, StreamMode::Ignore)),
            )
        });
        thread::sleep(Duration::from_millis(100));
        cancel_active_external_commands();

        let error = handle
            .join()
            .expect("join")
            .expect_err("command should be cancelled");

        assert!(error.to_string().contains("Android app target failed"));
    }

    #[test]
    fn required_command_failure_includes_captured_stderr() {
        let error = run_required(
            DevTarget::Android,
            failure_command().with_options(quiet_command_options(None, StreamMode::Ignore)),
        )
        .expect_err("command should fail");

        assert!(error.to_string().contains("toolchain detail"));
    }

    fn sleep_command() -> SpawnConfig {
        if cfg!(windows) {
            SpawnConfig::new(
                "powershell",
                ["-NoProfile", "-Command", "Start-Sleep -Seconds 5"],
            )
        } else {
            SpawnConfig::new("sh", ["-c", "sleep 5"])
        }
    }

    fn failure_command() -> SpawnConfig {
        if cfg!(windows) {
            SpawnConfig::new(
                "powershell",
                [
                    "-NoProfile",
                    "-Command",
                    "[Console]::Error.Write('toolchain detail'); exit 1",
                ],
            )
        } else {
            SpawnConfig::new("sh", ["-c", "printf 'toolchain detail' >&2; exit 1"])
        }
    }
}
