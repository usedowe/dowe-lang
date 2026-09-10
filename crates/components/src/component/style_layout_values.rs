#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapSize {
    Scale(ScaleValue),
    Px(u16),
}

impl GapSize {
    pub fn class_suffix(self) -> String {
        match self {
            Self::Scale(value) => value.class_suffix(),
            Self::Px(value) => format!("px-{value}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GapValue {
    Single(GapSize),
    Pair(GapSize, GapSize),
}

impl GapValue {
    pub fn class_suffix(&self) -> String {
        match self {
            Self::Single(value) => value.class_suffix(),
            Self::Pair(row, column) => format!("{}-{}", row.class_suffix(), column.class_suffix()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridTracks {
    Auto,
    Count(u16),
    Fractions(Vec<u16>),
}

impl GridTracks {
    pub fn class_suffix(&self) -> String {
        match self {
            Self::Auto => "auto".to_string(),
            Self::Count(value) => value.to_string(),
            Self::Fractions(values) => format!(
                "fr-{}",
                values
                    .iter()
                    .map(u16::to_string)
                    .collect::<Vec<_>>()
                    .join("-")
            ),
        }
    }

    pub fn count(&self) -> Option<u16> {
        match self {
            Self::Count(value) => Some(*value),
            Self::Auto | Self::Fractions(_) => None,
        }
    }

    pub fn weights(&self) -> Option<&[u16]> {
        match self {
            Self::Fractions(values) => Some(values),
            Self::Auto | Self::Count(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridAlignment {
    Start,
    End,
    EndSafe,
    Center,
    CenterSafe,
    Between,
    Around,
    Evenly,
    Stretch,
    Baseline,
    BaselineLast,
    Normal,
}

impl GridAlignment {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            "end-safe" => Some(Self::EndSafe),
            "center" => Some(Self::Center),
            "center-safe" => Some(Self::CenterSafe),
            "between" => Some(Self::Between),
            "around" => Some(Self::Around),
            "evenly" => Some(Self::Evenly),
            "stretch" => Some(Self::Stretch),
            "baseline" => Some(Self::Baseline),
            "baseline-last" => Some(Self::BaselineLast),
            "normal" => Some(Self::Normal),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
            Self::EndSafe => "end-safe",
            Self::Center => "center",
            Self::CenterSafe => "center-safe",
            Self::Between => "between",
            Self::Around => "around",
            Self::Evenly => "evenly",
            Self::Stretch => "stretch",
            Self::Baseline => "baseline",
            Self::BaselineLast => "baseline-last",
            Self::Normal => "normal",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Start,
            Self::End,
            Self::EndSafe,
            Self::Center,
            Self::CenterSafe,
            Self::Between,
            Self::Around,
            Self::Evenly,
            Self::Stretch,
            Self::Baseline,
            Self::BaselineLast,
            Self::Normal,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
}

impl FlexDirection {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "row" => Some(Self::Row),
            "column" => Some(Self::Column),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Column => "column",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Row, Self::Column]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexItem {
    Initial,
    Auto,
    None,
    Fill,
}

impl FlexItem {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "initial" => Some(Self::Initial),
            "auto" => Some(Self::Auto),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Auto => "auto",
            Self::None => "none",
            Self::Fill => "1",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Initial, Self::Auto, Self::None, Self::Fill]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Justify {
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
    Stretch,
    Normal,
    EndSafe,
    CenterSafe,
}

impl Justify {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "start" | "flex-start" => Some(Self::Start),
            "center" => Some(Self::Center),
            "end" | "flex-end" => Some(Self::End),
            "between" | "space-between" => Some(Self::Between),
            "around" | "space-around" => Some(Self::Around),
            "evenly" | "space-evenly" => Some(Self::Evenly),
            "stretch" => Some(Self::Stretch),
            "normal" => Some(Self::Normal),
            "end-safe" => Some(Self::EndSafe),
            "center-safe" => Some(Self::CenterSafe),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
            Self::Between => "between",
            Self::Around => "around",
            Self::Evenly => "evenly",
            Self::Stretch => "stretch",
            Self::Normal => "normal",
            Self::EndSafe => "end-safe",
            Self::CenterSafe => "center-safe",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Start,
            Self::Center,
            Self::End,
            Self::EndSafe,
            Self::CenterSafe,
            Self::Between,
            Self::Around,
            Self::Evenly,
            Self::Stretch,
            Self::Normal,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
    BaselineLast,
    EndSafe,
    CenterSafe,
}

impl Align {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "start" | "flex-start" => Some(Self::Start),
            "center" => Some(Self::Center),
            "end" | "flex-end" => Some(Self::End),
            "stretch" => Some(Self::Stretch),
            "baseline" => Some(Self::Baseline),
            "baseline-last" => Some(Self::BaselineLast),
            "end-safe" => Some(Self::EndSafe),
            "center-safe" => Some(Self::CenterSafe),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
            Self::Stretch => "stretch",
            Self::Baseline => "baseline",
            Self::BaselineLast => "baseline-last",
            Self::EndSafe => "end-safe",
            Self::CenterSafe => "center-safe",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Start,
            Self::Center,
            Self::End,
            Self::EndSafe,
            Self::CenterSafe,
            Self::Baseline,
            Self::BaselineLast,
            Self::Stretch,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Start,
    Center,
    End,
    Justify,
}

impl TextAlign {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "center" => Some(Self::Center),
            "end" => Some(Self::End),
            "justify" => Some(Self::Justify),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
            Self::Justify => "justify",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Start, Self::Center, Self::End, Self::Justify]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSize {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    TwoXl,
    ThreeXl,
    FourXl,
    FiveXl,
    SixXl,
    SevenXl,
    EightXl,
    NineXl,
}

