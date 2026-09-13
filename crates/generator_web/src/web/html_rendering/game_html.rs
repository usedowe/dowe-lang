fn append_game_attributes(
    extra: &mut String,
    game: &GameProps,
    context: &ReactiveRenderContext,
) {
    extra.push_str(&format!(
        r#" data-dowe-game data-dowe-game-reconnect="{}" data-dowe-game-reconnect-delay="{}""#,
        game.reconnect, game.reconnect_delay,
    ));
    if let Some(socket) = game.socket.as_deref() {
        let path = if game.socket_binding {
            context.signal_path(socket)
        } else {
            socket.to_string()
        };
        extra.push_str(&format!(
            r#" data-dowe-game-socket="{}"{}"#,
            escape_attr(&path),
            if game.socket_binding {
                r#" data-dowe-game-socket-binding="true""#
            } else {
                ""
            }
        ));
    }
    for (attribute, path) in [
        ("data-dowe-game-send", game.send.as_deref()),
        ("data-dowe-game-status", game.status.as_deref()),
    ] {
        if let Some(path) = path {
            extra.push_str(&format!(
                r#" {attribute}="{}""#,
                escape_attr(&context.signal_path(path))
            ));
        }
    }
    for (attribute, action) in [
        ("data-dowe-game-on-open", game.on_open.as_deref()),
        ("data-dowe-game-on-message", game.on_message.as_deref()),
        ("data-dowe-game-on-close", game.on_close.as_deref()),
        ("data-dowe-game-on-error", game.on_error.as_deref()),
        ("data-dowe-game-on-fire", game.on_fire.as_deref()),
    ] {
        if let Some(action) = action {
            extra.push_str(&format!(
                r#" {attribute}="{}""#,
                escape_attr(&context.action_id(action))
            ));
        }
    }
    extra.push_str(&format!(
        r#" data-dowe-game-renderer="{}" data-dowe-game-controls="{}" data-dowe-game-move-speed="{}" data-dowe-game-turn-speed="{}""#,
        game.renderer.as_str(),
        game.controls.as_str(),
        game.move_speed,
        game.turn_speed,
    ));
    for (attribute, path) in [
        ("data-dowe-game-world", game.world.as_deref()),
        ("data-dowe-game-camera", game.camera.as_deref()),
    ] {
        if let Some(path) = path {
            extra.push_str(&format!(
                r#" {attribute}="{}""#,
                escape_attr(&context.signal_path(path))
            ));
        }
    }
    if game.renderer.as_str() == "raycast3d" && game.on_key.is_none() {
        extra.push_str(r#" tabindex="0""#);
    }
}
