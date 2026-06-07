use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub type SpawnResult<T> = Result<T, SpawnError>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnError {
    pub command: String,
    pub phase: SpawnPhase,
    pub message: String,
}

impl SpawnError {
    pub fn new(command: impl Into<String>, phase: SpawnPhase, message: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            phase,
            message: message.into(),
        }
    }
}

impl Display for SpawnError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "spawn `{}` failed during {}: {}",
            self.command, self.phase, self.message
        )
    }
}

impl std::error::Error for SpawnError {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnPhase {
    Validation,
    CommandResolution,
    Cwd,
    Environment,
    Pipe,
    Pty,
    Start,
    StreamRead,
    StreamWrite,
    Resize,
    Termination,
    Wait,
}

impl Display for SpawnPhase {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Validation => "validation",
            Self::CommandResolution => "command resolution",
            Self::Cwd => "cwd validation",
            Self::Environment => "environment construction",
            Self::Pipe => "pipe creation",
            Self::Pty => "pty creation",
            Self::Start => "process start",
            Self::StreamRead => "stream read",
            Self::StreamWrite => "stream write",
            Self::Resize => "pty resize",
            Self::Termination => "termination",
            Self::Wait => "wait",
        };
        formatter.write_str(value)
    }
}
