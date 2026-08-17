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
    let compiled =
        compile_watch_project(compiler, paths.clone(), compile_server, previous_project).await;

    match compiled {
        Ok(mut project) => {
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
                next.server_inspector = current.server_inspector.clone();
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

#[cfg(test)]
mod tests {
    use super::{HOT_RELOAD_COMPLETED_MESSAGE, handle_watch_changes, run_watch_loop};
    use crate::dev_native_builds::NativeBuildCoordinator;
    use crate::watch::SourceWatcher;
    use crate::{
        DevEvent, DevEventBus, DevEventType, DevRuntimeState, DevTarget, DevTargetSelection, HostOs,
    };
    use dowe_compiler::{DevCompilerSession, ViewPlatform};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex, mpsc as standard_mpsc};
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::sync::{RwLock, broadcast, oneshot};
    use tokio::time::timeout;

    #[tokio::test]
    async fn watch_rebuild_updates_project_and_emits_reload() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path(), "Login");
        let mut compiler =
            DevCompilerSession::new(temp.path(), [ViewPlatform::Web]).expect("compiler");
        let project = compiler.compile_initial(true).expect("project");
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
            compiler,
            stop_receiver,
        ));

        wait_for_event(&mut receiver, DevEventType::WatchReady).await;
        write_page_fixture(temp.path(), "Changed");
        let rebuild = wait_for_event(&mut receiver, DevEventType::RebuildSucceeded).await;
        assert_eq!(
            rebuild.message.as_deref(),
            Some(HOT_RELOAD_COMPLETED_MESSAGE)
        );
        wait_for_event(&mut receiver, DevEventType::ModuleUpdate).await;

        let current = state.project.read().await;
        assert!(current.web.pages[0].body_html.contains("Changed"));
        assert_eq!(current.backend.endpoints.len(), 1);
        assert!(current.server_inspector.is_some());
        assert!(temp.path().join(".dowe/server/inspector.json").exists());
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
    async fn unchanged_page_save_does_not_emit_a_web_module_update() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path(), "Login");
        let mut compiler =
            DevCompilerSession::new(temp.path(), [ViewPlatform::Web]).expect("compiler");
        let project = compiler.compile_initial(false).expect("project");
        let state = DevRuntimeState {
            project: Arc::new(RwLock::new(Arc::new(project))),
            events: DevEventBus::new("unchanged-watch-test"),
            dev_origins: Vec::new(),
            cache_mode: crate::handlers::CacheRuntimeMode::Local,
        };
        let selection =
            DevTargetSelection::new([DevTarget::Web], HostOs::Linux).expect("selection");
        let mut receiver = state.events.subscribe();
        let (stop_sender, stop_receiver) = oneshot::channel();
        let handle = tokio::spawn(run_watch_loop(
            temp.path().to_path_buf(),
            selection,
            state,
            compiler,
            stop_receiver,
        ));

        wait_for_event(&mut receiver, DevEventType::WatchReady).await;
        write_page_fixture(temp.path(), "Login");
        wait_for_event(&mut receiver, DevEventType::RebuildSucceeded).await;
        let module_update = timeout(Duration::from_millis(300), async {
            loop {
                let event = receiver.recv().await.expect("event");
                if event.event_type == DevEventType::ModuleUpdate {
                    break event;
                }
            }
        })
        .await;
        assert!(module_update.is_err());

        let _ = stop_sender.send(());
        handle.await.expect("watch task").expect("watch result");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn web_update_does_not_wait_for_the_selected_native_target() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path(), "Login");
        let mut compiler =
            DevCompilerSession::new(temp.path(), [ViewPlatform::Web, ViewPlatform::Android])
                .expect("compiler");
        let project = Arc::new(compiler.compile_initial(false).expect("project"));
        let state = DevRuntimeState {
            project: Arc::new(RwLock::new(Arc::clone(&project))),
            events: DevEventBus::new("multi-target-watch-test"),
            dev_origins: Vec::new(),
            cache_mode: crate::handlers::CacheRuntimeMode::Local,
        };
        let selection =
            DevTargetSelection::new([DevTarget::Web, DevTarget::Android], HostOs::Macos)
                .expect("selection");
        let (build_sender, build_receiver) = standard_mpsc::channel();
        let (release_sender, release_receiver) = standard_mpsc::sync_channel(0);
        let release_receiver = Arc::new(Mutex::new(release_receiver));
        let mut native_builds = NativeBuildCoordinator::new_for_test(
            &selection,
            &state,
            &project,
            move |project, target, revision| {
                assert_eq!(target, DevTarget::Android);
                assert!(project.apps.files.is_empty());
                build_sender.send(()).expect("build observation");
                release_receiver
                    .lock()
                    .expect("build gate")
                    .recv()
                    .expect("build release");
                Ok(revision
                    .is_current()
                    .then(|| crate::dev_modules::PublishedDevModule {
                        target: target.as_str().to_string(),
                        version: "android-version".to_string(),
                        path: "/android".to_string(),
                        file: PathBuf::from("android"),
                    }))
            },
        );
        let mut receiver = state.events.subscribe();
        write_page_fixture(temp.path(), "Changed");
        let mut watcher = SourceWatcher::new(temp.path()).expect("watcher");

        handle_watch_changes(
            temp.path(),
            &selection,
            &state,
            &mut compiler,
            &mut native_builds,
            &mut watcher,
            vec!["pages/login.dowe".to_string()],
        )
        .await;

        let web = wait_for_event(&mut receiver, DevEventType::ModuleUpdate).await;
        assert_eq!(web.target.as_deref(), Some("web"));
        build_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("native build started");
        release_sender.send(()).expect("release native build");
        native_builds.shutdown();
    }

    #[tokio::test]
    async fn failed_rebuild_keeps_the_last_project_and_recovers_on_the_next_save() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path(), "Login");
        let mut compiler =
            DevCompilerSession::new(temp.path(), [ViewPlatform::Web]).expect("compiler");
        let project = compiler.compile_initial(false).expect("project");
        let state = DevRuntimeState {
            project: Arc::new(RwLock::new(Arc::new(project))),
            events: DevEventBus::new("recovery-watch-test"),
            dev_origins: Vec::new(),
            cache_mode: crate::handlers::CacheRuntimeMode::Local,
        };
        let selection =
            DevTargetSelection::new([DevTarget::Web], HostOs::Linux).expect("selection");
        let mut receiver = state.events.subscribe();
        let (stop_sender, stop_receiver) = oneshot::channel();
        let handle = tokio::spawn(run_watch_loop(
            temp.path().to_path_buf(),
            selection,
            state.clone(),
            compiler,
            stop_receiver,
        ));

        wait_for_event(&mut receiver, DevEventType::WatchReady).await;
        fs::write(
            temp.path().join("pages/login.dowe"),
            "page loginPage\n  Text\n    Login\n",
        )
        .expect("invalid page");
        wait_for_event(&mut receiver, DevEventType::RebuildFailed).await;
        assert!(
            state.project.read().await.web.pages[0]
                .body_html
                .contains("Login")
        );

        write_page_fixture(temp.path(), "Recovered");
        wait_for_event(&mut receiver, DevEventType::RebuildSucceeded).await;
        wait_for_event(&mut receiver, DevEventType::ModuleUpdate).await;
        assert!(
            state.project.read().await.web.pages[0]
                .body_html
                .contains("Recovered")
        );

        let _ = stop_sender.send(());
        handle.await.expect("watch task").expect("watch result");
    }

    #[tokio::test]
    async fn server_rebuild_replaces_the_server_model_and_emits_restart() {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path(), "Login");
        let mut compiler =
            DevCompilerSession::new(temp.path(), [ViewPlatform::Web]).expect("compiler");
        let project = compiler.compile_initial(true).expect("project");
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
            compiler,
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

    async fn wait_for_event(
        receiver: &mut broadcast::Receiver<DevEvent>,
        expected: DevEventType,
    ) -> DevEvent {
        timeout(Duration::from_secs(4), async {
            loop {
                let event = receiver.recv().await.expect("event");
                if event.event_type == expected {
                    break event;
                }
            }
        })
        .await
        .expect("event timeout")
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
