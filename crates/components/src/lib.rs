pub use dowe_stdlib::{StdlibArgument, StdlibCall, StdlibValue};

include!("component/model.rs");
include!("component/i18n.rs");
include!("component/code_highlighting.rs");
include!("component/typography.rs");
include!("component/design_tokens.rs");
include!("component/style_tokens.rs");
include!("component/font_catalog.rs");
include!("component/registry.rs");
include!("component/font_collection.rs");
include!("component/error.rs");
include!("component/builders.rs");
include!("component/tree_validation.rs");
include!("component/style_parsing.rs");
include!("component/navigation_parsing.rs");
include!("component/value_parsing.rs");
include!("component/tree_helpers.rs");

#[cfg(test)]
mod tests {
    include!("component/tests_registry_and_validation.rs");
    include!("component/tests_props_and_tree.rs");
}
