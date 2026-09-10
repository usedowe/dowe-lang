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
