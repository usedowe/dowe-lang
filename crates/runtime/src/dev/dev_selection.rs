use crate::error::{RuntimeError, RuntimeResult};
use dowe_compiler::{ProjectCapabilities, inspect_project_capabilities};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const DEV_TARGET_SELECTION_VERSION: u8 = 1;

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
        targets: persisted_dev_targets(selection),
        quit_simulators_on_exit,
    };
    let mut contents = serde_json::to_string_pretty(&stored)
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    contents.push('\n');
    fs::write(&path, contents)?;

    Ok(path)
}

fn persisted_dev_targets(selection: &DevTargetSelection) -> Vec<String> {
    DevTarget::canonical()
        .iter()
        .filter(|target| selection.contains(**target))
        .map(|target| target.as_str().to_string())
        .collect()
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
