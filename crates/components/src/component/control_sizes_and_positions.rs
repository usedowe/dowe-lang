#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideNavSize {
    Sm,
    Md,
    Lg,
}

impl SideNavSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Sm, Self::Md, Self::Lg]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableSize {
    Sm,
    Md,
    Lg,
}

impl TableSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Sm, Self::Md, Self::Lg]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableColumnAlign {
    Start,
    Center,
    End,
}

impl TableColumnAlign {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "center" => Some(Self::Center),
            "end" => Some(Self::End),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Start, Self::Center, Self::End]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerPosition {
    Start,
    End,
    Top,
    Bottom,
}

impl DrawerPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Start, Self::End, Self::Top, Self::Bottom]
    }
}
