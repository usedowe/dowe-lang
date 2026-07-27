use crate::config::Signal;
use crate::error::SpawnError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnEvent {
    Started {
        spawn_id: u64,
        system_pid: Option<u32>,
        command: String,
        pty: bool,
    },
    Stdout {
        spawn_id: u64,
        bytes: Vec<u8>,
    },
    Stderr {
        spawn_id: u64,
        bytes: Vec<u8>,
    },
    Terminal {
        spawn_id: u64,
        bytes: Vec<u8>,
    },
    StdinClosed {
        spawn_id: u64,
    },
    ResizeApplied {
        spawn_id: u64,
        rows: u16,
        cols: u16,
    },
    Exit {
        spawn_id: u64,
        output: SpawnOutput,
    },
    Timeout {
        spawn_id: u64,
        timeout_ms: u64,
        signal: Signal,
    },
    Canceled {
        spawn_id: u64,
        signal: Signal,
    },
    Error {
        spawn_id: u64,
        error: SpawnError,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnOutput {
    pub exit_code: Option<i32>,
    pub signal: Option<String>,
    pub success: bool,
    pub timed_out: bool,
    pub canceled: bool,
    pub spawn_error: Option<SpawnError>,
    pub duration_ms: u128,
    pub stdout_bytes: Vec<u8>,
    pub stderr_bytes: Vec<u8>,
    pub terminal_bytes: Vec<u8>,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub terminal_truncated: bool,
}

impl SpawnOutput {
    pub fn success(&self) -> bool {
        self.success
    }
}
