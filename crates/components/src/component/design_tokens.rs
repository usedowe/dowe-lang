use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::{Mutex, OnceLock};

#[derive(Default)]
struct ColorIdentifierRegistry {
    ids: HashMap<&'static str, u16>,
    names: Vec<&'static str>,
}

fn color_identifier_registry() -> &'static Mutex<ColorIdentifierRegistry> {
    static IDENTIFIERS: OnceLock<Mutex<ColorIdentifierRegistry>> = OnceLock::new();
    IDENTIFIERS.get_or_init(|| Mutex::new(ColorIdentifierRegistry::default()))
}

fn intern_color_identifier(value: &str) -> Option<u16> {
    let mut registry = color_identifier_registry()
        .lock()
        .expect("color identifier registry");
    if let Some(id) = registry.ids.get(value) {
        return Some(*id);
    }
    let id = u16::try_from(registry.names.len()).ok()?;
    let value = Box::leak(value.to_string().into_boxed_str());
    registry.ids.insert(value, id);
    registry.names.push(value);
    Some(id)
}

fn color_identifier(id: u16) -> &'static str {
    color_identifier_registry()
        .lock()
        .expect("color identifier registry")
        .names
        .get(usize::from(id))
        .copied()
        .expect("registered color identifier")
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ColorToken(u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignConfig {
    pub default_theme: String,
    pub themes: Vec<DesignTheme>,
    pub defaults: DesignDefaults,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignTheme {
    pub name: String,
    pub colors: BTreeMap<ColorToken, String>,
    pub radius: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignDefaults {
    pub font: BTreeMap<DesignComponentSlot, FontFamily>,
    pub radius: BTreeMap<DesignComponentSlot, RoundedSize>,
    pub shadow: BTreeMap<DesignComponentSlot, ShadowSize>,
    pub shadow_color: BTreeMap<DesignComponentSlot, ColorFamily>,
    pub border: BTreeMap<DesignComponentSlot, BorderWidth>,
    pub border_color: BTreeMap<DesignComponentSlot, ColorFamily>,
    pub scheme: BTreeMap<DesignComponentSlot, ColorFamily>,
    pub variant: BTreeMap<DesignComponentSlot, ComponentVariant>,
    pub tabs_variant: BTreeMap<DesignComponentSlot, TabsVariant>,
    pub size: BTreeMap<DesignComponentSlot, ButtonSize>,
}

impl Default for DesignDefaults {
    fn default() -> Self {
        Self::with_builtin_defaults()
    }
}

impl DesignDefaults {
    pub fn empty() -> Self {
        Self {
            font: BTreeMap::new(),
            radius: BTreeMap::new(),
            shadow: BTreeMap::new(),
            shadow_color: BTreeMap::new(),
            border: BTreeMap::new(),
            border_color: BTreeMap::new(),
            scheme: BTreeMap::new(),
            variant: BTreeMap::new(),
            tabs_variant: BTreeMap::new(),
            size: BTreeMap::new(),
        }
    }

    pub fn with_builtin_defaults() -> Self {
        let mut defaults = Self::empty();

        for (slot, scheme) in [
            (DesignComponentSlot::Button, ColorFamily::Primary),
            (DesignComponentSlot::IconButton, ColorFamily::Primary),
            (DesignComponentSlot::Card, ColorFamily::Surface),
            (DesignComponentSlot::Drawer, ColorFamily::Surface),
            (DesignComponentSlot::Toast, ColorFamily::Info),
            (DesignComponentSlot::Section, ColorFamily::Background),
            (DesignComponentSlot::Checkbox, ColorFamily::Primary),
            (DesignComponentSlot::Input, ColorFamily::Primary),
            (DesignComponentSlot::Date, ColorFamily::Primary),
            (DesignComponentSlot::Password, ColorFamily::Primary),
            (DesignComponentSlot::Select, ColorFamily::Primary),
            (DesignComponentSlot::Pin, ColorFamily::Primary),
            (DesignComponentSlot::AppBar, ColorFamily::Surface),
            (DesignComponentSlot::Footer, ColorFamily::Surface),
            (DesignComponentSlot::Modal, ColorFamily::Surface),
            (DesignComponentSlot::Dropdown, ColorFamily::Surface),
            (DesignComponentSlot::Tooltip, ColorFamily::Surface),
            (DesignComponentSlot::Tabs, ColorFamily::Primary),
        ] {
            defaults.scheme.insert(slot, scheme);
        }

        for (slot, variant) in [
            (DesignComponentSlot::Button, ComponentVariant::Solid),
            (DesignComponentSlot::IconButton, ComponentVariant::Solid),
            (DesignComponentSlot::Card, ComponentVariant::Solid),
            (DesignComponentSlot::Drawer, ComponentVariant::Solid),
            (DesignComponentSlot::Toast, ComponentVariant::Solid),
            (DesignComponentSlot::Section, ComponentVariant::Solid),
            (DesignComponentSlot::Accordion, ComponentVariant::Ghost),
            (DesignComponentSlot::Input, ComponentVariant::Outlined),
            (DesignComponentSlot::Date, ComponentVariant::Outlined),
            (DesignComponentSlot::Password, ComponentVariant::Outlined),
            (DesignComponentSlot::Select, ComponentVariant::Outlined),
            (DesignComponentSlot::Pin, ComponentVariant::Outlined),
            (DesignComponentSlot::AppBar, ComponentVariant::Solid),
            (DesignComponentSlot::Footer, ComponentVariant::Solid),
            (DesignComponentSlot::Modal, ComponentVariant::Solid),
            (DesignComponentSlot::Dropdown, ComponentVariant::Solid),
            (DesignComponentSlot::Tooltip, ComponentVariant::Solid),
        ] {
            defaults.variant.insert(slot, variant);
        }

        defaults
            .tabs_variant
            .insert(DesignComponentSlot::Tabs, TabsVariant::Pills);

        for slot in [
            DesignComponentSlot::Button,
            DesignComponentSlot::IconButton,
            DesignComponentSlot::Card,
            DesignComponentSlot::Toast,
        ] {
            defaults.radius.insert(slot, RoundedSize::Md);
        }

        defaults
    }

    pub fn with_builtin_overrides(configured: Self) -> Self {
        let mut defaults = Self::with_builtin_defaults();
        inherit_configured_ui(&mut defaults.radius, &configured.radius);
        inherit_configured_ui(&mut defaults.scheme, &configured.scheme);
        inherit_configured_ui(&mut defaults.variant, &configured.variant);
        inherit_configured_ui(&mut defaults.tabs_variant, &configured.tabs_variant);
        defaults.font.extend(configured.font);
        defaults.radius.extend(configured.radius);
        defaults.shadow.extend(configured.shadow);
        defaults.shadow_color.extend(configured.shadow_color);
        defaults.border.extend(configured.border);
        defaults.border_color.extend(configured.border_color);
        defaults.scheme.extend(configured.scheme);
        defaults.variant.extend(configured.variant);
        defaults.tabs_variant.extend(configured.tabs_variant);
        defaults.size.extend(configured.size);
        defaults
    }
}

fn inherit_configured_ui<T: Copy>(
    defaults: &mut BTreeMap<DesignComponentSlot, T>,
    configured: &BTreeMap<DesignComponentSlot, T>,
) {
    let Some(value) = configured.get(&DesignComponentSlot::Ui).copied() else {
        return;
    };
    for slot in DesignComponentSlot::all() {
        if defaults.contains_key(slot) && !configured.contains_key(slot) {
            defaults.insert(*slot, value);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DesignComponentSlot {
    Card,
    Button,
    IconButton,
    Drawer,
    Toast,
    Section,
    Accordion,
    Checkbox,
    Input,
    Date,
    Password,
    Select,
    Pin,
    AppBar,
    Footer,
    Modal,
    Dropdown,
    Tooltip,
    Tabs,
    Chip,
    Avatar,
    Text,
    Title,
    Ui,
}

impl Default for DesignConfig {
    fn default() -> Self {
        Self {
            default_theme: "light".to_string(),
            themes: vec![integrated_design_theme("light").expect("light design theme")],
            defaults: DesignDefaults::with_builtin_defaults(),
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

    pub fn ordered_color_tokens(&self) -> Vec<ColorToken> {
        ColorToken::all()
            .iter()
            .copied()
            .chain(
                self.colors
                    .keys()
                    .copied()
                    .filter(|token| !token.is_builtin()),
            )
            .collect()
    }

    pub fn contains_color_family(&self, family: ColorFamily, soft: bool) -> bool {
        family
            .theme_tokens(soft)
            .is_some_and(|tokens| tokens.into_iter().all(|token| self.colors.contains_key(&token)))
    }

    pub fn contains_color_token(&self, token: ColorToken) -> bool {
        self.colors.contains_key(&token)
    }
}

impl DesignComponentSlot {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "Card" => Some(Self::Card),
            "Button" => Some(Self::Button),
            "IconButton" => Some(Self::IconButton),
            "Drawer" => Some(Self::Drawer),
            "Toast" => Some(Self::Toast),
            "Section" => Some(Self::Section),
            "Accordion" => Some(Self::Accordion),
            "Checkbox" => Some(Self::Checkbox),
            "Input" => Some(Self::Input),
            "Date" => Some(Self::Date),
            "Password" => Some(Self::Password),
            "Select" => Some(Self::Select),
            "Pin" => Some(Self::Pin),
            "AppBar" => Some(Self::AppBar),
            "Footer" => Some(Self::Footer),
            "Modal" => Some(Self::Modal),
            "Dropdown" => Some(Self::Dropdown),
            "Tooltip" => Some(Self::Tooltip),
            "Tabs" => Some(Self::Tabs),
            "Chip" => Some(Self::Chip),
            "Avatar" => Some(Self::Avatar),
            "Text" => Some(Self::Text),
            "Title" => Some(Self::Title),
            "Ui" => Some(Self::Ui),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Card => "Card",
            Self::Button => "Button",
            Self::IconButton => "IconButton",
            Self::Drawer => "Drawer",
            Self::Toast => "Toast",
            Self::Section => "Section",
            Self::Accordion => "Accordion",
            Self::Checkbox => "Checkbox",
            Self::Input => "Input",
            Self::Date => "Date",
            Self::Password => "Password",
            Self::Select => "Select",
            Self::Pin => "Pin",
            Self::AppBar => "AppBar",
            Self::Footer => "Footer",
            Self::Modal => "Modal",
            Self::Dropdown => "Dropdown",
            Self::Tooltip => "Tooltip",
            Self::Tabs => "Tabs",
            Self::Chip => "Chip",
            Self::Avatar => "Avatar",
            Self::Text => "Text",
            Self::Title => "Title",
            Self::Ui => "Ui",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Card,
            Self::Button,
            Self::IconButton,
            Self::Drawer,
            Self::Toast,
            Self::Section,
            Self::Accordion,
            Self::Checkbox,
            Self::Input,
            Self::Date,
            Self::Password,
            Self::Select,
            Self::Pin,
            Self::AppBar,
            Self::Footer,
            Self::Modal,
            Self::Dropdown,
            Self::Tooltip,
            Self::Tabs,
            Self::Chip,
            Self::Avatar,
            Self::Text,
            Self::Title,
            Self::Ui,
        ]
    }
}

pub fn integrated_design_theme(name: &str) -> Option<DesignTheme> {
    match name {
        "light" => Some(theme_from_values(
            "light",
            &[
                (ColorToken::Primary, "#2563eb"),
                (ColorToken::PrimaryText, "#ffffff"),
                (ColorToken::PrimaryTitle, "#ffffff"),
                (ColorToken::Secondary, "#4f46e5"),
                (ColorToken::SecondaryText, "#ffffff"),
                (ColorToken::SecondaryTitle, "#ffffff"),
                (ColorToken::Tertiary, "#0f766e"),
                (ColorToken::TertiaryText, "#ffffff"),
                (ColorToken::TertiaryTitle, "#ffffff"),
                (ColorToken::Muted, "#64748b"),
                (ColorToken::MutedText, "#ffffff"),
                (ColorToken::MutedTitle, "#ffffff"),
                (ColorToken::Background, "#ffffff"),
                (ColorToken::BackgroundText, "#111827"),
                (ColorToken::BackgroundTitle, "#111827"),
                (ColorToken::Surface, "#f8fafc"),
                (ColorToken::SurfaceText, "#111827"),
                (ColorToken::SurfaceTitle, "#111827"),
                (ColorToken::Success, "#16a34a"),
                (ColorToken::SuccessText, "#ffffff"),
                (ColorToken::SuccessTitle, "#ffffff"),
                (ColorToken::Info, "#0284c7"),
                (ColorToken::InfoText, "#ffffff"),
                (ColorToken::InfoTitle, "#ffffff"),
                (ColorToken::Warning, "#d97706"),
                (ColorToken::WarningText, "#111827"),
                (ColorToken::WarningTitle, "#111827"),
                (ColorToken::Danger, "#dc2626"),
                (ColorToken::DangerText, "#ffffff"),
                (ColorToken::DangerTitle, "#ffffff"),
                (ColorToken::SoftPrimary, "#dbeafe"),
                (ColorToken::SoftPrimaryText, "#1e3a8a"),
                (ColorToken::SoftPrimaryTitle, "#1e3a8a"),
                (ColorToken::SoftSecondary, "#e0e7ff"),
                (ColorToken::SoftSecondaryText, "#312e81"),
                (ColorToken::SoftSecondaryTitle, "#312e81"),
                (ColorToken::SoftTertiary, "#ccfbf1"),
                (ColorToken::SoftTertiaryText, "#134e4a"),
                (ColorToken::SoftTertiaryTitle, "#134e4a"),
                (ColorToken::SoftMuted, "#e2e8f0"),
                (ColorToken::SoftMutedText, "#334155"),
                (ColorToken::SoftMutedTitle, "#334155"),
                (ColorToken::SoftSuccess, "#dcfce7"),
                (ColorToken::SoftSuccessText, "#14532d"),
                (ColorToken::SoftSuccessTitle, "#14532d"),
                (ColorToken::SoftInfo, "#e0f2fe"),
                (ColorToken::SoftInfoText, "#075985"),
                (ColorToken::SoftInfoTitle, "#075985"),
                (ColorToken::SoftWarning, "#fef3c7"),
                (ColorToken::SoftWarningText, "#78350f"),
                (ColorToken::SoftWarningTitle, "#78350f"),
                (ColorToken::SoftDanger, "#fee2e2"),
                (ColorToken::SoftDangerText, "#7f1d1d"),
                (ColorToken::SoftDangerTitle, "#7f1d1d"),
            ],
            8,
        )),
        "dark" => Some(theme_from_values(
            "dark",
            &[
                (ColorToken::Primary, "#93c5fd"),
                (ColorToken::PrimaryText, "#0f172a"),
                (ColorToken::PrimaryTitle, "#0f172a"),
                (ColorToken::Secondary, "#a5b4fc"),
                (ColorToken::SecondaryText, "#111827"),
                (ColorToken::SecondaryTitle, "#111827"),
                (ColorToken::Tertiary, "#5eead4"),
                (ColorToken::TertiaryText, "#042f2e"),
                (ColorToken::TertiaryTitle, "#042f2e"),
                (ColorToken::Muted, "#94a3b8"),
                (ColorToken::MutedText, "#0f172a"),
                (ColorToken::MutedTitle, "#0f172a"),
                (ColorToken::Background, "#020617"),
                (ColorToken::BackgroundText, "#f8fafc"),
                (ColorToken::BackgroundTitle, "#f8fafc"),
                (ColorToken::Surface, "#0f172a"),
                (ColorToken::SurfaceText, "#f8fafc"),
                (ColorToken::SurfaceTitle, "#f8fafc"),
                (ColorToken::Success, "#4ade80"),
                (ColorToken::SuccessText, "#052e16"),
                (ColorToken::SuccessTitle, "#052e16"),
                (ColorToken::Info, "#38bdf8"),
                (ColorToken::InfoText, "#082f49"),
                (ColorToken::InfoTitle, "#082f49"),
                (ColorToken::Warning, "#facc15"),
                (ColorToken::WarningText, "#422006"),
                (ColorToken::WarningTitle, "#422006"),
                (ColorToken::Danger, "#f87171"),
                (ColorToken::DangerText, "#450a0a"),
                (ColorToken::DangerTitle, "#450a0a"),
                (ColorToken::SoftPrimary, "#1e3a8a"),
                (ColorToken::SoftPrimaryText, "#dbeafe"),
                (ColorToken::SoftPrimaryTitle, "#dbeafe"),
                (ColorToken::SoftSecondary, "#312e81"),
                (ColorToken::SoftSecondaryText, "#e0e7ff"),
                (ColorToken::SoftSecondaryTitle, "#e0e7ff"),
                (ColorToken::SoftTertiary, "#134e4a"),
                (ColorToken::SoftTertiaryText, "#ccfbf1"),
                (ColorToken::SoftTertiaryTitle, "#ccfbf1"),
                (ColorToken::SoftMuted, "#334155"),
                (ColorToken::SoftMutedText, "#e2e8f0"),
                (ColorToken::SoftMutedTitle, "#e2e8f0"),
                (ColorToken::SoftSuccess, "#14532d"),
                (ColorToken::SoftSuccessText, "#dcfce7"),
                (ColorToken::SoftSuccessTitle, "#dcfce7"),
                (ColorToken::SoftInfo, "#075985"),
                (ColorToken::SoftInfoText, "#e0f2fe"),
                (ColorToken::SoftInfoTitle, "#e0f2fe"),
                (ColorToken::SoftWarning, "#78350f"),
                (ColorToken::SoftWarningText, "#fef3c7"),
                (ColorToken::SoftWarningTitle, "#fef3c7"),
                (ColorToken::SoftDanger, "#7f1d1d"),
                (ColorToken::SoftDangerText, "#fee2e2"),
                (ColorToken::SoftDangerTitle, "#fee2e2"),
            ],
            8,
        )),
        _ => None,
    }
}

fn theme_from_values(
    name: &str,
    colors: &[(ColorToken, &str)],
    radius: u16,
) -> DesignTheme {
    DesignTheme {
        name: name.to_string(),
        colors: colors
            .iter()
            .map(|(token, value)| (*token, (*value).to_string()))
            .collect(),
        radius,
    }
}

impl fmt::Debug for ColorToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl PartialOrd for ColorToken {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ColorToken {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_builtin() || other.is_builtin() {
            return self.0.cmp(&other.0);
        }
        self.as_str().cmp(other.as_str())
    }
}

#[allow(non_upper_case_globals)]
impl ColorToken {
    const CUSTOM_OFFSET: u16 = 54;
    pub const Primary: Self = Self(0);
    pub const PrimaryText: Self = Self(1);
    pub const PrimaryTitle: Self = Self(2);
    pub const Secondary: Self = Self(3);
    pub const SecondaryText: Self = Self(4);
    pub const SecondaryTitle: Self = Self(5);
    pub const Tertiary: Self = Self(6);
    pub const TertiaryText: Self = Self(7);
    pub const TertiaryTitle: Self = Self(8);
    pub const Muted: Self = Self(9);
    pub const MutedText: Self = Self(10);
    pub const MutedTitle: Self = Self(11);
    pub const Background: Self = Self(12);
    pub const BackgroundText: Self = Self(13);
    pub const BackgroundTitle: Self = Self(14);
    pub const Surface: Self = Self(15);
    pub const SurfaceText: Self = Self(16);
    pub const SurfaceTitle: Self = Self(17);
    pub const Success: Self = Self(18);
    pub const SuccessText: Self = Self(19);
    pub const SuccessTitle: Self = Self(20);
    pub const Info: Self = Self(21);
    pub const InfoText: Self = Self(22);
    pub const InfoTitle: Self = Self(23);
    pub const Warning: Self = Self(24);
    pub const WarningText: Self = Self(25);
    pub const WarningTitle: Self = Self(26);
    pub const Danger: Self = Self(27);
    pub const DangerText: Self = Self(28);
    pub const DangerTitle: Self = Self(29);
    pub const SoftPrimary: Self = Self(30);
    pub const SoftPrimaryText: Self = Self(31);
    pub const SoftPrimaryTitle: Self = Self(32);
    pub const SoftSecondary: Self = Self(33);
    pub const SoftSecondaryText: Self = Self(34);
    pub const SoftSecondaryTitle: Self = Self(35);
    pub const SoftTertiary: Self = Self(36);
    pub const SoftTertiaryText: Self = Self(37);
    pub const SoftTertiaryTitle: Self = Self(38);
    pub const SoftMuted: Self = Self(39);
    pub const SoftMutedText: Self = Self(40);
    pub const SoftMutedTitle: Self = Self(41);
    pub const SoftSuccess: Self = Self(42);
    pub const SoftSuccessText: Self = Self(43);
    pub const SoftSuccessTitle: Self = Self(44);
    pub const SoftInfo: Self = Self(45);
    pub const SoftInfoText: Self = Self(46);
    pub const SoftInfoTitle: Self = Self(47);
    pub const SoftWarning: Self = Self(48);
    pub const SoftWarningText: Self = Self(49);
    pub const SoftWarningTitle: Self = Self(50);
    pub const SoftDanger: Self = Self(51);
    pub const SoftDangerText: Self = Self(52);
    pub const SoftDangerTitle: Self = Self(53);

    fn custom(value: &str) -> Option<Self> {
        let id = intern_color_identifier(value)?;
        Self::CUSTOM_OFFSET.checked_add(id).map(Self)
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "primary" => Some(Self::Primary),
            "primaryText" => Some(Self::PrimaryText),
            "primaryTitle" => Some(Self::PrimaryTitle),
            "secondary" => Some(Self::Secondary),
            "secondaryText" => Some(Self::SecondaryText),
            "secondaryTitle" => Some(Self::SecondaryTitle),
            "tertiary" => Some(Self::Tertiary),
            "tertiaryText" => Some(Self::TertiaryText),
            "tertiaryTitle" => Some(Self::TertiaryTitle),
            "muted" => Some(Self::Muted),
            "mutedText" => Some(Self::MutedText),
            "mutedTitle" => Some(Self::MutedTitle),
            "background" => Some(Self::Background),
            "backgroundText" => Some(Self::BackgroundText),
            "backgroundTitle" => Some(Self::BackgroundTitle),
            "surface" => Some(Self::Surface),
            "surfaceText" => Some(Self::SurfaceText),
            "surfaceTitle" => Some(Self::SurfaceTitle),
            "success" => Some(Self::Success),
            "successText" => Some(Self::SuccessText),
            "successTitle" => Some(Self::SuccessTitle),
            "info" => Some(Self::Info),
            "infoText" => Some(Self::InfoText),
            "infoTitle" => Some(Self::InfoTitle),
            "warning" => Some(Self::Warning),
            "warningText" => Some(Self::WarningText),
            "warningTitle" => Some(Self::WarningTitle),
            "danger" => Some(Self::Danger),
            "dangerText" => Some(Self::DangerText),
            "dangerTitle" => Some(Self::DangerTitle),
            "softPrimary" => Some(Self::SoftPrimary),
            "softPrimaryText" => Some(Self::SoftPrimaryText),
            "softPrimaryTitle" => Some(Self::SoftPrimaryTitle),
            "softSecondary" => Some(Self::SoftSecondary),
            "softSecondaryText" => Some(Self::SoftSecondaryText),
            "softSecondaryTitle" => Some(Self::SoftSecondaryTitle),
            "softTertiary" => Some(Self::SoftTertiary),
            "softTertiaryText" => Some(Self::SoftTertiaryText),
            "softTertiaryTitle" => Some(Self::SoftTertiaryTitle),
            "softMuted" => Some(Self::SoftMuted),
            "softMutedText" => Some(Self::SoftMutedText),
            "softMutedTitle" => Some(Self::SoftMutedTitle),
            "softSuccess" => Some(Self::SoftSuccess),
            "softSuccessText" => Some(Self::SoftSuccessText),
            "softSuccessTitle" => Some(Self::SoftSuccessTitle),
            "softInfo" => Some(Self::SoftInfo),
            "softInfoText" => Some(Self::SoftInfoText),
            "softInfoTitle" => Some(Self::SoftInfoTitle),
            "softWarning" => Some(Self::SoftWarning),
            "softWarningText" => Some(Self::SoftWarningText),
            "softWarningTitle" => Some(Self::SoftWarningTitle),
            "softDanger" => Some(Self::SoftDanger),
            "softDangerText" => Some(Self::SoftDangerText),
            "softDangerTitle" => Some(Self::SoftDangerTitle),
            _ if is_valid_color_token_name(value) => Self::custom(value),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::PrimaryText => "primaryText",
            Self::PrimaryTitle => "primaryTitle",
            Self::Secondary => "secondary",
            Self::SecondaryText => "secondaryText",
            Self::SecondaryTitle => "secondaryTitle",
            Self::Tertiary => "tertiary",
            Self::TertiaryText => "tertiaryText",
            Self::TertiaryTitle => "tertiaryTitle",
            Self::Muted => "muted",
            Self::MutedText => "mutedText",
            Self::MutedTitle => "mutedTitle",
            Self::Background => "background",
            Self::BackgroundText => "backgroundText",
            Self::BackgroundTitle => "backgroundTitle",
            Self::Surface => "surface",
            Self::SurfaceText => "surfaceText",
            Self::SurfaceTitle => "surfaceTitle",
            Self::Success => "success",
            Self::SuccessText => "successText",
            Self::SuccessTitle => "successTitle",
            Self::Info => "info",
            Self::InfoText => "infoText",
            Self::InfoTitle => "infoTitle",
            Self::Warning => "warning",
            Self::WarningText => "warningText",
            Self::WarningTitle => "warningTitle",
            Self::Danger => "danger",
            Self::DangerText => "dangerText",
            Self::DangerTitle => "dangerTitle",
            Self::SoftPrimary => "softPrimary",
            Self::SoftPrimaryText => "softPrimaryText",
            Self::SoftPrimaryTitle => "softPrimaryTitle",
            Self::SoftSecondary => "softSecondary",
            Self::SoftSecondaryText => "softSecondaryText",
            Self::SoftSecondaryTitle => "softSecondaryTitle",
            Self::SoftTertiary => "softTertiary",
            Self::SoftTertiaryText => "softTertiaryText",
            Self::SoftTertiaryTitle => "softTertiaryTitle",
            Self::SoftMuted => "softMuted",
            Self::SoftMutedText => "softMutedText",
            Self::SoftMutedTitle => "softMutedTitle",
            Self::SoftSuccess => "softSuccess",
            Self::SoftSuccessText => "softSuccessText",
            Self::SoftSuccessTitle => "softSuccessTitle",
            Self::SoftInfo => "softInfo",
            Self::SoftInfoText => "softInfoText",
            Self::SoftInfoTitle => "softInfoTitle",
            Self::SoftWarning => "softWarning",
            Self::SoftWarningText => "softWarningText",
            Self::SoftWarningTitle => "softWarningTitle",
            Self::SoftDanger => "softDanger",
            Self::SoftDangerText => "softDangerText",
            Self::SoftDangerTitle => "softDangerTitle",
            _ => color_identifier(self.0 - Self::CUSTOM_OFFSET),
        }
    }

    pub fn is_builtin(self) -> bool {
        self.0 < Self::CUSTOM_OFFSET
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Primary,
            Self::PrimaryText,
            Self::PrimaryTitle,
            Self::Secondary,
            Self::SecondaryText,
            Self::SecondaryTitle,
            Self::Tertiary,
            Self::TertiaryText,
            Self::TertiaryTitle,
            Self::Muted,
            Self::MutedText,
            Self::MutedTitle,
            Self::Background,
            Self::BackgroundText,
            Self::BackgroundTitle,
            Self::Surface,
            Self::SurfaceText,
            Self::SurfaceTitle,
            Self::Success,
            Self::SuccessText,
            Self::SuccessTitle,
            Self::Info,
            Self::InfoText,
            Self::InfoTitle,
            Self::Warning,
            Self::WarningText,
            Self::WarningTitle,
            Self::Danger,
            Self::DangerText,
            Self::DangerTitle,
            Self::SoftPrimary,
            Self::SoftPrimaryText,
            Self::SoftPrimaryTitle,
            Self::SoftSecondary,
            Self::SoftSecondaryText,
            Self::SoftSecondaryTitle,
            Self::SoftTertiary,
            Self::SoftTertiaryText,
            Self::SoftTertiaryTitle,
            Self::SoftMuted,
            Self::SoftMutedText,
            Self::SoftMutedTitle,
            Self::SoftSuccess,
            Self::SoftSuccessText,
            Self::SoftSuccessTitle,
            Self::SoftInfo,
            Self::SoftInfoText,
            Self::SoftInfoTitle,
            Self::SoftWarning,
            Self::SoftWarningText,
            Self::SoftWarningTitle,
            Self::SoftDanger,
            Self::SoftDangerText,
            Self::SoftDangerTitle,
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ColorFamily(u16);

impl fmt::Debug for ColorFamily {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl PartialOrd for ColorFamily {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ColorFamily {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_builtin() || other.is_builtin() {
            return self.0.cmp(&other.0);
        }
        self.as_str().cmp(other.as_str())
    }
}

#[allow(non_upper_case_globals)]
impl ColorFamily {
    const CUSTOM_OFFSET: u16 = 10;
    pub const Primary: Self = Self(0);
    pub const Secondary: Self = Self(1);
    pub const Tertiary: Self = Self(2);
    pub const Muted: Self = Self(3);
    pub const Background: Self = Self(4);
    pub const Surface: Self = Self(5);
    pub const Success: Self = Self(6);
    pub const Info: Self = Self(7);
    pub const Warning: Self = Self(8);
    pub const Danger: Self = Self(9);

    fn custom(value: &str) -> Option<Self> {
        let id = intern_color_identifier(value)?;
        Self::CUSTOM_OFFSET.checked_add(id).map(Self)
    }

    pub fn color_token(self) -> ColorToken {
        match self {
            Self::Primary => ColorToken::Primary,
            Self::Secondary => ColorToken::Secondary,
            Self::Tertiary => ColorToken::Tertiary,
            Self::Muted => ColorToken::Muted,
            Self::Background => ColorToken::Background,
            Self::Surface => ColorToken::Surface,
            Self::Success => ColorToken::Success,
            Self::Info => ColorToken::Info,
            Self::Warning => ColorToken::Warning,
            Self::Danger => ColorToken::Danger,
            _ => custom_color_token(self.as_str()),
        }
    }

    pub fn text_token(self) -> ColorToken {
        match self {
            Self::Primary => ColorToken::PrimaryText,
            Self::Secondary => ColorToken::SecondaryText,
            Self::Tertiary => ColorToken::TertiaryText,
            Self::Muted => ColorToken::MutedText,
            Self::Background => ColorToken::BackgroundText,
            Self::Surface => ColorToken::SurfaceText,
            Self::Success => ColorToken::SuccessText,
            Self::Info => ColorToken::InfoText,
            Self::Warning => ColorToken::WarningText,
            Self::Danger => ColorToken::DangerText,
            _ => custom_color_role_token(self.as_str(), "Text"),
        }
    }

    pub fn title_token(self) -> ColorToken {
        match self {
            Self::Primary => ColorToken::PrimaryTitle,
            Self::Secondary => ColorToken::SecondaryTitle,
            Self::Tertiary => ColorToken::TertiaryTitle,
            Self::Muted => ColorToken::MutedTitle,
            Self::Background => ColorToken::BackgroundTitle,
            Self::Surface => ColorToken::SurfaceTitle,
            Self::Success => ColorToken::SuccessTitle,
            Self::Info => ColorToken::InfoTitle,
            Self::Warning => ColorToken::WarningTitle,
            Self::Danger => ColorToken::DangerTitle,
            _ => custom_color_role_token(self.as_str(), "Title"),
        }
    }

    pub fn soft_color_token(self) -> ColorToken {
        match self {
            Self::Primary => ColorToken::SoftPrimary,
            Self::Secondary => ColorToken::SoftSecondary,
            Self::Tertiary => ColorToken::SoftTertiary,
            Self::Muted => ColorToken::SoftMuted,
            Self::Background => ColorToken::Background,
            Self::Surface => ColorToken::Surface,
            Self::Success => ColorToken::SoftSuccess,
            Self::Info => ColorToken::SoftInfo,
            Self::Warning => ColorToken::SoftWarning,
            Self::Danger => ColorToken::SoftDanger,
            _ => custom_soft_color_token(self.as_str(), ""),
        }
    }

    pub fn soft_text_token(self) -> ColorToken {
        match self {
            Self::Primary => ColorToken::SoftPrimaryText,
            Self::Secondary => ColorToken::SoftSecondaryText,
            Self::Tertiary => ColorToken::SoftTertiaryText,
            Self::Muted => ColorToken::SoftMutedText,
            Self::Background => ColorToken::BackgroundText,
            Self::Surface => ColorToken::SurfaceText,
            Self::Success => ColorToken::SoftSuccessText,
            Self::Info => ColorToken::SoftInfoText,
            Self::Warning => ColorToken::SoftWarningText,
            Self::Danger => ColorToken::SoftDangerText,
            _ => custom_soft_color_token(self.as_str(), "Text"),
        }
    }

    pub fn soft_title_token(self) -> ColorToken {
        match self {
            Self::Primary => ColorToken::SoftPrimaryTitle,
            Self::Secondary => ColorToken::SoftSecondaryTitle,
            Self::Tertiary => ColorToken::SoftTertiaryTitle,
            Self::Muted => ColorToken::SoftMutedTitle,
            Self::Background => ColorToken::BackgroundTitle,
            Self::Surface => ColorToken::SurfaceTitle,
            Self::Success => ColorToken::SoftSuccessTitle,
            Self::Info => ColorToken::SoftInfoTitle,
            Self::Warning => ColorToken::SoftWarningTitle,
            Self::Danger => ColorToken::SoftDangerTitle,
            _ => custom_soft_color_token(self.as_str(), "Title"),
        }
    }

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
            _ if is_valid_custom_color_family_name(value) => Self::custom(value),
            _ => None,
        }
    }

    pub fn from_theme_name(value: &str) -> Option<(Self, bool)> {
        if let Some(family) = Self::from_name(value) {
            return Some((family, false));
        }
        let suffix = value.strip_prefix("soft")?;
        let mut chars = suffix.chars();
        let first = chars.next()?;
        if !first.is_ascii_uppercase() {
            return None;
        }
        let mut base = first.to_ascii_lowercase().to_string();
        base.extend(chars);
        let family = Self::from_name(&base)?;
        (!matches!(family, Self::Background | Self::Surface)).then_some((family, true))
    }

    pub fn theme_tokens(self, soft: bool) -> Option<[ColorToken; 3]> {
        if soft {
            if matches!(self, Self::Background | Self::Surface) {
                return None;
            }
            return Some([
                self.soft_color_token(),
                self.soft_text_token(),
                self.soft_title_token(),
            ]);
        }
        Some([self.color_token(), self.text_token(), self.title_token()])
    }

    pub fn theme_names() -> &'static [&'static str] {
        &[
            "primary",
            "secondary",
            "tertiary",
            "muted",
            "background",
            "surface",
            "success",
            "info",
            "warning",
            "danger",
            "softPrimary",
            "softSecondary",
            "softTertiary",
            "softMuted",
            "softSuccess",
            "softInfo",
            "softWarning",
            "softDanger",
        ]
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
            _ => color_identifier(self.0 - Self::CUSTOM_OFFSET),
        }
    }

    pub fn theme_name(self, soft: bool) -> String {
        if !soft {
            return self.as_str().to_string();
        }
        let mut chars = self.as_str().chars();
        let first = chars.next().expect("color family");
        let mut name = format!("soft{}", first.to_ascii_uppercase());
        name.extend(chars);
        name
    }

    pub fn is_builtin(self) -> bool {
        self.0 < Self::CUSTOM_OFFSET
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

fn custom_color_token(value: &str) -> ColorToken {
    ColorToken::custom(value).expect("custom color token")
}

fn custom_color_role_token(value: &str, role: &str) -> ColorToken {
    custom_color_token(&format!("{value}{role}"))
}

fn custom_soft_color_token(value: &str, role: &str) -> ColorToken {
    let mut chars = value.chars();
    let first = chars.next().expect("custom color family");
    let mut name = format!("soft{}", first.to_ascii_uppercase());
    name.extend(chars);
    name.push_str(role);
    custom_color_token(&name)
}

fn is_valid_color_token_name(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    if value
        .strip_prefix("on")
        .and_then(|suffix| suffix.as_bytes().first())
        .is_some_and(u8::is_ascii_uppercase)
    {
        return false;
    }
    bytes.iter().all(u8::is_ascii_alphanumeric)
}

fn is_valid_custom_color_family_name(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > 48 || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    if !bytes.iter().all(u8::is_ascii_alphanumeric)
        || value.ends_with("Text")
        || value.ends_with("Title")
        || value == "soft"
        || matches!(
            value,
            "theme" | "design" | "fonts" | "colors" | "color" | "text" | "title"
        )
    {
        return false;
    }
    if value
        .strip_prefix("soft")
        .and_then(|suffix| suffix.as_bytes().first())
        .is_some_and(u8::is_ascii_uppercase)
    {
        return false;
    }
    !value
        .strip_prefix("on")
        .and_then(|suffix| suffix.as_bytes().first())
        .is_some_and(u8::is_ascii_uppercase)
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
    Syne,
    Jost,
    Puritan,
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
            "syne" => Some(Self::Syne),
            "jost" => Some(Self::Jost),
            "puritan" => Some(Self::Puritan),
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
            Self::Syne => "syne",
            Self::Jost => "jost",
            Self::Puritan => "puritan",
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
            Self::Syne,
            Self::Jost,
            Self::Puritan,
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
    Line,
}

impl ComponentVariant {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "solid" => Some(Self::Solid),
            "soft" => Some(Self::Soft),
            "outlined" | "outline" => Some(Self::Outlined),
            "ghost" => Some(Self::Ghost),
            "line" => Some(Self::Line),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Soft => "soft",
            Self::Outlined => "outlined",
            Self::Ghost => "ghost",
            Self::Line => "line",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Solid,
            Self::Soft,
            Self::Outlined,
            Self::Ghost,
            Self::Line,
        ]
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

    pub fn icon_button_control_size(self) -> ScaleValue {
        match self {
            Self::Xs => ScaleValue::from_half_steps(12),
            Self::Sm => ScaleValue::from_half_steps(16),
            Self::Md => ScaleValue::from_half_steps(20),
            Self::Lg => ScaleValue::from_half_steps(24),
            Self::Xl => ScaleValue::from_half_steps(28),
        }
    }

    pub fn icon_button_icon_size(self) -> ScaleValue {
        match self {
            Self::Xs => ScaleValue::from_half_steps(8),
            Self::Sm => ScaleValue::from_half_steps(10),
            Self::Md => ScaleValue::from_half_steps(12),
            Self::Lg => ScaleValue::from_half_steps(16),
            Self::Xl => ScaleValue::from_half_steps(20),
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
