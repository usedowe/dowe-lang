fn android_runtime_data_code_svg() -> &'static str {
    concat!(
        include!("data_code_svg_candlestick.rs"),
        include!("data_code_svg_charts.rs"),
        include!("data_code_svg_tables.rs"),
    )
}
