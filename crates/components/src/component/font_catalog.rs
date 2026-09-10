include!("font_catalog_system_and_sans.rs");
include!("font_catalog_editorial.rs");

pub const FONT_CATALOG: &[FontCatalogEntry] = &[
    font_catalog_system_entry!(),
    font_catalog_inter_entry!(),
    font_catalog_roboto_entry!(),
    font_catalog_montserrat_entry!(),
    font_catalog_lato_entry!(),
    font_catalog_poppins_entry!(),
    font_catalog_manrope_entry!(),
    font_catalog_quicksand_entry!(),
    font_catalog_lora_entry!(),
    font_catalog_syne_entry!(),
    font_catalog_jost_entry!(),
    font_catalog_puritan_entry!(),
];
