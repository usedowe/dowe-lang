use super::*;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppOutput {
    pub files: Vec<GeneratedFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewTargetRoutes {
    pub web: Vec<ViewRoute>,
    pub desktop: Vec<ViewRoute>,
    pub android: Vec<ViewRoute>,
    pub ios: Vec<ViewRoute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ViewPlatform {
    Web,
    Desktop,
    Android,
    Ios,
}

impl ViewPlatform {
    pub fn all() -> &'static [Self] {
        &[Self::Web, Self::Desktop, Self::Android, Self::Ios]
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "web" => Some(Self::Web),
            "desktop" => Some(Self::Desktop),
            "android" => Some(Self::Android),
            "ios" => Some(Self::Ios),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Desktop => "desktop",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFile {
    pub relative_path: PathBuf,
    pub content: String,
    pub kind: String,
    pub target: String,
}
