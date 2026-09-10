fn swift_runtime_overlays() -> &'static str {
    concat!(
        include!("overlays_presenters.rs"),
        include!("overlays_modals.rs"),
        include!("overlays_floating_controls.rs"),
    )
}
