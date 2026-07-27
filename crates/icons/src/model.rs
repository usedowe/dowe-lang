use crate::{IconError, IconResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IconTarget {
    Web,
    Desktop,
    Ios,
    Android,
}

impl IconTarget {
    pub const ALL: [Self; 4] = [Self::Web, Self::Desktop, Self::Ios, Self::Android];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Desktop => "desktop",
            Self::Ios => "ios",
            Self::Android => "android",
        }
    }
}

impl FromStr for IconTarget {
    type Err = IconError;

    fn from_str(value: &str) -> IconResult<Self> {
        match value {
            "web" => Ok(Self::Web),
            "desktop" => Ok(Self::Desktop),
            "ios" => Ok(Self::Ios),
            "android" => Ok(Self::Android),
            _ => Err(IconError::new(format!(
                "unknown icon target `{value}`; expected web, desktop, ios or android"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IconRounded {
    None,
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Full,
}

impl IconRounded {
    pub const ALL: [Self; 7] = [
        Self::None,
        Self::Xs,
        Self::Sm,
        Self::Md,
        Self::Lg,
        Self::Xl,
        Self::Full,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Full => "full",
        }
    }

    pub(crate) fn ratio(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Xs => 0.0625,
            Self::Sm => 0.125,
            Self::Md => 0.1875,
            Self::Lg => 0.25,
            Self::Xl => 0.375,
            Self::Full => 0.5,
        }
    }
}

impl FromStr for IconRounded {
    type Err = IconError;

    fn from_str(value: &str) -> IconResult<Self> {
        match value {
            "none" => Ok(Self::None),
            "xs" => Ok(Self::Xs),
            "sm" => Ok(Self::Sm),
            "md" => Ok(Self::Md),
            "lg" => Ok(Self::Lg),
            "xl" => Ok(Self::Xl),
            "full" => Ok(Self::Full),
            _ => Err(IconError::new(format!(
                "unknown icon rounded value `{value}`; expected none, xs, sm, md, lg, xl or full"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateIconOptions {
    pub root: PathBuf,
    pub source: PathBuf,
    pub background: String,
    pub rounded: IconRounded,
    pub targets: Vec<IconTarget>,
}

impl GenerateIconOptions {
    pub fn new(
        root: impl AsRef<Path>,
        source: impl AsRef<Path>,
        background: impl Into<String>,
        rounded: IconRounded,
    ) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            source: source.as_ref().to_path_buf(),
            background: background.into(),
            rounded,
            targets: IconTarget::ALL.to_vec(),
        }
    }

    pub fn with_targets(mut self, targets: impl IntoIterator<Item = IconTarget>) -> Self {
        self.targets = targets.into_iter().collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IconReport {
    pub output_dir: PathBuf,
    pub manifest: PathBuf,
    pub targets: Vec<IconTarget>,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IconColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl IconColor {
    pub(crate) fn parse(value: &str) -> IconResult<Self> {
        if value.len() != 7 || !value.starts_with('#') {
            return Err(IconError::new(
                "icon background must use #RRGGBB hexadecimal format",
            ));
        }
        let parse = |range| {
            u8::from_str_radix(&value[range], 16)
                .map_err(|_| IconError::new("icon background must use #RRGGBB hexadecimal format"))
        };
        Ok(Self {
            red: parse(1..3)?,
            green: parse(3..5)?,
            blue: parse(5..7)?,
        })
    }

    pub(crate) fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}
