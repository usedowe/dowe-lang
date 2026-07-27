#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidTextSize {
    pub min: &'static str,
    pub preferred_base: &'static str,
    pub preferred_viewport: &'static str,
    pub max: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextTypography {
    pub font_size: FluidTextSize,
    pub line_height: &'static str,
    pub weight: TextWeight,
    pub letter_spacing_em: &'static str,
}

pub fn text_typography(title: bool, value: TextSize) -> TextTypography {
    if title {
        title_text_typography(value)
    } else {
        body_text_typography(value)
    }
}

pub fn text_weight_number(value: TextWeight) -> &'static str {
    match value {
        TextWeight::Thin => "100",
        TextWeight::Extralight => "200",
        TextWeight::Light => "300",
        TextWeight::Regular => "400",
        TextWeight::Medium => "500",
        TextWeight::Semibold => "600",
        TextWeight::Bold => "700",
        TextWeight::Extrabold => "800",
        TextWeight::Black => "900",
    }
}

pub fn text_spacing_em(value: TextSpacing) -> &'static str {
    match value {
        TextSpacing::Tightest => "-0.06",
        TextSpacing::Tighter => "-0.04",
        TextSpacing::Tight => "-0.02",
        TextSpacing::Normal => "0",
        TextSpacing::Wide => "0.02",
        TextSpacing::Wider => "0.04",
        TextSpacing::Widest => "0.06",
    }
}

fn body_text_typography(value: TextSize) -> TextTypography {
    let font_size = match value {
        TextSize::Xs => FluidTextSize {
            min: "11",
            preferred_base: "10.4",
            preferred_viewport: "0.15",
            max: "12",
        },
        TextSize::Sm => FluidTextSize {
            min: "12",
            preferred_base: "11.2",
            preferred_viewport: "0.2",
            max: "14",
        },
        TextSize::Md => FluidTextSize {
            min: "14",
            preferred_base: "13.12",
            preferred_viewport: "0.25",
            max: "16",
        },
        TextSize::Lg => FluidTextSize {
            min: "16",
            preferred_base: "15.2",
            preferred_viewport: "0.3",
            max: "18",
        },
        TextSize::Xl => FluidTextSize {
            min: "17",
            preferred_base: "16",
            preferred_viewport: "0.35",
            max: "20",
        },
        TextSize::TwoXl => FluidTextSize {
            min: "18",
            preferred_base: "16.8",
            preferred_viewport: "0.45",
            max: "22",
        },
        TextSize::ThreeXl => FluidTextSize {
            min: "20",
            preferred_base: "18.4",
            preferred_viewport: "0.55",
            max: "24",
        },
        TextSize::FourXl => FluidTextSize {
            min: "22",
            preferred_base: "19.52",
            preferred_viewport: "0.8",
            max: "28",
        },
        TextSize::FiveXl => FluidTextSize {
            min: "24",
            preferred_base: "20.8",
            preferred_viewport: "1",
            max: "32",
        },
        TextSize::SixXl => FluidTextSize {
            min: "28",
            preferred_base: "23.2",
            preferred_viewport: "1.25",
            max: "36",
        },
        TextSize::SevenXl => FluidTextSize {
            min: "32",
            preferred_base: "25.6",
            preferred_viewport: "1.5",
            max: "40",
        },
        TextSize::EightXl => FluidTextSize {
            min: "36",
            preferred_base: "28",
            preferred_viewport: "2",
            max: "48",
        },
        TextSize::NineXl => FluidTextSize {
            min: "40",
            preferred_base: "30.4",
            preferred_viewport: "2.8",
            max: "60",
        },
    };
    TextTypography {
        font_size,
        line_height: body_line_height(value),
        weight: TextWeight::Regular,
        letter_spacing_em: "0",
    }
}

fn title_text_typography(value: TextSize) -> TextTypography {
    let font_size = match value {
        TextSize::Xs => FluidTextSize {
            min: "14",
            preferred_base: "12.8",
            preferred_viewport: "0.2",
            max: "16",
        },
        TextSize::Sm => FluidTextSize {
            min: "16",
            preferred_base: "14.4",
            preferred_viewport: "0.25",
            max: "18",
        },
        TextSize::Md => FluidTextSize {
            min: "18",
            preferred_base: "16",
            preferred_viewport: "0.35",
            max: "20",
        },
        TextSize::Lg => FluidTextSize {
            min: "20",
            preferred_base: "17.6",
            preferred_viewport: "0.45",
            max: "24",
        },
        TextSize::Xl => FluidTextSize {
            min: "24",
            preferred_base: "20",
            preferred_viewport: "0.75",
            max: "30",
        },
        TextSize::TwoXl => FluidTextSize {
            min: "28",
            preferred_base: "22.4",
            preferred_viewport: "1",
            max: "36",
        },
        TextSize::ThreeXl => FluidTextSize {
            min: "32",
            preferred_base: "25.6",
            preferred_viewport: "1.5",
            max: "48",
        },
        TextSize::FourXl => FluidTextSize {
            min: "36",
            preferred_base: "28.8",
            preferred_viewport: "2",
            max: "60",
        },
        TextSize::FiveXl => FluidTextSize {
            min: "40",
            preferred_base: "32",
            preferred_viewport: "2.75",
            max: "72",
        },
        TextSize::SixXl => FluidTextSize {
            min: "48",
            preferred_base: "36",
            preferred_viewport: "3.5",
            max: "80",
        },
        TextSize::SevenXl => FluidTextSize {
            min: "56",
            preferred_base: "40",
            preferred_viewport: "4.5",
            max: "96",
        },
        TextSize::EightXl => FluidTextSize {
            min: "64",
            preferred_base: "44",
            preferred_viewport: "5.5",
            max: "112",
        },
        TextSize::NineXl => FluidTextSize {
            min: "72",
            preferred_base: "48",
            preferred_viewport: "7",
            max: "128",
        },
    };
    TextTypography {
        font_size,
        line_height: title_line_height(value),
        weight: title_weight(value),
        letter_spacing_em: title_letter_spacing(value),
    }
}

fn body_line_height(value: TextSize) -> &'static str {
    match value {
        TextSize::Xs | TextSize::Sm => "1.5",
        TextSize::Md | TextSize::Lg | TextSize::Xl => "1.6",
        TextSize::TwoXl => "1.55",
        TextSize::ThreeXl => "1.5",
        TextSize::FourXl => "1.45",
        TextSize::FiveXl => "1.4",
        TextSize::SixXl => "1.35",
        TextSize::SevenXl => "1.3",
        TextSize::EightXl => "1.25",
        TextSize::NineXl => "1.2",
    }
}

fn title_line_height(value: TextSize) -> &'static str {
    match value {
        TextSize::Xs | TextSize::Sm => "1.35",
        TextSize::Md | TextSize::Lg => "1.3",
        TextSize::Xl => "1.25",
        TextSize::TwoXl => "1.2",
        TextSize::ThreeXl => "1.15",
        TextSize::FourXl => "1.1",
        TextSize::FiveXl => "1.05",
        TextSize::SixXl | TextSize::SevenXl | TextSize::EightXl | TextSize::NineXl => "1",
    }
}

fn title_weight(value: TextSize) -> TextWeight {
    match value {
        TextSize::Xs | TextSize::Sm | TextSize::Md | TextSize::Lg => TextWeight::Semibold,
        TextSize::Xl
        | TextSize::TwoXl
        | TextSize::ThreeXl
        | TextSize::FourXl
        | TextSize::FiveXl
        | TextSize::SixXl => TextWeight::Bold,
        TextSize::SevenXl | TextSize::EightXl | TextSize::NineXl => TextWeight::Extrabold,
    }
}

fn title_letter_spacing(value: TextSize) -> &'static str {
    match value {
        TextSize::Xs | TextSize::Sm => "-0.01",
        TextSize::Md | TextSize::Lg => "-0.015",
        TextSize::Xl => "-0.02",
        TextSize::TwoXl => "-0.025",
        TextSize::ThreeXl => "-0.03",
        TextSize::FourXl => "-0.035",
        TextSize::FiveXl => "-0.04",
        TextSize::SixXl => "-0.045",
        TextSize::SevenXl => "-0.05",
        TextSize::EightXl => "-0.055",
        TextSize::NineXl => "-0.06",
    }
}
