pub const VIEW_IR_SCHEMA_VERSION: u32 = dowe_components::VIEW_IR_SCHEMA_VERSION;

include!("web/inspector.rs");
include!("web/prop_consumption.rs");
include!("web/artifacts.rs");
include!("web/routes_and_css_collection.rs");
include!("web/css_rules.rs");
include!("web/html_rendering.rs");
include!("web/html_classes_and_escape.rs");

#[cfg(test)]
mod tests {
    include!("web/tests.rs");
}
