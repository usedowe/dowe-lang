use super::{DevRunOptions, DevTarget, DevTargetDeviceSelection, DevTargetSelection};
use crate::dev_targets::{cancel_active_external_commands, start_external_target};
use crate::dev_watch::run_watch_loop;
use crate::error::{RuntimeError, RuntimeResult};
use crate::logging::{LoadingStatus, log_info};
use crate::server::{DevServerTargets, RunningDevServers, start_dev_servers};
use dowe_compiler::{CompiledProject, DevCompilerSession, ViewPlatform};
use dowe_spawn::{ChildProcess, ProcessControl, SpawnConfig, run};
use futures_util::stream::{FuturesUnordered, StreamExt};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::time::Duration;

const LOADING_TICK_INTERVAL: Duration = Duration::from_millis(120);

pub struct RunningDevSession {
    pub root: PathBuf,
    pub targets: DevTargetSelection,
    pub servers: RunningDevServers,
    compiler: DevCompilerSession,
    external_processes: Vec<RunningExternalProcess>,
    external_cleanups: Vec<RunningExternalCleanup>,
}

pub(crate) struct RunningExternalProcess {
    pub(crate) target: DevTarget,
    pub(crate) child: ChildProcess,
}

pub(crate) struct RunningExternalCleanup {
    pub(crate) target: DevTarget,
    pub(crate) config: SpawnConfig,
}

#[derive(Default)]
pub(crate) struct ExternalTargetStartup {
    pub(crate) processes: Vec<RunningExternalProcess>,
    pub(crate) cleanups: Vec<RunningExternalCleanup>,
}

impl ExternalTargetStartup {
    pub(crate) fn from_processes(processes: Vec<RunningExternalProcess>) -> Self {
        Self {
            processes,
            cleanups: Vec::new(),
        }
    }

    pub(crate) fn extend(&mut self, startup: ExternalTargetStartup) {
        self.processes.extend(startup.processes);
        self.cleanups.extend(startup.cleanups);
    }
}

pub async fn run_dev(root: impl AsRef<Path>, selection: DevTargetSelection) -> RuntimeResult<()> {
    run_dev_with_options(root, selection, DevRunOptions::default()).await
}

pub async fn run_dev_with_options(
    root: impl AsRef<Path>,
    selection: DevTargetSelection,
    options: DevRunOptions,
) -> RuntimeResult<()> {
    let root = root.as_ref();
    let platforms = selected_view_platforms(&selection);
    let mut compiler = DevCompilerSession::new(root, platforms).map_err(RuntimeError::from)?;
    let defer_apps = selection.contains(DevTarget::Desktop)
        || selection.contains(DevTarget::Android)
        || selection.contains(DevTarget::Ios);
    let mut project = if defer_apps {
        compiler.compile_initial_web(
            selection.contains(DevTarget::Server) || selection.contains(DevTarget::Desktop),
        )
    } else {
        compiler.compile_initial(
            selection.contains(DevTarget::Server) || selection.contains(DevTarget::Desktop),
        )
    }
    .map_err(RuntimeError::from)?;
    if !selection.contains(DevTarget::Server) {
        project.server_inspector = None;
        let inspector_root = project.root.join(".dowe/server");
        if inspector_root.exists() {
            fs::remove_dir_all(&inspector_root)
                .map_err(|error| RuntimeError::new(error.to_string()))?;
        }
    }
    let session =
        start_dev_session_with_compiler_options(project, selection, options, compiler).await?;
    session.wait().await
}

pub(crate) fn selected_view_platforms(selection: &DevTargetSelection) -> Vec<ViewPlatform> {
    selection
        .targets()
        .iter()
        .filter_map(|target| match target {
            DevTarget::Server => None,
            DevTarget::Web => Some(ViewPlatform::Web),
            DevTarget::Desktop => Some(ViewPlatform::Desktop),
            DevTarget::Android => Some(ViewPlatform::Android),
            DevTarget::Ios => Some(ViewPlatform::Ios),
        })
        .collect()
}

pub async fn start_dev_session(
    project: CompiledProject,
    selection: DevTargetSelection,
) -> RuntimeResult<RunningDevSession> {
    start_dev_session_with_options(project, selection, DevRunOptions::default()).await
}

pub async fn start_dev_session_with_options(
    project: CompiledProject,
    selection: DevTargetSelection,
    options: DevRunOptions,
) -> RuntimeResult<RunningDevSession> {
    let compiler = DevCompilerSession::new(&project.root, selected_view_platforms(&selection))
        .map_err(RuntimeError::from)?;
    start_dev_session_with_compiler_options(project, selection, options, compiler).await
}

async fn start_dev_session_with_compiler_options(
    mut project: CompiledProject,
    selection: DevTargetSelection,
    options: DevRunOptions,
    compiler: DevCompilerSession,
) -> RuntimeResult<RunningDevSession> {
    if !selection.contains(DevTarget::Server) {
        project.server_inspector = None;
        let inspector_root = project.root.join(".dowe/server");
        if inspector_root.exists() {
            fs::remove_dir_all(&inspector_root)
                .map_err(|error| RuntimeError::new(error.to_string()))?;
        }
    }
    let server_targets = dev_server_targets(&selection);
    let servers = match start_dev_servers(project.clone(), server_targets).await {
        Ok(servers) => servers,
        Err(error) => return Err(error),
    };
    if project.apps.files.is_empty()
        && (selection.contains(DevTarget::Desktop)
            || selection.contains(DevTarget::Android)
            || selection.contains(DevTarget::Ios))
    {
        log_info("Native app artifacts generating in parallel");
        let compiler_for_apps = compiler.clone();
        let mut project_for_apps = project.clone();
        let app_result = tokio::task::spawn_blocking(move || {
            compiler_for_apps
                .complete_dev_app_outputs(&mut project_for_apps)
                .map(|()| project_for_apps)
        })
        .await;
        match app_result {
            Ok(Ok(completed)) => {
                project = completed;
                log_info("Native app artifacts ready");
                let state = servers.runtime_state();
                *state.project.write().await = Arc::new(project.clone());
            }
            Ok(Err(error)) => {
                let _ = servers.shutdown().await;
                return Err(RuntimeError::from(error));
            }
            Err(error) => {
                let _ = servers.shutdown().await;
                return Err(RuntimeError::new(format!(
                    "initial native app generation failed: {error}"
                )));
            }
        }
    }
    let mut session = RunningDevSession {
        root: project.root.clone(),
        targets: selection.clone(),
        servers,
        compiler,
        external_processes: Vec::new(),
        external_cleanups: Vec::new(),
    };
    let desktop_origin = session
        .servers
        .views_addr
        .map(|addr| format!("http://{addr}/"));
    let dev_origin = session
        .servers
        .views_addr
        .map(|addr| format!("http://{addr}"));

    match start_external_targets(
        &project,
        &selection,
        desktop_origin,
        dev_origin,
        &options.devices,
    )
    .await
    {
        Ok(startup) => {
            session.external_processes.extend(startup.processes);
            session.external_cleanups.extend(startup.cleanups);
        }
        Err((error, startup)) => {
            session.external_processes.extend(startup.processes);
            session.external_cleanups.extend(startup.cleanups);
            let _ = session.shutdown().await;
            return Err(error);
        }
    }

    Ok(session)
}

pub(crate) fn dev_server_targets(selection: &DevTargetSelection) -> DevServerTargets {
    DevServerTargets {
        backend: selection.contains(DevTarget::Server),
        views: selection.contains(DevTarget::Web)
            || selection.contains(DevTarget::Desktop)
            || selection.contains(DevTarget::Android)
            || selection.contains(DevTarget::Ios),
        desktop: selection.contains(DevTarget::Desktop),
    }
}

async fn start_external_targets(
    project: &CompiledProject,
    selection: &DevTargetSelection,
    desktop_origin: Option<String>,
    dev_origin: Option<String>,
    devices: &DevTargetDeviceSelection,
) -> Result<ExternalTargetStartup, (RuntimeError, ExternalTargetStartup)> {
    let targets = [DevTarget::Desktop, DevTarget::Android, DevTarget::Ios]
        .into_iter()
        .filter(|target| selection.contains(*target))
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return Ok(ExternalTargetStartup::default());
    }

    let mut pending = targets.iter().copied().collect::<BTreeSet<_>>();
    let loading_status = LoadingStatus::start(loading_status_message(pending.iter().copied()));
    let animate_loading = loading_status.is_interactive();
    let mut tasks = FuturesUnordered::new();
    for target in targets {
        let project = project.clone();
        let desktop_origin = desktop_origin.clone();
        let dev_origin = dev_origin.clone();
        let devices = devices.clone();
        tasks.push(tokio::task::spawn_blocking(move || {
            (
                target,
                start_external_target(
                    &project,
                    target,
                    desktop_origin.as_deref(),
                    dev_origin.as_deref(),
                    &devices,
                ),
            )
        }));
    }

    let mut loading_tick = Box::pin(tokio::time::sleep(LOADING_TICK_INTERVAL));
    let mut shutdown_signal = Box::pin(tokio::signal::ctrl_c());
    let mut startup = ExternalTargetStartup::default();
    let mut first_error = None;
    let mut cancelling = false;

    while !tasks.is_empty() {
        tokio::select! {
            signal = &mut shutdown_signal, if !cancelling => {
                cancel_active_external_commands();
                if let Err(error) = signal
                    && first_error.is_none()
                {
                    first_error = Some(RuntimeError::from(error));
                } else if first_error.is_none() {
                    first_error = Some(RuntimeError::new("development session cancelled"));
                }
                cancelling = true;
            }
            result = tasks.next() => {
                let Some(result) = result else {
                    break;
                };
                match result {
                    Ok((target, Ok(target_startup))) => {
                        pending.remove(&target);
                        startup.extend(target_startup);
                        if !pending.is_empty() {
                            loading_status.update(loading_status_message(pending.iter().copied()));
                        }
                    }
                    Ok((target, Err(error))) => {
                        pending.remove(&target);
                        if !pending.is_empty() {
                            loading_status.update(loading_status_message(pending.iter().copied()));
                        }
                        record_external_startup_failure(
                            &mut first_error,
                            &mut cancelling,
                            error,
                            cancel_active_external_commands,
                        );
                    }
                    Err(error) => {
                        record_external_startup_failure(
                            &mut first_error,
                            &mut cancelling,
                            RuntimeError::from(error),
                            cancel_active_external_commands,
                        );
                    }
                }
            }
            _ = &mut loading_tick, if animate_loading && !pending.is_empty() => {
                loading_status.tick();
                loading_tick.as_mut().reset(tokio::time::Instant::now() + LOADING_TICK_INTERVAL);
            }
        }
    }

    loading_status.finish();
    if let Some(error) = first_error {
        Err((error, startup))
    } else {
        Ok(startup)
    }
}

pub(crate) fn record_external_startup_failure(
    first_error: &mut Option<RuntimeError>,
    cancelling: &mut bool,
    error: RuntimeError,
    cancel: impl FnOnce(),
) {
    if first_error.is_none() {
        *first_error = Some(error);
    }
    if !*cancelling {
        cancel();
        *cancelling = true;
    }
}

fn target_labels(targets: impl IntoIterator<Item = DevTarget>) -> String {
    targets
        .into_iter()
        .map(|target| target.label())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn loading_status_message(targets: impl IntoIterator<Item = DevTarget>) -> String {
    format!("Loading dev targets: {}", target_labels(targets))
}

impl RunningDevSession {
    pub async fn shutdown(self) -> RuntimeResult<()> {
        let RunningDevSession {
            servers,
            mut external_processes,
            external_cleanups,
            ..
        } = self;
        cancel_external_processes(&external_processes);
        run_external_cleanups(&external_cleanups);
        let external_result = wait_cancelled_external_processes(&mut external_processes);
        let server_result = servers.shutdown().await;
        external_result?;
        server_result
    }

    pub async fn wait(self) -> RuntimeResult<()> {
        let RunningDevSession {
            root,
            targets,
            servers,
            compiler,
            mut external_processes,
            external_cleanups,
            ..
        } = self;
        let state = servers.runtime_state();
        let (stop_sender, stop_receiver) = oneshot::channel();
        let watch_handle = tokio::spawn(run_watch_loop(
            root,
            targets.clone(),
            state,
            compiler,
            stop_receiver,
        ));

        let mut result = if servers.has_any() {
            let server_result = servers.wait().await;
            cancel_external_processes(&external_processes);
            run_external_cleanups(&external_cleanups);
            let external_result = wait_cancelled_external_processes(&mut external_processes);
            first_error(server_result, external_result)
        } else if !external_processes.is_empty() {
            wait_external_processes_with_signal(external_processes, external_cleanups).await
        } else if !external_cleanups.is_empty() {
            wait_external_cleanups_with_signal(external_cleanups).await
        } else {
            wait_external_processes(&mut external_processes)
        };

        let _ = stop_sender.send(());
        match watch_handle.await {
            Ok(Ok(())) => {}
            Ok(Err(error)) if result.is_ok() => result = Err(error),
            Ok(Err(_)) => {}
            Err(error) if result.is_ok() => result = Err(RuntimeError::from(error)),
            Err(_) => {}
        }

        result
    }
}

fn first_error(first: RuntimeResult<()>, second: RuntimeResult<()>) -> RuntimeResult<()> {
    match (first, second) {
        (Err(error), _) => Err(error),
        (_, Err(error)) => Err(error),
        _ => Ok(()),
    }
}

fn cancel_external_processes(processes: &[RunningExternalProcess]) {
    for process in processes {
        let _ = process.child.cancel();
    }
}

fn cancel_external_controls(controls: &[ProcessControl]) {
    for control in controls {
        let _ = control.cancel();
    }
}

fn run_external_cleanups(cleanups: &[RunningExternalCleanup]) {
    for cleanup in cleanups {
        let _target = cleanup.target;
        let _ = run(cleanup.config.clone());
    }
}

async fn wait_external_processes_with_signal(
    mut processes: Vec<RunningExternalProcess>,
    cleanups: Vec<RunningExternalCleanup>,
) -> RuntimeResult<()> {
    let controls = processes
        .iter()
        .map(|process| process.child.controller())
        .collect::<Vec<_>>();
    let mut wait_handle =
        tokio::task::spawn_blocking(move || wait_external_processes(&mut processes));
    let result = tokio::select! {
        signal = tokio::signal::ctrl_c() => {
            signal.map_err(RuntimeError::from)?;
            cancel_external_controls(&controls);
            wait_handle.await.map_err(RuntimeError::from)?
        }
        result = &mut wait_handle => result.map_err(RuntimeError::from)?,
    };
    run_external_cleanups(&cleanups);
    result
}

async fn wait_external_cleanups_with_signal(
    cleanups: Vec<RunningExternalCleanup>,
) -> RuntimeResult<()> {
    tokio::signal::ctrl_c().await.map_err(RuntimeError::from)?;
    run_external_cleanups(&cleanups);
    Ok(())
}

fn wait_external_processes(processes: &mut Vec<RunningExternalProcess>) -> RuntimeResult<()> {
    let mut first_error = None;
    let processes = std::mem::take(processes);

    for process in processes {
        match process.child.wait() {
            Ok(output) if output.success => {}
            Ok(output) => {
                if first_error.is_none() {
                    first_error = Some(RuntimeError::new(format!(
                        "{} exited with status {:?}",
                        process.target.label(),
                        output.exit_code
                    )));
                }
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(RuntimeError::new(format!(
                        "{} failed: {error}",
                        process.target.label()
                    )));
                }
            }
        }
    }

    if let Some(error) = first_error {
        Err(error)
    } else {
        Ok(())
    }
}

fn wait_cancelled_external_processes(
    processes: &mut Vec<RunningExternalProcess>,
) -> RuntimeResult<()> {
    let mut first_error = None;
    for process in std::mem::take(processes) {
        if let Err(error) = process.child.wait()
            && first_error.is_none()
        {
            first_error = Some(RuntimeError::new(format!(
                "{} failed to stop: {error}",
                process.target.label()
            )));
        }
    }
    first_error.map_or(Ok(()), Err)
}
