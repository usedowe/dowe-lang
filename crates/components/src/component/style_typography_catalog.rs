#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSpacing {
    Tightest,
    Tighter,
    Tight,
    Normal,
    Wide,
    Wider,
    Widest,
}

impl TextSpacing {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "tightest" => Some(Self::Tightest),
            "tighter" => Some(Self::Tighter),
            "tight" => Some(Self::Tight),
            "normal" => Some(Self::Normal),
            "wide" => Some(Self::Wide),
            "wider" => Some(Self::Wider),
            "widest" => Some(Self::Widest),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tightest => "tightest",
            Self::Tighter => "tighter",
            Self::Tight => "tight",
            Self::Normal => "normal",
            Self::Wide => "wide",
            Self::Wider => "wider",
            Self::Widest => "widest",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Tightest,
            Self::Tighter,
            Self::Tight,
            Self::Normal,
            Self::Wide,
            Self::Wider,
            Self::Widest,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontCatalogEntry {
    pub token: FontFamily,
    pub display_name: &'static str,
    pub web_stack: &'static str,
    pub ios_family_name: &'static str,
    pub android_family_name: &'static str,
    pub package_assets: bool,
    pub weights: &'static [FontCatalogWeight],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontCatalogWeight {
    pub weight: TextWeight,
    pub numeric_weight: u16,
    pub asset_stem: &'static str,
}

pub fn font_catalog() -> &'static [FontCatalogEntry] {
    FONT_CATALOG
}
