pub fn diagram_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut nodes = None;
    let mut edges = None;
    let mut fit_view = true;
    let mut pan_on_drag = true;
    let mut zoom_on_scroll = true;
    let mut minimap = false;
    let mut controls = true;
    let mut show_grid = true;
    let mut empty_label = "No nodes yet".to_string();
    let mut on_node_click = None;
    let mut on_node_drag = None;
    let mut on_connect = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "nodes" => nodes = Some(parse_reference_path(&prop.name, &prop.value)?),
            "edges" => edges = Some(parse_reference_path(&prop.name, &prop.value)?),
            "fitView" => fit_view = parse_static_bool(&prop.name, &prop.value)?,
            "panOnDrag" => pan_on_drag = parse_static_bool(&prop.name, &prop.value)?,
            "zoomOnScroll" => zoom_on_scroll = parse_static_bool(&prop.name, &prop.value)?,
            "minimap" => minimap = parse_static_bool(&prop.name, &prop.value)?,
            "controls" => controls = parse_static_bool(&prop.name, &prop.value)?,
            "showGrid" => show_grid = parse_static_bool(&prop.name, &prop.value)?,
            "emptyLabel" => empty_label = parse_required_string(&prop.name, &prop.value)?,
            "onNodeClick" => on_node_click = Some(parse_required_string(&prop.name, &prop.value)?),
            "onNodeDrag" => on_node_drag = Some(parse_required_string(&prop.name, &prop.value)?),
            "onConnect" => on_connect = Some(parse_required_string(&prop.name, &prop.value)?),
            _ if is_chart_style_prop(&prop.name) => style_props.push(prop),
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::Diagram, &prop.name)),
        }
    }
    let nodes = nodes.ok_or_else(|| ComponentError::invalid_prop("nodes", "signal array path"))?;
    let edges = edges.ok_or_else(|| ComponentError::invalid_prop("edges", "signal array path"))?;
    let mut style = parse_variant_props(BuiltinComponent::Diagram, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    if style.style.sizing.h.is_none() {
        style.style.sizing.h = Some(ResponsiveValue::scalar(SizeValue::Scale(
            ScaleValue::from_half_steps(150),
        )));
    }
    Ok(ViewNode::Diagram {
        props: DiagramProps {
            style,
            nodes,
            edges,
            fit_view,
            pan_on_drag,
            zoom_on_scroll,
            minimap,
            controls,
            show_grid,
            empty_label,
            on_node_click,
            on_node_drag,
            on_connect,
        },
    })
}
