fn render_compose_countdown(
    props: &CountdownProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweCountdown(target = {}, showDays = {}, showHours = {}, showMinutes = {}, showSeconds = {}, size = {}, daysLabel = {}, hoursLabel = {}, minutesLabel = {}, secondsLabel = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, onComplete = {}, modifier = {})\n",
        compose_string_literal(&props.target),
        props.show_days,
        props.show_hours,
        props.show_minutes,
        props.show_seconds,
        compose_string_literal(props.size.as_str()),
        compose_string_literal(&props.days_label),
        compose_string_literal(&props.hours_label),
        compose_string_literal(&props.minutes_label),
        compose_string_literal(&props.seconds_label),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_optional_component_action(props.on_complete.as_deref(), None, context),
        modifier_for_style(&props.style.style),
    ));
}

fn render_compose_map(
    props: &MapProps,
    markers: &[MapMarker],
    waypoints: &[MapWaypoint],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweMap(centerLat = {}, centerLng = {}, zoom = {}, height = {}, width = {}, showControls = {}, showScale = {}, showLocationControl = {}, interactive = {}, markers = {}, waypoints = {}, backgroundColor = {}, contentColor = {}, onLocation = {}, onLocationError = {}, onRoute = {}, modifier = {})\n",
        compose_string_literal(&props.center_lat),
        compose_string_literal(&props.center_lng),
        props.zoom,
        compose_string_literal(&props.height),
        compose_string_literal(&props.width),
        props.show_controls,
        props.show_scale,
        props.show_location_control,
        props.interactive,
        compose_map_markers(markers, context),
        compose_map_waypoints(waypoints),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_optional_component_action(props.on_location.as_deref(), None, context),
        compose_optional_component_action(props.on_location_error.as_deref(), None, context),
        compose_optional_component_action(props.on_route.as_deref(), None, context),
        modifier_for_style(&props.style.style),
    ));
}

fn render_compose_badge(
    props: &BadgeProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweBadge(text = {}, position = {}, backgroundColor = {}, contentColor = {}, modifier = {}) {{\n",
        compose_string_literal(&props.text),
        compose_string_literal(props.position.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        modifier_for_style(&props.style.style)
    ));
    for child in children {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_chip(
    props: &ChipProps,
    value: &str,
    start: Option<&SideNavIcon>,
    end: Option<&SideNavIcon>,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let size = props.style.size.unwrap_or(ButtonSize::Md);
    let mut modifier = modifier_for_style(&props.style.style);
    if props.style.element.on_click.is_some() {
        modifier.push_str(&format!(
            ".clickable(onClick = {})",
            compose_component_action(props.style.element.on_click.as_deref(), None, context)
        ));
    }
    let compact =
        props.style.style.sizing.w.is_none() && props.style.style.sizing.w_binding.is_none();
    output.push_str(&format!(
        "{pad}DoweChip(text = {}, size = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, modifier = {}, compact = {}, onClose = {}, start = ",
        compose_string_literal(value),
        compose_string_literal(size.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_variant_border(&props.style),
        modifier,
        compact,
        compose_optional_component_action(props.on_close.as_deref(), None, context)
    ));
    render_compose_optional_icon_lambda(start, indent, output);
    output.push_str(", end = ");
    render_compose_optional_icon_lambda(end, indent, output);
    output.push_str(")\n");
}

fn render_compose_skeleton(
    props: &SkeletonProps,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweSkeleton(variant = {}, animation = {}, modifier = {})\n",
        compose_string_literal(props.variant.as_str()),
        compose_string_literal(props.animation.as_str()),
        modifier_for_container_style(&props.style, flow)
    ));
}
