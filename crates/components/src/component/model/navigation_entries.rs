#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlayEntry {
    Item(OverlayItemProps),
    Divider,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayItemProps {
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<SideNavIcon>,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandEntry {
    Item(OverlayItemProps),
    Group {
        label: String,
        icon: Option<SideNavIcon>,
        items: Vec<OverlayItemProps>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsProps {
    pub style: StyleProps,
    pub variant: TabsVariant,
    pub color: ColorFamily,
    pub position: TabsPosition,
    pub variant_explicit: bool,
    pub color_explicit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem {
    pub id: String,
    pub label: String,
    pub i18n: Option<String>,
    pub children: Vec<ViewNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavMenuItem {
    Item(NavMenuItemProps),
    Submenu {
        props: NavMenuItemProps,
        items: Vec<NavMenuItemProps>,
    },
    Megamenu {
        props: NavMenuItemProps,
        content: Vec<ViewNode>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavMenuItemProps {
    pub label: String,
    pub i18n: Option<String>,
    pub description: Option<String>,
    pub description_i18n: Option<String>,
    pub icon: Option<SideNavIcon>,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsPosition {
    Top,
    Bottom,
    Start,
    End,
}

impl TabsPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Start => "start",
            Self::End => "end",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Top, Self::Bottom, Self::Start, Self::End]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarStatus {
    Online,
    Offline,
    Busy,
    Away,
}

impl AvatarStatus {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "online" => Some(Self::Online),
            "offline" => Some(Self::Offline),
            "busy" => Some(Self::Busy),
            "away" => Some(Self::Away),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
            Self::Busy => "busy",
            Self::Away => "away",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Online, Self::Offline, Self::Busy, Self::Away]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayCornerPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl OverlayCornerPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "top-left" => Some(Self::TopLeft),
            "top-right" => Some(Self::TopRight),
            "bottom-left" => Some(Self::BottomLeft),
            "bottom-right" => Some(Self::BottomRight),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::TopLeft,
            Self::TopRight,
            Self::BottomLeft,
            Self::BottomRight,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayPosition {
    Top,
    Bottom,
    Start,
    End,
}

impl OverlayPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Start => "start",
            Self::End => "end",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Top, Self::Bottom, Self::Start, Self::End]
    }
}

