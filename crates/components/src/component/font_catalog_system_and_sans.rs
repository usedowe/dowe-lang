macro_rules! font_catalog_system_entry {
    () => {
        FontCatalogEntry {
            token: FontFamily::System,
            display_name: "system-ui",
            web_stack: "system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif",
            ios_family_name: ".system",
            android_family_name: "sans-serif",
            package_assets: false,
            weights: &[],
        }
    };
}

macro_rules! font_catalog_inter_entry {
    () => {
FontCatalogEntry {
        token: FontFamily::Inter,
        display_name: "Inter",
        web_stack: "\"Dowe Inter\",Inter,system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif",
        ios_family_name: "Inter",
        android_family_name: "Inter",
        package_assets: true,
        weights: &[
            FontCatalogWeight {
                weight: TextWeight::Thin,
                numeric_weight: 100,
                asset_stem: "inter-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Extralight,
                numeric_weight: 200,
                asset_stem: "inter-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Light,
                numeric_weight: 300,
                asset_stem: "inter-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Regular,
                numeric_weight: 400,
                asset_stem: "inter-regular",
            },
            FontCatalogWeight {
                weight: TextWeight::Medium,
                numeric_weight: 500,
                asset_stem: "inter-medium",
            },
            FontCatalogWeight {
                weight: TextWeight::Semibold,
                numeric_weight: 600,
                asset_stem: "inter-semibold",
            },
            FontCatalogWeight {
                weight: TextWeight::Bold,
                numeric_weight: 700,
                asset_stem: "inter-bold",
            },
            FontCatalogWeight {
                weight: TextWeight::Extrabold,
                numeric_weight: 800,
                asset_stem: "inter-extrabold",
            },
            FontCatalogWeight {
                weight: TextWeight::Black,
                numeric_weight: 900,
                asset_stem: "inter-extrabold",
            },
        ],
    }
    };
}

macro_rules! font_catalog_roboto_entry {
    () => {
FontCatalogEntry {
        token: FontFamily::Roboto,
        display_name: "Roboto",
        web_stack: "\"Dowe Roboto\",Roboto,system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif",
        ios_family_name: "Roboto",
        android_family_name: "Roboto",
        package_assets: true,
        weights: &[
            FontCatalogWeight {
                weight: TextWeight::Thin,
                numeric_weight: 100,
                asset_stem: "roboto-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Extralight,
                numeric_weight: 200,
                asset_stem: "roboto-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Light,
                numeric_weight: 300,
                asset_stem: "roboto-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Regular,
                numeric_weight: 400,
                asset_stem: "roboto-regular",
            },
            FontCatalogWeight {
                weight: TextWeight::Medium,
                numeric_weight: 500,
                asset_stem: "roboto-medium",
            },
            FontCatalogWeight {
                weight: TextWeight::Semibold,
                numeric_weight: 600,
                asset_stem: "roboto-semibold",
            },
            FontCatalogWeight {
                weight: TextWeight::Bold,
                numeric_weight: 700,
                asset_stem: "roboto-bold",
            },
            FontCatalogWeight {
                weight: TextWeight::Extrabold,
                numeric_weight: 800,
                asset_stem: "roboto-extrabold",
            },
            FontCatalogWeight {
                weight: TextWeight::Black,
                numeric_weight: 900,
                asset_stem: "roboto-extrabold",
            },
        ],
    }
    };
}

macro_rules! font_catalog_montserrat_entry {
    () => {
FontCatalogEntry {
        token: FontFamily::Montserrat,
        display_name: "Montserrat",
        web_stack: "\"Dowe Montserrat\",Montserrat,system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif",
        ios_family_name: "Montserrat",
        android_family_name: "Montserrat",
        package_assets: true,
        weights: &[
            FontCatalogWeight {
                weight: TextWeight::Thin,
                numeric_weight: 100,
                asset_stem: "montserrat-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Extralight,
                numeric_weight: 200,
                asset_stem: "montserrat-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Light,
                numeric_weight: 300,
                asset_stem: "montserrat-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Regular,
                numeric_weight: 400,
                asset_stem: "montserrat-regular",
            },
            FontCatalogWeight {
                weight: TextWeight::Medium,
                numeric_weight: 500,
                asset_stem: "montserrat-medium",
            },
            FontCatalogWeight {
                weight: TextWeight::Semibold,
                numeric_weight: 600,
                asset_stem: "montserrat-semibold",
            },
            FontCatalogWeight {
                weight: TextWeight::Bold,
                numeric_weight: 700,
                asset_stem: "montserrat-bold",
            },
            FontCatalogWeight {
                weight: TextWeight::Extrabold,
                numeric_weight: 800,
                asset_stem: "montserrat-extrabold",
            },
            FontCatalogWeight {
                weight: TextWeight::Black,
                numeric_weight: 900,
                asset_stem: "montserrat-extrabold",
            },
        ],
    }
    };
}

macro_rules! font_catalog_lato_entry {
    () => {
FontCatalogEntry {
        token: FontFamily::Lato,
        display_name: "Lato",
        web_stack: "\"Dowe Lato\",Lato,system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif",
        ios_family_name: "Lato",
        android_family_name: "Lato",
        package_assets: true,
        weights: &[
            FontCatalogWeight {
                weight: TextWeight::Thin,
                numeric_weight: 100,
                asset_stem: "lato-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Extralight,
                numeric_weight: 200,
                asset_stem: "lato-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Light,
                numeric_weight: 300,
                asset_stem: "lato-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Regular,
                numeric_weight: 400,
                asset_stem: "lato-regular",
            },
            FontCatalogWeight {
                weight: TextWeight::Medium,
                numeric_weight: 500,
                asset_stem: "lato-medium",
            },
            FontCatalogWeight {
                weight: TextWeight::Semibold,
                numeric_weight: 600,
                asset_stem: "lato-semibold",
            },
            FontCatalogWeight {
                weight: TextWeight::Bold,
                numeric_weight: 700,
                asset_stem: "lato-bold",
            },
            FontCatalogWeight {
                weight: TextWeight::Extrabold,
                numeric_weight: 800,
                asset_stem: "lato-extrabold",
            },
            FontCatalogWeight {
                weight: TextWeight::Black,
                numeric_weight: 900,
                asset_stem: "lato-extrabold",
            },
        ],
    }
    };
}

macro_rules! font_catalog_poppins_entry {
    () => {
FontCatalogEntry {
        token: FontFamily::Poppins,
        display_name: "Poppins",
        web_stack: "\"Dowe Poppins\",Poppins,system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif",
        ios_family_name: "Poppins",
        android_family_name: "Poppins",
        package_assets: true,
        weights: &[
            FontCatalogWeight {
                weight: TextWeight::Thin,
                numeric_weight: 100,
                asset_stem: "poppins-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Extralight,
                numeric_weight: 200,
                asset_stem: "poppins-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Light,
                numeric_weight: 300,
                asset_stem: "poppins-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Regular,
                numeric_weight: 400,
                asset_stem: "poppins-regular",
            },
            FontCatalogWeight {
                weight: TextWeight::Medium,
                numeric_weight: 500,
                asset_stem: "poppins-medium",
            },
            FontCatalogWeight {
                weight: TextWeight::Semibold,
                numeric_weight: 600,
                asset_stem: "poppins-semibold",
            },
            FontCatalogWeight {
                weight: TextWeight::Bold,
                numeric_weight: 700,
                asset_stem: "poppins-bold",
            },
            FontCatalogWeight {
                weight: TextWeight::Extrabold,
                numeric_weight: 800,
                asset_stem: "poppins-extrabold",
            },
            FontCatalogWeight {
                weight: TextWeight::Black,
                numeric_weight: 900,
                asset_stem: "poppins-extrabold",
            },
        ],
    }
    };
}

macro_rules! font_catalog_manrope_entry {
    () => {
FontCatalogEntry {
        token: FontFamily::Manrope,
        display_name: "Manrope",
        web_stack: "\"Dowe Manrope\",Manrope,system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif",
        ios_family_name: "Manrope",
        android_family_name: "Manrope",
        package_assets: true,
        weights: &[
            FontCatalogWeight {
                weight: TextWeight::Thin,
                numeric_weight: 100,
                asset_stem: "manrope-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Extralight,
                numeric_weight: 200,
                asset_stem: "manrope-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Light,
                numeric_weight: 300,
                asset_stem: "manrope-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Regular,
                numeric_weight: 400,
                asset_stem: "manrope-regular",
            },
            FontCatalogWeight {
                weight: TextWeight::Medium,
                numeric_weight: 500,
                asset_stem: "manrope-medium",
            },
            FontCatalogWeight {
                weight: TextWeight::Semibold,
                numeric_weight: 600,
                asset_stem: "manrope-semibold",
            },
            FontCatalogWeight {
                weight: TextWeight::Bold,
                numeric_weight: 700,
                asset_stem: "manrope-bold",
            },
            FontCatalogWeight {
                weight: TextWeight::Extrabold,
                numeric_weight: 800,
                asset_stem: "manrope-extrabold",
            },
            FontCatalogWeight {
                weight: TextWeight::Black,
                numeric_weight: 900,
                asset_stem: "manrope-extrabold",
            },
        ],
    }
    };
}

macro_rules! font_catalog_quicksand_entry {
    () => {
FontCatalogEntry {
        token: FontFamily::Quicksand,
        display_name: "Quicksand",
        web_stack: "\"Dowe Quicksand\",Quicksand,system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif",
        ios_family_name: "Quicksand",
        android_family_name: "Quicksand",
        package_assets: true,
        weights: &[
            FontCatalogWeight {
                weight: TextWeight::Thin,
                numeric_weight: 100,
                asset_stem: "quicksand-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Extralight,
                numeric_weight: 200,
                asset_stem: "quicksand-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Light,
                numeric_weight: 300,
                asset_stem: "quicksand-light",
            },
            FontCatalogWeight {
                weight: TextWeight::Regular,
                numeric_weight: 400,
                asset_stem: "quicksand-regular",
            },
            FontCatalogWeight {
                weight: TextWeight::Medium,
                numeric_weight: 500,
                asset_stem: "quicksand-medium",
            },
            FontCatalogWeight {
                weight: TextWeight::Semibold,
                numeric_weight: 600,
                asset_stem: "quicksand-semibold",
            },
            FontCatalogWeight {
                weight: TextWeight::Bold,
                numeric_weight: 700,
                asset_stem: "quicksand-bold",
            },
            FontCatalogWeight {
                weight: TextWeight::Extrabold,
                numeric_weight: 800,
                asset_stem: "quicksand-extrabold",
            },
            FontCatalogWeight {
                weight: TextWeight::Black,
                numeric_weight: 900,
                asset_stem: "quicksand-extrabold",
            },
        ],
    }
    };
}
