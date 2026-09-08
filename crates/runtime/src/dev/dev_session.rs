use super::{DevRunOptions, DevTarget, DevTargetDeviceSelection, DevTargetSelection};

use crate::dev_watch::run_watch_loop;
use crate::error::{RuntimeError, RuntimeResult};
use crate::logging::log_dev_info;
use crate::server::RunningDevServers;
use dowe_compiler::{CompiledProject, DevCompilerSession};
use dowe_spawn::{ChildProcess, SpawnConfig};

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;

mod cleanup;
mod external_targets;
mod startup;
mod wait;

use cleanup::{
    cancel_external_processes, first_error, run_external_cleanups,
    wait_cancelled_external_processes, wait_external_processes,
};

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

pub(crate) use startup::{dev_server_targets, selected_view_platforms};
pub use startup::{run_dev, run_dev_with_options, run_studio, start_studio_session};

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
    project.local_databases = true;
    project.studio_preview = options.studio_preview;
    let mut project = Arc::new(project);
    let server_targets = dev_server_targets(&selection);
    let servers =
        match crate::server::start_dev_servers_shared(project.clone(), server_targets).await {
            Ok(servers) => servers,
            Err(error) => return Err(error),
        };
    project = {
        let state = servers.runtime_state();
        let current = state.project.read().await;
        current.clone()
    };
    if project.apps.files.is_empty()
        && (selection.contains(DevTarget::Desktop)
            || selection.contains(DevTarget::Android)
            || selection.contains(DevTarget::Ios))
    {
        log_dev_info("Native app artifacts generating in parallel");
        let compiler_for_apps = compiler.clone();
        let project_for_apps = project.clone();
        let app_result = std::thread::Builder::new()
            .name("dowe-native-app-generation".to_string())
            .stack_size(64 * 1024 * 1024)
            .spawn(move || {
                let mut project_for_apps = (*project_for_apps).clone();
                compiler_for_apps
                    .complete_dev_app_outputs(&mut project_for_apps)
                    .map(|()| project_for_apps)
            })
            .map_err(|error| RuntimeError::new(error.to_string()))?
            .join()
            .map_err(|_| RuntimeError::new("native app generation thread panicked"))?;
        match app_result {
            Ok(completed) => {
                project = Arc::new(completed);
                log_dev_info("Native app artifacts ready");
                let state = servers.runtime_state();
                *state.project.write().await = project.clone();
            }
            Err(error) => {
                let _ = servers.shutdown().await;
                return Err(RuntimeError::from(error));
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

    match external_targets::start(
        project.clone(),
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

    pub fn spawn_watch(
        &self,
    ) -> (
        oneshot::Sender<()>,
        tokio::task::JoinHandle<RuntimeResult<()>>,
    ) {
        let (stop_sender, stop_receiver) = oneshot::channel();
        let handle = tokio::spawn(run_watch_loop(
            self.root.clone(),
            self.targets.clone(),
            self.servers.runtime_state(),
            self.compiler.clone(),
            stop_receiver,
        ));
        (stop_sender, handle)
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
            wait::processes_with_signal(external_processes, external_cleanups).await
        } else if !external_cleanups.is_empty() {
            wait::cleanups_with_signal(external_cleanups).await
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
