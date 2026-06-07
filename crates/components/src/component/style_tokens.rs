pub const TAILWIND_SCALE: &[ScaleValue] = &[
    ScaleValue::from_half_steps(0),
    ScaleValue::from_half_steps(1),
    ScaleValue::from_half_steps(2),
    ScaleValue::from_half_steps(3),
    ScaleValue::from_half_steps(4),
    ScaleValue::from_half_steps(5),
    ScaleValue::from_half_steps(6),
    ScaleValue::from_half_steps(7),
    ScaleValue::from_half_steps(8),
    ScaleValue::from_half_steps(10),
    ScaleValue::from_half_steps(12),
    ScaleValue::from_half_steps(14),
    ScaleValue::from_half_steps(16),
    ScaleValue::from_half_steps(18),
    ScaleValue::from_half_steps(20),
    ScaleValue::from_half_steps(22),
    ScaleValue::from_half_steps(24),
    ScaleValue::from_half_steps(28),
    ScaleValue::from_half_steps(32),
    ScaleValue::from_half_steps(40),
    ScaleValue::from_half_steps(48),
    ScaleValue::from_half_steps(56),
    ScaleValue::from_half_steps(64),
    ScaleValue::from_half_steps(72),
    ScaleValue::from_half_steps(80),
    ScaleValue::from_half_steps(88),
    ScaleValue::from_half_steps(96),
    ScaleValue::from_half_steps(104),
    ScaleValue::from_half_steps(112),
    ScaleValue::from_half_steps(120),
    ScaleValue::from_half_steps(128),
    ScaleValue::from_half_steps(144),
    ScaleValue::from_half_steps(160),
    ScaleValue::from_half_steps(192),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeValue {
    Scale(ScaleValue),
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundedSize {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Full,
}

impl RoundedSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "xs" => Some(Self::Xs),
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Full => "full",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Xs, Self::Sm, Self::Md, Self::Lg, Self::Xl, Self::Full]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderWidth(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSpan(pub u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverSource(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlayPaint {
    BlackOpacity(String),
    Color(ColorToken),
    Rgba(String),
    LinearGradient(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionBackground {
    Soft,
    Aurora,
    Sunrise,
    Ocean,
    Meadow,
    Slate,
}

impl SectionBackground {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "soft" => Some(Self::Soft),
            "aurora" => Some(Self::Aurora),
            "sunrise" => Some(Self::Sunrise),
            "ocean" => Some(Self::Ocean),
            "meadow" => Some(Self::Meadow),
            "slate" => Some(Self::Slate),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Soft => "soft",
            Self::Aurora => "aurora",
            Self::Sunrise => "sunrise",
            Self::Ocean => "ocean",
            Self::Meadow => "meadow",
            Self::Slate => "slate",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Soft,
            Self::Aurora,
            Self::Sunrise,
            Self::Ocean,
            Self::Meadow,
            Self::Slate,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewAnimation {
    None,
    FadeIn,
    SlideUp,
    SlideDown,
    SlideLeft,
    SlideRight,
    ScaleIn,
}

impl ViewAnimation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "fadeIn" => Some(Self::FadeIn),
            "slideUp" => Some(Self::SlideUp),
            "slideDown" => Some(Self::SlideDown),
            "slideLeft" => Some(Self::SlideLeft),
            "slideRight" => Some(Self::SlideRight),
            "scaleIn" => Some(Self::ScaleIn),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::FadeIn => "fadeIn",
            Self::SlideUp => "slideUp",
            Self::SlideDown => "slideDown",
            Self::SlideLeft => "slideLeft",
            Self::SlideRight => "slideRight",
            Self::ScaleIn => "scaleIn",
        }
    }

    pub fn class_suffix(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::FadeIn => "fade-in",
            Self::SlideUp => "slide-up",
            Self::SlideDown => "slide-down",
            Self::SlideLeft => "slide-left",
            Self::SlideRight => "slide-right",
            Self::ScaleIn => "scale-in",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::None,
            Self::FadeIn,
            Self::SlideUp,
            Self::SlideDown,
            Self::SlideLeft,
            Self::SlideRight,
            Self::ScaleIn,
        ]
    }
}

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
    Template(String),
}

impl GridTracks {
    pub fn class_suffix(&self) -> String {
        match self {
            Self::Auto => "auto".to_string(),
            Self::Count(value) => value.to_string(),
            Self::Template(value) => format!("tpl-{}", stable_slug(value)),
        }
    }

    pub fn count(&self) -> Option<u16> {
        match self {
            Self::Count(value) => Some(*value),
            Self::Template(value) => Some(value.split_whitespace().count() as u16),
            Self::Auto => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridAlignment {
    Start,
    Center,
    End,
    Stretch,
}

impl GridAlignment {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "center" => Some(Self::Center),
            "end" => Some(Self::End),
            "stretch" => Some(Self::Stretch),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
            Self::Stretch => "stretch",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Start, Self::Center, Self::End, Self::Stretch]
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
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Start,
            Self::Center,
            Self::End,
            Self::Between,
            Self::Around,
            Self::Evenly,
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
}

impl Align {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "start" | "flex-start" => Some(Self::Start),
            "center" => Some(Self::Center),
            "end" | "flex-end" => Some(Self::End),
            "stretch" => Some(Self::Stretch),
            "baseline" => Some(Self::Baseline),
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
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Start,
            Self::Center,
            Self::End,
            Self::Stretch,
            Self::Baseline,
        ]
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

pub const INPUT_MIN_HEIGHT: ScaleValue = ScaleValue::from_half_steps(20);
pub const INPUT_HORIZONTAL_PADDING: ScaleValue = ScaleValue::from_half_steps(6);
pub const INPUT_TEXT_SIZE: TextSize = TextSize::Md;

impl TextSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "xs" => Some(Self::Xs),
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            "2xl" => Some(Self::TwoXl),
            "3xl" => Some(Self::ThreeXl),
            "4xl" => Some(Self::FourXl),
            "5xl" => Some(Self::FiveXl),
            "6xl" => Some(Self::SixXl),
            "7xl" => Some(Self::SevenXl),
            "8xl" => Some(Self::EightXl),
            "9xl" => Some(Self::NineXl),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::TwoXl => "2xl",
            Self::ThreeXl => "3xl",
            Self::FourXl => "4xl",
            Self::FiveXl => "5xl",
            Self::SixXl => "6xl",
            Self::SevenXl => "7xl",
            Self::EightXl => "8xl",
            Self::NineXl => "9xl",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Xs,
            Self::Sm,
            Self::Md,
            Self::Lg,
            Self::Xl,
            Self::TwoXl,
            Self::ThreeXl,
            Self::FourXl,
            Self::FiveXl,
            Self::SixXl,
            Self::SevenXl,
            Self::EightXl,
            Self::NineXl,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWeight {
    Thin,
    Extralight,
    Light,
    Regular,
    Medium,
    Semibold,
    Bold,
    Extrabold,
    Black,
}

impl TextWeight {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "thin" => Some(Self::Thin),
            "extralight" => Some(Self::Extralight),
            "light" => Some(Self::Light),
            "regular" => Some(Self::Regular),
            "medium" => Some(Self::Medium),
            "semibold" => Some(Self::Semibold),
            "bold" => Some(Self::Bold),
            "extrabold" => Some(Self::Extrabold),
            "black" => Some(Self::Black),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Thin => "thin",
            Self::Extralight => "extralight",
            Self::Light => "light",
            Self::Regular => "regular",
            Self::Medium => "medium",
            Self::Semibold => "semibold",
            Self::Bold => "bold",
            Self::Extrabold => "extrabold",
            Self::Black => "black",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Thin,
            Self::Extralight,
            Self::Light,
            Self::Regular,
            Self::Medium,
            Self::Semibold,
            Self::Bold,
            Self::Extrabold,
            Self::Black,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSpacing {
    Tightest,
    Tighter,
    Tight,
    Normal,
    Wide,
    Wider,
    Widest,
}

impl TextSpacing {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "tightest" => Some(Self::Tightest),
            "tighter" => Some(Self::Tighter),
            "tight" => Some(Self::Tight),
            "normal" => Some(Self::Normal),
            "wide" => Some(Self::Wide),
            "wider" => Some(Self::Wider),
            "widest" => Some(Self::Widest),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tightest => "tightest",
            Self::Tighter => "tighter",
            Self::Tight => "tight",
            Self::Normal => "normal",
            Self::Wide => "wide",
            Self::Wider => "wider",
            Self::Widest => "widest",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Tightest,
            Self::Tighter,
            Self::Tight,
            Self::Normal,
            Self::Wide,
            Self::Wider,
            Self::Widest,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontCatalogEntry {
    pub token: FontFamily,
    pub display_name: &'static str,
    pub web_stack: &'static str,
    pub ios_family_name: &'static str,
    pub android_family_name: &'static str,
    pub package_assets: bool,
    pub weights: &'static [FontCatalogWeight],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontCatalogWeight {
    pub weight: TextWeight,
    pub numeric_weight: u16,
    pub asset_stem: &'static str,
}

pub fn font_catalog() -> &'static [FontCatalogEntry] {
    FONT_CATALOG
}
