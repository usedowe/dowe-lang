use std::fmt::{Display, Formatter};
use std::path::Path;

pub type CodeGraphResult<T> = Result<T, CodeGraphError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeGraphError {
    message: String,
}

impl CodeGraphError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn at_path(path: &Path, message: impl AsRef<str>) -> Self {
        Self::new(format!("{}: {}", path.display(), message.as_ref()))
    }
}

impl Display for CodeGraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CodeGraphError {}

impl From<std::io::Error> for CodeGraphError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<serde_json::Error> for CodeGraphError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error.to_string())
    }
}
