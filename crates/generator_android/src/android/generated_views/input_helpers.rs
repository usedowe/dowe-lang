fn android_runtime_input_helpers() -> &'static str {
    concat!(
        include!("input_helpers_state.rs"),
        include!("input_helpers_controls.rs"),
        include!("input_helpers_validation.rs"),
        include!("input_helpers_shared.rs"),
    )
}
