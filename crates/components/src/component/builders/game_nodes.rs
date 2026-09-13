pub fn game_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut scene = None;
    let mut renderer = GameRenderer::Canvas2d;
    let mut world = None;
    let mut camera = None;
    let mut controls = GameControls::None;
    let mut move_speed = 2;
    let mut turn_speed = 120;
    let mut view_width = 320;
    let mut view_height = 180;
    let mut fit = CanvasFit::Contain;
    let mut fps = 60;
    let mut autoplay = true;
    let mut background = CanvasBackground::Transparent;
    let mut pixelated = false;
    let mut label = None;
    let mut on_pointer = None;
    let mut on_key = None;
    let mut on_fire = None;
    let mut on_motion = None;
    let mut motion_rate = 30;
    let mut socket = None;
    let mut socket_binding = false;
    let mut send = None;
    let mut status = None;
    let mut on_open = None;
    let mut on_message = None;
    let mut on_close = None;
    let mut on_error = None;
    let mut reconnect = true;
    let mut reconnect_delay = 1000;
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "scene" => scene = Some(parse_reference_path(&prop.name, &prop.value)?),
            "renderer" => renderer = parse_game_renderer(&prop.name, &prop.value)?,
            "world" => world = Some(parse_reference_path(&prop.name, &prop.value)?),
            "camera" => camera = Some(parse_reference_path(&prop.name, &prop.value)?),
            "controls" => controls = parse_game_controls(&prop.name, &prop.value)?,
            "moveSpeed" => move_speed = parse_game_move_speed(&prop.name, &prop.value)?,
            "turnSpeed" => turn_speed = parse_game_turn_speed(&prop.name, &prop.value)?,
            "viewWidth" => view_width = parse_positive_u16(&prop.name, &prop.value)?,
            "viewHeight" => view_height = parse_positive_u16(&prop.name, &prop.value)?,
            "fit" => fit = parse_canvas_fit(&prop.name, &prop.value)?,
            "fps" => fps = parse_canvas_fps(&prop.name, &prop.value)?,
            "autoplay" => autoplay = parse_static_bool(&prop.name, &prop.value)?,
            "background" => background = parse_canvas_background(&prop.name, &prop.value)?,
            "pixelated" => pixelated = parse_static_bool(&prop.name, &prop.value)?,
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "onPointer" => {
                on_pointer = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?)
            }
            "onKey" => on_key = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?),
            "onFire" => on_fire = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?),
            "onMotion" => {
                on_motion = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?)
            }
            "motionRate" => motion_rate = parse_canvas_motion_rate(&prop.name, &prop.value)?,
            "socket" => match &prop.value {
                PropValue::Binding(binding) if is_reference_path(&binding.path) => {
                    socket = Some(binding.path.clone());
                    socket_binding = true;
                }
                _ => {
                    let value = parse_required_string(&prop.name, &prop.value)?;
                    if !valid_game_socket(&value) {
                        return Err(ComponentError::invalid_prop(
                            &prop.name,
                            "ws/wss URL or absolute backend path",
                        ));
                    }
                    socket = Some(value);
                }
            },
            "send" => send = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?),
            "status" => {
                status = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "writable signal string path",
                )?)
            }
            "onOpen" => on_open = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?),
            "onMessage" => {
                on_message = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?)
            }
            "onClose" => {
                on_close = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?)
            }
            "onError" => {
                on_error = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?)
            }
            "reconnect" => reconnect = parse_static_bool(&prop.name, &prop.value)?,
            "reconnectDelay" => {
                reconnect_delay = parse_game_reconnect_delay(&prop.name, &prop.value)?
            }
            _ => style_props.push(prop),
        }
    }

    let mut style = parse_style_props(BuiltinComponent::Canvas, &style_props, StylePropMode::Box)?;
    if style.sizing.w.is_none() {
        style.sizing.w = Some(ResponsiveValue::scalar(SizeValue::Full));
    }
    if style.sizing.h.is_none() {
        style.sizing.h = Some(ResponsiveValue::scalar(SizeValue::Scale(
            ScaleValue::from_half_steps(96),
        )));
    }

    match renderer {
        GameRenderer::Canvas2d => {
            if scene.is_none() {
                return Err(ComponentError::invalid_prop(
                    "scene",
                    "signal array path required by renderer canvas2d",
                ));
            }
            if world.is_some()
                || camera.is_some()
                || controls != GameControls::None
                || on_fire.is_some()
            {
                return Err(ComponentError::invalid_prop_combination(
                    "world, camera, controls and onFire require renderer raycast3d",
                ));
            }
        }
        GameRenderer::Raycast3d => {
            if world.is_none() {
                return Err(ComponentError::invalid_prop(
                    "world",
                    "signal object path required by renderer raycast3d",
                ));
            }
            if camera.is_none() {
                return Err(ComponentError::invalid_prop(
                    "camera",
                    "signal object path required by renderer raycast3d",
                ));
            }
            if scene.is_some() {
                return Err(ComponentError::invalid_prop_combination(
                    "scene is only supported by renderer canvas2d",
                ));
            }
            if on_motion.is_some() {
                return Err(ComponentError::invalid_prop_combination(
                    "onMotion is only supported by renderer canvas2d",
                ));
            }
        }
    }

    Ok(ViewNode::Game {
        props: GameProps {
            style,
            renderer,
            scene,
            world,
            camera,
            controls,
            move_speed,
            turn_speed,
            view_width,
            view_height,
            fit,
            fps,
            autoplay,
            background,
            pixelated,
            label: label
                .ok_or_else(|| ComponentError::invalid_prop("label", "non-empty static string"))?,
            on_pointer,
            on_key,
            on_fire,
            on_motion,
            motion_rate,
            socket,
            socket_binding,
            send,
            status,
            on_open,
            on_message,
            on_close,
            on_error,
            reconnect,
            reconnect_delay,
        },
    })
}

fn parse_game_renderer(name: &str, value: &PropValue) -> ComponentResult<GameRenderer> {
    let value = parse_required_string(name, value)?;
    GameRenderer::from_name(&value).ok_or_else(|| ComponentError::invalid_prop(name, "canvas2d or raycast3d"))
}

fn parse_game_controls(name: &str, value: &PropValue) -> ComponentResult<GameControls> {
    let value = parse_required_string(name, value)?;
    GameControls::from_name(&value).ok_or_else(|| ComponentError::invalid_prop(name, "none or doom"))
}

fn parse_game_move_speed(name: &str, value: &PropValue) -> ComponentResult<u16> {
    parse_game_speed(name, value, 1..=20, "integer from 1 through 20")
}

fn parse_game_turn_speed(name: &str, value: &PropValue) -> ComponentResult<u16> {
    parse_game_speed(name, value, 1..=360, "integer from 1 through 360")
}

fn parse_game_speed(
    name: &str,
    value: &PropValue,
    range: std::ops::RangeInclusive<u16>,
    expected: &str,
) -> ComponentResult<u16> {
    match value {
        PropValue::Number(value) => value
            .parse::<u16>()
            .ok()
            .filter(|value| range.contains(value))
            .ok_or_else(|| ComponentError::invalid_prop(name, expected)),
        PropValue::String(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(name, expected)),
    }
}

fn parse_game_reconnect_delay(name: &str, value: &PropValue) -> ComponentResult<u16> {
    match value {
        PropValue::Number(value) => value
            .parse::<u16>()
            .ok()
            .filter(|value| (100..=60_000).contains(value))
            .ok_or_else(|| ComponentError::invalid_prop(name, "integer from 100 through 60000")),
        PropValue::String(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(
            name,
            "integer from 100 through 60000",
        )),
    }
}

fn valid_game_socket(value: &str) -> bool {
    if value.is_empty() || value.chars().any(char::is_whitespace) || value.contains('\\') {
        return false;
    }
    if value.starts_with("ws://") || value.starts_with("wss://") {
        return value.len() > 6 && !value[6..].starts_with('/');
    }
    if !value.starts_with('/') || value.starts_with("//") || value.contains("://") {
        return false;
    }
    let path = value.split(['?', '#']).next().unwrap_or_default();
    !path.split('/').any(|segment| matches!(segment, "." | ".."))
}
