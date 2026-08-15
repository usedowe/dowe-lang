use crate::error::{DeployError, DeployResult};
use dowe_compiler::inspect_project_capabilities;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeployEnvironment {
    #[default]
    Live,
    Stage,
    Uat,
}

impl DeployEnvironment {
    pub const ALL: [Self; 3] = [Self::Live, Self::Stage, Self::Uat];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Stage => "stage",
            Self::Uat => "uat",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Live => "Live",
            Self::Stage => "Stage",
            Self::Uat => "UAT",
        }
    }

    pub fn compile_environment(self) -> dowe_compiler::CompileEnvironment {
        match self {
            Self::Live => dowe_compiler::CompileEnvironment::Live,
            Self::Stage => dowe_compiler::CompileEnvironment::Stage,
            Self::Uat => dowe_compiler::CompileEnvironment::Uat,
        }
    }

    pub fn requires_access(self) -> bool {
        !matches!(self, Self::Live)
    }
}

impl Display for DeployEnvironment {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for DeployEnvironment {
    type Err = DeployError;

    fn from_str(value: &str) -> DeployResult<Self> {
        Self::ALL
            .into_iter()
            .find(|environment| environment.as_str() == value)
            .ok_or_else(|| DeployError::new(format!("unknown deploy environment `{value}`")))
    }
}

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
    Dowe,
    Docker,
    Ssh,
    Cloudflare,
    #[serde(rename = "cloudflare-pages")]
    CloudflarePages,
    Vercel,
    Android,
    Ios,
}

impl DeployTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Dowe => "dowe",
            Self::Docker => "docker",
            Self::Ssh => "ssh",
            Self::Cloudflare => "cloudflare",
            Self::CloudflarePages => "cloudflare-pages",
            Self::Vercel => "vercel",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }

    pub fn surface(self) -> DeploySurface {
        match self {
            Self::Static | Self::CloudflarePages => DeploySurface::Web,
            Self::Dowe | Self::Docker | Self::Ssh | Self::Cloudflare | Self::Vercel => {
                DeploySurface::Server
            }
            Self::Android => DeploySurface::Android,
            Self::Ios => DeploySurface::Ios,
        }
    }

    pub fn supports_surface(self, surface: DeploySurface) -> bool {
        match self {
            Self::Dowe | Self::Docker | Self::Vercel => {
                matches!(surface, DeploySurface::Server | DeploySurface::Web)
            }
            _ => self.surface() == surface,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Static => "Static files",
            Self::Dowe => "Dowe Cloud",
            Self::Docker => "Docker",
            Self::Ssh => "SSH",
            Self::Cloudflare => "Cloudflare Worker",
            Self::CloudflarePages => "Cloudflare Pages",
            Self::Vercel => "Vercel",
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
            "dowe" => Ok(Self::Dowe),
            "docker" => Ok(Self::Docker),
            "ssh" => Ok(Self::Ssh),
            "cloudflare" => Ok(Self::Cloudflare),
            "cloudflare-pages" => Ok(Self::CloudflarePages),
            "vercel" => Ok(Self::Vercel),
            "android" => Ok(Self::Android),
            "ios" => Ok(Self::Ios),
            _ => Err(DeployError::new(format!("unknown deploy target `{value}`"))),
        }
    }
}

pub fn available_deploy_surfaces(
    root: impl AsRef<Path>,
    environment: DeployEnvironment,
) -> DeployResult<Vec<DeploySurface>> {
    let capabilities = inspect_project_capabilities(root.as_ref())?;
    Ok(DeploySurface::canonical()
        .iter()
        .copied()
        .filter(|surface| match surface {
            DeploySurface::Server => capabilities.server,
            DeploySurface::Web => capabilities.views,
            DeploySurface::Android => capabilities.views && environment == DeployEnvironment::Live,
            DeploySurface::Ios => {
                capabilities.views
                    && environment == DeployEnvironment::Live
                    && cfg!(target_os = "macos")
            }
        })
        .collect())
}

pub fn deploy_targets_for_surface(surface: DeploySurface) -> &'static [DeployTarget] {
    match surface {
        DeploySurface::Server => &[
            DeployTarget::Dowe,
            DeployTarget::Docker,
            DeployTarget::Ssh,
            DeployTarget::Cloudflare,
            DeployTarget::Vercel,
        ],
        DeploySurface::Web => &[
            DeployTarget::Dowe,
            DeployTarget::Docker,
            DeployTarget::CloudflarePages,
            DeployTarget::Vercel,
        ],
        DeploySurface::Android => &[DeployTarget::Android],
        DeploySurface::Ios => &[DeployTarget::Ios],
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeployOptions {
    pub root: PathBuf,
    pub environment: DeployEnvironment,
    pub target: DeployTarget,
    pub surface: Option<DeploySurface>,
    pub name: Option<String>,
    pub publish: bool,
    pub dry_run: bool,
    pub registry: Option<String>,
    pub image: Option<String>,
    pub track: Option<String>,
    pub ssh_host: Option<String>,
    pub ssh_user: Option<String>,
    pub ssh_key_file: Option<PathBuf>,
}

impl DeployOptions {
    pub fn new(root: impl Into<PathBuf>, target: DeployTarget) -> Self {
        Self {
            root: root.into(),
            environment: DeployEnvironment::Live,
            target,
            surface: None,
            name: None,
            publish: false,
            dry_run: false,
            registry: None,
            image: None,
            track: None,
            ssh_host: None,
            ssh_user: None,
            ssh_key_file: None,
        }
    }

    pub fn surface(&self) -> DeploySurface {
        self.surface.unwrap_or_else(|| self.target.surface())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployReport {
    pub environment: DeployEnvironment,
    pub target: DeployTarget,
    pub output_dir: PathBuf,
    pub files: Vec<PathBuf>,
    pub command: Option<Vec<String>>,
    pub published: bool,
    pub image_ref: Option<String>,
    pub image_built: bool,
    pub artifact: Option<PathBuf>,
    pub access_protected: bool,
    pub deployment_id: Option<String>,
    pub url: Option<String>,
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
