pub const INPUT_MIN_HEIGHT: ScaleValue = ScaleValue::from_half_steps(20);
pub const INPUT_HORIZONTAL_PADDING: ScaleValue = ScaleValue::from_half_steps(6);
pub const INPUT_TEXT_SIZE: TextSize = TextSize::Md;
pub const FORM_CONTROL_FLOATING_HEIGHT_INCREMENT: ScaleValue = ScaleValue::from_half_steps(4);

pub fn form_control_min_height(size: ButtonSize, label_floating: bool) -> ScaleValue {
    let base = match size {
        ButtonSize::Xs => ScaleValue::from_half_steps(12),
        ButtonSize::Sm => ScaleValue::from_half_steps(16),
        ButtonSize::Md => INPUT_MIN_HEIGHT,
        ButtonSize::Lg => ScaleValue::from_half_steps(24),
        ButtonSize::Xl => ScaleValue::from_half_steps(28),
    };
    ScaleValue::from_half_steps(
        base.0
            + if label_floating {
                FORM_CONTROL_FLOATING_HEIGHT_INCREMENT.0
            } else {
                0
            },
    )
}

pub fn form_control_text_size(size: ButtonSize) -> TextSize {
    match size {
        ButtonSize::Xs => TextSize::Xs,
        ButtonSize::Sm => TextSize::Sm,
        ButtonSize::Md => TextSize::Md,
        ButtonSize::Lg => TextSize::Lg,
        ButtonSize::Xl => TextSize::Xl,
    }
}

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

