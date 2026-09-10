fn swift_runtime_navigation_controls() -> &'static str {
    concat!(
        include!("navigation_controls_tabs.rs"),
        include!("navigation_controls_side_nav.rs"),
        include!("navigation_controls_menu.rs"),
    )
}
