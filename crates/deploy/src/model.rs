use crate::error::{DeployError, DeployResult};
use dowe_compiler::inspect_project_capabilities;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeploySurface {
    Server,
    Web,
    Android,
    Ios,
}

impl DeploySurface {
    pub fn canonical() -> &'static [Self] {
        &[Self::Server, Self::Web, Self::Android, Self::Ios]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Server => "server",
            Self::Web => "web",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Server => "Server",
            Self::Web => "Web",
            Self::Android => "Android",
            Self::Ios => "iOS",
        }
    }
}

impl Display for DeploySurface {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for DeploySurface {
    type Err = DeployError;

    fn from_str(value: &str) -> DeployResult<Self> {
        match value {
            "server" => Ok(Self::Server),
            "web" => Ok(Self::Web),
            "android" => Ok(Self::Android),
            "ios" => Ok(Self::Ios),
            _ => Err(DeployError::new(format!(
                "unknown deploy surface `{value}`"
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeployTarget {
    Static,
    Docker,
    Cloudflare,
    #[serde(rename = "cloudflare-pages")]
    CloudflarePages,
    Android,
    Ios,
}

impl DeployTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Docker => "docker",
            Self::Cloudflare => "cloudflare",
            Self::CloudflarePages => "cloudflare-pages",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }

    pub fn surface(self) -> DeploySurface {
        match self {
            Self::Static | Self::CloudflarePages => DeploySurface::Web,
            Self::Docker | Self::Cloudflare => DeploySurface::Server,
            Self::Android => DeploySurface::Android,
            Self::Ios => DeploySurface::Ios,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Static => "Static files",
            Self::Docker => "Docker",
            Self::Cloudflare => "Cloudflare Worker",
            Self::CloudflarePages => "Cloudflare Pages",
            Self::Android => "Google Play",
            Self::Ios => "App Store Connect",
        }
    }
}

impl Display for DeployTarget {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for DeployTarget {
    type Err = DeployError;

    fn from_str(value: &str) -> DeployResult<Self> {
        match value {
            "static" => Ok(Self::Static),
            "docker" => Ok(Self::Docker),
            "cloudflare" => Ok(Self::Cloudflare),
            "cloudflare-pages" => Ok(Self::CloudflarePages),
            "android" => Ok(Self::Android),
            "ios" => Ok(Self::Ios),
            _ => Err(DeployError::new(format!("unknown deploy target `{value}`"))),
        }
    }
}

pub fn available_deploy_surfaces(root: impl AsRef<Path>) -> DeployResult<Vec<DeploySurface>> {
    let capabilities = inspect_project_capabilities(root.as_ref())?;
    Ok(DeploySurface::canonical()
        .iter()
        .copied()
        .filter(|surface| match surface {
            DeploySurface::Server => capabilities.server,
            DeploySurface::Web => capabilities.views,
            DeploySurface::Android => capabilities.views,
            DeploySurface::Ios => capabilities.views && cfg!(target_os = "macos"),
        })
        .collect())
}

pub fn deploy_targets_for_surface(surface: DeploySurface) -> &'static [DeployTarget] {
    match surface {
        DeploySurface::Server => &[DeployTarget::Docker, DeployTarget::Cloudflare],
        DeploySurface::Web => &[DeployTarget::CloudflarePages],
        DeploySurface::Android => &[DeployTarget::Android],
        DeploySurface::Ios => &[DeployTarget::Ios],
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeployOptions {
    pub root: PathBuf,
    pub target: DeployTarget,
    pub name: Option<String>,
    pub publish: bool,
    pub dry_run: bool,
    pub registry: Option<String>,
    pub image: Option<String>,
    pub track: Option<String>,
}

impl DeployOptions {
    pub fn new(root: impl Into<PathBuf>, target: DeployTarget) -> Self {
        Self {
            root: root.into(),
            target,
            name: None,
            publish: false,
            dry_run: false,
            registry: None,
            image: None,
            track: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployReport {
    pub target: DeployTarget,
    pub output_dir: PathBuf,
    pub files: Vec<PathBuf>,
    pub command: Option<Vec<String>>,
    pub published: bool,
    pub image_ref: Option<String>,
    pub image_built: bool,
    pub artifact: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildTarget {
    Android,
    Ios,
    Macos,
    Windows,
    Linux,
}

impl BuildTarget {
    pub const ALL: [Self; 5] = [
        Self::Android,
        Self::Ios,
        Self::Macos,
        Self::Windows,
        Self::Linux,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Android => "android",
            Self::Ios => "ios",
            Self::Macos => "macos",
            Self::Windows => "windows",
            Self::Linux => "linux",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Android => "Android APK",
            Self::Ios => "iOS IPA",
            Self::Macos => "macOS DMG",
            Self::Windows => "Windows EXE",
            Self::Linux => "Linux executable",
        }
    }

    pub fn requires_macos(self) -> bool {
        matches!(self, Self::Ios | Self::Macos)
    }
}

impl Display for BuildTarget {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for BuildTarget {
    type Err = DeployError;

    fn from_str(value: &str) -> DeployResult<Self> {
        Self::ALL
            .into_iter()
            .find(|target| target.as_str() == value)
            .ok_or_else(|| DeployError::new(format!("unknown build target `{value}`")))
    }
}

pub fn available_build_targets() -> Vec<BuildTarget> {
    BuildTarget::ALL
        .into_iter()
        .filter(|target| match target {
            BuildTarget::Ios | BuildTarget::Macos => cfg!(target_os = "macos"),
            BuildTarget::Windows => cfg!(target_os = "windows"),
            BuildTarget::Linux => cfg!(target_os = "linux"),
            BuildTarget::Android => true,
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildOptions {
    pub root: PathBuf,
    pub target: BuildTarget,
    pub dry_run: bool,
}

impl BuildOptions {
    pub fn new(root: impl Into<PathBuf>, target: BuildTarget) -> Self {
        Self {
            root: root.into(),
            target,
            dry_run: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildReport {
    pub target: BuildTarget,
    pub output_dir: PathBuf,
    pub artifact: PathBuf,
    pub commands: Vec<Vec<String>>,
    pub built: bool,
}
