use crate::dev::{DevTarget, DevTargetSelection};
use crate::dev_modules::{DevModuleRevision, PublishedDevModule};
use crate::dev_targets::{
    build_hot_module_if_current, cancel_active_external_commands,
    cancel_active_external_commands_for,
};
use crate::error::{RuntimeError, RuntimeResult};
use crate::logging::log_error;
use crate::{DevEventType, DevRuntimeState};
use dowe_compiler::{AppOutput, CompiledProject, ViewPlatform, generate_dev_app_output};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;
use tokio::task::JoinHandle;

type NativeBuildFunction = Arc<
    dyn Fn(
            &CompiledProject,
            DevTarget,
            &DevModuleRevision,
        ) -> RuntimeResult<Option<PublishedDevModule>>
        + Send
        + Sync,
>;

pub(crate) struct NativeBuildCoordinator {
    revision: u64,
    workers: BTreeMap<DevTarget, NativeBuildWorker>,
    handles: Vec<JoinHandle<()>>,
    stopped: bool,
}

struct NativeBuildWorker {
    latest: Arc<Mutex<u64>>,
    projects: Arc<Mutex<NativeBuildProjects>>,
    sender: watch::Sender<Option<NativeBuildRequest>>,
}

struct NativeBuildProjects {
    published: Option<Arc<CompiledProject>>,
    requested: Option<Arc<CompiledProject>>,
    requested_revision: u64,
}

#[derive(Clone)]
struct NativeBuildRequest {
    revision: u64,
    project: Arc<CompiledProject>,
    paths: Vec<String>,
}

fn build_generated_hot_module_if_current(
    project: &CompiledProject,
    target: DevTarget,
    revision: &DevModuleRevision,
) -> RuntimeResult<Option<PublishedDevModule>> {
    let generated = generated_native_app_output(project, target)?;
    let files = generated
        .as_ref()
        .map(|output| output.files.as_slice())
        .unwrap_or(project.apps.files.as_slice());
    build_hot_module_if_current(&project.root, files, target, revision)
}

fn generated_native_app_output(
    project: &CompiledProject,
    target: DevTarget,
) -> RuntimeResult<Option<AppOutput>> {
    if project
        .apps
        .files
        .iter()
        .any(|file| file.target == target.as_str())
    {
        return Ok(None);
    }
    let platform = match target {
        DevTarget::Android => ViewPlatform::Android,
        DevTarget::Ios => ViewPlatform::Ios,
        _ => {
            return Err(RuntimeError::new(format!(
                "{} does not use generated native app output",
                target.label()
            )));
        }
    };
    generate_dev_app_output(project, platform)
        .map(Some)
        .map_err(RuntimeError::from)
}

impl NativeBuildCoordinator {
    pub(crate) fn new(
        selection: &DevTargetSelection,
        state: &DevRuntimeState,
        project: &Arc<CompiledProject>,
    ) -> Self {
        Self::new_with_builder_and_initial_ios(
            selection,
            state,
            project,
            Arc::new(build_generated_hot_module_if_current),
        )
    }

    fn new_with_builder_and_initial_ios(
        selection: &DevTargetSelection,
        state: &DevRuntimeState,
        project: &Arc<CompiledProject>,
        builder: NativeBuildFunction,
    ) -> Self {
        let mut coordinator = Self::new_with_builder(selection, state, project, builder);
        if selection.contains(DevTarget::Ios) {
            coordinator.enqueue_initial(project, DevTarget::Ios);
        }
        coordinator
    }

    fn new_with_builder(
        selection: &DevTargetSelection,
        state: &DevRuntimeState,
        project: &Arc<CompiledProject>,
        builder: NativeBuildFunction,
    ) -> Self {
        let mut workers = BTreeMap::new();
        let mut handles = Vec::new();
        for target in [DevTarget::Android, DevTarget::Ios]
            .into_iter()
            .filter(|target| selection.contains(*target))
        {
            let latest = Arc::new(Mutex::new(0));
            let projects = Arc::new(Mutex::new(NativeBuildProjects {
                published: Some(Arc::clone(project)),
                requested: None,
                requested_revision: 0,
            }));
            let (sender, receiver) = watch::channel(None);
            handles.push(tokio::spawn(run_native_build_worker(
                target,
                Arc::clone(&latest),
                Arc::clone(&projects),
                receiver,
                state.clone(),
                Arc::clone(&builder),
            )));
            workers.insert(
                target,
                NativeBuildWorker {
                    latest,
                    projects,
                    sender,
                },
            );
        }
        Self {
            revision: 0,
            workers,
            handles,
            stopped: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(
        selection: &DevTargetSelection,
        state: &DevRuntimeState,
        project: &Arc<CompiledProject>,
        builder: impl Fn(
            &CompiledProject,
            DevTarget,
            &DevModuleRevision,
        ) -> RuntimeResult<Option<PublishedDevModule>>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self::new_with_builder(selection, state, project, Arc::new(builder))
    }

    pub(crate) fn enqueue(&mut self, project: Arc<CompiledProject>, paths: Vec<String>) {
        self.revision = self.revision.saturating_add(1);
        for (target, worker) in &mut self.workers {
            {
                let mut projects = worker.projects.lock().expect("native build project lock");
                let matches_requested = projects.requested.as_ref().is_some_and(|requested| {
                    native_target_inputs_equal(&project, requested, *target)
                });
                let matches_published = projects.requested.is_none()
                    && projects.published.as_ref().is_some_and(|published| {
                        native_target_inputs_equal(&project, published, *target)
                    });
                if matches_requested || matches_published {
                    continue;
                }
                projects.requested = Some(Arc::clone(&project));
                projects.requested_revision = self.revision;
            }
            *worker.latest.lock().expect("native build revision lock") = self.revision;
            cancel_active_external_commands_for(*target);
            worker.sender.send_replace(Some(NativeBuildRequest {
                revision: self.revision,
                project: Arc::clone(&project),
                paths: paths.clone(),
            }));
        }
    }

    fn enqueue_initial(&mut self, project: &Arc<CompiledProject>, target: DevTarget) {
        let Some(worker) = self.workers.get_mut(&target) else {
            return;
        };
        self.revision = self.revision.saturating_add(1);
        {
            let mut projects = worker.projects.lock().expect("native build project lock");
            projects.published = None;
            projects.requested = Some(Arc::clone(project));
            projects.requested_revision = self.revision;
        }
        *worker.latest.lock().expect("native build revision lock") = self.revision;
        worker.sender.send_replace(Some(NativeBuildRequest {
            revision: self.revision,
            project: Arc::clone(project),
            paths: Vec::new(),
        }));
    }

    pub(crate) fn shutdown(mut self) {
        self.stop();
    }

    fn stop(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        if self.workers.is_empty() {
            return;
        }
        for worker in self.workers.values() {
            if let Ok(mut latest) = worker.latest.lock() {
                *latest = latest.saturating_add(1);
            }
        }
        cancel_active_external_commands();
        for handle in &self.handles {
            handle.abort();
        }
    }
}

impl Drop for NativeBuildCoordinator {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn run_native_build_worker(
    target: DevTarget,
    latest: Arc<Mutex<u64>>,
    projects: Arc<Mutex<NativeBuildProjects>>,
    mut receiver: watch::Receiver<Option<NativeBuildRequest>>,
    state: DevRuntimeState,
    builder: NativeBuildFunction,
) {
    while receiver.changed().await.is_ok() {
        let Some(request) = receiver.borrow_and_update().clone() else {
            continue;
        };
        let revision = DevModuleRevision::new(request.revision, Arc::clone(&latest));
        if !revision.is_current() {
            continue;
        }
        state.events.emit(
            DevEventType::ModuleBuildStarted,
            Some(target.as_str()),
            None::<String>,
            request.paths.clone(),
        );
        let project = Arc::clone(&request.project);
        let build_revision = revision.clone();
        let build = Arc::clone(&builder);
        let result =
            tokio::task::spawn_blocking(move || build(project.as_ref(), target, &build_revision))
                .await;
        match result {
            Ok(Ok(Some(module))) => {
                mark_native_build_published(&projects, &request);
                let _ = revision.run_if_current(|| {
                    state.events.emit_module_update(
                        module.target,
                        module.version,
                        request.paths.clone(),
                    );
                });
            }
            Ok(Ok(None)) => {
                mark_native_build_retryable(&projects, &request);
            }
            Ok(Err(error)) => {
                mark_native_build_retryable(&projects, &request);
                let _ = revision.run_if_current(|| {
                    log_error(native_build_failure_message(target, &error));
                    state.events.emit(
                        DevEventType::ModuleBuildFailed,
                        Some(target.as_str()),
                        Some(error.to_string()),
                        request.paths.clone(),
                    );
                });
            }
            Err(error) => {
                mark_native_build_retryable(&projects, &request);
                let _ = revision.run_if_current(|| {
                    log_error(native_build_failure_message(target, &error));
                    state.events.emit(
                        DevEventType::ModuleBuildFailed,
                        Some(target.as_str()),
                        Some(error.to_string()),
                        request.paths.clone(),
                    );
                });
            }
        }
    }
}

