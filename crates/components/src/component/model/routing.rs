#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRoute {
    pub id: String,
    pub route_path: String,
    pub layout_tree: ViewNode,
    pub page_tree: ViewNode,
    pub sections: Vec<ViewSection>,
    pub navigation_actions: Vec<ViewNavigationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewSection {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewNavigationAction {
    pub id: String,
    pub action: NavigationAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewMetadata {
    pub name: String,
    pub content: String,
}

pub const VIEW_META_NAMES: &[&str] = &[
    "title",
    "description",
    "keywords",
    "robots",
    "canonical",
    "og:title",
    "og:description",
    "og:image",
    "og:image:alt",
    "og:type",
    "og:url",
    "og:site_name",
    "twitter:card",
    "twitter:title",
    "twitter:description",
    "twitter:image",
    "twitter:image:alt",
    "twitter:site",
    "twitter:creator",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationAction {
    Internal {
        path: String,
        fragment: Option<String>,
        operation: NavigationOperation,
    },
    Section {
        fragment: String,
        operation: NavigationOperation,
    },
    External {
        url: String,
        web_target: WebTarget,
        native_external_mode: NativeExternalMode,
    },
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationOperation {
    Push,
    Replace,
}

impl NavigationOperation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "push" => Some(Self::Push),
            "replace" => Some(Self::Replace),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Push => "push",
            Self::Replace => "replace",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Push, Self::Replace]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebTarget {
    SelfTarget,
    Blank,
}

impl WebTarget {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "self" => Some(Self::SelfTarget),
            "blank" => Some(Self::Blank),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelfTarget => "self",
            Self::Blank => "blank",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::SelfTarget, Self::Blank]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeExternalMode {
    System,
    Webview,
}

impl NativeExternalMode {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "webview" => Some(Self::Webview),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Webview => "webview",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::System, Self::Webview]
    }
}
