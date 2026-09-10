fn register_rendered_node_props(node: &ViewNode, context: &ReactiveRenderContext) {
    if let Some(component) = form_component(node) {
        let element = dowe_components::node_element_props(node);
        if element.and_then(|props| props.bind.as_deref()).is_some() {
            context.register_consumed_prop(component, "bind", "ElementProps.bind");
        }
        if let Some(props) = form_variant_props(node) {
            if props.variant.is_some()
                || props.variant_binding.is_some()
                || props.reactive.variant.is_some()
            {
                context.register_consumed_prop(component, "variant", "VariantProps.variant");
            }
            if props.color.is_some() || props.color_binding.is_some() || props.reactive.scheme.is_some() {
                context.register_consumed_prop(component, "scheme", "VariantProps.color");
            }
            if props.size.is_some() || props.size_binding.is_some() || props.reactive.size.is_some() {
                context.register_consumed_prop(component, "size", "VariantProps.size");
            }
            if props.style.rounded.is_some()
                || props.style.rounded_binding.is_some()
                || props.reactive.rounded.is_some()
            {
                context.register_consumed_prop(component, "rounded", "VariantProps.style.rounded");
            }
        }
        if element.and_then(|props| props.on_change.as_ref()).is_some() {
            context.register_consumed_prop(component, "onChange", "ElementProps.on_change");
        }
        if element.and_then(|props| props.on_input.as_ref()).is_some() {
            context.register_consumed_prop(component, "onInput", "ElementProps.on_input");
        }
    }
    register_structural_node_consumed_props(node, context);
    register_media_and_form_consumed_props(node, context);
    register_content_consumed_props(node, context);
}

fn register_structural_item_consumed_props(
    component: BuiltinComponent,
    item: dowe_components::ViewItemKind,
    context: &ReactiveRenderContext,
    props: &[(&'static str, &'static str)],
) {
    for (prop, field) in props {
        let mut registry = context.consumed_props.borrow_mut();
        dowe_components::register_consumed_item(&mut registry, component, item, *prop, *field);
    }
}

fn register_structural_consumed_props(
    component: BuiltinComponent,
    context: &ReactiveRenderContext,
    props: &[(&'static str, &'static str)],
) {
    for (prop, field) in props {
        context.register_consumed_prop(component, prop, field);
    }
}

fn register_chart_consumed_props(component: BuiltinComponent, context: &ReactiveRenderContext) {
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
        BuiltinComponent::Candlestick => vec![
            ("stream", "CandlestickProps.stream"),
            ("upColor", "CandlestickProps.up_color"),
            ("downColor", "CandlestickProps.down_color"),
            ("maxPoints", "CandlestickProps.max_points"),
        ],
        BuiltinComponent::ArcChart => vec![
            ("centerText", "ArcChartProps.center_text"),
            ("centerValue", "ArcChartProps.center_value"),
            ("thickness", "ArcChartProps.thickness"),
            ("gap", "ArcChartProps.gap"),
            ("startAngle", "ArcChartProps.start_angle"),
            ("endAngle", "ArcChartProps.end_angle"),
            ("showGlow", "ArcChartProps.show_glow"),
        ],
        BuiltinComponent::AreaChart => vec![
            ("curve", "AreaChartProps.curve"),
            ("strokeWidth", "AreaChartProps.stroke_width"),
            ("fillOpacity", "AreaChartProps.fill_opacity"),
            ("stacked", "AreaChartProps.stacked"),
            ("showPoints", "AreaChartProps.show_points"),
            ("showGlow", "AreaChartProps.show_glow"),
        ],
        BuiltinComponent::BarChart => vec![
            ("grouped", "BarChartProps.grouped"),
            ("stacked", "BarChartProps.stacked"),
            ("showValues", "BarChartProps.show_values"),
            ("barRadius", "BarChartProps.bar_radius"),
            ("showGlow", "BarChartProps.show_glow"),
        ],
        BuiltinComponent::LineChart => vec![
            ("curve", "LineChartProps.curve"),
            ("strokeWidth", "LineChartProps.stroke_width"),
            ("pointRadius", "LineChartProps.point_radius"),
            ("showGradientFill", "LineChartProps.show_gradient_fill"),
            ("showGlow", "LineChartProps.show_glow"),
        ],
        BuiltinComponent::PieChart => vec![
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

fn form_component(node: &ViewNode) -> Option<BuiltinComponent> {
    match node {
        ViewNode::Input { .. } => Some(BuiltinComponent::Input),
        ViewNode::Select { .. } => Some(BuiltinComponent::Select),
        ViewNode::Checkbox { .. } => Some(BuiltinComponent::Checkbox),
        ViewNode::Toggle { .. } => Some(BuiltinComponent::Toggle),
        ViewNode::RadioGroup { props, .. } => Some(if matches!(
            props.presentation,
            RadioGroupPresentation::Card
        ) {
            BuiltinComponent::RadioCard
        } else {
            BuiltinComponent::RadioGroup
        }),
        ViewNode::Slider { .. } => Some(BuiltinComponent::Slider),
        ViewNode::Date { .. } => Some(BuiltinComponent::Date),
        ViewNode::DateRange { .. } => Some(BuiltinComponent::DateRange),
        ViewNode::Password { .. } => Some(BuiltinComponent::Password),
        ViewNode::Phone { .. } => Some(BuiltinComponent::Phone),
        ViewNode::Pin { .. } => Some(BuiltinComponent::Pin),
        ViewNode::Textarea { .. } => Some(BuiltinComponent::Textarea),
        ViewNode::Color { .. } => Some(BuiltinComponent::Color),
        ViewNode::Dropzone { .. } => Some(BuiltinComponent::Dropzone),
        ViewNode::ComboBox { .. } => Some(BuiltinComponent::ComboBox),
        ViewNode::CsvField { .. } => Some(BuiltinComponent::CsvField),
        ViewNode::DragDrop { .. } => Some(BuiltinComponent::DragDrop),
        ViewNode::Editor { .. } => Some(BuiltinComponent::Editor),
        ViewNode::ImageCropper { .. } => Some(BuiltinComponent::ImageCropper),
        _ => None,
    }
}

fn form_variant_props(node: &ViewNode) -> Option<&VariantProps> {
    match node {
        ViewNode::Input { props } | ViewNode::Select { props, .. } => Some(props),
        ViewNode::Checkbox { props } => Some(&props.style),
        ViewNode::Color { props } => Some(&props.style),
        ViewNode::Date { props } => Some(&props.style),
        ViewNode::DateRange { props } => Some(&props.style),
        ViewNode::RadioGroup { props, .. } => Some(&props.style),
        ViewNode::Toggle { props } => Some(&props.style),
        ViewNode::Slider { props } => Some(&props.style),
        ViewNode::Dropzone { props } => Some(&props.style),
        ViewNode::ComboBox { props, .. } => Some(&props.style),
        ViewNode::CsvField { props, .. } => Some(&props.style),
        ViewNode::DragDrop { props, .. } => Some(&props.style),
        ViewNode::Editor { props } => Some(&props.style),
        ViewNode::ImageCropper { props } => Some(&props.style),
        ViewNode::Password { props } => Some(&props.style),
        ViewNode::Phone { props } => Some(&props.style),
        ViewNode::Pin { props } => Some(&props.style),
        ViewNode::Textarea { props } => Some(&props.style),
        _ => None,
    }
}
