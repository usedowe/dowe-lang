use std::fmt::{Display, Formatter};

pub type DeployResult<T> = Result<T, DeployError>;

#[derive(Debug)]
pub struct DeployError {
    message: String,
}

impl DeployError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for DeployError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for DeployError {}

impl From<std::io::Error> for DeployError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<dowe_compiler::DoweError> for DeployError {
    fn from(error: dowe_compiler::DoweError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<serde_json::Error> for DeployError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error.to_string())
    }
}
