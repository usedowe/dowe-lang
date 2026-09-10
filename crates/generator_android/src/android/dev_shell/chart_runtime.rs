fn dev_activity_chart_runtime() -> &'static str {
    concat!(
        include!("chart_types_and_factory.rs"),
        include!("chart_view.rs"),
        include!("chart_data_helpers.rs"),
    )
}
