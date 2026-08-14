#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandlestickProps {
    pub style: VariantProps,
    pub data: String,
    pub stream: Option<String>,
    pub up_color: ColorToken,
    pub down_color: ColorToken,
    pub empty_label: String,
    pub max_points: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartCommonProps {
    pub style: VariantProps,
    pub data: Option<String>,
    pub series: Option<String>,
    pub size: ChartSize,
    pub palette: ChartPalette,
    pub legend_position: ChartLegendPosition,
    pub empty_label: String,
    pub loading: bool,
    pub hide_legend: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcChartProps {
    pub common: ChartCommonProps,
    pub center_text: Option<String>,
    pub center_value: Option<String>,
    pub thickness: u16,
    pub gap: u16,
    pub start_angle: i16,
    pub end_angle: i16,
    pub show_inline_labels: bool,
    pub hide_values: bool,
    pub show_glow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaChartProps {
    pub common: ChartCommonProps,
    pub curve: ChartCurve,
    pub stroke_width: u16,
    pub fill_opacity: u16,
    pub stacked: bool,
    pub hide_line: bool,
    pub show_points: bool,
    pub hide_grid: bool,
    pub hide_x_axis: bool,
    pub hide_y_axis: bool,
    pub show_glow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarChartProps {
    pub common: ChartCommonProps,
    pub grouped: bool,
    pub stacked: bool,
    pub show_values: bool,
    pub bar_radius: u16,
    pub hide_grid: bool,
    pub show_glow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineChartProps {
    pub common: ChartCommonProps,
    pub curve: ChartCurve,
    pub stroke_width: u16,
    pub point_radius: u16,
    pub hide_points: bool,
    pub hide_grid: bool,
    pub hide_x_axis: bool,
    pub hide_y_axis: bool,
    pub show_gradient_fill: bool,
    pub show_glow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PieChartProps {
    pub common: ChartCommonProps,
    pub donut: bool,
    pub donut_width: u16,
    pub center_label: Option<String>,
    pub center_value: Option<String>,
    pub start_angle: i16,
    pub pad_angle: u16,
    pub hide_labels: bool,
    pub hide_values: bool,
    pub hide_percentages: bool,
    pub show_glow: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartSize {
    Sm,
    Md,
    Lg,
    Xl,
}

impl ChartSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }

    pub fn circular_height(self) -> ScaleValue {
        match self {
            Self::Sm => ScaleValue::from_half_steps(40),
            Self::Md => ScaleValue::from_half_steps(56),
            Self::Lg => ScaleValue::from_half_steps(75),
            Self::Xl => ScaleValue::from_half_steps(100),
        }
    }

    pub fn cartesian_height(self) -> ScaleValue {
        match self {
            Self::Sm => ScaleValue::from_half_steps(50),
            Self::Md => ScaleValue::from_half_steps(75),
            Self::Lg => ScaleValue::from_half_steps(100),
            Self::Xl => ScaleValue::from_half_steps(125),
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Sm, Self::Md, Self::Lg, Self::Xl]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartPalette {
    Default,
    Rainbow,
    Ocean,
    Sunset,
    Forest,
    Neon,
}

impl ChartPalette {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "default" => Some(Self::Default),
            "rainbow" => Some(Self::Rainbow),
            "ocean" => Some(Self::Ocean),
            "sunset" => Some(Self::Sunset),
            "forest" => Some(Self::Forest),
            "neon" => Some(Self::Neon),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Rainbow => "rainbow",
            Self::Ocean => "ocean",
            Self::Sunset => "sunset",
            Self::Forest => "forest",
            Self::Neon => "neon",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Default,
            Self::Rainbow,
            Self::Ocean,
            Self::Sunset,
            Self::Forest,
            Self::Neon,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartLegendPosition {
    Top,
    Right,
    Bottom,
    Left,
    None,
}

impl ChartLegendPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "top" => Some(Self::Top),
            "right" => Some(Self::Right),
            "bottom" => Some(Self::Bottom),
            "left" => Some(Self::Left),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::None => "none",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Top, Self::Right, Self::Bottom, Self::Left, Self::None]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartCurve {
    Linear,
    Smooth,
}

impl ChartCurve {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "linear" => Some(Self::Linear),
            "smooth" => Some(Self::Smooth),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Smooth => "smooth",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Linear, Self::Smooth]
    }
}

