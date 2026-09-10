fn android_runtime_foundation() -> &'static str {
    concat!(
        include!("foundation_imports_and_styles.rs"),
        include!("foundation_motion_and_gestures.rs"),
        include!("foundation_components.rs"),
    )
}
