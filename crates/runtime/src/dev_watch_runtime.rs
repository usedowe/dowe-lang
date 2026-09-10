use crate::dev::{DevTarget, DevTargetSelection};
use crate::dev_modules::web_module_version;
use crate::dev_native_builds::NativeBuildCoordinator;
use crate::error::RuntimeResult;
use crate::logging::{log_error, log_info};
use crate::server_actions::execute_server_action;
use crate::watch::SourceWatcher;
use crate::{DevEventType, DevRuntimeState};
use dowe_compiler::{CompiledProject, DevChangeScope, DevCompilerSession, classify_dev_changes};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout;

const HOT_RELOAD_COMPLETED_MESSAGE: &str = "Hot reload completed (0 errors)";
const HOT_RELOAD_FAILED_MESSAGE: &str = "Hot reload failed";
const WATCH_QUIET_WINDOW: Duration = Duration::from_millis(40);

pub(crate) async fn run_watch_loop(
    root: PathBuf,
    selection: DevTargetSelection,
    state: DevRuntimeState,
    mut compiler: DevCompilerSession,
    mut stop: oneshot::Receiver<()>,
) -> RuntimeResult<()> {
    let mut watcher = SourceWatcher::new(&root)?;
    let initial_project = {
        let project = state.project.read().await;
        Arc::clone(&project)
    };
    let mut native_builds = NativeBuildCoordinator::new(&selection, &state, &initial_project);
    state.events.emit(
        DevEventType::WatchReady,
        None::<String>,
        Some("watching src"),
        Vec::new(),
    );

    loop {
        tokio::select! {
            _ = &mut stop => {
                native_builds.shutdown();
                state.events.emit(DevEventType::Shutdown, None::<String>, None::<String>, Vec::new());
                return Ok(());
            }
            paths = watcher.receive() => {
                handle_watch_changes(
                    &root,
                    &selection,
                    &state,
                    &mut compiler,
                    &mut native_builds,
                    &mut watcher,
                    paths?,
                ).await;
            }
        }
    }
}

async fn handle_watch_changes(
    root: &Path,
    selection: &DevTargetSelection,
    state: &DevRuntimeState,
    compiler: &mut DevCompilerSession,
    native_builds: &mut NativeBuildCoordinator,
    watcher: &mut SourceWatcher,
    paths: Vec<String>,
) {
    let paths = match debounce_changes(watcher, paths.clone()).await {
        Ok(paths) => paths,
        Err(error) => {
            let error = error.to_string();
            report_hot_reload_failure(&error);
            state.events.emit(
                DevEventType::RebuildFailed,
                None::<String>,
                Some(error),
                paths,
            );
            return;
        }
    };
    state.events.emit(
        DevEventType::ChangeDetected,
        None::<String>,
        None::<String>,
        paths.clone(),
    );
    state.events.emit(
        DevEventType::RebuildStarted,
        None::<String>,
        None::<String>,
        paths.clone(),
    );

    let compile_server = matches!(classify_dev_changes(root, &paths), DevChangeScope::Project)
        && (selection.contains(DevTarget::Server) || selection.contains(DevTarget::Desktop));
    let previous_project = {
        let current = state.project.read().await;
        Arc::clone(&current)
    };
    let previous_web_version = if selection.contains(DevTarget::Web) {
        Some(web_module_version(&previous_project))
    } else {
        None
    };
    let compiled = compile_watch_project(
        compiler,
        paths.clone(),
        compile_server,
        previous_project.clone(),
    )
    .await;

    match compiled {
        Ok(mut project) => {
            Arc::make_mut(&mut project).studio_preview = previous_project.studio_preview;
            if compile_server {
                Arc::make_mut(&mut project).local_databases = true;
                if let Err(error) = crate::database_bootstrap::prepare_databases(&project).await {
                    let error = error.to_string();
                    report_hot_reload_failure(&error);
                    state.events.emit(
                        DevEventType::RebuildFailed,
                        None::<String>,
                        Some(error),
                        paths,
                    );
                    return;
                }
            } else {
                let current = state.project.read().await;
                let next = Arc::make_mut(&mut project);
                next.backend = current.backend.clone();
                next.desktop_server = current.desktop_server.clone();
                next.databases = current.databases.clone();
                next.native_ipc = current.native_ipc.clone();
                next.server_inspector = current.server_inspector.clone();
                next.local_databases = current.local_databases;
            }
            apply_active_development_bindings(&mut project, &previous_project, selection);
            let server_init_action = (compile_server && selection.contains(DevTarget::Server))
                .then(|| project.backend.init_action.clone());

            {
                let mut current = state.project.write().await;
                *current = Arc::clone(&project);
            }

            state.events.emit(
                DevEventType::RebuildSucceeded,
                None::<String>,
                Some(HOT_RELOAD_COMPLETED_MESSAGE),
                paths.clone(),
            );
            log_info(HOT_RELOAD_COMPLETED_MESSAGE);

            if selection.contains(DevTarget::Web)
                && previous_web_version.as_deref() != Some(&web_module_version(&project))
            {
                state.events.emit_module_update(
                    DevTarget::Web.as_str(),
                    web_module_version(&project),
                    paths.clone(),
                );
            }
            native_builds.enqueue(Arc::clone(&project), paths.clone());

            if let Some(init_action) = server_init_action {
                state.events.emit(
                    DevEventType::TargetRestarting,
                    Some(DevTarget::Server.as_str()),
                    None::<String>,
                    paths.clone(),
                );
                execute_server_action(&init_action);
            }

            if compile_server && selection.contains(DevTarget::Server) {
                state.events.emit(
                    DevEventType::TargetReady,
                    Some(DevTarget::Server.as_str()),
                    None::<String>,
                    paths.clone(),
                );
            }

            for target in [DevTarget::Desktop] {
                if selection.contains(target) {
                    state.events.emit(
                        DevEventType::Reload,
                        Some(target.as_str()),
                        None::<String>,
                        paths.clone(),
                    );
                }
            }
        }
        Err(error) => {
            report_hot_reload_failure(&error);
            state.events.emit(
                DevEventType::RebuildFailed,
                None::<String>,
                Some(error),
                paths,
            );
        }
    }
}

fn apply_active_development_bindings(
    project: &mut Arc<CompiledProject>,
    current: &CompiledProject,
    selection: &DevTargetSelection,
) {
    let next = Arc::make_mut(project);
    if selection.contains(DevTarget::Server) {
        next.backend.port = current.backend.port;
        if let Some(inspector) = &mut next.server_inspector {
            inspector.port = current.backend.port;
        }
        preserve_environment_value(next, current, "BACKEND_URL");
        preserve_environment_value(next, current, "SERVER_URL");
    }
    if selection.contains(DevTarget::Desktop) {
        if let (Some(next_server), Some(current_server)) =
            (&mut next.desktop_server, &current.desktop_server)
        {
            next_server.port = current_server.port;
        }
        preserve_environment_value(next, current, "BACKEND_DESKTOP_URL");
        preserve_environment_value(next, current, "SERVER_DESKTOP_URL");
    }
}

fn preserve_environment_value(
    project: &mut CompiledProject,
    current: &CompiledProject,
    name: &str,
) {
    let Some(active) = current
        .environment_config
        .variables
        .iter()
        .find(|variable| variable.name == name)
        .cloned()
    else {
        return;
    };
    if let Some(variable) = project
        .environment_config
        .variables
        .iter_mut()
        .find(|variable| variable.name == name)
    {
        *variable = active;
    } else {
        project.environment_config.variables.push(active);
    }
}

fn report_hot_reload_failure(error: &str) {
    log_error(HOT_RELOAD_FAILED_MESSAGE);
    log_error(error);
}

async fn compile_watch_project(
    compiler: &mut DevCompilerSession,
    paths: Vec<String>,
    compile_server: bool,
    previous: Arc<CompiledProject>,
) -> Result<Arc<CompiledProject>, String> {
    let (sender, receiver) = oneshot::channel();
    let mut next = compiler.clone();
    std::thread::Builder::new()
        .name("dowe-watch-compiler".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let result = next
                .rebuild_snapshot_from(&paths, compile_server, &previous)
                .map(Arc::new)
                .map_err(|error| error.to_string());
            let _ = sender.send((next, result));
        })
        .map_err(|error| error.to_string())?;
    let (next, result) = receiver
        .await
        .map_err(|_| "watch compiler thread stopped before returning a result".to_string())?;
    *compiler = next;
    result
}

async fn debounce_changes(
    watcher: &mut SourceWatcher,
    paths: Vec<String>,
) -> RuntimeResult<Vec<String>> {
    let mut paths = paths.into_iter().collect::<BTreeSet<_>>();

    loop {
        match timeout(WATCH_QUIET_WINDOW, watcher.receive()).await {
            Ok(Ok(next)) => paths.extend(next),
            Ok(Err(error)) => return Err(error),
            Err(_) => break,
        }
    }

    Ok(paths.into_iter().collect())
}
