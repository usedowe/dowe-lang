use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorToken {
    Primary,
    OnPrimary,
    Secondary,
    OnSecondary,
    Tertiary,
    OnTertiary,
    Muted,
    OnMuted,
    Background,
    OnBackground,
    Surface,
    OnSurface,
    Success,
    OnSuccess,
    Info,
    OnInfo,
    Warning,
    OnWarning,
    Danger,
    OnDanger,
    SoftPrimary,
    OnSoftPrimary,
    SoftSecondary,
    OnSoftSecondary,
    SoftTertiary,
    OnSoftTertiary,
    SoftMuted,
    OnSoftMuted,
    SoftSuccess,
    OnSoftSuccess,
    SoftInfo,
    OnSoftInfo,
    SoftWarning,
    OnSoftWarning,
    SoftDanger,
    OnSoftDanger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignConfig {
    pub default_theme: String,
    pub themes: Vec<DesignTheme>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignTheme {
    pub name: String,
    pub colors: BTreeMap<ColorToken, String>,
    pub radii: DesignRadii,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesignRadii {
    pub radius: u16,
    pub radius_box: u16,
    pub radius_ui: u16,
}

impl Default for DesignConfig {
    fn default() -> Self {
        Self {
            default_theme: "light".to_string(),
            themes: vec![integrated_design_theme("light").expect("light design theme")],
        }
    }
}

impl DesignConfig {
    pub fn default_theme(&self) -> &DesignTheme {
        self.theme(&self.default_theme)
            .or_else(|| self.themes.first())
            .expect("design theme")
    }

    pub fn theme(&self, name: &str) -> Option<&DesignTheme> {
        self.themes.iter().find(|theme| theme.name == name)
    }
}

impl DesignTheme {
    pub fn color_value(&self, token: ColorToken) -> &str {
        self.colors
            .get(&token)
            .map(String::as_str)
            .expect("design color token")
    }
}

pub fn integrated_design_theme(name: &str) -> Option<DesignTheme> {
    match name {
        "light" => Some(theme_from_values(
            "light",
            &[
                (ColorToken::Primary, "#2563eb"),
                (ColorToken::OnPrimary, "#ffffff"),
                (ColorToken::Secondary, "#4f46e5"),
                (ColorToken::OnSecondary, "#ffffff"),
                (ColorToken::Tertiary, "#0f766e"),
                (ColorToken::OnTertiary, "#ffffff"),
                (ColorToken::Muted, "#64748b"),
                (ColorToken::OnMuted, "#ffffff"),
                (ColorToken::Background, "#ffffff"),
                (ColorToken::OnBackground, "#111827"),
                (ColorToken::Surface, "#f8fafc"),
                (ColorToken::OnSurface, "#111827"),
                (ColorToken::Success, "#16a34a"),
                (ColorToken::OnSuccess, "#ffffff"),
                (ColorToken::Info, "#0284c7"),
                (ColorToken::OnInfo, "#ffffff"),
                (ColorToken::Warning, "#d97706"),
                (ColorToken::OnWarning, "#111827"),
                (ColorToken::Danger, "#dc2626"),
                (ColorToken::OnDanger, "#ffffff"),
                (ColorToken::SoftPrimary, "#dbeafe"),
                (ColorToken::OnSoftPrimary, "#1e3a8a"),
                (ColorToken::SoftSecondary, "#e0e7ff"),
                (ColorToken::OnSoftSecondary, "#312e81"),
                (ColorToken::SoftTertiary, "#ccfbf1"),
                (ColorToken::OnSoftTertiary, "#134e4a"),
                (ColorToken::SoftMuted, "#e2e8f0"),
                (ColorToken::OnSoftMuted, "#334155"),
                (ColorToken::SoftSuccess, "#dcfce7"),
                (ColorToken::OnSoftSuccess, "#14532d"),
                (ColorToken::SoftInfo, "#e0f2fe"),
                (ColorToken::OnSoftInfo, "#075985"),
                (ColorToken::SoftWarning, "#fef3c7"),
                (ColorToken::OnSoftWarning, "#78350f"),
                (ColorToken::SoftDanger, "#fee2e2"),
                (ColorToken::OnSoftDanger, "#7f1d1d"),
            ],
            DesignRadii {
                radius: 8,
                radius_box: 12,
                radius_ui: 6,
            },
        )),
        "dark" => Some(theme_from_values(
            "dark",
            &[
                (ColorToken::Primary, "#93c5fd"),
                (ColorToken::OnPrimary, "#0f172a"),
                (ColorToken::Secondary, "#a5b4fc"),
                (ColorToken::OnSecondary, "#111827"),
                (ColorToken::Tertiary, "#5eead4"),
                (ColorToken::OnTertiary, "#042f2e"),
                (ColorToken::Muted, "#94a3b8"),
                (ColorToken::OnMuted, "#0f172a"),
                (ColorToken::Background, "#020617"),
                (ColorToken::OnBackground, "#f8fafc"),
                (ColorToken::Surface, "#0f172a"),
                (ColorToken::OnSurface, "#f8fafc"),
                (ColorToken::Success, "#4ade80"),
                (ColorToken::OnSuccess, "#052e16"),
                (ColorToken::Info, "#38bdf8"),
                (ColorToken::OnInfo, "#082f49"),
                (ColorToken::Warning, "#facc15"),
                (ColorToken::OnWarning, "#422006"),
                (ColorToken::Danger, "#f87171"),
                (ColorToken::OnDanger, "#450a0a"),
                (ColorToken::SoftPrimary, "#1e3a8a"),
                (ColorToken::OnSoftPrimary, "#dbeafe"),
                (ColorToken::SoftSecondary, "#312e81"),
                (ColorToken::OnSoftSecondary, "#e0e7ff"),
                (ColorToken::SoftTertiary, "#134e4a"),
                (ColorToken::OnSoftTertiary, "#ccfbf1"),
                (ColorToken::SoftMuted, "#334155"),
                (ColorToken::OnSoftMuted, "#e2e8f0"),
                (ColorToken::SoftSuccess, "#14532d"),
                (ColorToken::OnSoftSuccess, "#dcfce7"),
                (ColorToken::SoftInfo, "#075985"),
                (ColorToken::OnSoftInfo, "#e0f2fe"),
                (ColorToken::SoftWarning, "#78350f"),
                (ColorToken::OnSoftWarning, "#fef3c7"),
                (ColorToken::SoftDanger, "#7f1d1d"),
                (ColorToken::OnSoftDanger, "#fee2e2"),
            ],
            DesignRadii {
                radius: 8,
                radius_box: 12,
                radius_ui: 6,
            },
        )),
        _ => None,
    }
}

fn theme_from_values(
    name: &str,
    colors: &[(ColorToken, &str)],
    radii: DesignRadii,
) -> DesignTheme {
    DesignTheme {
        name: name.to_string(),
        colors: colors
            .iter()
            .map(|(token, value)| (*token, (*value).to_string()))
            .collect(),
        radii,
    }
}

impl ColorToken {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "primary" => Some(Self::Primary),
            "onPrimary" => Some(Self::OnPrimary),
            "secondary" => Some(Self::Secondary),
            "onSecondary" => Some(Self::OnSecondary),
            "tertiary" => Some(Self::Tertiary),
            "onTertiary" => Some(Self::OnTertiary),
            "muted" => Some(Self::Muted),
            "onMuted" => Some(Self::OnMuted),
            "background" => Some(Self::Background),
            "onBackground" => Some(Self::OnBackground),
            "surface" => Some(Self::Surface),
            "onSurface" => Some(Self::OnSurface),
            "success" => Some(Self::Success),
            "onSuccess" => Some(Self::OnSuccess),
            "info" => Some(Self::Info),
            "onInfo" => Some(Self::OnInfo),
            "warning" => Some(Self::Warning),
            "onWarning" => Some(Self::OnWarning),
            "danger" => Some(Self::Danger),
            "onDanger" => Some(Self::OnDanger),
            "softPrimary" => Some(Self::SoftPrimary),
            "onSoftPrimary" => Some(Self::OnSoftPrimary),
            "softSecondary" => Some(Self::SoftSecondary),
            "onSoftSecondary" => Some(Self::OnSoftSecondary),
            "softTertiary" => Some(Self::SoftTertiary),
            "onSoftTertiary" => Some(Self::OnSoftTertiary),
            "softMuted" => Some(Self::SoftMuted),
            "onSoftMuted" => Some(Self::OnSoftMuted),
            "softSuccess" => Some(Self::SoftSuccess),
            "onSoftSuccess" => Some(Self::OnSoftSuccess),
            "softInfo" => Some(Self::SoftInfo),
            "onSoftInfo" => Some(Self::OnSoftInfo),
            "softWarning" => Some(Self::SoftWarning),
            "onSoftWarning" => Some(Self::OnSoftWarning),
            "softDanger" => Some(Self::SoftDanger),
            "onSoftDanger" => Some(Self::OnSoftDanger),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::OnPrimary => "onPrimary",
            Self::Secondary => "secondary",
            Self::OnSecondary => "onSecondary",
            Self::Tertiary => "tertiary",
            Self::OnTertiary => "onTertiary",
            Self::Muted => "muted",
            Self::OnMuted => "onMuted",
            Self::Background => "background",
            Self::OnBackground => "onBackground",
            Self::Surface => "surface",
            Self::OnSurface => "onSurface",
            Self::Success => "success",
            Self::OnSuccess => "onSuccess",
            Self::Info => "info",
            Self::OnInfo => "onInfo",
            Self::Warning => "warning",
            Self::OnWarning => "onWarning",
            Self::Danger => "danger",
            Self::OnDanger => "onDanger",
            Self::SoftPrimary => "softPrimary",
            Self::OnSoftPrimary => "onSoftPrimary",
            Self::SoftSecondary => "softSecondary",
            Self::OnSoftSecondary => "onSoftSecondary",
            Self::SoftTertiary => "softTertiary",
            Self::OnSoftTertiary => "onSoftTertiary",
            Self::SoftMuted => "softMuted",
            Self::OnSoftMuted => "onSoftMuted",
            Self::SoftSuccess => "softSuccess",
            Self::OnSoftSuccess => "onSoftSuccess",
            Self::SoftInfo => "softInfo",
            Self::OnSoftInfo => "onSoftInfo",
            Self::SoftWarning => "softWarning",
            Self::OnSoftWarning => "onSoftWarning",
            Self::SoftDanger => "softDanger",
            Self::OnSoftDanger => "onSoftDanger",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Primary,
            Self::OnPrimary,
            Self::Secondary,
            Self::OnSecondary,
            Self::Tertiary,
            Self::OnTertiary,
            Self::Muted,
            Self::OnMuted,
            Self::Background,
            Self::OnBackground,
            Self::Surface,
            Self::OnSurface,
            Self::Success,
            Self::OnSuccess,
            Self::Info,
            Self::OnInfo,
            Self::Warning,
            Self::OnWarning,
            Self::Danger,
            Self::OnDanger,
            Self::SoftPrimary,
            Self::OnSoftPrimary,
            Self::SoftSecondary,
            Self::OnSoftSecondary,
            Self::SoftTertiary,
            Self::OnSoftTertiary,
            Self::SoftMuted,
            Self::OnSoftMuted,
            Self::SoftSuccess,
            Self::OnSoftSuccess,
            Self::SoftInfo,
            Self::OnSoftInfo,
            Self::SoftWarning,
            Self::OnSoftWarning,
            Self::SoftDanger,
            Self::OnSoftDanger,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFamily {
    Primary,
    Secondary,
    Tertiary,
    Muted,
    Background,
    Surface,
    Success,
    Info,
    Warning,
    Danger,
}

impl ColorFamily {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "primary" => Some(Self::Primary),
            "secondary" => Some(Self::Secondary),
            "tertiary" => Some(Self::Tertiary),
            "muted" => Some(Self::Muted),
            "background" => Some(Self::Background),
            "surface" => Some(Self::Surface),
            "success" => Some(Self::Success),
            "info" => Some(Self::Info),
            "warning" => Some(Self::Warning),
            "danger" => Some(Self::Danger),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Tertiary => "tertiary",
            Self::Muted => "muted",
            Self::Background => "background",
            Self::Surface => "surface",
            Self::Success => "success",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Danger => "danger",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Primary,
            Self::Secondary,
            Self::Tertiary,
            Self::Muted,
            Self::Background,
            Self::Surface,
            Self::Success,
            Self::Info,
            Self::Warning,
            Self::Danger,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FontFamily {
    System,
    Inter,
    Roboto,
    Montserrat,
    Lato,
    Poppins,
    Manrope,
    Quicksand,
    Lora,
}

impl FontFamily {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "inter" => Some(Self::Inter),
            "roboto" => Some(Self::Roboto),
            "montserrat" => Some(Self::Montserrat),
            "lato" => Some(Self::Lato),
            "poppins" => Some(Self::Poppins),
            "manrope" => Some(Self::Manrope),
            "quicksand" => Some(Self::Quicksand),
            "lora" => Some(Self::Lora),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Inter => "inter",
            Self::Roboto => "roboto",
            Self::Montserrat => "montserrat",
            Self::Lato => "lato",
            Self::Poppins => "poppins",
            Self::Manrope => "manrope",
            Self::Quicksand => "quicksand",
            Self::Lora => "lora",
        }
    }

    pub fn display_name(self) -> &'static str {
        self.catalog_entry().display_name
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::System,
            Self::Inter,
            Self::Roboto,
            Self::Montserrat,
            Self::Lato,
            Self::Poppins,
            Self::Manrope,
            Self::Quicksand,
            Self::Lora,
        ]
    }

    pub fn catalog_entry(self) -> &'static FontCatalogEntry {
        FONT_CATALOG
            .iter()
            .find(|entry| entry.token == self)
            .expect("font catalog entry")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentVariant {
    Solid,
    Soft,
    Outlined,
    Ghost,
}

impl ComponentVariant {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "solid" => Some(Self::Solid),
            "soft" => Some(Self::Soft),
            "outlined" => Some(Self::Outlined),
            "ghost" => Some(Self::Ghost),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Soft => "soft",
            Self::Outlined => "outlined",
            Self::Ghost => "ghost",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Solid, Self::Soft, Self::Outlined, Self::Ghost]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl ButtonSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "xs" => Some(Self::Xs),
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
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
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Xs, Self::Sm, Self::Md, Self::Lg, Self::Xl]
    }

    pub fn padding_x(self) -> ScaleValue {
        match self {
            Self::Xs => ScaleValue::from_half_steps(5),
            Self::Sm => ScaleValue::from_half_steps(6),
            Self::Md => ScaleValue::from_half_steps(8),
            Self::Lg => ScaleValue::from_half_steps(10),
            Self::Xl => ScaleValue::from_half_steps(12),
        }
    }

    pub fn padding_y(self) -> ScaleValue {
        match self {
            Self::Xs => ScaleValue::from_half_steps(3),
            Self::Sm => ScaleValue::from_half_steps(4),
            Self::Md => ScaleValue::from_half_steps(5),
            Self::Lg => ScaleValue::from_half_steps(6),
            Self::Xl => ScaleValue::from_half_steps(7),
        }
    }

    pub fn min_height(self) -> ScaleValue {
        match self {
            Self::Xs => ScaleValue::from_half_steps(14),
            Self::Sm => ScaleValue::from_half_steps(16),
            Self::Md => ScaleValue::from_half_steps(20),
            Self::Lg => ScaleValue::from_half_steps(22),
            Self::Xl => ScaleValue::from_half_steps(24),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideNavSize {
    Sm,
    Md,
    Lg,
}

impl SideNavSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Sm, Self::Md, Self::Lg]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableSize {
    Sm,
    Md,
    Lg,
}

impl TableSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Sm, Self::Md, Self::Lg]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableColumnAlign {
    Start,
    Center,
    End,
}

impl TableColumnAlign {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "center" => Some(Self::Center),
            "end" => Some(Self::End),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Start, Self::Center, Self::End]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerPosition {
    Start,
    End,
    Top,
    Bottom,
}

impl DrawerPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Start, Self::End, Self::Top, Self::Bottom]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleValue(pub u16);

impl ScaleValue {
    pub const fn from_half_steps(value: u16) -> Self {
        Self(value)
    }

    pub fn class_suffix(self) -> String {
        if self.0 % 2 == 0 {
            (self.0 / 2).to_string()
        } else {
            format!("{}.5", self.0 / 2)
        }
    }

    pub fn native_units(self) -> u16 {
        self.0 * 2
    }
}
