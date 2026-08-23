pub use dowe_stdlib::{StdlibArgument, StdlibCall, StdlibValue};

include!("component/model.rs");
include!("component/i18n.rs");
include!("component/code_highlighting.rs");
include!("component/typography.rs");
include!("component/style_tokens.rs");
include!("component/design_tokens.rs");
include!("component/design_defaults.rs");
include!("component/font_catalog.rs");
include!("component/registry.rs");
include!("component/font_collection.rs");
include!("component/icon_catalog.rs");
include!("component/phone_catalog.rs");
include!("component/error.rs");
include!("component/builders.rs");
include!("component/tree_validation.rs");
include!("component/style_parsing.rs");
include!("component/navigation_parsing.rs");
include!("component/value_parsing.rs");
include!("component/text_binding.rs");
include!("component/tree_helpers.rs");
include!("component/side_nav_memory.rs");

#[cfg(test)]
mod tests {
    include!("component/tests_registry_and_validation.rs");
    include!("component/tests_props_helpers.rs");
    include!("component/tests_props_defaults.rs");
    include!("component/tests_props_validation.rs");
    include!("component/tests_typography_and_svg.rs");
    include!("component/tests_navigation_and_tree.rs");
    include!("component/tests_tree_helpers.rs");
    include!("component/tests_form_validation.rs");
}
