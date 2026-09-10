fn render_dev_android_chart(
    chart_type: &str,
    props: &ChartCommonProps,
    pie_props: Option<&PieChartProps>,
    arc_props: Option<&ArcChartProps>,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    context: &ComposeReactiveContext,
    output: &mut String,
) {
    let view = next_dev_view(counter);
    let data_path = props
        .data
        .as_deref()
        .map(|value| format!("\"{}\"", escape_java(&context.signal_path(value))))
        .unwrap_or_else(|| "null".to_string());
    let series_path = props
        .series
        .as_deref()
        .map(|value| format!("\"{}\"", escape_java(&context.signal_path(value))))
        .unwrap_or_else(|| "null".to_string());
    let pie_args = pie_props
        .map(|pie| {
            format!(
                ", {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
                pie.donut,
                pie.donut_width,
                pie.center_label
                    .as_deref()
                    .map(|value| format!("\"{}\"", escape_java(value)))
                    .unwrap_or_else(|| "null".to_string()),
                pie.center_value
                    .as_deref()
                    .map(|value| format!("\"{}\"", escape_java(value)))
                    .unwrap_or_else(|| "null".to_string()),
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
                format!(
                    ", false, 60, null, null, {}, 0, false, false, false, false",
                    arc.start_angle
                )
            })
        })
        .unwrap_or_else(|| {
            ", false, 60, null, null, -90, 0, false, false, false, false".to_string()
        });
    let arc_args = arc_props
        .map(|arc| {
            format!(
                ", {}, {}, {}, {}, {}, {}, {}",
                arc.center_text
                    .as_deref()
                    .map(|value| format!("\"{}\"", escape_java(value)))
                    .unwrap_or_else(|| "null".to_string()),
                arc.thickness,
                arc.gap,
                arc.end_angle,
                arc.show_inline_labels,
                arc.hide_values,
                arc.show_glow,
            )
        })
        .unwrap_or_else(|| ", null, 16, 8, 270, false, false, false".to_string());
    output.push_str(&format!(
        "        DoweChartView {view} = doweChart(\"{}\", {data_path}, {series_path}, \"{}\", \"{}\", \"{}\", {}, {}, {}, {}, {}{pie_args}{arc_args});\n",
        escape_java(chart_type),
        props.palette.as_str(),
        props.legend_position.as_str(),
        escape_java(&props.empty_label),
        props.loading,
        props.hide_legend,
        dev_card_variant_container(&props.style),
        dev_card_variant_content(&props.style),
        dev_card_border(&props.style)
    ));
    apply_dev_android_style(&props.style.style, &view, false, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}
