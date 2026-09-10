fn swift_runtime_canvas() -> &'static str {
    concat!(
        include!("canvas_image_store.rs"),
        include!("canvas_view.rs"),
        include!("canvas_input_bridge.rs"),
    )
}
