use std::fmt::{Display, Formatter};

pub type QueueResult<T> = Result<T, QueueError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueError {
    InvalidName(String),
    InvalidTopic(String),
    InvalidRequest(String),
    QueueNotFound(String),
    Authentication(String),
    Authorization(String),
    InvalidReceipt(String),
    DurabilityError(String),
    Corruption(String),
    Remote(String),
    Unsupported(String),
    Io(String),
}

impl QueueError {
    pub fn category(&self) -> &'static str {
        match self {
            Self::InvalidName(_) => "InvalidName",
            Self::InvalidTopic(_) => "InvalidTopic",
            Self::InvalidRequest(_) => "InvalidRequest",
            Self::QueueNotFound(_) => "QueueNotFound",
            Self::Authentication(_) => "Authentication",
            Self::Authorization(_) => "Authorization",
            Self::InvalidReceipt(_) => "InvalidReceipt",
            Self::DurabilityError(_) => "DurabilityError",
            Self::Corruption(_) => "Corruption",
            Self::Remote(_) => "Remote",
            Self::Unsupported(_) => "Unsupported",
            Self::Io(_) => "Io",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::InvalidName(message)
            | Self::InvalidTopic(message)
            | Self::InvalidRequest(message)
            | Self::QueueNotFound(message)
            | Self::Authentication(message)
            | Self::Authorization(message)
            | Self::InvalidReceipt(message)
            | Self::DurabilityError(message)
            | Self::Corruption(message)
            | Self::Remote(message)
            | Self::Unsupported(message)
            | Self::Io(message) => message,
        }
    }
}

impl Display for QueueError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.category(), self.message())
    }
}

impl std::error::Error for QueueError {}

impl From<std::io::Error> for QueueError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<serde_json::Error> for QueueError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidRequest(error.to_string())
    }
}
