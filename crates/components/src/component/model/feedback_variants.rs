#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Success,
    Error,
    Info,
    Warning,
}

impl AlertKind {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "success" => Some(Self::Success),
            "error" => Some(Self::Error),
            "info" => Some(Self::Info),
            "warning" => Some(Self::Warning),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Info => "info",
            Self::Warning => "warning",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Success, Self::Error, Self::Info, Self::Warning]
    }

    pub fn color(self) -> ColorFamily {
        match self {
            Self::Success => ColorFamily::Success,
            Self::Error => ColorFamily::Danger,
            Self::Info => ColorFamily::Info,
            Self::Warning => ColorFamily::Warning,
        }
    }
}

