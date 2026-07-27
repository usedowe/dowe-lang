use std::fmt::{Display, Formatter};

pub type KvResult<T> = Result<T, KvError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KvError {
    NotFound(String),
    InvalidName(String),
    InvalidRequest(String),
    Authentication(String),
    Authorization(String),
    Remote(String),
    DurabilityError(String),
    Corruption(String),
    Io(String),
}

impl KvError {
    pub fn category(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NotFound",
            Self::InvalidName(_) => "InvalidName",
            Self::InvalidRequest(_) => "InvalidRequest",
            Self::Authentication(_) => "Authentication",
            Self::Authorization(_) => "Authorization",
            Self::Remote(_) => "Remote",
            Self::DurabilityError(_) => "DurabilityError",
            Self::Corruption(_) => "Corruption",
            Self::Io(_) => "Io",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::NotFound(message)
            | Self::InvalidName(message)
            | Self::InvalidRequest(message)
            | Self::Authentication(message)
            | Self::Authorization(message)
            | Self::Remote(message)
            | Self::DurabilityError(message)
            | Self::Corruption(message)
            | Self::Io(message) => message,
        }
    }
}

impl Display for KvError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.category(), self.message())
    }
}

impl std::error::Error for KvError {}

impl From<std::io::Error> for KvError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<serde_json::Error> for KvError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidRequest(error.to_string())
    }
}
