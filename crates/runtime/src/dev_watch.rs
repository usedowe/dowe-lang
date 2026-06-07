use crate::dev::{DevTarget, DevTargetSelection};
use crate::error::RuntimeResult;
use crate::logging::log_error;
use crate::server_actions::execute_server_action;
use crate::watch::SourceWatcher;
use crate::{DevEventType, DevRuntimeState};
use dowe_compiler::compile_dev;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
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
                state.events.emit(DevEventType::Shutdown, None::<String>, None::<String>, Vec::new());
                return Ok(());
            }
            _ = ticks.tick() => {
                let paths = watcher.poll()?;
                if !paths.is_empty() {
                    handle_watch_changes(&root, &selection, &state, &mut watcher, paths).await;
                }
            }
        }
    }
}

async fn handle_watch_changes(
    root: &Path,
    selection: &DevTargetSelection,
    state: &DevRuntimeState,
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

    match compile_dev(root) {
        Ok(project) => {
            if selection.contains(DevTarget::Server) {
                state.events.emit(
                    DevEventType::TargetRestarting,
                    Some(DevTarget::Server.as_str()),
                    None::<String>,
                    paths.clone(),
                );
                execute_server_action(&project.backend.init_action);
            }

            {
                let mut current = state.project.write().await;
                *current = project;
            }

            state.events.emit(
                DevEventType::RebuildSucceeded,
                None::<String>,
                None::<String>,
                paths.clone(),
            );

            if selection.contains(DevTarget::Server) {
                state.events.emit(
                    DevEventType::TargetReady,
                    Some(DevTarget::Server.as_str()),
                    None::<String>,
                    paths.clone(),
                );
            }

            for target in [
                DevTarget::Web,
                DevTarget::Desktop,
                DevTarget::Android,
                DevTarget::Ios,
            ] {
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
            let message = error.to_string();
            log_error(&message);
            state.events.emit(
                DevEventType::RebuildFailed,
                None::<String>,
                Some(message),
                paths,
            );
        }
    }
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
            project: Arc::new(RwLock::new(project)),
            events: DevEventBus::new("watch-test"),
            dev_origins: Vec::new(),
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
        write_fixture(temp.path(), "Changed");
        wait_for_event(&mut receiver, DevEventType::Reload).await;

        let current = state.project.read().await;
        assert!(current.web.pages[0].body_html.contains("Changed"));

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
        fs::create_dir_all(root.join("src/layouts")).expect("layouts");
        fs::create_dir_all(root.join("src/pages")).expect("pages");
        fs::write(
            root.join("src/main.dowe"),
            r#"main
  server port:0
    route "/api/status"
      response text:"OK"
    init
      log "Server inicializado""#,
        )
        .expect("server");
        fs::write(
            root.join("src/views.dowe"),
            r#"import AuthLayout from "./layouts/auth"
import loginPage from "./pages/login"

views
  route path:"/" layout:AuthLayout
    page path:"" component:loginPage"#,
        )
        .expect("views");
        fs::write(
            root.join("src/layouts/auth.dowe"),
            r#"layout AuthLayout
  Box
    Text
      Layout
    children"#,
        )
        .expect("layout");
        fs::write(
            root.join("src/pages/login.dowe"),
            format!(
                r#"page loginPage
  Box
    Text
      {page_text}"#
            ),
        )
        .expect("page");
    }
}
