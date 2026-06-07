include!("android/artifacts.rs");
include!("android/generated_views.rs");
include!("android/reactive_runtime.rs");
include!("android/reactive_lowering.rs");
include!("android/dev_layout_partitioning.rs");
include!("android/dev_shell.rs");
include!("android/dev_reactive_lowering.rs");
include!("android/dev_java_runtime.rs");
include!("android/dev_rendering.rs");
include!("android/compose_rendering.rs");
include!("android/compose_styles.rs");
include!("android/compose_values_and_text.rs");
include!("android/design_tokens_and_names.rs");
include!("android/escaping.rs");

#[cfg(test)]
mod tests {
    include!("android/tests.rs");
}
