use crate::error::{DeployError, DeployResult};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeployTarget {
    Static,
    Docker,
    Ssh,
    Cloudflare,
}

impl DeployTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Docker => "docker",
            Self::Ssh => "ssh",
            Self::Cloudflare => "cloudflare",
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
            "ssh" => Ok(Self::Ssh),
            "cloudflare" => Ok(Self::Cloudflare),
            _ => Err(DeployError::new(format!("unknown deploy target `{value}`"))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeployOptions {
    pub root: PathBuf,
    pub target: DeployTarget,
    pub name: Option<String>,
    pub publish: bool,
    pub dry_run: bool,
    pub host: Option<String>,
    pub remote_dir: Option<String>,
    pub server_binary: Option<PathBuf>,
}

impl DeployOptions {
    pub fn new(root: impl Into<PathBuf>, target: DeployTarget) -> Self {
        Self {
            root: root.into(),
            target,
            name: None,
            publish: false,
            dry_run: false,
            host: None,
            remote_dir: None,
            server_binary: None,
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
}
