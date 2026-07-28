use crate::dev_targets::{cancel_active_external_commands, start_external_target};
use crate::dev_watch::run_watch_loop;
use crate::error::{RuntimeError, RuntimeResult};
use crate::logging::LoadingStatus;
use crate::server::{DevServerTargets, RunningDevServers, start_dev_servers};
use dowe_compiler::{
    CompiledProject, ProjectCapabilities, compile_dev, inspect_project_capabilities,
};
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

    fn is_configured_for(self, capabilities: ProjectCapabilities) -> bool {
        match self {
            Self::Server => capabilities.server,
            Self::Web | Self::Desktop | Self::Android | Self::Ios => capabilities.views,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevTargetPreferences {
    pub selection: DevTargetSelection,
    pub quit_simulators_on_exit: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DevRunOptions {
    pub devices: DevTargetDeviceSelection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevTargetDeviceSelection {
    pub android: Option<AndroidDeviceSelection>,
    pub ios: Option<IosSimulatorSelection>,
    pub quit_simulators_on_exit: bool,
}

impl Default for DevTargetDeviceSelection {
    fn default() -> Self {
        Self {
            android: None,
            ios: None,
            quit_simulators_on_exit: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AndroidDeviceSelection {
    Connected { serial: String },
    Avd { name: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AndroidDeviceOption {
    label: String,
    selection: AndroidDeviceSelection,
}

impl AndroidDeviceOption {
    pub(crate) fn new(label: impl Into<String>, selection: AndroidDeviceSelection) -> Self {
        Self {
            label: label.into(),
            selection,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn selection(&self) -> &AndroidDeviceSelection {
        &self.selection
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IosSimulatorSelection {
    udid: String,
}

impl IosSimulatorSelection {
    pub(crate) fn new(udid: impl Into<String>) -> Self {
        Self { udid: udid.into() }
    }

    pub fn udid(&self) -> &str {
        &self.udid
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IosSimulatorOption {
    label: String,
    name: String,
    udid: String,
    runtime: String,
    state: String,
}

impl IosSimulatorOption {
    pub(crate) fn new(
        name: impl Into<String>,
        udid: impl Into<String>,
        runtime: impl Into<String>,
        state: impl Into<String>,
    ) -> Self {
        let name = name.into();
        let udid = udid.into();
        let runtime = runtime.into();
        let state = state.into();
        let label = if state == "Booted" {
            format!("{name} ({runtime}, Booted)")
        } else {
            format!("{name} ({runtime}, {state})")
        };
        Self {
            label,
            name,
            udid,
            runtime,
            state,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn udid(&self) -> &str {
        &self.udid
    }

    pub fn runtime(&self) -> &str {
        &self.runtime
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn selection(&self) -> IosSimulatorSelection {
        IosSimulatorSelection::new(self.udid.clone())
    }

    pub fn is_booted(&self) -> bool {
        self.state == "Booted"
    }
}

#[derive(Serialize, Deserialize)]
struct StoredDevTargetSelection {
    version: u8,
    targets: Vec<String>,
    #[serde(default = "default_quit_simulators_on_exit")]
    quit_simulators_on_exit: bool,
}

fn default_quit_simulators_on_exit() -> bool {
    true
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

pub fn available_dev_targets_for_project(
    root: impl AsRef<Path>,
    host: HostOs,
) -> RuntimeResult<Vec<DevTarget>> {
    let capabilities = inspect_project_capabilities(root.as_ref()).map_err(RuntimeError::from)?;
    Ok(DevTarget::canonical()
        .iter()
        .copied()
        .filter(|target| target.is_available_on(host) && target.is_configured_for(capabilities))
        .collect())
}

pub fn default_dev_targets_for_project(
    root: impl AsRef<Path>,
    host: HostOs,
) -> RuntimeResult<DevTargetSelection> {
    let available = available_dev_targets_for_project(root, host)?;
    let defaults = [DevTarget::Server, DevTarget::Web]
        .into_iter()
        .filter(|target| available.contains(target))
        .collect::<Vec<_>>();
    if defaults.is_empty() {
        return Err(RuntimeError::new(
            "main.dowe does not configure any dev targets; add `server` or `views` under `main`",
        ));
    }
    DevTargetSelection::new(defaults, host)
}

pub fn validate_dev_target_selection_for_project(
    root: impl AsRef<Path>,
    host: HostOs,
    selection: &DevTargetSelection,
) -> RuntimeResult<()> {
    let available = available_dev_targets_for_project(root, host)?;
    for target in selection.targets() {
        if !available.contains(target) {
            return Err(RuntimeError::new(format!(
                "target `{target}` is not configured in main.dowe"
            )));
        }
    }
    Ok(())
}

pub fn available_android_devices() -> RuntimeResult<Vec<AndroidDeviceOption>> {
    crate::dev_targets::android_device_options()
}

pub fn available_ios_simulators() -> RuntimeResult<Vec<IosSimulatorOption>> {
    crate::dev_targets::ios_simulator_options()
}

pub fn dev_target_selection_path(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join(".dowe/dev/target-selection.json")
}

pub fn load_dev_target_selection(
    root: impl AsRef<Path>,
    host: HostOs,
) -> RuntimeResult<Option<DevTargetSelection>> {
    Ok(load_dev_target_preferences(root, host)?.map(|preferences| preferences.selection))
}

pub fn load_dev_target_preferences(
    root: impl AsRef<Path>,
    host: HostOs,
) -> RuntimeResult<Option<DevTargetPreferences>> {
    let path = dev_target_selection_path(root);
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => return Ok(None),
    };

    Ok(parse_stored_dev_target_preferences(
        &contents,
        host,
        &available_dev_targets(host),
    ))
}

pub fn load_dev_target_preferences_for_project(
    root: impl AsRef<Path>,
    host: HostOs,
) -> RuntimeResult<Option<DevTargetPreferences>> {
    let root = root.as_ref();
    let available = available_dev_targets_for_project(root, host)?;
    let path = dev_target_selection_path(root);
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => return Ok(None),
    };
    Ok(parse_stored_dev_target_preferences(
        &contents, host, &available,
    ))
}

pub fn save_dev_target_selection(
    root: impl AsRef<Path>,
    selection: &DevTargetSelection,
) -> RuntimeResult<PathBuf> {
    let root = root.as_ref();
    let quit_simulators_on_exit = fs::read_to_string(dev_target_selection_path(root))
        .ok()
        .and_then(|contents| serde_json::from_str::<StoredDevTargetSelection>(&contents).ok())
        .filter(|stored| stored.version == DEV_TARGET_SELECTION_VERSION)
        .map(|stored| stored.quit_simulators_on_exit)
        .unwrap_or_else(default_quit_simulators_on_exit);
    save_dev_target_preferences(root, selection, quit_simulators_on_exit)
}

pub fn save_dev_target_preferences(
    root: impl AsRef<Path>,
    selection: &DevTargetSelection,
    quit_simulators_on_exit: bool,
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
        quit_simulators_on_exit,
    };
    let mut contents = serde_json::to_string_pretty(&stored)
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    contents.push('\n');
    fs::write(&path, contents)?;

    Ok(path)
}

fn parse_stored_dev_target_preferences(
    contents: &str,
    host: HostOs,
    available: &[DevTarget],
) -> Option<DevTargetPreferences> {
    let stored = serde_json::from_str::<StoredDevTargetSelection>(contents).ok()?;
    if stored.version != DEV_TARGET_SELECTION_VERSION {
        return None;
    }

    let mut targets = Vec::new();
    for value in stored.targets {
        let target = value.parse::<DevTarget>().ok()?;
        if available.contains(&target) {
            targets.push(target);
        }
    }

    if targets.is_empty() {
        return None;
    }

    let selection = DevTargetSelection::new(targets, host).ok()?;
    Some(DevTargetPreferences {
        selection,
        quit_simulators_on_exit: stored.quit_simulators_on_exit,
    })
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
    run_dev_with_options(root, selection, DevRunOptions::default()).await
}

pub async fn run_dev_with_options(
    root: impl AsRef<Path>,
    selection: DevTargetSelection,
    options: DevRunOptions,
) -> RuntimeResult<()> {
    let project = compile_dev(root).map_err(RuntimeError::from)?;
    let session = start_dev_session_with_options(project, selection, options).await?;
    session.wait().await
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
    let server_targets = DevServerTargets {
        backend: selection.contains(DevTarget::Server),
        views: selection.contains(DevTarget::Web)
            || selection.contains(DevTarget::Android)
            || selection.contains(DevTarget::Ios),
        desktop: selection.contains(DevTarget::Desktop),
    };
    let servers = match start_dev_servers(project.clone(), server_targets).await {
        Ok(servers) => servers,
        Err(error) => return Err(error),
    };
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

fn record_external_startup_failure(
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

#[cfg(test)]
mod tests {
    use super::{
        DevTarget, DevTargetDeviceSelection, DevTargetSelection, HostOs, available_dev_targets,
        available_dev_targets_for_project, default_dev_targets, default_dev_targets_for_project,
        dev_target_selection_path, load_dev_target_preferences,
        load_dev_target_preferences_for_project, load_dev_target_selection, loading_status_message,
        record_external_startup_failure, save_dev_target_preferences, save_dev_target_selection,
        validate_dev_target_selection_for_project,
    };
    use crate::error::RuntimeError;
    use std::cell::Cell;
    use std::fs;
    use std::path::Path;
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
    fn exposes_only_view_targets_for_a_views_only_project() {
        let temp = TempDir::new().expect("tempdir");
        write_main(temp.path(), "main\n  views:viewRoutes\n");

        let targets =
            available_dev_targets_for_project(temp.path(), HostOs::Macos).expect("targets");
        let defaults =
            default_dev_targets_for_project(temp.path(), HostOs::Macos).expect("defaults");

        assert_eq!(
            targets,
            [
                DevTarget::Web,
                DevTarget::Desktop,
                DevTarget::Android,
                DevTarget::Ios
            ]
        );
        assert_eq!(defaults.targets(), &[DevTarget::Web]);
    }

    #[test]
    fn exposes_only_server_for_a_server_only_project() {
        let temp = TempDir::new().expect("tempdir");
        write_main(temp.path(), "main\n  server port:8080\n");

        let targets =
            available_dev_targets_for_project(temp.path(), HostOs::Linux).expect("targets");
        let defaults =
            default_dev_targets_for_project(temp.path(), HostOs::Linux).expect("defaults");

        assert_eq!(targets, [DevTarget::Server]);
        assert_eq!(defaults.targets(), &[DevTarget::Server]);
    }

    #[test]
    fn rejects_targets_missing_from_main() {
        let temp = TempDir::new().expect("tempdir");
        write_main(temp.path(), "main\n  views:viewRoutes\n");
        let selection =
            DevTargetSelection::new([DevTarget::Server], HostOs::Linux).expect("selection");

        let error =
            validate_dev_target_selection_for_project(temp.path(), HostOs::Linux, &selection)
                .expect_err("error");

        assert!(error.to_string().contains("main.dowe"));
        assert!(error.to_string().contains("server"));
    }

    #[test]
    fn defaults_to_quitting_simulators_on_exit() {
        assert!(DevTargetDeviceSelection::default().quit_simulators_on_exit);
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
    fn cancels_pending_startups_after_first_target_failure() {
        let cancelled = Cell::new(false);
        let mut first_error = None;
        let mut cancelling = false;

        record_external_startup_failure(
            &mut first_error,
            &mut cancelling,
            RuntimeError::new("Android app failed"),
            || cancelled.set(true),
        );

        assert!(cancelled.get());
        assert!(cancelling);
        assert_eq!(
            first_error.expect("first error").to_string(),
            "Android app failed"
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
            "{\n  \"version\": 1,\n  \"targets\": [\n    \"server\",\n    \"android\"\n  ],\n  \"quit_simulators_on_exit\": true\n}\n"
        );
        let loaded = load_dev_target_selection(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored selection");

        assert_eq!(loaded.targets(), &[DevTarget::Server, DevTarget::Android]);
    }

    #[test]
    fn persists_disabled_simulator_quit_preference() {
        let temp = TempDir::new().expect("tempdir");
        let selection =
            DevTargetSelection::new([DevTarget::Android], HostOs::Linux).expect("selection");

        save_dev_target_preferences(temp.path(), &selection, false).expect("save");
        let loaded = load_dev_target_preferences(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored preferences");

        assert_eq!(loaded.selection, selection);
        assert!(!loaded.quit_simulators_on_exit);
    }

    #[test]
    fn legacy_selection_defaults_to_quitting_simulators() {
        let temp = TempDir::new().expect("tempdir");
        let path = dev_target_selection_path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, r#"{"version":1,"targets":["android"]}"#).expect("write");

        let loaded = load_dev_target_preferences(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored preferences");

        assert!(loaded.quit_simulators_on_exit);
    }

    #[test]
    fn target_updates_preserve_simulator_quit_preference() {
        let temp = TempDir::new().expect("tempdir");
        let mobile = DevTargetSelection::new([DevTarget::Android], HostOs::Linux).expect("mobile");
        save_dev_target_preferences(temp.path(), &mobile, false).expect("save preference");
        let web = DevTargetSelection::new([DevTarget::Web], HostOs::Linux).expect("web");

        save_dev_target_selection(temp.path(), &web).expect("save targets");
        let loaded = load_dev_target_preferences(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored preferences");

        assert_eq!(loaded.selection, web);
        assert!(!loaded.quit_simulators_on_exit);
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
    fn filters_persisted_targets_by_project_capabilities() {
        let temp = TempDir::new().expect("tempdir");
        write_main(temp.path(), "main\n  views:viewRoutes\n");
        let path = dev_target_selection_path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(
            &path,
            r#"{"version":1,"targets":["server","web","android"]}"#,
        )
        .expect("write");

        let loaded = load_dev_target_preferences_for_project(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored selection");

        assert_eq!(
            loaded.selection.targets(),
            &[DevTarget::Web, DevTarget::Android]
        );
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

    fn write_main(root: &Path, source: &str) {
        fs::create_dir_all(root.join("src")).expect("src");
        fs::write(root.join("main.dowe"), source).expect("main");
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
