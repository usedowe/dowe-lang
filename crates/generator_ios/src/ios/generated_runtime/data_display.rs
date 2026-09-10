fn swift_runtime_data_display() -> &'static str {
    concat!(
        include!("data_display_market.rs"),
        include!("data_display_charts.rs"),
        include!("data_display_interaction.rs"),
    )
}
