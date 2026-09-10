fn swift_runtime_foundation() -> &'static str {
    concat!(
        include!("foundation_imports_and_keys.rs"),
        include!("foundation_design_types.rs"),
        include!("foundation_modifiers.rs"),
        include!("foundation_overlays.rs"),
    )
}
