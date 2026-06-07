use std::fmt::{Display, Formatter};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RuntimeError {}

impl From<std::io::Error> for RuntimeError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<dowe_compiler::DoweError> for RuntimeError {
    fn from(error: dowe_compiler::DoweError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<dowe_spawn::SpawnError> for RuntimeError {
    fn from(error: dowe_spawn::SpawnError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<tokio::task::JoinError> for RuntimeError {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::new(error.to_string())
    }
}
