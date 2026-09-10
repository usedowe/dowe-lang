fn dev_activity_svg_parser() -> &'static str {
    concat!(
        include!("svg_runtime_view.rs"),
        include!("svg_path_parser.rs"),
    )
}
