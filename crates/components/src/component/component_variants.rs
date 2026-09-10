#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentVariant {
    Solid,
    Outlined,
    Ghost,
    Line,
}

impl ComponentVariant {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "solid" => Some(Self::Solid),
            "outlined" | "outline" => Some(Self::Outlined),
            "ghost" => Some(Self::Ghost),
            "line" => Some(Self::Line),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Outlined => "outlined",
            Self::Ghost => "ghost",
            Self::Line => "line",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Solid, Self::Outlined, Self::Ghost, Self::Line]
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

    pub fn chip_icon_size(self) -> ScaleValue {
        match self {
            Self::Xs => ScaleValue::from_half_steps(6),
            Self::Sm => ScaleValue::from_half_steps(7),
            Self::Md => ScaleValue::from_half_steps(8),
            Self::Lg => ScaleValue::from_half_steps(10),
            Self::Xl => ScaleValue::from_half_steps(12),
        }
    }
}
