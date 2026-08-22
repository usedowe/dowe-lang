#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisibilityCondition {
    Static(ResponsiveValue<bool>),
    Signal(String),
    NumberComparison {
        path: String,
        comparison: ReactiveNumberComparison,
    },
    StringEquality {
        path: String,
        value: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgPathFill {
    None,
    CurrentColor,
    Color(ColorToken),
    RawFill {
        color: &'static str,
        opacity: u8,
        even_odd: bool,
    },
    Fill {
        color: Option<ColorToken>,
        opacity: u8,
        even_odd: bool,
    },
    RawStroke {
        color: &'static str,
        opacity: u8,
        width: u16,
        line_cap: SvgLineCap,
        line_join: SvgLineJoin,
    },
    LiteralFill {
        red: u8,
        green: u8,
        blue: u8,
        opacity: u8,
        even_odd: bool,
    },
    LiteralStroke {
        red: u8,
        green: u8,
        blue: u8,
        opacity: u8,
        width: u16,
        line_cap: SvgLineCap,
        line_join: SvgLineJoin,
    },
    Stroke {
        color: Option<ColorToken>,
        opacity: u8,
        width: u16,
        line_cap: SvgLineCap,
        line_join: SvgLineJoin,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgLineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgLineJoin {
    Miter,
    Round,
    Bevel,
}

