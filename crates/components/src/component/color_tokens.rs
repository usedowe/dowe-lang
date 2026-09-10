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
    const CUSTOM_OFFSET: u16 = 33;
    pub const Primary: Self = Self(0);
    pub const PrimaryText: Self = Self(1);
    pub const PrimaryTitle: Self = Self(2);
    pub const Secondary: Self = Self(3);
    pub const SecondaryText: Self = Self(4);
    pub const SecondaryTitle: Self = Self(5);
    pub const Accent: Self = Self(6);
    pub const AccentText: Self = Self(7);
    pub const AccentTitle: Self = Self(8);
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
    pub const White: Self = Self(30);
    pub const Black: Self = Self(31);
    pub const Transparent: Self = Self(32);
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
            "accent" => Some(Self::Accent),
            "accentText" => Some(Self::AccentText),
            "accentTitle" => Some(Self::AccentTitle),
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
            "white" => Some(Self::White),
            "black" => Some(Self::Black),
            "transparent" => Some(Self::Transparent),
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
            Self::Accent => "accent",
            Self::AccentText => "accentText",
            Self::AccentTitle => "accentTitle",
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
            Self::White => "white",
            Self::Black => "black",
            Self::Transparent => "transparent",
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
            Self::Accent,
            Self::AccentText,
            Self::AccentTitle,
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
    pub const Accent: Self = Self(2);
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
            Self::Accent => ColorToken::Accent,
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
            Self::Accent => ColorToken::AccentText,
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
            Self::Accent => ColorToken::AccentTitle,
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

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "primary" => Some(Self::Primary),
            "secondary" => Some(Self::Secondary),
            "accent" => Some(Self::Accent),
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
        Self::from_name(value).map(|family| (family, false))
    }

    pub fn theme_tokens(self) -> Option<[ColorToken; 3]> {
        Some([self.color_token(), self.text_token(), self.title_token()])
    }

    pub fn theme_names() -> &'static [&'static str] {
        &[
            "primary",
            "secondary",
            "accent",
            "muted",
            "background",
            "surface",
            "success",
            "info",
            "warning",
            "danger",
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Accent => "accent",
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

    pub fn theme_name(self) -> String {
        self.as_str().to_string()
    }

    pub fn is_builtin(self) -> bool {
        self.0 < Self::CUSTOM_OFFSET
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Primary,
            Self::Secondary,
            Self::Accent,
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

fn is_valid_color_token_name(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    if value.starts_with("soft") {
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
        || value.starts_with("soft")
        || value.ends_with("Text")
        || value.ends_with("Title")
        || matches!(
            value,
            "theme" | "design" | "fonts" | "colors" | "color" | "text" | "title"
        )
    {
        return false;
    }
    !value
        .strip_prefix("on")
        .and_then(|suffix| suffix.as_bytes().first())
        .is_some_and(u8::is_ascii_uppercase)
}
