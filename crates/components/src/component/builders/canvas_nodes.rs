pub fn canvas_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut scene = None;
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
    let mut on_motion = None;
    let mut motion_rate = 30;
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "scene" => scene = Some(parse_reference_path(&prop.name, &prop.value)?),
            "viewWidth" => view_width = parse_positive_u16(&prop.name, &prop.value)?,
            "viewHeight" => view_height = parse_positive_u16(&prop.name, &prop.value)?,
            "fit" => fit = parse_canvas_fit(&prop.name, &prop.value)?,
            "fps" => fps = parse_canvas_fps(&prop.name, &prop.value)?,
            "autoplay" => autoplay = parse_static_bool(&prop.name, &prop.value)?,
            "background" => background = parse_canvas_background(&prop.name, &prop.value)?,
            "pixelated" => pixelated = parse_static_bool(&prop.name, &prop.value)?,
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "onPointer" => on_pointer = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?),
            "onKey" => on_key = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?),
            "onMotion" => on_motion = Some(parse_signal_path(&prop.name, &prop.value, "signal path")?),
            "motionRate" => motion_rate = parse_canvas_motion_rate(&prop.name, &prop.value)?,
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

    Ok(ViewNode::Canvas {
        props: CanvasProps {
            style,
            scene: scene.ok_or_else(|| ComponentError::invalid_prop("scene", "signal array path"))?,
            view_width,
            view_height,
            fit,
            fps,
            autoplay,
            background,
            pixelated,
            label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty static string"))?,
            on_pointer,
            on_key,
            on_motion,
            motion_rate,
        },
    })
}

fn parse_canvas_motion_rate(name: &str, value: &PropValue) -> ComponentResult<u8> {
    match value {
        PropValue::Number(value) => value
            .parse::<u8>()
            .ok()
            .filter(|value| (1..=60).contains(value))
            .ok_or_else(|| ComponentError::invalid_prop(name, "integer from 1 through 60")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) | PropValue::Binding(_) => {
            Err(ComponentError::invalid_prop(name, "integer from 1 through 60"))
        }
    }
}

fn parse_canvas_fit(name: &str, value: &PropValue) -> ComponentResult<CanvasFit> {
    let value = parse_required_string(name, value)?;
    CanvasFit::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "contain, cover or stretch"))
}

fn parse_canvas_fps(name: &str, value: &PropValue) -> ComponentResult<u8> {
    match value {
        PropValue::Number(value) => value
            .parse::<u8>()
            .ok()
            .filter(|value| (1..=120).contains(value))
            .ok_or_else(|| ComponentError::invalid_prop(name, "integer from 1 through 120")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) | PropValue::Binding(_) => {
            Err(ComponentError::invalid_prop(name, "integer from 1 through 120"))
        }
    }
}

fn parse_canvas_background(name: &str, value: &PropValue) -> ComponentResult<CanvasBackground> {
    let value = parse_required_string(name, value)?;
    if value == "transparent" {
        return Ok(CanvasBackground::Transparent);
    }
    ColorToken::from_name(&value)
        .map(CanvasBackground::Color)
        .ok_or_else(|| ComponentError::invalid_prop(name, "transparent or color token"))
}
