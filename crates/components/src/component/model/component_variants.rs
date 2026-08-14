#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewIcon {
    Plus,
    Link,
    Edit,
    Trash,
    Search,
    Settings,
    Upload,
    File,
    Dismiss,
    Moon,
    Sun,
}

impl ViewIcon {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "plus" => Some(Self::Plus),
            "link" => Some(Self::Link),
            "edit" => Some(Self::Edit),
            "trash" => Some(Self::Trash),
            "search" => Some(Self::Search),
            "settings" => Some(Self::Settings),
            "upload" => Some(Self::Upload),
            "file" => Some(Self::File),
            "dismiss" => Some(Self::Dismiss),
            "moon" => Some(Self::Moon),
            "sun" => Some(Self::Sun),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plus => "plus",
            Self::Link => "link",
            Self::Edit => "edit",
            Self::Trash => "trash",
            Self::Search => "search",
            Self::Settings => "settings",
            Self::Upload => "upload",
            Self::File => "file",
            Self::Dismiss => "dismiss",
            Self::Moon => "moon",
            Self::Sun => "sun",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Plus,
            Self::Link,
            Self::Edit,
            Self::Trash,
            Self::Search,
            Self::Settings,
            Self::Upload,
            Self::File,
            Self::Dismiss,
            Self::Moon,
            Self::Sun,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonVariant {
    Text,
    Circular,
    Rectangular,
    Rounded,
}

impl SkeletonVariant {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "circular" => Some(Self::Circular),
            "rectangular" => Some(Self::Rectangular),
            "rounded" => Some(Self::Rounded),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Circular => "circular",
            Self::Rectangular => "rectangular",
            Self::Rounded => "rounded",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Text, Self::Circular, Self::Rectangular, Self::Rounded]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonAnimation {
    Pulse,
    Wave,
    None,
}

impl SkeletonAnimation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "pulse" => Some(Self::Pulse),
            "wave" => Some(Self::Wave),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pulse => "pulse",
            Self::Wave => "wave",
            Self::None => "none",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Pulse, Self::Wave, Self::None]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Primary,
    Secondary,
    Muted,
    Success,
    Info,
    Warning,
    Danger,
    Error,
}

impl ToastKind {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "primary" => Some(Self::Primary),
            "secondary" => Some(Self::Secondary),
            "muted" => Some(Self::Muted),
            "success" => Some(Self::Success),
            "info" => Some(Self::Info),
            "warning" => Some(Self::Warning),
            "danger" => Some(Self::Danger),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Muted => "muted",
            Self::Success => "success",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Error => "error",
        }
    }

    pub fn color(self) -> ColorFamily {
        match self {
            Self::Primary => ColorFamily::Primary,
            Self::Secondary => ColorFamily::Secondary,
            Self::Muted => ColorFamily::Muted,
            Self::Success => ColorFamily::Success,
            Self::Info => ColorFamily::Info,
            Self::Warning => ColorFamily::Warning,
            Self::Danger | Self::Error => ColorFamily::Danger,
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Primary,
            Self::Secondary,
            Self::Muted,
            Self::Success,
            Self::Info,
            Self::Warning,
            Self::Danger,
            Self::Error,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatBoxMode {
    Conversation,
    Prompt,
}

impl ChatBoxMode {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "conversation" => Some(Self::Conversation),
            "prompt" => Some(Self::Prompt),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Conversation => "conversation",
            Self::Prompt => "prompt",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Conversation, Self::Prompt]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyKind {
    Playlist,
    Result,
    Data,
    Template,
}

impl EmptyKind {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "playlist" => Some(Self::Playlist),
            "result" => Some(Self::Result),
            "data" => Some(Self::Data),
            "template" => Some(Self::Template),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Playlist => "playlist",
            Self::Result => "result",
            Self::Data => "data",
            Self::Template => "template",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Playlist, Self::Result, Self::Data, Self::Template]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarqueeSpeed {
    Slow,
    Normal,
    Fast,
}

impl MarqueeSpeed {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "slow" => Some(Self::Slow),
            "normal" => Some(Self::Normal),
            "fast" => Some(Self::Fast),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Slow => "slow",
            Self::Normal => "normal",
            Self::Fast => "fast",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Slow, Self::Normal, Self::Fast]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarqueeOrientation {
    Horizontal,
    Vertical,
}

impl MarqueeOrientation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Horizontal, Self::Vertical]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsVariant {
    Solid,
    Outlined,
    Line,
    Ghost,
    Pills,
    Stepper,
}

impl TabsVariant {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "solid" => Some(Self::Solid),
            "outlined" | "outline" => Some(Self::Outlined),
            "line" => Some(Self::Line),
            "ghost" => Some(Self::Ghost),
            "pills" => Some(Self::Pills),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Outlined => "outlined",
            Self::Line => "line",
            Self::Ghost => "ghost",
            Self::Pills => "pills",
            Self::Stepper => "stepper",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Solid,
            Self::Outlined,
            Self::Line,
            Self::Ghost,
            Self::Pills,
        ]
    }
}

