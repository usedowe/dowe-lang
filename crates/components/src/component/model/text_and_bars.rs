#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextProps {
    pub style: StyleProps,
    pub align: Option<ResponsiveValue<TextAlign>>,
    pub size: Option<ResponsiveValue<TextSize>>,
    pub weight: Option<ResponsiveValue<TextWeight>>,
    pub letter_spacing: Option<ResponsiveValue<TextSpacing>>,
    pub i18n: Option<String>,
    pub title: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertProps {
    pub style: VariantProps,
    pub kind: AlertKind,
    pub message: String,
    pub visible: Option<String>,
    pub on_close: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgProps {
    pub style: StyleProps,
    pub view_box: SvgViewBox,
    pub data: Option<String>,
    pub icon_name: Option<String>,
    pub icon_fallback: Option<String>,
    pub icon_fill: Option<ColorToken>,
    pub icon_stroke: Option<ColorToken>,
    pub motion: Option<SvgMotion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgMotion {
    pub source: &'static str,
    pub fill: Option<ColorToken>,
    pub stroke: Option<ColorToken>,
    pub animated: bool,
}

impl SvgProps {
    pub fn is_animated(&self) -> bool {
        self.motion.as_ref().is_some_and(|source| source.animated)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BarProps {
    pub style: VariantProps,
    pub bordered: bool,
    pub blurred: bool,
    pub boxed: bool,
    pub floating: bool,
    pub position: BarPosition,
    pub hide_on_scroll: bool,
    pub dock_on_scroll: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BottomBarTab {
    pub label: String,
    pub i18n: Option<String>,
    pub featured: bool,
    pub icon: SideNavIcon,
    pub navigation: NavigationAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarPosition {
    #[default]
    Static,
    Sticky,
    Fixed,
}

impl BarPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "static" => Some(Self::Static),
            "sticky" => Some(Self::Sticky),
            "fixed" => Some(Self::Fixed),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Sticky => "sticky",
            Self::Fixed => "fixed",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Static, Self::Sticky, Self::Fixed]
    }
}
