#[allow(unused_variables)]
fn render_compose_display_iframe(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Iframe { props } = node else { return; };
    let pad = " ".repeat(indent);
            let sandbox = props
                .sandbox
                .as_ref()
                .map(|tokens| {
                    format!(
                        "listOf({})",
                        tokens
                            .iter()
                            .map(|token| compose_string_literal(token))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "{pad}DoweIframe(source = {}, title = {}, sandbox = {sandbox}, autoplay = {}, modifier = {}, shape = RoundedCornerShape({}))\n",
                compose_string_literal(&props.src),
                compose_string_literal(&props.title),
                props.allow.iter().any(|token| token == "autoplay"),
                modifier_for_style(&props.style),
                compose_card_radius(&props.style),
            ));
}

#[allow(unused_variables)]
fn render_compose_display_device(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Device { props, iframe } = node else { return; };
    let pad = " ".repeat(indent);
            let sandbox = iframe
                .sandbox
                .as_ref()
                .map(|tokens| {
                    format!(
                        "listOf({})",
                        tokens
                            .iter()
                            .map(|token| compose_string_literal(token))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            let icons = props
                .options
                .iter()
                .map(|option| {
                    format!(
                        "DoweDeviceIcon(profile = {}, viewBox = {}, paths = {})",
                        compose_string_literal(option.profile.as_str()),
                        compose_svg_view_box(&option.icon.props.view_box),
                        compose_svg_paths(&option.icon.paths),
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let bound_profile = props.bind.as_deref().map(|path| format!("state.text(\"{}\")", escape_kotlin(&context.signal_path(path)))).unwrap_or_else(|| compose_string_literal(props.device.as_str()));
            let bind_path = props.bind.as_deref().map(|path| compose_string_literal(&context.signal_path(path))).unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "{pad}DoweDevicePreview(initialProfile = {}, bind = {}, boundProfile = {}, onProfileChange = {}, source = {}, title = {}, sandbox = {sandbox}, autoplay = {}, hideControls = {}, icons = listOf({icons}), modifier = {})\n", 
                compose_string_literal(props.device.as_str()),
                bind_path,
                bound_profile,
                props.bind.as_deref().map(|path| format!("{{ value -> state.write(\"{}\", value) }}", escape_kotlin(&context.signal_path(path)))).unwrap_or_else(|| "{ _ -> }".to_string()),
                compose_string_literal(&iframe.src),
                compose_string_literal(&iframe.title),
                iframe.allow.iter().any(|token| token == "autoplay"),
                props.hide_controls,
                modifier_for_style(&props.style),
            ));
}

#[allow(unused_variables)]
fn render_compose_display_canvas(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Canvas { props } = node else { return; };
    let pad = " ".repeat(indent);
            let background = match props.background {
                CanvasBackground::Transparent => "Color.Transparent".to_string(),
                CanvasBackground::Color(color) => color_ref(color).to_string(),
            };
            let draw_mode_path = props
                .draw_mode_binding
                .then(|| context.signal_path(&props.draw_mode));
            let layer_bind = props
                .layer_bind
                .as_deref()
                .map(|path| context.signal_path(path));
            let selected_layer = props
                .selected_layer
                .as_deref()
                .map(|path| context.signal_path(path));
            output.push_str(&format!(
                "{pad}DoweCanvas(state = state, scenePath = {}, viewWidth = {}f, viewHeight = {}f, fit = {}, fps = {}, autoplay = {}, pixelated = {}, backgroundColor = {}, label = {}, onPointer = {}, onKey = {}, onMotion = {}, motionRate = {}, draw = {}, drawMode = {}, drawModePath = {}, layersPath = {}, selectedPath = {}, onLayerAdd = {}, onLayerChange = {}, onLayerRemove = {}, onLayerSelect = {}, modifier = {})\n",
                compose_string_literal(&context.signal_path(&props.scene)),
                props.view_width,
                props.view_height,
                compose_string_literal(props.fit.as_str()),
                props.fps,
                props.autoplay,
                props.pixelated,
                background,
                compose_string_literal(&props.label),
                compose_optional_string(props.on_pointer.as_deref().and_then(|value| context.action_id(value))),
                compose_optional_string(props.on_key.as_deref().and_then(|value| context.action_id(value))),
                compose_optional_string(props.on_motion.as_deref().and_then(|value| context.action_id(value))),
                props.motion_rate,
                props.draw,
                compose_string_literal(&props.draw_mode),
                compose_optional_string(draw_mode_path.as_deref()),
                compose_optional_string(layer_bind.as_deref()),
                compose_optional_string(selected_layer.as_deref()),
                compose_optional_string(props.on_layer_add.as_deref().and_then(|value| context.action_id(value))),
                compose_optional_string(props.on_layer_change.as_deref().and_then(|value| context.action_id(value))),
                compose_optional_string(props.on_layer_remove.as_deref().and_then(|value| context.action_id(value))),
                compose_optional_string(props.on_layer_select.as_deref().and_then(|value| context.action_id(value))),
                modifier_for_style(&props.style),
            ));
}

#[allow(unused_variables)]
fn render_compose_display_diagram(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Diagram { props } = node else { return; };
    let pad = " ".repeat(indent);
            output.push_str(&format!(
                        "{pad}DoweDiagram(state = state, nodesPath = {}, edgesPath = {}, fitView = {}, panOnDrag = {}, zoomOnScroll = {}, controls = {}, minimap = {}, showGrid = {}, emptyLabel = {}, onNodeClick = {}, onNodeDrag = {}, onConnect = {}, backgroundColor = {}, contentColor = {}, modifier = {})\n",
                        compose_string_literal(&context.signal_path(&props.nodes)),
                        compose_string_literal(&context.signal_path(&props.edges)),
                        props.fit_view,
                        props.pan_on_drag,
                        props.zoom_on_scroll,
                        props.controls,
                        props.minimap,
                        props.show_grid,
                        compose_string_literal(&props.empty_label),
                        compose_optional_string(props.on_node_click.as_deref().and_then(|value| context.action_id(value))),
                        compose_optional_string(props.on_node_drag.as_deref().and_then(|value| context.action_id(value))),
                        compose_optional_string(props.on_connect.as_deref().and_then(|value| context.action_id(value))),
                        card_variant_container(&props.style),
                        card_variant_content(&props.style),
                        modifier_for_style(&props.style.style),
                    ));
}

#[allow(unused_variables)]
fn render_compose_display_candlestick(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Candlestick { props } = node else { return; };
    let pad = " ".repeat(indent);
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                card_variant_content(&props.style)
            } else {
                "null"
            };
            output.push_str(&format!(
                        "{pad}DoweCandlestick(state = state, dataPath = {}, stream = {}, upColor = {}, downColor = {}, emptyLabel = {}, maxPoints = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
                        compose_string_literal(&context.signal_path(&props.data)),
                        compose_optional_string(props.stream.as_deref()),
                        color_ref(props.up_color),
                        color_ref(props.down_color),
                        compose_string_literal(&props.empty_label),
                        props.max_points,
                        modifier_for_style(&props.style.style),
                        compose_card_radius(&props.style.style),
                        card_variant_container(&props.style),
                        card_variant_content(&props.style),
                    ));
}

#[allow(unused_variables)]
fn render_compose_display_arc_chart(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::ArcChart { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_chart(
                "arc",
                &props.common,
                None,
                Some(props),
                indent,
                output,
                context,
            );
}

#[allow(unused_variables)]
fn render_compose_display_area_chart(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::AreaChart { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_chart("area", &props.common, None, None, indent, output, context);
}

#[allow(unused_variables)]
fn render_compose_display_bar_chart(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::BarChart { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_chart("bar", &props.common, None, None, indent, output, context);
}

#[allow(unused_variables)]
fn render_compose_display_line_chart(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::LineChart { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_chart("line", &props.common, None, None, indent, output, context);
}

#[allow(unused_variables)]
fn render_compose_display_pie_chart(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::PieChart { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_chart(
                "pie",
                &props.common,
                Some(props),
                None,
                indent,
                output,
                context,
            );
}

#[allow(unused_variables)]
fn render_compose_display_table(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Table { props } = node else { return; };
    let pad = " ".repeat(indent);
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                table_variant_content(&props.style)
            } else {
                "null"
            };
            output.push_str(&format!(
                        "{pad}DoweTable(state = state, dataPath = {}, columns = {}, size = {}, striped = {}, bordered = {}, dividers = {}, emptyTitle = {}, emptyDescription = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
                        compose_string_literal(&context.signal_path(&props.data)),
                        compose_table_columns(&props.columns),
                        compose_table_size(props.size),
                        props.striped,
                        props.bordered,
                        props.dividers,
                        compose_string_literal(&props.empty_title),
                        compose_string_literal(&props.empty_description),
                        modifier_for_style(&props.style.style),
                        compose_card_radius(&props.style.style),
                        table_variant_container(&props.style),
                        table_variant_content(&props.style),
                    ));
}

