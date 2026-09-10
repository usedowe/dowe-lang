#[allow(unused_variables)]
fn render_compose_display_countdown(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Countdown { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_countdown(props, indent, output, context);
}

#[allow(unused_variables)]
fn render_compose_display_map(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Map { props, markers, waypoints, } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_map(props, markers, waypoints, indent, output, context);
}

#[allow(unused_variables)]
fn render_compose_display_divider(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Divider { props } = node else { return; };
    let pad = " ".repeat(indent);
            output.push_str(&format!(
                "{pad}Box(modifier = {}.background({}))\n",
                modifier_for_divider(props, flow),
                color_ref(family_color(props.color))
            ));
}

#[allow(unused_variables)]
fn render_compose_display_title(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Title { props, value } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_text(
                true,
                props,
                value,
                props.style.font.as_ref().or(inherited_font),
                indent,
                output,
                default_family,
                context,
            );
}

#[allow(unused_variables)]
fn render_compose_display_text(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Text { props, value } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_text(
                false,
                props,
                value,
                props.style.font.as_ref().or(inherited_font),
                indent,
                output,
                default_family,
                context,
            );
}

#[allow(unused_variables)]
fn render_compose_display_alert(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Alert { props } = node else { return; };
    let pad = " ".repeat(indent);
            if let Some(visible) = props.visible.as_deref() {
                output.push_str(&format!(
                    "{pad}if (state.bool(\"{}\")) {{\n",
                    escape_kotlin(&context.signal_path(visible))
                ));
            }
            let alert_pad = if props.visible.is_some() {
                format!("{pad}    ")
            } else {
                pad.clone()
            };
            let shape = compose_control_radius(&props.style.style);
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                format!(
                    ".border(1.dp, {}, RoundedCornerShape({shape}))",
                    variant_content(&props.style)
                )
            } else {
                String::new()
            };
            output.push_str(&format!(
                        "{alert_pad}Row(modifier = {}.clip(RoundedCornerShape({shape})).background({}){border}.padding(horizontal = 14.dp, vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {{\n",
                        modifier_for_style(&props.style.style),
                        variant_container(&props.style)
                    ));
            output.push_str(&format!(
                        "{alert_pad}    Text({}, modifier = Modifier.weight(1f), color = {}, fontFamily = {})\n",
                        compose_text_expression(&props.message, None, context),
                        variant_content(&props.style),
                        compose_font_value(
                            props.style.style.font.as_ref().or(inherited_font),
                            default_family
                        )
                    ));
            if let Some(action) = props
                .on_close
                .as_deref()
                .and_then(|name| context.action_id(name))
            {
                output.push_str(&format!(
                            "{alert_pad}    Button(onClick = {{ actionScope.launch {{ state.run(\"{}\") }} }}, contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp)) {{ Text(\"x\") }}\n",
                            escape_kotlin(action)
                        ));
            }
            output.push_str(&format!("{alert_pad}}}\n"));
            if props.visible.is_some() {
                output.push_str(&format!("{pad}}}\n"));
            }
}

#[allow(unused_variables)]
fn render_compose_display_svg(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Svg { props, paths } = node else { return; };
    let pad = " ".repeat(indent);
            if let Some(data) = props.data.as_deref() {
                let payload = if let Some(item) = context.item_value(data) {
                    let path = context.item_path(data).unwrap_or_else(|| data.to_string());
                    format!("state.json(\"{}\", {item})", escape_kotlin(&path))
                } else {
                    format!(
                        "state.json(\"{}\")",
                        escape_kotlin(&context.signal_path(data))
                    )
                };
                output.push_str(&format!(
                    "{pad}DoweRuntimeSvg(payload = {payload}, modifier = {}, color = {}, animated = {})\n",
                    compose_svg_modifier(props),
                    compose_svg_color(&props.style),
                    props.is_animated()
                ));
                return;
            }
            if let Some(name) = props.icon_name.as_deref() {
                let name = if let Some(item) = context.item_value(name) {
                    let path = context.item_path(name).unwrap_or_else(|| name.to_string());
                    format!("state.text(\"{}\", {item})", escape_kotlin(&path))
                } else {
                    format!(
                        "state.text(\"{}\")",
                        escape_kotlin(&context.signal_path(name))
                    )
                };
                let fallback =
                    compose_string_literal(props.icon_fallback.as_deref().unwrap_or_default());
                let color = props
                    .icon_fill
                    .or(props.icon_stroke)
                    .map(color_ref)
                    .map(str::to_string)
                    .unwrap_or_else(|| compose_svg_color(&props.style));
                output.push_str(&format!(
                    "{pad}DoweDynamicIcon(name = {name}, fallback = {fallback}, modifier = {}, color = {}, animated = {})\n",
                    modifier_for_style(&props.style),
                    color,
                    props.is_animated()
                ));
                return;
            }
            output.push_str(&format!(
                "{pad}DoweSvg(viewBox = {}, modifier = {}, color = {}, paths = {}, animated = {})\n",
                compose_svg_view_box(&props.view_box),
                compose_svg_modifier(props),
                compose_svg_color(&props.style),
                compose_svg_paths(paths),
                props.is_animated()
            ));
}


fn compose_video_icons() -> String {
    let icon = |name: &str| {
        let icon = solar_control_icon(name).expect("bundled Video control icon");
        format!(
            "DoweVideoIcon(viewBox = {}, paths = {})",
            compose_svg_view_box(&icon.props.view_box),
            compose_svg_paths(&icon.paths)
        )
    };
    format!(
        "DoweVideoIcons(play = {}, pause = {}, volume = {}, muted = {}, pictureInPicture = {}, fullscreen = {})",
        icon("play"),
        icon("pause"),
        icon("volume-loud"),
        icon("volume-cross"),
        icon("pip"),
        icon("full-screen")
    )
}

fn render_compose_chart(
    chart_type: &str,
    props: &ChartCommonProps,
    pie_props: Option<&PieChartProps>,
    arc_props: Option<&ArcChartProps>,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            card_variant_content(&props.style)
        } else {
            "null"
        };
    let data_path = props
        .data
        .as_deref()
        .map(|value| compose_string_literal(&context.signal_path(value)))
        .unwrap_or_else(|| "null".to_string());
    let series_path = props
        .series
        .as_deref()
        .map(|value| compose_string_literal(&context.signal_path(value)))
        .unwrap_or_else(|| "null".to_string());
    let pie_args = pie_props
        .map(|pie| {
            format!(
                ", donut = {}, donutWidth = {}, centerLabel = {}, centerValue = {}, startAngle = {}, padAngle = {}, hideLabels = {}, hideValues = {}, hidePercentages = {}, showGlow = {}",
                pie.donut,
                pie.donut_width,
                compose_optional_string(pie.center_label.as_deref()),
                compose_optional_string(pie.center_value.as_deref()),
                pie.start_angle,
                pie.pad_angle,
                pie.hide_labels,
                pie.hide_values,
                pie.hide_percentages,
                pie.show_glow,
            )
        })
        .or_else(|| {
            arc_props.map(|arc| {
                format!(", startAngle = {}f", arc.start_angle)
            })
        })
        .unwrap_or_default();
    let arc_args = arc_props
        .map(|arc| {
            format!(
                ", centerText = {}, centerValue = {}, thickness = {}, gap = {}, endAngle = {}, showInlineLabels = {}, arcHideValues = {}, arcShowGlow = {}",
                compose_optional_string(arc.center_text.as_deref()),
                compose_optional_string(arc.center_value.as_deref()),
                arc.thickness,
                arc.gap,
                arc.end_angle,
                arc.show_inline_labels,
                arc.hide_values,
                arc.show_glow,
            )
        })
        .unwrap_or_default();
    output.push_str(&format!(
        "{pad}DoweChart(state = state, chartType = {}, dataPath = {}, seriesPath = {}, palette = {}, legendPosition = {}, emptyLabel = {}, loading = {}, hideLegend = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border}{pie_args}{arc_args})\n",
        compose_string_literal(chart_type),
        data_path,
        series_path,
        compose_string_literal(props.palette.as_str()),
        compose_string_literal(props.legend_position.as_str()),
        compose_string_literal(&props.empty_label),
        props.loading,
        props.hide_legend,
        modifier_for_style(&props.style.style),
        compose_card_radius(&props.style.style),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
    ));
}

