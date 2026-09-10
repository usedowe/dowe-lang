fn register_compose_structural_item_consumed_props(
    component: dowe_components::BuiltinComponent,
    item: dowe_components::ViewItemKind,
    context: &ComposeReactiveContext,
    props: &[(&'static str, &'static str)],
) {
    for (prop, field) in props {
        let mut registry = context.consumed_props.borrow_mut();
        dowe_components::register_consumed_item(&mut registry, component, item, *prop, *field);
    }
}

fn register_compose_structural_consumed_props(
    component: dowe_components::BuiltinComponent,
    context: &ComposeReactiveContext,
    props: &[(&'static str, &'static str)],
) {
    for (prop, field) in props {
        context.register_consumed_prop(component, prop, field);
    }
}

fn register_compose_chart_consumed_props(
    component: dowe_components::BuiltinComponent,
    context: &ComposeReactiveContext,
) {
    for (prop, field) in [
        ("data", "ChartCommonProps.data"),
        ("series", "ChartCommonProps.series"),
        ("size", "ChartCommonProps.size"),
        ("palette", "ChartCommonProps.palette"),
        ("loading", "ChartCommonProps.loading"),
    ] {
        context.register_consumed_prop(component, prop, field);
    }
    let fields = match component {
        dowe_components::BuiltinComponent::Candlestick => vec![
            ("stream", "CandlestickProps.stream"),
            ("upColor", "CandlestickProps.up_color"),
            ("downColor", "CandlestickProps.down_color"),
            ("maxPoints", "CandlestickProps.max_points"),
        ],
        dowe_components::BuiltinComponent::ArcChart => vec![
            ("centerText", "ArcChartProps.center_text"),
            ("centerValue", "ArcChartProps.center_value"),
            ("thickness", "ArcChartProps.thickness"),
            ("gap", "ArcChartProps.gap"),
            ("startAngle", "ArcChartProps.start_angle"),
            ("endAngle", "ArcChartProps.end_angle"),
            ("showGlow", "ArcChartProps.show_glow"),
        ],
        dowe_components::BuiltinComponent::AreaChart => vec![
            ("curve", "AreaChartProps.curve"),
            ("strokeWidth", "AreaChartProps.stroke_width"),
            ("fillOpacity", "AreaChartProps.fill_opacity"),
            ("stacked", "AreaChartProps.stacked"),
            ("showPoints", "AreaChartProps.show_points"),
            ("showGlow", "AreaChartProps.show_glow"),
        ],
        dowe_components::BuiltinComponent::BarChart => vec![
            ("grouped", "BarChartProps.grouped"),
            ("stacked", "BarChartProps.stacked"),
            ("showValues", "BarChartProps.show_values"),
            ("barRadius", "BarChartProps.bar_radius"),
            ("showGlow", "BarChartProps.show_glow"),
        ],
        dowe_components::BuiltinComponent::LineChart => vec![
            ("curve", "LineChartProps.curve"),
            ("strokeWidth", "LineChartProps.stroke_width"),
            ("pointRadius", "LineChartProps.point_radius"),
            ("showGradientFill", "LineChartProps.show_gradient_fill"),
            ("showGlow", "LineChartProps.show_glow"),
        ],
        dowe_components::BuiltinComponent::PieChart => vec![
            ("donut", "PieChartProps.donut"),
            ("donutWidth", "PieChartProps.donut_width"),
            ("centerLabel", "PieChartProps.center_label"),
            ("centerValue", "PieChartProps.center_value"),
            ("startAngle", "PieChartProps.start_angle"),
            ("padAngle", "PieChartProps.pad_angle"),
            ("hideLabels", "PieChartProps.hide_labels"),
            ("hideValues", "PieChartProps.hide_values"),
            ("hidePercentages", "PieChartProps.hide_percentages"),
            ("showGlow", "PieChartProps.show_glow"),
        ],
        _ => Vec::new(),
    };
    for (prop, field) in fields {
        context.register_consumed_prop(component, prop, field);
    }
}
