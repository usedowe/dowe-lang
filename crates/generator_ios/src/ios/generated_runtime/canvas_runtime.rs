fn swift_runtime_canvas() -> &'static str {
    static RUNTIME: OnceLock<String> = OnceLock::new();
    RUNTIME
        .get_or_init(|| {
            let mut output = concat!(
                include!("canvas_image_store.rs"),
                include!("canvas_view.rs"),
                include!("canvas_input_bridge.rs"),
            )
            .to_string();
            output.push_str(swift_runtime_game_raycast());
            output.push_str(swift_runtime_game_socket());
            output
        })
        .as_str()
}
