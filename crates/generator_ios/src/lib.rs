pub const VIEW_IR_SCHEMA_VERSION: u32 = dowe_components::VIEW_IR_SCHEMA_VERSION;

include!("ios/artifacts.rs");
include!("ios/prop_consumption.rs");
include!("ios/dynamic_icon_catalog.rs");
include!("ios/generated_runtime.rs");
include!("ios/reactive_runtime.rs");
include!("ios/reactive_lowering.rs");
include!("ios/layout_partitioning.rs");
include!("ios/layout_sections.rs");
include!("ios/route_sections.rs");
include!("ios/swift_rendering.rs");
include!("ios/swift_modifiers.rs");
include!("ios/swift_values_and_text.rs");
include!("ios/design_tokens_and_names.rs");

#[cfg(test)]
mod tests {
    include!("ios/tests.rs");
}
