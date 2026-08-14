#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageAspect {
    Horizontal,
    Vertical,
    Square,
    Auto,
}

impl ImageAspect {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            "square" => Some(Self::Square),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
            Self::Square => "square",
            Self::Auto => "auto",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Horizontal, Self::Vertical, Self::Square, Self::Auto]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageObjectFit {
    Cover,
    Contain,
    Fill,
    None,
}

impl ImageObjectFit {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "cover" => Some(Self::Cover),
            "contain" => Some(Self::Contain),
            "fill" => Some(Self::Fill),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cover => "cover",
            Self::Contain => "contain",
            Self::Fill => "fill",
            Self::None => "none",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Cover, Self::Contain, Self::Fill, Self::None]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageLoading {
    Lazy,
    Eager,
}

impl ImageLoading {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselOrientation {
    Horizontal,
    Vertical,
}

impl CarouselOrientation {
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
pub enum CarouselVariant {
    Simple,
    Snapping,
    Masonry,
    Rtl,
    Sticky,
    Controls,
    Dots,
    Thumbnails,
    CoverFlow,
    Slideshow,
    Stories,
    SmartStack,
    CardStack,
    Flipbook,
}

impl CarouselVariant {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "simple" => Some(Self::Simple),
            "snapping" => Some(Self::Snapping),
            "masonry" => Some(Self::Masonry),
            "rtl" => Some(Self::Rtl),
            "sticky" => Some(Self::Sticky),
            "controls" => Some(Self::Controls),
            "dots" => Some(Self::Dots),
            "thumbnails" => Some(Self::Thumbnails),
            "coverFlow" => Some(Self::CoverFlow),
            "slideshow" => Some(Self::Slideshow),
            "stories" => Some(Self::Stories),
            "smartStack" => Some(Self::SmartStack),
            "cardStack" => Some(Self::CardStack),
            "flipbook" => Some(Self::Flipbook),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Snapping => "snapping",
            Self::Masonry => "masonry",
            Self::Rtl => "rtl",
            Self::Sticky => "sticky",
            Self::Controls => "controls",
            Self::Dots => "dots",
            Self::Thumbnails => "thumbnails",
            Self::CoverFlow => "coverFlow",
            Self::Slideshow => "slideshow",
            Self::Stories => "stories",
            Self::SmartStack => "smartStack",
            Self::CardStack => "cardStack",
            Self::Flipbook => "flipbook",
        }
    }

    pub fn class_name(self) -> &'static str {
        match self {
            Self::CoverFlow => "cover-flow",
            Self::SmartStack => "smart-stack",
            Self::CardStack => "card-stack",
            _ => self.as_str(),
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Simple,
            Self::Snapping,
            Self::Masonry,
            Self::Rtl,
            Self::Sticky,
            Self::Controls,
            Self::Dots,
            Self::Thumbnails,
            Self::CoverFlow,
            Self::Slideshow,
            Self::Stories,
            Self::SmartStack,
            Self::CardStack,
            Self::Flipbook,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselIndicatorType {
    Bar,
    Dot,
}

impl CarouselIndicatorType {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "bar" => Some(Self::Bar),
            "dot" => Some(Self::Dot),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bar => "bar",
            Self::Dot => "dot",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Bar, Self::Dot]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoAspect {
    Horizontal,
    Vertical,
    Square,
}

impl VideoAspect {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            "square" => Some(Self::Square),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
            Self::Square => "square",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Horizontal, Self::Vertical, Self::Square]
    }
}

