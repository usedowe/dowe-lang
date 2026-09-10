impl Default for DesignConfig {
    fn default() -> Self {
        Self {
            default_theme: "light".to_string(),
            themes: vec![
                integrated_design_theme("light").expect("light design theme"),
                integrated_design_theme("dark").expect("dark design theme"),
            ],
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
        match token.as_str() {
            "white" => "#FFFFFF",
            "black" => "#000000",
            "transparent" => "#00000000",
            _ => self
                .colors
                .get(&token)
                .map(String::as_str)
                .expect("design color token"),
        }
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

    pub fn contains_color_family(&self, family: ColorFamily) -> bool {
        family.theme_tokens().is_some_and(|tokens| {
            tokens
                .into_iter()
                .all(|token| self.colors.contains_key(&token))
        })
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
            "DateRange" => Some(Self::DateRange),
            "Color" => Some(Self::Color),
            "Textarea" => Some(Self::Textarea),
            "Password" => Some(Self::Password),
            "Select" => Some(Self::Select),
            "Pin" => Some(Self::Pin),
            "SideNav" => Some(Self::SideNav),
            "Sidebar" => Some(Self::Sidebar),
            "NavMenu" => Some(Self::NavMenu),
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
            Self::DateRange => "DateRange",
            Self::Color => "Color",
            Self::Textarea => "Textarea",
            Self::Password => "Password",
            Self::Select => "Select",
            Self::Pin => "Pin",
            Self::SideNav => "SideNav",
            Self::Sidebar => "Sidebar",
            Self::NavMenu => "NavMenu",
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
            Self::DateRange,
            Self::Color,
            Self::Textarea,
            Self::Password,
            Self::Select,
            Self::Pin,
            Self::SideNav,
            Self::Sidebar,
            Self::NavMenu,
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
                (ColorToken::Primary, "#1F3A5F"),
                (ColorToken::PrimaryText, "#EBF2FA"),
                (ColorToken::PrimaryTitle, "#FFFFFF"),
                (ColorToken::Secondary, "#6BC670"),
                (ColorToken::SecondaryText, "#0F291E"),
                (ColorToken::SecondaryTitle, "#040D05"),
                (ColorToken::Accent, "#3F7A8A"),
                (ColorToken::AccentText, "#F0F7F9"),
                (ColorToken::AccentTitle, "#FFFFFF"),
                (ColorToken::Muted, "#E2E8F0"),
                (ColorToken::MutedText, "#334155"),
                (ColorToken::MutedTitle, "#1F3A5F"),
                (ColorToken::Background, "#F3F1EE"),
                (ColorToken::BackgroundText, "#334155"),
                (ColorToken::BackgroundTitle, "#1F3A5F"),
                (ColorToken::Surface, "#FFFFFF"),
                (ColorToken::SurfaceText, "#334155"),
                (ColorToken::SurfaceTitle, "#1F3A5F"),
                (ColorToken::Success, "#16A34A"),
                (ColorToken::SuccessText, "#E8F5E9"),
                (ColorToken::SuccessTitle, "#FFFFFF"),
                (ColorToken::Info, "#0084D1"),
                (ColorToken::InfoText, "#E1F5FE"),
                (ColorToken::InfoTitle, "#FFFFFF"),
                (ColorToken::Warning, "#D08700"),
                (ColorToken::WarningText, "#1F1400"),
                (ColorToken::WarningTitle, "#0D0900"),
                (ColorToken::Danger, "#E7000B"),
                (ColorToken::DangerText, "#FFEBEE"),
                (ColorToken::DangerTitle, "#FFFFFF"),
            ],
            8,
        )),
        "dark" => Some(theme_from_values(
            "dark",
            &[
                (ColorToken::Primary, "#F3F1EE"),
                (ColorToken::PrimaryText, "#334155"),
                (ColorToken::PrimaryTitle, "#1F3A5F"),
                (ColorToken::Secondary, "#6BC670"),
                (ColorToken::SecondaryText, "#0F291E"),
                (ColorToken::SecondaryTitle, "#040D05"),
                (ColorToken::Accent, "#3F7A8A"),
                (ColorToken::AccentText, "#F0F7F9"),
                (ColorToken::AccentTitle, "#FFFFFF"),
                (ColorToken::Muted, "#334155"),
                (ColorToken::MutedText, "#D5DEE9"),
                (ColorToken::MutedTitle, "#F3F7FC"),
                (ColorToken::Background, "#111827"),
                (ColorToken::BackgroundText, "#E5E7EB"),
                (ColorToken::BackgroundTitle, "#F9FAFB"),
                (ColorToken::Surface, "#1F2937"),
                (ColorToken::SurfaceText, "#E5E7EB"),
                (ColorToken::SurfaceTitle, "#F9FAFB"),
                (ColorToken::Success, "#16A34A"),
                (ColorToken::SuccessText, "#E8F5E9"),
                (ColorToken::SuccessTitle, "#FFFFFF"),
                (ColorToken::Info, "#0084D1"),
                (ColorToken::InfoText, "#E1F5FE"),
                (ColorToken::InfoTitle, "#FFFFFF"),
                (ColorToken::Warning, "#D08700"),
                (ColorToken::WarningText, "#1F1400"),
                (ColorToken::WarningTitle, "#0D0900"),
                (ColorToken::Danger, "#E7000B"),
                (ColorToken::DangerText, "#FFEBEE"),
                (ColorToken::DangerTitle, "#FFFFFF"),
            ],
            8,
        )),
        _ => None,
    }
}

fn theme_from_values(name: &str, colors: &[(ColorToken, &str)], radius: u16) -> DesignTheme {
    DesignTheme {
        name: name.to_string(),
        colors: colors
            .iter()
            .map(|(token, value)| (*token, (*value).to_string()))
            .collect(),
        radius,
    }
}

