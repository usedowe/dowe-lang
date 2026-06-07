include!("ios/artifacts.rs");
include!("ios/generated_runtime.rs");
include!("ios/reactive_runtime.rs");
include!("ios/reactive_lowering.rs");
include!("ios/layout_partitioning.rs");
include!("ios/layout_sections.rs");
include!("ios/swift_rendering.rs");
include!("ios/swift_modifiers.rs");
include!("ios/swift_values_and_text.rs");
include!("ios/design_tokens_and_names.rs");

#[cfg(test)]
mod tests {
    include!("ios/tests.rs");
}
