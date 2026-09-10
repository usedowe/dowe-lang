fn swift_runtime_empty_motion_text() -> &'static str {
    concat!(
        include!("empty_state.rs"),
        include!("motion_marquee_and_typewriter.rs"),
        include!("rich_text.rs"),
        include!("record_view.rs"),
    )
}
