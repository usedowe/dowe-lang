fn swift_runtime_layout_helpers() -> &'static str {
    concat!(
        include!("layout_responsive_and_flow.rs"),
        include!("layout_grid.rs"),
        include!("layout_docking.rs"),
    )
}
