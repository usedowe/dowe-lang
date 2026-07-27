use std::fmt::{Display, Formatter};
use std::path::Path;

pub type DoweResult<T> = Result<T, DoweError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoweError {
    message: String,
}

impl DoweError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn at_path(path: &Path, message: impl AsRef<str>) -> Self {
        Self::new(format!("{}: {}", path.display(), message.as_ref()))
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for DoweError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for DoweError {}

impl From<std::io::Error> for DoweError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}
