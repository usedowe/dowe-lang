#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyProps {
    pub style: VariantProps,
    pub kind: EmptyKind,
    pub title: Option<String>,
    pub description: Option<String>,
    pub action_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarqueeProps {
    pub style: StyleProps,
    pub speed: MarqueeSpeed,
    pub pause_on_hover: bool,
    pub reverse: bool,
    pub orientation: MarqueeOrientation,
    pub fade: bool,
    pub fade_color: ColorToken,
    pub gap: ScaleValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeWriterProps {
    pub style: StyleProps,
    pub type_speed: u64,
    pub delete_speed: u64,
    pub after_typed: u64,
    pub after_deleted: u64,
    pub repeat: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeWriterItem {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichTextMark {
    pub text: String,
    pub style: RichTextMarkStyle,
    pub color: ColorFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RichTextMarkStyle {
    Mark,
    Grad,
    Pill,
    Slant,
    Glow,
    Under,
    Strike,
    Box,
    Wave,
    Neon,
    Pop,
    Tag,
}

impl RichTextMarkStyle {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "mark" => Some(Self::Mark),
            "grad" => Some(Self::Grad),
            "pill" => Some(Self::Pill),
            "slant" => Some(Self::Slant),
            "glow" => Some(Self::Glow),
            "under" => Some(Self::Under),
            "strike" => Some(Self::Strike),
            "box" => Some(Self::Box),
            "wave" => Some(Self::Wave),
            "neon" => Some(Self::Neon),
            "pop" => Some(Self::Pop),
            "tag" => Some(Self::Tag),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mark => "mark",
            Self::Grad => "grad",
            Self::Pill => "pill",
            Self::Slant => "slant",
            Self::Glow => "glow",
            Self::Under => "under",
            Self::Strike => "strike",
            Self::Box => "box",
            Self::Wave => "wave",
            Self::Neon => "neon",
            Self::Pop => "pop",
            Self::Tag => "tag",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Mark,
            Self::Grad,
            Self::Pill,
            Self::Slant,
            Self::Glow,
            Self::Under,
            Self::Strike,
            Self::Box,
            Self::Wave,
            Self::Neon,
            Self::Pop,
            Self::Tag,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordProps {
    pub style: VariantProps,
    pub name: String,
    pub url: Option<String>,
    pub disabled: bool,
    pub max_duration: Option<u16>,
    pub on_start: Option<String>,
    pub on_pause: Option<String>,
    pub on_resume: Option<String>,
    pub on_stop: Option<String>,
    pub on_discard: Option<String>,
    pub on_confirm: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleGroupProps {
    pub style: VariantProps,
    pub kind: ToggleGroupKind,
    pub pagination: Option<PaginationProps>,
    pub value: Option<String>,
    pub selected: String,
    pub multiple: bool,
    pub size: ButtonSize,
    pub wide: bool,
    pub vertical: bool,
    pub disabled: bool,
    pub aria_label: Option<String>,
    pub on_change: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationProps {
    pub total: PaginationTotal,
    pub page_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaginationTotal {
    Static(u32),
    Signal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleGroupKind {
    #[default]
    Selection,
    Pagination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleGroupItem {
    pub id: String,
    pub label: String,
    pub icon: Option<ViewIcon>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollapsibleProps {
    pub style: VariantProps,
    pub label: String,
    pub default_open: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountdownProps {
    pub style: VariantProps,
    pub target: String,
    pub show_days: bool,
    pub show_hours: bool,
    pub show_minutes: bool,
    pub show_seconds: bool,
    pub size: CountdownSize,
    pub days_label: String,
    pub hours_label: String,
    pub minutes_label: String,
    pub seconds_label: String,
    pub on_complete: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountdownSize {
    Sm,
    Md,
    Lg,
    Xl,
}

impl CountdownSize {
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

    pub fn all() -> &'static [Self] {
        &[Self::Sm, Self::Md, Self::Lg, Self::Xl]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapProps {
    pub style: VariantProps,
    pub center_lat: String,
    pub center_lng: String,
    pub zoom: u16,
    pub height: String,
    pub width: String,
    pub show_controls: bool,
    pub show_scale: bool,
    pub show_location_control: bool,
    pub interactive: bool,
    pub route_start_lat: Option<String>,
    pub route_start_lng: Option<String>,
    pub route_end_lat: Option<String>,
    pub route_end_lng: Option<String>,
    pub on_location: Option<String>,
    pub on_location_error: Option<String>,
    pub on_route: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapMarker {
    pub id: String,
    pub lat: String,
    pub lng: String,
    pub label: Option<String>,
    pub popup: Option<String>,
    pub icon: MapMarkerIcon,
    pub on_click: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapWaypoint {
    pub lat: String,
    pub lng: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapMarkerIcon {
    Default,
    Start,
    End,
    Waypoint,
}

impl MapMarkerIcon {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "default" => Some(Self::Default),
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            "waypoint" => Some(Self::Waypoint),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Start => "start",
            Self::End => "end",
            Self::Waypoint => "waypoint",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioProps {
    pub style: VariantProps,
    pub src: String,
    pub subtitle: Option<String>,
    pub avatar_src: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageProps {
    pub style: VariantProps,
    pub src: String,
    pub reactive_src: Option<String>,
    pub alt: String,
    pub aspect: ImageAspect,
    pub object_fit: ImageObjectFit,
    pub loading: ImageLoading,
    pub hide_controls: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccordionProps {
    pub style: VariantProps,
    pub multiple: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccordionItem {
    pub id: String,
    pub label: String,
    pub disabled: bool,
    pub default_open: bool,
    pub children: Vec<ViewNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarouselProps {
    pub style: VariantProps,
    pub variant: CarouselVariant,
    pub autoplay: bool,
    pub autoplay_interval: u16,
    pub disable_loop: bool,
    pub hide_controls: bool,
    pub hide_indicators: bool,
    pub show_navigation: bool,
    pub show_counter: bool,
    pub orientation: CarouselOrientation,
    pub size: ButtonSize,
    pub indicator_type: CarouselIndicatorType,
    pub title: Option<String>,
    pub slide_width: Option<u16>,
    pub slide_height: Option<u16>,
    pub slides_per_view: u16,
    pub gap: u16,
}

impl CarouselProps {
    pub fn shows_controls(&self) -> bool {
        !self.hide_controls || self.variant == CarouselVariant::Controls
    }

    pub fn shows_indicators(&self) -> bool {
        !self.hide_indicators
    }

    pub fn has_variant_indicators(&self) -> bool {
        matches!(self.variant, CarouselVariant::Dots | CarouselVariant::Thumbnails)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarouselSlide {
    pub id: String,
    pub children: Vec<ViewNode>,
}
