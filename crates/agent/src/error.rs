use std::fmt::{Display, Formatter};
use std::path::Path;

pub type AgentResult<T> = Result<T, AgentError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentError {
    message: String,
}

impl AgentError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn at_path(path: &Path, message: impl AsRef<str>) -> Self {
        Self::new(format!("{}: {}", path.display(), message.as_ref()))
    }
}

impl Display for AgentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AgentError {}
