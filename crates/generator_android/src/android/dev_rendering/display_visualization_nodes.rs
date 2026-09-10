fn render_dev_android_visualization_display_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    _inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    _children_method: Option<&str>,
) -> bool {
    if !matches!(
        node,
        ViewNode::Canvas { .. }
            | ViewNode::Diagram { .. }
            | ViewNode::Candlestick { .. }
            | ViewNode::ArcChart { .. }
            | ViewNode::AreaChart { .. }
            | ViewNode::BarChart { .. }
            | ViewNode::LineChart { .. }
            | ViewNode::PieChart { .. }
    ) {
        return false;
    }
    match node {
        ViewNode::Canvas { props } => {
            let view = next_dev_view(counter);
            let on_pointer = props
                .on_pointer
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_key = props
                .on_key
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_motion = props
                .on_motion
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let layer_path = |value: Option<&String>| {
                value
                    .map(|value| format!("\"{}\"", escape_java(&context.signal_path(value))))
                    .unwrap_or_else(|| "null".to_string())
            };
            let action = |value: Option<&String>| {
                value
                    .and_then(|value| context.action_id(value))
                    .map(|value| format!("\"{}\"", escape_java(value)))
                    .unwrap_or_else(|| "null".to_string())
            };
            let draw_mode_path = if props.draw_mode_binding {
                layer_path(Some(&props.draw_mode))
            } else {
                "null".to_string()
            };
            let layers_path = layer_path(props.layer_bind.as_ref());
            let selected_path = layer_path(props.selected_layer.as_ref());
            let on_layer_add = action(props.on_layer_add.as_ref());
            let on_layer_change = action(props.on_layer_change.as_ref());
            let on_layer_remove = action(props.on_layer_remove.as_ref());
            let on_layer_select = action(props.on_layer_select.as_ref());
            let background = match props.background {
                CanvasBackground::Transparent => "Color.TRANSPARENT".to_string(),
                CanvasBackground::Color(color) => java_color(color).to_string(),
            };
            let border_width = props
                .style
                .border
                .as_ref()
                .map(dev_border_value)
                .unwrap_or_else(|| "null".to_string());
            let border_color = props
                .style
                .border_color
                .map(family_color)
                .map(java_color)
                .unwrap_or("DOWE_BACKGROUND_TEXT");
            output.push_str(&format!(
                "        DoweCanvasView {view} = doweCanvas(\"{}\", {}f, {}f, \"{}\", {}, {}, {}, {}, \"{}\", {on_pointer}, {on_key}, {on_motion}, {}, {}, \"{}\", {draw_mode_path}, {layers_path}, {selected_path}, {on_layer_add}, {on_layer_change}, {on_layer_remove}, {on_layer_select}, {border_width}, {border_color}, {});\n",
                escape_java(&context.signal_path(&props.scene)),
                props.view_width,
                props.view_height,
                props.fit.as_str(),
                props.fps,
                props.autoplay,
                props.pixelated,
                background,
                escape_java(&props.label),
                props.motion_rate,
                props.draw,
                escape_java(&props.draw_mode),
                dev_style_radius(&props.style),
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Diagram { props } => {
            let view = next_dev_view(counter);
            let on_node_click = props
                .on_node_click
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_node_drag = props
                .on_node_drag
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_connect = props
                .on_connect
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        DoweDiagramView {view} = doweDiagram(\"{}\", \"{}\", {}, {}, {}, {}, {}, {}, {}, {on_node_click}, {on_node_drag}, {on_connect}, DOWE_SURFACE, DOWE_BACKGROUND_TEXT, {});\n",
                escape_java(&context.signal_path(&props.nodes)),
                escape_java(&context.signal_path(&props.edges)),
                props.fit_view,
                props.pan_on_drag,
                props.zoom_on_scroll,
                props.controls,
                props.minimap,
                props.show_grid,
                format!("\"{}\"", escape_java(&props.empty_label)),
                dev_style_radius(&props.style.style),
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Candlestick { props } => {
            let view = next_dev_view(counter);
            let stream = props
                .stream
                .as_deref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                                        "        DoweCandlestickView {view} = doweCandlestick(\"{}\", {stream}, {}, {}, \"{}\", {}, {}, {}, {});\n",
                                        escape_java(&context.signal_path(&props.data)),
                                        java_color(props.up_color),
                                        java_color(props.down_color),
                                        escape_java(&props.empty_label),
                                        props.max_points,
                                        dev_card_variant_container(&props.style),
                                        dev_card_variant_content(&props.style),
                                        dev_card_border(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::ArcChart { props } => {
            render_dev_android_chart(
                "arc",
                &props.common,
                None,
                Some(props),
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::AreaChart { props } => {
            render_dev_android_chart(
                "area",
                &props.common,
                None,
                None,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::BarChart { props } => {
            render_dev_android_chart(
                "bar",
                &props.common,
                None,
                None,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::LineChart { props } => {
            render_dev_android_chart(
                "line",
                &props.common,
                None,
                None,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        ViewNode::PieChart { props } => {
            render_dev_android_chart(
                "pie",
                &props.common,
                Some(props),
                None,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                context,
                output,
            );
        }
        _ => {}
    }
    true
}
