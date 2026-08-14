#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ElementProps {
    pub id: Option<String>,
    pub font: Option<ResponsiveValue<FontFamily>>,
    pub bind: Option<String>,
    pub on_click: Option<String>,
    pub show: Option<VisibilityCondition>,
    pub form: Option<Box<FormControlValidation>>,
}

impl ElementProps {
    pub fn form_validation(&self) -> Option<&FormControlValidation> {
        self.form.as_deref()
    }

    pub fn form_validation_mut(&mut self) -> &mut FormControlValidation {
        self.form
            .get_or_insert_with(|| Box::new(FormControlValidation::default()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyleProps {
    pub element: ElementProps,
    pub font: Option<ResponsiveValue<FontFamily>>,
    pub bg: Option<ResponsiveValue<ColorToken>>,
    pub text: Option<ResponsiveValue<ColorToken>>,
    pub cover: Option<ResponsiveValue<CoverSource>>,
    pub overlay: Option<ResponsiveValue<OverlayPaint>>,
    pub background: Option<ResponsiveValue<SectionBackground>>,
    pub boxed: bool,
    pub extras: Option<Box<StyleExtras>>,
    pub spacing: SpacingProps,
    pub sizing: SizingProps,
    pub rounded: Option<ResponsiveValue<RoundedSize>>,
    pub border: Option<ResponsiveValue<BorderWidth>>,
    pub border_color: Option<ColorFamily>,
    pub shadow: Option<ResponsiveValue<ShadowSize>>,
    pub shadow_color: Option<ColorFamily>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyleExtras {
    pub motion: ViewMotionStyle,
    pub grid_item: GridItemProps,
    pub position: PositionProps,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewMotionStyle {
    pub animation: Option<ViewAnimation>,
    pub rotate: Option<ResponsiveValue<ViewRotation>>,
    pub scale: Option<ResponsiveValue<ViewScale>>,
    pub translate_x: Option<ResponsiveValue<ViewTranslation>>,
    pub translate_y: Option<ResponsiveValue<ViewTranslation>>,
    pub transition: Option<ViewTransition>,
    pub gesture: Option<ViewGesture>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoxPosition {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
}

impl BoxPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "static" => Some(Self::Static),
            "relative" => Some(Self::Relative),
            "absolute" => Some(Self::Absolute),
            "fixed" => Some(Self::Fixed),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Relative => "relative",
            Self::Absolute => "absolute",
            Self::Fixed => "fixed",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Static, Self::Relative, Self::Absolute, Self::Fixed]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PositionProps {
    pub mode: BoxPosition,
    pub top: Option<ResponsiveValue<ScaleValue>>,
    pub right: Option<ResponsiveValue<ScaleValue>>,
    pub bottom: Option<ResponsiveValue<ScaleValue>>,
    pub left: Option<ResponsiveValue<ScaleValue>>,
}

impl StyleProps {
    pub fn animation(&self) -> Option<ViewAnimation> {
        self.motion().animation
    }

    pub fn set_animation(&mut self, animation: Option<ViewAnimation>) {
        if animation.is_some() || self.extras.is_some() {
            self.motion_mut().animation = animation;
        }
    }

    pub fn motion(&self) -> &ViewMotionStyle {
        self.extras
            .as_deref()
            .map(|extras| &extras.motion)
            .unwrap_or(&ViewMotionStyle::EMPTY)
    }

    pub fn motion_mut(&mut self) -> &mut ViewMotionStyle {
        &mut self
            .extras
            .get_or_insert_with(|| Box::new(StyleExtras::default()))
            .motion
    }

    pub fn grid_item(&self) -> &GridItemProps {
        self.extras
            .as_deref()
            .map(|extras| &extras.grid_item)
            .unwrap_or(&GridItemProps::EMPTY)
    }

    pub fn grid_item_mut(&mut self) -> &mut GridItemProps {
        &mut self
            .extras
            .get_or_insert_with(|| Box::new(StyleExtras::default()))
            .grid_item
    }

    pub fn position(&self) -> &PositionProps {
        self.extras
            .as_deref()
            .map(|extras| &extras.position)
            .unwrap_or(&PositionProps::STATIC)
    }

    pub fn position_mut(&mut self) -> &mut PositionProps {
        &mut self
            .extras
            .get_or_insert_with(|| Box::new(StyleExtras::default()))
            .position
    }
}

impl ViewMotionStyle {
    pub const EMPTY: Self = Self {
        animation: None,
        rotate: None,
        scale: None,
        translate_x: None,
        translate_y: None,
        transition: None,
        gesture: None,
    };
}

impl GridItemProps {
    pub const EMPTY: Self = Self {
        col_span: None,
        row_span: None,
    };
}

impl PositionProps {
    pub const STATIC: Self = Self {
        mode: BoxPosition::Static,
        top: None,
        right: None,
        bottom: None,
        left: None,
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutProps {
    pub style: StyleProps,
    pub direction: ResponsiveValue<FlexDirection>,
    pub wrap: bool,
    pub justify: Option<ResponsiveValue<Justify>>,
    pub align: Option<ResponsiveValue<Align>>,
    pub gap: Option<ResponsiveValue<GapValue>>,
}

impl Default for LayoutProps {
    fn default() -> Self {
        Self {
            style: StyleProps::default(),
            direction: ResponsiveValue::scalar(FlexDirection::Row),
            wrap: false,
            justify: None,
            align: None,
            gap: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GridProps {
    pub style: StyleProps,
    pub columns: Option<ResponsiveValue<GridTracks>>,
    pub rows: Option<ResponsiveValue<GridTracks>>,
    pub justify: Option<ResponsiveValue<GridAlignment>>,
    pub align: Option<ResponsiveValue<GridAlignment>>,
    pub gap: Option<ResponsiveValue<GapValue>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VariantProps {
    pub element: ElementProps,
    pub style: StyleProps,
    pub variant: Option<ComponentVariant>,
    pub color: Option<ColorFamily>,
    pub size: Option<ButtonSize>,
    pub label: Option<String>,
    pub i18n: Option<String>,
    pub placeholder: Option<String>,
    pub label_floating: bool,
    pub icon_start: Option<SideNavIcon>,
    pub icon_end: Option<SideNavIcon>,
    pub loading_icon: Option<SideNavIcon>,
    pub icon_only: bool,
    pub navigation: Option<NavigationAction>,
    pub reactive: ReactiveVariantProps,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BrandProps {
    pub style: StyleProps,
    pub navigation: Option<NavigationAction>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerProps {
    pub style: StyleProps,
    pub navigation: NavigationAction,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReactiveVariantProps {
    pub variant: Option<String>,
    pub scheme: Option<String>,
    pub size: Option<String>,
    pub rounded: Option<String>,
    pub loading: Option<String>,
    pub disabled: Option<String>,
    pub icon_start_when: Option<String>,
    pub icon_end_when: Option<String>,
    pub icon_start_comparison: Option<ReactiveNumberComparison>,
    pub icon_end_comparison: Option<ReactiveNumberComparison>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactiveNumberComparison {
    pub operator: NumberComparisonOperator,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberComparisonOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl NumberComparisonOperator {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
        }
    }
}
