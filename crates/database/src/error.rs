use std::fmt::{Display, Formatter};

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    NotFound(String),
    AlreadyExists(String),
    InvalidName(String),
    InvalidUlid(String),
    InvalidQuery(String),
    Authentication(String),
    Authorization(String),
    Remote(String),
    TypeError(String),
    TransactionConflict(String),
    DurabilityError(String),
    Corruption(String),
    UnsupportedFormat(String),
    Io(String),
}

impl StoreError {
    pub fn category(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NotFound",
            Self::AlreadyExists(_) => "AlreadyExists",
            Self::InvalidName(_) => "InvalidName",
            Self::InvalidUlid(_) => "InvalidUlid",
            Self::InvalidQuery(_) => "InvalidQuery",
            Self::Authentication(_) => "Authentication",
            Self::Authorization(_) => "Authorization",
            Self::Remote(_) => "Remote",
            Self::TypeError(_) => "TypeError",
            Self::TransactionConflict(_) => "TransactionConflict",
            Self::DurabilityError(_) => "DurabilityError",
            Self::Corruption(_) => "Corruption",
            Self::UnsupportedFormat(_) => "UnsupportedFormat",
            Self::Io(_) => "Io",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::NotFound(message)
            | Self::AlreadyExists(message)
            | Self::InvalidName(message)
            | Self::InvalidUlid(message)
            | Self::InvalidQuery(message)
            | Self::Authentication(message)
            | Self::Authorization(message)
            | Self::Remote(message)
            | Self::TypeError(message)
            | Self::TransactionConflict(message)
            | Self::DurabilityError(message)
            | Self::Corruption(message)
            | Self::UnsupportedFormat(message)
            | Self::Io(message) => message,
        }
    }
}

impl Display for StoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.category(), self.message())
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<dowe_id::IdError> for StoreError {
    fn from(error: dowe_id::IdError) -> Self {
        Self::InvalidUlid(error.to_string())
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidQuery(error.to_string())
    }
}
