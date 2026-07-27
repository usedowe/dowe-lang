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
}

impl DeploySurface {
    pub fn canonical() -> &'static [Self] {
        &[Self::Server, Self::Web]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Server => "server",
            Self::Web => "web",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Server => "Server",
            Self::Web => "Web",
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
}

impl DeployTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Docker => "docker",
            Self::Cloudflare => "cloudflare",
            Self::CloudflarePages => "cloudflare-pages",
        }
    }

    pub fn surface(self) -> DeploySurface {
        match self {
            Self::Static | Self::CloudflarePages => DeploySurface::Web,
            Self::Docker | Self::Cloudflare => DeploySurface::Server,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Static => "Static files",
            Self::Docker => "Docker",
            Self::Cloudflare => "Cloudflare Worker",
            Self::CloudflarePages => "Cloudflare Pages",
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
        })
        .collect())
}

pub fn deploy_targets_for_surface(surface: DeploySurface) -> &'static [DeployTarget] {
    match surface {
        DeploySurface::Server => &[DeployTarget::Docker, DeployTarget::Cloudflare],
        DeploySurface::Web => &[DeployTarget::CloudflarePages],
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
}
