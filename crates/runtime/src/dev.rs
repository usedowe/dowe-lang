use crate::dev_targets::{cancel_active_external_commands, start_external_target};
use crate::dev_watch::run_watch_loop;
use crate::error::{RuntimeError, RuntimeResult};
use crate::logging::LoadingStatus;
use crate::server::{DevServerTargets, RunningDevServers, start_dev_servers};
use dowe_compiler::{CompiledProject, compile_dev};
use dowe_spawn::{ChildProcess, ProcessControl, SpawnConfig, run};
use futures_util::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::sync::oneshot;
use tokio::time::Duration;

const DEV_TARGET_SELECTION_VERSION: u8 = 1;
const LOADING_TICK_INTERVAL: Duration = Duration::from_millis(120);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostOs {
    Macos,
    Linux,
    Windows,
    Other,
}

impl HostOs {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Other
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DevTarget {
    Server,
    Web,
    Desktop,
    Android,
    Ios,
}

impl DevTarget {
    pub fn canonical() -> &'static [Self] {
        &[
            Self::Server,
            Self::Web,
            Self::Desktop,
            Self::Android,
            Self::Ios,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Server => "server",
            Self::Web => "web",
            Self::Desktop => "desktop",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Server => "Backend server",
            Self::Web => "Views server",
            Self::Desktop => "Desktop app",
            Self::Android => "Android app",
            Self::Ios => "iOS app",
        }
    }

    pub fn is_available_on(self, host: HostOs) -> bool {
        match self {
            Self::Server | Self::Web => true,
            Self::Desktop | Self::Android => {
                matches!(host, HostOs::Macos | HostOs::Linux | HostOs::Windows)
            }
            Self::Ios => host == HostOs::Macos,
        }
    }
}

impl Display for DevTarget {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for DevTarget {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "server" => Ok(Self::Server),
            "web" => Ok(Self::Web),
            "desktop" => Ok(Self::Desktop),
            "android" => Ok(Self::Android),
            "ios" => Ok(Self::Ios),
            _ => Err(format!("unknown dev target `{value}`")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevTargetSelection {
    targets: Vec<DevTarget>,
}

#[derive(Serialize, Deserialize)]
struct StoredDevTargetSelection {
    version: u8,
    targets: Vec<String>,
}

impl DevTargetSelection {
    pub fn new(targets: impl IntoIterator<Item = DevTarget>, host: HostOs) -> RuntimeResult<Self> {
        let requested = targets.into_iter().collect::<BTreeSet<_>>();
        if requested.is_empty() {
            return Err(RuntimeError::new("select at least one dev target"));
        }

        for target in &requested {
            if !target.is_available_on(host) {
                return Err(RuntimeError::new(format!(
                    "target `{target}` is not available on this host"
                )));
            }
        }

        let targets = DevTarget::canonical()
            .iter()
            .copied()
            .filter(|target| requested.contains(target))
            .collect::<Vec<_>>();

        Ok(Self { targets })
    }

    pub fn parse(
        values: impl IntoIterator<Item = impl AsRef<str>>,
        host: HostOs,
    ) -> RuntimeResult<Self> {
        let mut targets = Vec::new();

        for value in values {
            let target = value
                .as_ref()
                .parse::<DevTarget>()
                .map_err(RuntimeError::new)?;
            targets.push(target);
        }

        Self::new(targets, host)
    }

    pub fn contains(&self, target: DevTarget) -> bool {
        self.targets.contains(&target)
    }

    pub fn targets(&self) -> &[DevTarget] {
        &self.targets
    }
}

pub fn available_dev_targets(host: HostOs) -> Vec<DevTarget> {
    DevTarget::canonical()
        .iter()
        .copied()
        .filter(|target| target.is_available_on(host))
        .collect()
}

pub fn default_dev_targets(host: HostOs) -> DevTargetSelection {
    DevTargetSelection::new([DevTarget::Server, DevTarget::Web], host)
        .expect("default dev targets are always available")
}

pub fn dev_target_selection_path(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join(".dowe/dev/target-selection.json")
}

pub fn load_dev_target_selection(
    root: impl AsRef<Path>,
    host: HostOs,
) -> RuntimeResult<Option<DevTargetSelection>> {
    let path = dev_target_selection_path(root);
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => return Ok(None),
    };

    Ok(parse_stored_dev_target_selection(&contents, host))
}

pub fn save_dev_target_selection(
    root: impl AsRef<Path>,
    selection: &DevTargetSelection,
) -> RuntimeResult<PathBuf> {
    let path = dev_target_selection_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let stored = StoredDevTargetSelection {
        version: DEV_TARGET_SELECTION_VERSION,
        targets: selection
            .targets()
            .iter()
            .map(|target| target.as_str().to_string())
            .collect(),
    };
    let mut contents = serde_json::to_string_pretty(&stored)
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    contents.push('\n');
    fs::write(&path, contents)?;

    Ok(path)
}

fn parse_stored_dev_target_selection(contents: &str, host: HostOs) -> Option<DevTargetSelection> {
    let stored = serde_json::from_str::<StoredDevTargetSelection>(contents).ok()?;
    if stored.version != DEV_TARGET_SELECTION_VERSION {
        return None;
    }

    let mut targets = Vec::new();
    for value in stored.targets {
        let target = value.parse::<DevTarget>().ok()?;
        if target.is_available_on(host) {
            targets.push(target);
        }
    }

    if targets.is_empty() {
        return None;
    }

    DevTargetSelection::new(targets, host).ok()
}

pub struct RunningDevSession {
    pub root: PathBuf,
    pub targets: DevTargetSelection,
    pub servers: RunningDevServers,
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
    let project = compile_dev(root).map_err(RuntimeError::from)?;
    let session = start_dev_session(project, selection).await?;
    session.wait().await
}

pub async fn start_dev_session(
    project: CompiledProject,
    selection: DevTargetSelection,
) -> RuntimeResult<RunningDevSession> {
    let server_targets = DevServerTargets {
        backend: selection.contains(DevTarget::Server),
        views: selection.contains(DevTarget::Web),
        desktop: selection.contains(DevTarget::Desktop),
    };
    let servers = start_dev_servers(project.clone(), server_targets).await?;
    let mut session = RunningDevSession {
        root: project.root.clone(),
        targets: selection.clone(),
        servers,
        external_processes: Vec::new(),
        external_cleanups: Vec::new(),
    };
    let desktop_origin = session
        .servers
        .desktop_addr
        .map(|addr| format!("http://{addr}/"));

    match start_external_targets(&project, &selection, desktop_origin).await {
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

async fn start_external_targets(
    project: &CompiledProject,
    selection: &DevTargetSelection,
    desktop_origin: Option<String>,
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
        tasks.push(tokio::task::spawn_blocking(move || {
            (
                target,
                start_external_target(&project, target, desktop_origin.as_deref()),
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
                        if first_error.is_none() {
                            first_error = Some(error);
                        }
                    }
                    Err(error) => {
                        if first_error.is_none() {
                            first_error = Some(RuntimeError::from(error));
                        }
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

fn target_labels(targets: impl IntoIterator<Item = DevTarget>) -> String {
    targets
        .into_iter()
        .map(|target| target.label())
        .collect::<Vec<_>>()
        .join(", ")
}

fn loading_status_message(targets: impl IntoIterator<Item = DevTarget>) -> String {
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
        let external_result = wait_external_processes(&mut external_processes);
        let server_result = servers.shutdown().await;
        external_result?;
        server_result
    }

    pub async fn wait(self) -> RuntimeResult<()> {
        let RunningDevSession {
            root,
            targets,
            servers,
            mut external_processes,
            external_cleanups,
            ..
        } = self;
        let state = servers.runtime_state();
        let (stop_sender, stop_receiver) = oneshot::channel();
        let watch_handle =
            tokio::spawn(run_watch_loop(root, targets.clone(), state, stop_receiver));

        let mut result = if servers.has_any() {
            let server_result = servers.wait().await;
            cancel_external_processes(&external_processes);
            run_external_cleanups(&external_cleanups);
            let external_result = wait_external_processes(&mut external_processes);
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

#[cfg(test)]
mod tests {
    use super::{
        DevTarget, DevTargetSelection, HostOs, available_dev_targets, default_dev_targets,
        dev_target_selection_path, load_dev_target_selection, loading_status_message,
        save_dev_target_selection,
    };
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn filters_ios_by_platform() {
        assert!(available_dev_targets(HostOs::Macos).contains(&DevTarget::Ios));
        assert!(!available_dev_targets(HostOs::Linux).contains(&DevTarget::Ios));
        assert!(!available_dev_targets(HostOs::Windows).contains(&DevTarget::Ios));
    }

    #[test]
    fn defaults_to_server_and_web() {
        let selection = default_dev_targets(HostOs::Linux);

        assert_eq!(selection.targets(), &[DevTarget::Server, DevTarget::Web]);
    }

    #[test]
    fn deduplicates_and_sorts_targets() {
        let selection = DevTargetSelection::new(
            [DevTarget::Android, DevTarget::Server, DevTarget::Android],
            HostOs::Linux,
        )
        .expect("selection");

        assert_eq!(
            selection.targets(),
            &[DevTarget::Server, DevTarget::Android]
        );
    }

    #[test]
    fn rejects_ios_outside_macos() {
        let error = DevTargetSelection::new([DevTarget::Ios], HostOs::Linux).expect_err("error");

        assert!(error.to_string().contains("ios"));
    }

    #[test]
    fn rejects_empty_target_selection() {
        let error = DevTargetSelection::new([], HostOs::Linux).expect_err("error");

        assert!(error.to_string().contains("select at least one"));
    }

    #[test]
    fn formats_loading_status_with_pending_target_labels() {
        assert_eq!(
            loading_status_message([DevTarget::Android, DevTarget::Ios]),
            "Loading dev targets: Android app, iOS app"
        );
    }

    #[test]
    fn persists_dev_target_selection_under_dowe_dev() {
        let temp = TempDir::new().expect("tempdir");
        let selection = DevTargetSelection::new(
            [DevTarget::Android, DevTarget::Server, DevTarget::Android],
            HostOs::Linux,
        )
        .expect("selection");

        let path = save_dev_target_selection(temp.path(), &selection).expect("save");

        assert_eq!(path, temp.path().join(".dowe/dev/target-selection.json"));
        assert_eq!(path, dev_target_selection_path(temp.path()));
        let contents = fs::read_to_string(path).expect("contents");
        assert_eq!(
            contents,
            "{\n  \"version\": 1,\n  \"targets\": [\n    \"server\",\n    \"android\"\n  ]\n}\n"
        );
        let loaded = load_dev_target_selection(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored selection");

        assert_eq!(loaded.targets(), &[DevTarget::Server, DevTarget::Android]);
    }

    #[test]
    fn filters_unavailable_persisted_dev_targets() {
        let temp = TempDir::new().expect("tempdir");
        let path = dev_target_selection_path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, r#"{"version":1,"targets":["server","ios"]}"#).expect("write");

        let loaded = load_dev_target_selection(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored selection");

        assert_eq!(loaded.targets(), &[DevTarget::Server]);
    }

    #[test]
    fn ignores_empty_persisted_dev_targets_after_platform_filtering() {
        let temp = TempDir::new().expect("tempdir");
        let path = dev_target_selection_path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, r#"{"version":1,"targets":["ios"]}"#).expect("write");

        let loaded = load_dev_target_selection(temp.path(), HostOs::Linux).expect("load");

        assert!(loaded.is_none());
    }

    #[test]
    fn ignores_invalid_persisted_dev_target_selection() {
        let temp = TempDir::new().expect("tempdir");
        let path = dev_target_selection_path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, r#"{"version":1,"targets":["server","watch"]}"#).expect("write");

        let loaded = load_dev_target_selection(temp.path(), HostOs::Macos).expect("load");

        assert!(loaded.is_none());
    }
}
