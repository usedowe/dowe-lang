pub fn canvas_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    canvas_component_node_for(BuiltinComponent::Canvas, props)
}

pub fn draw_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    canvas_component_node_for(BuiltinComponent::Draw, props)
}

fn canvas_component_node_for(
    component: BuiltinComponent,
    props: Vec<ComponentProp>,
) -> ComponentResult<ViewNode> {
    let mut scene = None;
    let mut layer_bind = None;
    let mut selected_layer = None;
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
    let mut draw = false;
    let mut draw_mode = DrawMode::Pen.as_str().to_string();
    let mut draw_mode_binding = false;
    let mut on_layer_add = None;
    let mut on_layer_change = None;
    let mut on_layer_remove = None;
    let mut on_layer_select = None;
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "scene" => scene = Some(parse_reference_path(&prop.name, &prop.value)?),
            "bind" if component == BuiltinComponent::Draw => {
                layer_bind = Some(parse_reference_path(&prop.name, &prop.value)?)
            }
            "selected" if component == BuiltinComponent::Draw => {
                selected_layer = Some(parse_signal_path(&prop.name, &prop.value, "signal string path")?)
            }
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
            "draw" => draw = parse_static_bool(&prop.name, &prop.value)?,
            "drawMode" => match &prop.value {
                PropValue::Binding(binding) => {
                    if !is_reference_path(&binding.path) {
                        return Err(ComponentError::invalid_prop(&prop.name, "pen, rect, circle or signal string path"));
                    }
                    draw_mode = binding.path.clone();
                    draw_mode_binding = true;
                }
                _ => draw_mode = parse_draw_mode(&prop.name, &prop.value, component)?.as_str().to_string(),
            },
            "onLayerAdd" if component == BuiltinComponent::Draw => {
                on_layer_add = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "onLayerChange" if component == BuiltinComponent::Draw => {
                on_layer_change = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "onLayerRemove" if component == BuiltinComponent::Draw => {
                on_layer_remove = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "onLayerSelect" if component == BuiltinComponent::Draw => {
                on_layer_select = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            _ => style_props.push(prop),
        }
    }

    let scene = scene.or_else(|| layer_bind.clone()).ok_or_else(|| {
        ComponentError::invalid_prop("scene", "signal array path or Draw bind signal array path")
    })?;
    if component != BuiltinComponent::Draw
        && (layer_bind.is_some()
            || selected_layer.is_some()
            || on_layer_add.is_some()
            || on_layer_change.is_some()
            || on_layer_remove.is_some()
            || on_layer_select.is_some())
    {
        return Err(ComponentError::invalid_prop_combination(
            "layer props are only supported by Draw",
        ));
    }
    if let Some(layer_bind) = layer_bind.as_deref() {
        if layer_bind != scene {
            return Err(ComponentError::invalid_prop_combination(
                "Draw scene and bind must reference the same Signal array",
            ));
        }
    }
    if component == BuiltinComponent::Draw && layer_bind.is_some() {
        draw = true;
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
            is_draw: component == BuiltinComponent::Draw,
            scene,
            layer_bind,
            selected_layer,
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
            draw,
            draw_mode,
            draw_mode_binding,
            on_layer_add,
            on_layer_change,
            on_layer_remove,
            on_layer_select,
        },
    })
}

fn parse_draw_mode(
    name: &str,
    value: &PropValue,
    component: BuiltinComponent,
) -> ComponentResult<DrawMode> {
    let value = parse_required_string(name, value)?;
    DrawMode::from_name(&value)
        .filter(|mode| {
            component == BuiltinComponent::Draw
                || !matches!(mode, DrawMode::Select | DrawMode::Erase)
        })
        .ok_or_else(|| {
            ComponentError::invalid_prop(
                name,
                if component == BuiltinComponent::Draw {
                    "pen, rect, circle, select or erase"
                } else {
                    "pen, rect or circle"
                },
            )
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
