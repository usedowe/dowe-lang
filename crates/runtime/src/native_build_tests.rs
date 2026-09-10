#[cfg(test)]
mod tests {
    use super::{
        NativeBuildRequest, generated_native_app_output, native_build_failure_message,
        native_target_inputs_equal,
    };
    use crate::{
        DevEventBus, DevEventType, DevRuntimeState, DevTarget, DevTargetSelection, HostOs,
    };
    use dowe_compiler::{DevCompilerSession, ViewPlatform, compile_dev};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex, mpsc as standard_mpsc};
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::sync::{RwLock, watch};
    use tokio::time::timeout;

    #[test]
    fn labels_native_build_failures_for_terminal_output() {
        assert_eq!(
            native_build_failure_message(DevTarget::Ios, &"generated Swift is invalid"),
            "iOS app module build failed: generated Swift is invalid"
        );
    }

    #[tokio::test]
    async fn pending_native_requests_collapse_to_the_latest_revision() {
        let temp = TempDir::new().expect("tempdir");
        write_project(temp.path(), "first", "one");
        let project = Arc::new(compile_dev(temp.path()).expect("project"));
        let (sender, mut receiver) = watch::channel(None);
        for revision in 1..=3 {
            sender.send_replace(Some(NativeBuildRequest {
                revision,
                project: Arc::clone(&project),
                paths: vec![format!("revision-{revision}")],
            }));
        }

        receiver.changed().await.expect("request");
        let latest = receiver.borrow_and_update().clone().expect("latest");

        assert_eq!(latest.revision, 3);
        assert_eq!(latest.paths, ["revision-3"]);
    }

    #[test]
    fn server_only_change_preserves_native_inputs() {
        let temp = TempDir::new().expect("tempdir");
        write_project(temp.path(), "first", "one");
        let first = compile_dev(temp.path()).expect("first");
        write_project(temp.path(), "first", "two");
        let second = compile_dev(temp.path()).expect("second");

        for target in [DevTarget::Android, DevTarget::Ios] {
            assert!(native_target_inputs_equal(&first, &second, target));
        }
    }

    #[test]
    fn view_change_updates_each_native_input() {
        let temp = TempDir::new().expect("tempdir");
        write_project(temp.path(), "first", "one");
        let first = compile_dev(temp.path()).expect("first");
        write_project(temp.path(), "second", "one");
        let second = compile_dev(temp.path()).expect("second");

        for target in [DevTarget::Android, DevTarget::Ios] {
            assert!(!native_target_inputs_equal(&first, &second, target));
        }
    }

    #[test]
    fn native_workers_generate_target_bytes_from_an_app_free_snapshot() {
        let temp = TempDir::new().expect("tempdir");
        write_project(temp.path(), "first", "one");
        let mut compiler = DevCompilerSession::new(
            temp.path(),
            [ViewPlatform::Web, ViewPlatform::Android, ViewPlatform::Ios],
        )
        .expect("compiler");
        let project = compiler.compile_initial_web(false).expect("snapshot");
        assert!(project.apps.files.is_empty());

        for target in [DevTarget::Android, DevTarget::Ios] {
            let generated = generated_native_app_output(&project, target)
                .expect("generation")
                .expect("generated output");
            assert!(
                generated
                    .files
                    .iter()
                    .any(|file| file.target == target.as_str())
            );
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn initial_ios_module_build_is_queued_without_blocking_target_startup() {
        let temp = TempDir::new().expect("tempdir");
        write_project(temp.path(), "first", "one");
        let project = Arc::new(compile_dev(temp.path()).expect("project"));
        let state = DevRuntimeState {
            project: Arc::new(RwLock::new(Arc::clone(&project))),
            events: DevEventBus::new("ios-initial-module"),
            dev_origins: Vec::new(),
            cache_mode: crate::handlers::CacheRuntimeMode::Local,
        };
        let selection =
            DevTargetSelection::new([DevTarget::Ios], HostOs::Macos).expect("selection");
        let (build_sender, build_receiver) = standard_mpsc::channel();
        let builder = Arc::new(
            move |_: &dowe_compiler::CompiledProject,
                  target: DevTarget,
                  revision: &crate::dev_modules::DevModuleRevision| {
                build_sender.send(target).expect("build observation");
                Ok(revision
                    .is_current()
                    .then(|| crate::dev_modules::PublishedDevModule {
                        target: target.as_str().to_string(),
                        version: "initial-version".to_string(),
                        path: "/ios".to_string(),
                        file: PathBuf::from("ios"),
                    }))
            },
        );
        let coordinator = super::NativeBuildCoordinator::new_with_builder_and_initial_ios(
            &selection, &state, &project, builder,
        );

        assert_eq!(
            build_receiver
                .recv_timeout(Duration::from_secs(2))
                .expect("initial build"),
            DevTarget::Ios
        );
        coordinator.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn native_targets_publish_in_their_actual_completion_order() {
        let temp = TempDir::new().expect("tempdir");
        write_project(temp.path(), "first", "one");
        let first = Arc::new(compile_dev(temp.path()).expect("first"));
        write_project(temp.path(), "second", "one");
        let second = Arc::new(compile_dev(temp.path()).expect("second"));
        let state = DevRuntimeState {
            project: Arc::new(RwLock::new(Arc::clone(&first))),
            events: DevEventBus::new("native-order"),
            dev_origins: Vec::new(),
            cache_mode: crate::handlers::CacheRuntimeMode::Local,
        };
        let selection =
            DevTargetSelection::new([DevTarget::Android, DevTarget::Ios], HostOs::Macos)
                .expect("selection");
        let (android_sender, android_receiver) = standard_mpsc::sync_channel(0);
        let (ios_sender, ios_receiver) = standard_mpsc::sync_channel(0);
        let android_receiver = Arc::new(Mutex::new(android_receiver));
        let ios_receiver = Arc::new(Mutex::new(ios_receiver));
        let builder = Arc::new(
            move |_: &dowe_compiler::CompiledProject,
                  target: DevTarget,
                  revision: &crate::dev_modules::DevModuleRevision| {
                match target {
                    DevTarget::Android => android_receiver
                        .lock()
                        .expect("android gate")
                        .recv()
                        .expect("android release"),
                    DevTarget::Ios => ios_receiver
                        .lock()
                        .expect("ios gate")
                        .recv()
                        .expect("ios release"),
                    _ => unreachable!(),
                }
                Ok(revision
                    .is_current()
                    .then(|| crate::dev_modules::PublishedDevModule {
                        target: target.as_str().to_string(),
                        version: format!("{}-version", target.as_str()),
                        path: format!("/{}", target.as_str()),
                        file: PathBuf::from(target.as_str()),
                    }))
            },
        );
        let mut receiver = state.events.subscribe();
        let mut coordinator =
            super::NativeBuildCoordinator::new_with_builder(&selection, &state, &first, builder);

        coordinator.enqueue(second, vec!["pages/index.dowe".to_string()]);
        wait_for_started_targets(&mut receiver).await;
        android_sender.send(()).expect("release android");
        let android = wait_for_module_update(&mut receiver).await;
        assert_eq!(android.target.as_deref(), Some("android"));
        assert!(
            timeout(
                Duration::from_millis(100),
                wait_for_module_update(&mut receiver)
            )
            .await
            .is_err()
        );
        ios_sender.send(()).expect("release ios");
        let ios = wait_for_module_update(&mut receiver).await;
        assert_eq!(ios.target.as_deref(), Some("ios"));

        coordinator.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn active_native_build_skips_intermediate_pending_revisions() {
        let temp = TempDir::new().expect("tempdir");
        write_project(temp.path(), "first", "one");
        let first = Arc::new(compile_dev(temp.path()).expect("first"));
        write_project(temp.path(), "second", "one");
        let second = Arc::new(compile_dev(temp.path()).expect("second"));
        write_project(temp.path(), "third", "one");
        let third = Arc::new(compile_dev(temp.path()).expect("third"));
        write_project(temp.path(), "fourth", "one");
        let fourth = Arc::new(compile_dev(temp.path()).expect("fourth"));
        let state = DevRuntimeState {
            project: Arc::new(RwLock::new(Arc::clone(&first))),
            events: DevEventBus::new("native-latest"),
            dev_origins: Vec::new(),
            cache_mode: crate::handlers::CacheRuntimeMode::Local,
        };
        let selection =
            DevTargetSelection::new([DevTarget::Android], HostOs::Macos).expect("selection");
        let (release_sender, release_receiver) = standard_mpsc::sync_channel(0);
        let release_receiver = Arc::new(Mutex::new(release_receiver));
        let (build_sender, build_receiver) = standard_mpsc::channel();
        let invocation = Arc::new(AtomicUsize::new(0));
        let builder = Arc::new({
            let invocation = Arc::clone(&invocation);
            move |project: &dowe_compiler::CompiledProject,
                  target: DevTarget,
                  revision: &crate::dev_modules::DevModuleRevision| {
                let body = &project.web.pages[0].body_html;
                let label = ["second", "third", "fourth"]
                    .into_iter()
                    .find(|label| body.contains(label))
                    .expect("page label")
                    .to_string();
                build_sender.send(label).expect("build observation");
                if invocation.fetch_add(1, Ordering::SeqCst) == 0 {
                    release_receiver
                        .lock()
                        .expect("build gate")
                        .recv()
                        .expect("build release");
                }
                Ok(revision
                    .is_current()
                    .then(|| crate::dev_modules::PublishedDevModule {
                        target: target.as_str().to_string(),
                        version: "latest-version".to_string(),
                        path: "/android".to_string(),
                        file: PathBuf::from("android"),
                    }))
            }
        });
        let mut receiver = state.events.subscribe();
        let mut coordinator =
            super::NativeBuildCoordinator::new_with_builder(&selection, &state, &first, builder);

        coordinator.enqueue(second, vec!["second".to_string()]);
        assert_eq!(
            build_receiver
                .recv_timeout(Duration::from_secs(2))
                .expect("first build"),
            "second"
        );
        coordinator.enqueue(third, vec!["third".to_string()]);
        coordinator.enqueue(fourth, vec!["fourth".to_string()]);
        release_sender.send(()).expect("release first build");
        let update = wait_for_module_update(&mut receiver).await;

        assert_eq!(update.target.as_deref(), Some("android"));
        assert_eq!(update.paths, ["fourth"]);
        assert_eq!(
            build_receiver
                .recv_timeout(Duration::from_secs(2))
                .expect("latest build"),
            "fourth"
        );
        assert!(build_receiver.try_recv().is_err());
        coordinator.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn failed_native_input_can_be_retried_without_native_output_change() {
        let temp = TempDir::new().expect("tempdir");
        write_project(temp.path(), "first", "one");
        let first = Arc::new(compile_dev(temp.path()).expect("first"));
        write_project(temp.path(), "second", "one");
        let second = Arc::new(compile_dev(temp.path()).expect("second"));
        write_project(temp.path(), "second", "two");
        let retry = Arc::new(compile_dev(temp.path()).expect("retry"));
        assert!(native_target_inputs_equal(
            &second,
            &retry,
            DevTarget::Android
        ));
        let state = DevRuntimeState {
            project: Arc::new(RwLock::new(Arc::clone(&first))),
            events: DevEventBus::new("native-retry"),
            dev_origins: Vec::new(),
            cache_mode: crate::handlers::CacheRuntimeMode::Local,
        };
        let selection =
            DevTargetSelection::new([DevTarget::Android], HostOs::Macos).expect("selection");
        let invocation = Arc::new(AtomicUsize::new(0));
        let builder = Arc::new({
            let invocation = Arc::clone(&invocation);
            move |_: &dowe_compiler::CompiledProject,
                  target: DevTarget,
                  _: &crate::dev_modules::DevModuleRevision| {
                if invocation.fetch_add(1, Ordering::SeqCst) == 0 {
                    return Err(crate::RuntimeError::new("transient native failure"));
                }
                Ok(Some(crate::dev_modules::PublishedDevModule {
                    target: target.as_str().to_string(),
                    version: "retry-version".to_string(),
                    path: "/android".to_string(),
                    file: PathBuf::from("android"),
                }))
            }
        });
        let mut receiver = state.events.subscribe();
        let mut coordinator =
            super::NativeBuildCoordinator::new_with_builder(&selection, &state, &first, builder);

        coordinator.enqueue(second, vec!["page".to_string()]);
        wait_for_event(&mut receiver, DevEventType::ModuleBuildFailed).await;
        coordinator.enqueue(retry, vec!["server".to_string()]);
        let update = wait_for_module_update(&mut receiver).await;

        assert_eq!(update.target.as_deref(), Some("android"));
        assert_eq!(update.paths, ["server"]);
        assert_eq!(invocation.load(Ordering::SeqCst), 2);
        coordinator.shutdown();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropping_coordinator_invalidates_an_active_revision() {
        let temp = TempDir::new().expect("tempdir");
        write_project(temp.path(), "first", "one");
        let first = Arc::new(compile_dev(temp.path()).expect("first"));
        write_project(temp.path(), "second", "one");
        let second = Arc::new(compile_dev(temp.path()).expect("second"));
        let state = DevRuntimeState {
            project: Arc::new(RwLock::new(Arc::clone(&first))),
            events: DevEventBus::new("native-drop"),
            dev_origins: Vec::new(),
            cache_mode: crate::handlers::CacheRuntimeMode::Local,
        };
        let selection =
            DevTargetSelection::new([DevTarget::Android], HostOs::Macos).expect("selection");
        let (revision_sender, revision_receiver) = standard_mpsc::channel();
        let (release_sender, release_receiver) = standard_mpsc::sync_channel(0);
        let release_receiver = Arc::new(Mutex::new(release_receiver));
        let builder = Arc::new(
            move |_: &dowe_compiler::CompiledProject,
                  _: DevTarget,
                  revision: &crate::dev_modules::DevModuleRevision| {
                revision_sender
                    .send(revision.clone())
                    .expect("revision observation");
                release_receiver
                    .lock()
                    .expect("build gate")
                    .recv()
                    .expect("build release");
                Ok(None)
            },
        );
        let mut coordinator =
            super::NativeBuildCoordinator::new_with_builder(&selection, &state, &first, builder);

        coordinator.enqueue(second, vec!["second".to_string()]);
        let revision = revision_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("active revision");
        drop(coordinator);

        assert!(!revision.is_current());
        release_sender.send(()).expect("release build");
    }

    async fn wait_for_started_targets(
        receiver: &mut tokio::sync::broadcast::Receiver<crate::DevEvent>,
    ) {
        let mut targets = std::collections::BTreeSet::new();
        timeout(Duration::from_secs(2), async {
            while targets.len() < 2 {
                let event = receiver.recv().await.expect("event");
                if event.event_type == DevEventType::ModuleBuildStarted {
                    targets.insert(event.target.expect("target"));
                }
            }
        })
        .await
        .expect("start timeout");
        assert_eq!(targets, ["android".to_string(), "ios".to_string()].into());
    }

    async fn wait_for_module_update(
        receiver: &mut tokio::sync::broadcast::Receiver<crate::DevEvent>,
    ) -> crate::DevEvent {
        timeout(Duration::from_secs(2), async {
            loop {
                let event = receiver.recv().await.expect("event");
                if event.event_type == DevEventType::ModuleUpdate {
                    return event;
                }
            }
        })
        .await
        .expect("module update timeout")
    }

    async fn wait_for_event(
        receiver: &mut tokio::sync::broadcast::Receiver<crate::DevEvent>,
        event_type: DevEventType,
    ) {
        timeout(Duration::from_secs(2), async {
            loop {
                if receiver.recv().await.expect("event").event_type == event_type {
                    return;
                }
            }
        })
        .await
        .expect("event timeout");
    }

    fn write_project(root: &std::path::Path, text: &str, server_text: &str) {
        fs::create_dir_all(root.join("pages")).expect("pages");
        fs::create_dir_all(root.join("routes")).expect("routes");
        fs::write(
            root.join("main.dowe"),
            format!(
                "import routes from \"@/routes/view\"\n\nmain\n  views:routes\n  server port:0\n    route \"/api/status\"\n      response text:\"{server_text}\"\n"
            ),
        )
        .expect("main");
        fs::write(
            root.join("routes/view.dowe"),
            "import indexPage from \"../pages/index\"\n\nviews routes\n  route path:\"/\" page:indexPage\n",
        )
        .expect("routes");
        fs::write(
            root.join("pages/index.dowe"),
            format!("page indexPage\n  Text\n    \"{text}\"\n"),
        )
        .expect("page");
    }
}
