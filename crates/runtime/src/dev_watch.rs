use crate::dev::{DevTarget, DevTargetSelection, selected_view_platforms};
use crate::dev_modules::web_module_version;
use crate::dev_native_builds::NativeBuildCoordinator;
use crate::error::RuntimeResult;
use crate::logging::log_error;
use crate::server_actions::execute_server_action;
use crate::watch::SourceWatcher;
use crate::{DevEventType, DevRuntimeState};
use dowe_compiler::{
    CompiledProject, DevChangeScope, ViewPlatform, classify_dev_changes, compile_dev_for_platforms,
    compile_dev_views_for_platforms,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::{MissedTickBehavior, interval, sleep};

pub(crate) async fn run_watch_loop(
    root: PathBuf,
    selection: DevTargetSelection,
    state: DevRuntimeState,
    mut stop: oneshot::Receiver<()>,
) -> RuntimeResult<()> {
    let mut watcher = SourceWatcher::new(&root)?;
    let initial_project = {
        let project = state.project.read().await;
        Arc::clone(&project)
    };
    let mut native_builds = NativeBuildCoordinator::new(&selection, &state, &initial_project);
    let mut ticks = interval(Duration::from_millis(250));
    ticks.set_missed_tick_behavior(MissedTickBehavior::Delay);
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
            _ = ticks.tick() => {
                let paths = watcher.poll()?;
                if !paths.is_empty() {
                    handle_watch_changes(
                        &root,
                        &selection,
                        &state,
                        &mut native_builds,
                        &mut watcher,
                        paths,
                    ).await;
                }
            }
        }
    }
}

async fn handle_watch_changes(
    root: &Path,
    selection: &DevTargetSelection,
    state: &DevRuntimeState,
    native_builds: &mut NativeBuildCoordinator,
    watcher: &mut SourceWatcher,
    paths: Vec<String>,
) {
    let paths = debounce_changes(watcher, paths).await;
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
    let compiled = compile_watch_project(
        root.to_path_buf(),
        selected_view_platforms(selection),
        compile_server,
    )
    .await;

    match compiled {
        Ok(mut project) => {
            if compile_server {
                Arc::make_mut(&mut project).local_databases = true;
                if let Err(error) = crate::database_bootstrap::prepare_databases(&project).await {
                    let error = error.to_string();
                    log_error(&error);
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
                next.local_databases = current.local_databases;
            }
            let server_init_action = (compile_server && selection.contains(DevTarget::Server))
                .then(|| project.backend.init_action.clone());

            {
                let mut current = state.project.write().await;
                *current = Arc::clone(&project);
            }

            state.events.emit(
                DevEventType::RebuildSucceeded,
                None::<String>,
                None::<String>,
                paths.clone(),
            );

            if selection.contains(DevTarget::Web) {
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
            log_error(&error);
            state.events.emit(
                DevEventType::RebuildFailed,
                None::<String>,
                Some(error),
                paths,
            );
        }
    }
}

async fn compile_watch_project(
    root: PathBuf,
    platforms: Vec<ViewPlatform>,
    compile_server: bool,
) -> Result<Arc<CompiledProject>, String> {
    let (sender, receiver) = oneshot::channel();
    std::thread::Builder::new()
        .name("dowe-watch-compiler".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let result = if compile_server {
                compile_dev_for_platforms(&root, platforms)
            } else {
                compile_dev_views_for_platforms(&root, platforms)
            }
            .map(Arc::new)
            .map_err(|error| error.to_string());
            let _ = sender.send(result);
        })
        .map_err(|error| error.to_string())?;
    receiver
        .await
        .map_err(|_| "watch compiler thread stopped before returning a result".to_string())?
}

async fn debounce_changes(watcher: &mut SourceWatcher, paths: Vec<String>) -> Vec<String> {
    let mut paths = paths.into_iter().collect::<BTreeSet<_>>();

    loop {
        sleep(Duration::from_millis(150)).await;
        let Ok(next) = watcher.poll() else {
            break;
        };
        if next.is_empty() {
            break;
        }
        paths.extend(next);
    }

    paths.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::run_watch_loop;
    use crate::{
        DevEvent, DevEventBus, DevEventType, DevRuntimeState, DevTarget, DevTargetSelection, HostOs,
    };
    use dowe_compiler::compile_dev;
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::sync::{RwLock, broadcast, oneshot};
    use tokio::time::timeout;

    #[tokio::test]
    async fn watch_rebuild_updates_project_and_emits_reload() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path(), "Login");
        let project = compile_dev(temp.path()).expect("project");
        let state = DevRuntimeState {
            project: Arc::new(RwLock::new(Arc::new(project))),
            events: DevEventBus::new("watch-test"),
            dev_origins: Vec::new(),
            cache_mode: crate::handlers::CacheRuntimeMode::Local,
        };
        let selection = DevTargetSelection::new([DevTarget::Server, DevTarget::Web], HostOs::Linux)
            .expect("selection");
        let mut receiver = state.events.subscribe();
        let (stop_sender, stop_receiver) = oneshot::channel();
        let handle = tokio::spawn(run_watch_loop(
            temp.path().to_path_buf(),
            selection,
            state.clone(),
            stop_receiver,
        ));

        wait_for_event(&mut receiver, DevEventType::WatchReady).await;
        write_page_fixture(temp.path(), "Changed");
        wait_for_event(&mut receiver, DevEventType::ModuleUpdate).await;

        let current = state.project.read().await;
        assert!(current.web.pages[0].body_html.contains("Changed"));
        assert_eq!(current.backend.endpoints.len(), 1);
        assert!(current.apps.files.is_empty());
        assert!(!temp.path().join(".dowe/apps").exists());
        drop(current);
        tokio::task::yield_now().await;
        while let Ok(event) = receiver.try_recv() {
            assert_ne!(event.event_type, DevEventType::TargetRestarting);
            assert_ne!(event.event_type, DevEventType::TargetReady);
        }

        let _ = stop_sender.send(());
        handle.await.expect("watch task").expect("watch result");
    }

    #[tokio::test]
    async fn server_rebuild_replaces_the_server_model_and_emits_restart() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path(), "Login");
        let project = compile_dev(temp.path()).expect("project");
        let state = DevRuntimeState {
            project: Arc::new(RwLock::new(Arc::new(project))),
            events: DevEventBus::new("server-watch-test"),
            dev_origins: Vec::new(),
            cache_mode: crate::handlers::CacheRuntimeMode::Local,
        };
        let selection = DevTargetSelection::new([DevTarget::Server, DevTarget::Web], HostOs::Linux)
            .expect("selection");
        let mut receiver = state.events.subscribe();
        let (stop_sender, stop_receiver) = oneshot::channel();
        let handle = tokio::spawn(run_watch_loop(
            temp.path().to_path_buf(),
            selection,
            state.clone(),
            stop_receiver,
        ));

        wait_for_event(&mut receiver, DevEventType::WatchReady).await;
        write_server_fixture(temp.path(), true);
        wait_for_event(&mut receiver, DevEventType::TargetRestarting).await;

        let current = state.project.read().await;
        assert_eq!(current.backend.endpoints.len(), 2);

        let _ = stop_sender.send(());
        handle.await.expect("watch task").expect("watch result");
    }

    async fn wait_for_event(receiver: &mut broadcast::Receiver<DevEvent>, expected: DevEventType) {
        timeout(Duration::from_secs(4), async {
            loop {
                let event = receiver.recv().await.expect("event");
                if event.event_type == expected {
                    break;
                }
            }
        })
        .await
        .expect("event timeout");
    }

    fn write_fixture(root: &Path, page_text: &str) {
        fs::create_dir_all(root.join("layouts")).expect("layouts");
        fs::create_dir_all(root.join("pages")).expect("pages");
        fs::create_dir_all(root.join("routes")).expect("routes");
        write_server_fixture(root, false);
        fs::write(
            root.join("routes/view.dowe"),
            r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"

views viewRoutes
  group path:"/" layout:AuthLayout
    route path:"" page:loginPage"#,
        )
        .expect("views");
        fs::write(
            root.join("layouts/auth.dowe"),
            r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
        )
        .expect("layout");
        write_page_fixture(root, page_text);
    }

    fn write_server_fixture(root: &Path, include_second_route: bool) {
        let second_route = if include_second_route {
            r#"
    route "/api/ready"
      response text:"READY""#
        } else {
            ""
        };
        fs::write(
            root.join("main.dowe"),
            format!(
                r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/status"
      response text:"OK"
{second_route}
    init
      log "Server inicializado""#
            ),
        )
        .expect("server");
    }

    fn write_page_fixture(root: &Path, page_text: &str) {
        fs::write(
            root.join("pages/login.dowe"),
            format!(
                r#"page loginPage
  Box
    Text
      "{page_text}""#
            ),
        )
        .expect("page");
    }
}
