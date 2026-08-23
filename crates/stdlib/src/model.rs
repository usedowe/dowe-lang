use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StdlibCall {
    pub namespace: String,
    pub function: String,
    pub args: Vec<StdlibArgument>,
}

impl StdlibCall {
    pub fn name(&self) -> String {
        format!("{}.{}", self.namespace, self.function)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StdlibArgument {
    pub name: String,
    pub value: StdlibValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StdlibValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Reference(String),
    Array(Vec<StdlibValue>),
    Object(Vec<(String, StdlibValue)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdlibSurface {
    Server,
    Views,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdlibReturnKind {
    Unknown,
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibSignature {
    pub namespace: String,
    pub function: String,
    pub required: &'static [&'static str],
    pub optional: &'static [&'static str],
    pub return_kind: StdlibReturnKind,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibError {
    pub code: StdlibErrorCode,
    pub message: String,
}

impl StdlibError {
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self {
            code: StdlibErrorCode::InvalidArgument,
            message: message.into(),
        }
    }

    pub fn limit_exceeded(message: impl Into<String>) -> Self {
        Self {
            code: StdlibErrorCode::LimitExceeded,
            message: message.into(),
        }
    }

    pub fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: StdlibErrorCode::ParseError,
            message: message.into(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            code: StdlibErrorCode::Unsupported,
            message: message.into(),
        }
    }

    pub fn non_finite_number(message: impl Into<String>) -> Self {
        Self {
            code: StdlibErrorCode::NonFiniteNumber,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for StdlibError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for StdlibError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdlibErrorCode {
    InvalidArgument,
    LimitExceeded,
    ParseError,
    Unsupported,
    NonFiniteNumber,
}

impl StdlibErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidArgument => "stdlib_invalid_argument",
            Self::LimitExceeded => "stdlib_limit_exceeded",
            Self::ParseError => "stdlib_parse_error",
            Self::Unsupported => "stdlib_unsupported",
            Self::NonFiniteNumber => "stdlib_non_finite_number",
        }
    }
}

pub type StdlibResult<T> = Result<T, StdlibError>;
