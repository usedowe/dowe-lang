fn swift_runtime_svg_runtime() -> &'static str {
    concat!(
        include!("svg_value_types.rs"),
        include!("svg_shape_rendering.rs"),
        include!("svg_view_rendering.rs"),
    )
}
