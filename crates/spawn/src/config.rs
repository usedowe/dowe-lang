use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnConfig {
    pub command: String,
    pub args: Vec<String>,
    pub options: SpawnOptions,
}

impl SpawnConfig {
    pub fn new(
        command: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            command: command.into(),
            args: args.into_iter().map(Into::into).collect(),
            options: SpawnOptions::default(),
        }
    }

    pub fn with_options(mut self, options: SpawnOptions) -> Self {
        self.options = options;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnOptions {
    pub cwd: Option<PathBuf>,
    pub env_mode: EnvMode,
    pub env: BTreeMap<String, String>,
    pub env_remove: Vec<String>,
    pub stdin: StreamMode,
    pub stdout: StreamMode,
    pub stderr: StreamMode,
    pub pty: Option<PtyOptions>,
    pub timeout_ms: Option<u64>,
    pub kill_target: KillTarget,
    pub kill_grace_ms: Option<u64>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub max_output_bytes: Option<usize>,
}

impl Default for SpawnOptions {
    fn default() -> Self {
        Self {
            cwd: None,
            env_mode: EnvMode::Inherit,
            env: BTreeMap::new(),
            env_remove: Vec::new(),
            stdin: StreamMode::Ignore,
            stdout: StreamMode::Pipe,
            stderr: StreamMode::Pipe,
            pty: None,
            timeout_ms: None,
            kill_target: KillTarget::Process,
            kill_grace_ms: Some(500),
            uid: None,
            gid: None,
            max_output_bytes: Some(1024 * 1024),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvMode {
    Inherit,
    Clean,
    Replace,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamMode {
    Inherit,
    Pipe,
    Ignore,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KillTarget {
    Process,
    Group,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtyOptions {
    pub rows: u16,
    pub cols: u16,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

impl Default for PtyOptions {
    fn default() -> Self {
        Self {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    Interrupt,
    Terminate,
    Kill,
}
