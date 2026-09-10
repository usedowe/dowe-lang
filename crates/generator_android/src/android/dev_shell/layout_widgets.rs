fn dev_activity_layout_widgets() -> &'static str {
    concat!(
        include!("layout_widgets_scaffold.rs"),
        include!("layout_widgets_navigation.rs"),
        include!("layout_widgets_interactions.rs"),
    )
}
