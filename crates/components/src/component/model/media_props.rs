#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoProps {
    pub style: VariantProps,
    pub src: String,
    pub poster: Option<String>,
    pub autoplay: bool,
    pub aspect: VideoAspect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IframeProps {
    pub style: StyleProps,
    pub src: String,
    pub title: String,
    pub loading: IframeLoading,
    pub allow: Vec<String>,
    pub sandbox: Option<Vec<String>>,
    pub allow_fullscreen: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceProps {
    pub style: StyleProps,
    pub device: DeviceProfile,
    pub options: Vec<DeviceOption>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceOption {
    pub profile: DeviceProfile,
    pub icon: SideNavIcon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceProfile {
    Mobile,
    Tablet,
    Laptop,
    Monitor,
}

impl DeviceProfile {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "mobile" => Some(Self::Mobile),
            "tablet" => Some(Self::Tablet),
            "laptop" => Some(Self::Laptop),
            "monitor" => Some(Self::Monitor),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mobile => "mobile",
            Self::Tablet => "tablet",
            Self::Laptop => "laptop",
            Self::Monitor => "monitor",
        }
    }

    pub fn dimensions(self) -> (u16, u16) {
        match self {
            Self::Mobile => (390, 844),
            Self::Tablet => (768, 1024),
            Self::Laptop => (1440, 900),
            Self::Monitor => (1920, 1080),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IframeLoading {
    Lazy,
    Eager,
}

impl IframeLoading {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "lazy" => Some(Self::Lazy),
            "eager" => Some(Self::Eager),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lazy => "lazy",
            Self::Eager => "eager",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Lazy, Self::Eager]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasProps {
    pub style: StyleProps,
    pub scene: String,
    pub view_width: u16,
    pub view_height: u16,
    pub fit: CanvasFit,
    pub fps: u8,
    pub autoplay: bool,
    pub background: CanvasBackground,
    pub pixelated: bool,
    pub label: String,
    pub on_pointer: Option<String>,
    pub on_key: Option<String>,
    pub on_motion: Option<String>,
    pub motion_rate: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasFit {
    Contain,
    Cover,
    Stretch,
}

impl CanvasFit {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "contain" => Some(Self::Contain),
            "cover" => Some(Self::Cover),
            "stretch" => Some(Self::Stretch),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Contain => "contain",
            Self::Cover => "cover",
            Self::Stretch => "stretch",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasBackground {
    Transparent,
    Color(ColorToken),
}

