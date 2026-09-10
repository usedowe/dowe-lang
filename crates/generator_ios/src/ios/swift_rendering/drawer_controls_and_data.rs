fn render_swift_pagination(
    props: &ToggleGroupProps,
    items: &[ToggleGroupItem],
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = props
        .value
        .as_deref()
        .map(|path| {
            format!(
                "state.binding(\"{}\")",
                escape_swift(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| format!(".constant({})", swift_string_literal(&props.selected)));
    let page_count = props
        .pagination
        .as_ref()
        .map(|pagination| match &pagination.total {
            dowe_components::PaginationTotal::Static(total) => {
                total.div_ceil(pagination.page_size).max(1).to_string()
            }
            dowe_components::PaginationTotal::Signal(total) => {
                let path = escape_swift(&context.signal_path(total));
                let offset = pagination.page_size - 1;
                format!(
                    "max(1, min(25, (max(0, Int(state.text(\"{path}\")) ?? 0) + {offset}) / {}))",
                    pagination.page_size
                )
            }
        })
        .unwrap_or_else(|| items.len().max(1).to_string());
    let previous = solar_control_icon("arrow-left").expect("bundled Pagination previous icon");
    let next = solar_control_icon("arrow-right").expect("bundled Pagination next icon");
    output.push_str(&format!(
        "{pad}DowePagination(value: {binding}, pageCount: {page_count}, size: {}, disabled: {}, ariaLabel: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, onChange: {}, previousIcon: {{\n",
        swift_string_literal(props.size.as_str()),
        props.disabled,
        swift_optional_literal(props.aria_label.as_deref()),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_variant_border(&props.style),
        swift_optional_component_action(props.on_change.as_deref(), None, context),
    ));
    render_swift_side_icon(&previous, indent + 4, output);
    output.push_str(&format!("{pad}}}, nextIcon: {{\n"));
    render_swift_side_icon(&next, indent + 4, output);
    output.push_str(&format!("{pad}}})\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_collapsible(
    props: &CollapsibleProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let arrow = solar_control_icon("alt-arrow-down").expect("bundled Collapsible arrow icon");
    let content_color = card_variant_content(&props.style);
    output.push_str(&format!(
        "{pad}DoweCollapsible(label: {}, defaultOpen: {}, disabled: {}, backgroundColor: {}, contentColor: {content_color}, borderColor: {}, radius: {}, arrowIcon: {{\n",
        swift_string_literal(&props.label),
        props.default_open,
        props.disabled,
        card_variant_container(&props.style),
        swift_variant_border(&props.style),
        swift_card_radius(&props.style.style),
    ));
    render_swift_button_icon(&arrow, content_color, indent + 4, output);
    output.push_str(&format!("{pad}}}) {{\n"));
    for child in children {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            flow,
            props.style.style.font.as_ref().or(inherited_font),
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_countdown(
    props: &CountdownProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweCountdown(target: {}, showDays: {}, showHours: {}, showMinutes: {}, showSeconds: {}, size: {}, daysLabel: {}, hoursLabel: {}, minutesLabel: {}, secondsLabel: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, onComplete: {})\n",
        swift_string_literal(&props.target),
        props.show_days,
        props.show_hours,
        props.show_minutes,
        props.show_seconds,
        swift_string_literal(props.size.as_str()),
        swift_string_literal(&props.days_label),
        swift_string_literal(&props.hours_label),
        swift_string_literal(&props.minutes_label),
        swift_string_literal(&props.seconds_label),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_variant_border(&props.style),
        swift_optional_component_action(props.on_complete.as_deref(), None, context),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_map(
    props: &MapProps,
    markers: &[MapMarker],
    waypoints: &[MapWaypoint],
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweMap(centerLat: {}, centerLng: {}, zoom: {}, height: {}, width: {}, showControls: {}, showScale: {}, showLocationControl: {}, interactive: {}, markers: {}, waypoints: {}, backgroundColor: {}, contentColor: {}, onLocation: {}, onLocationError: {}, onRoute: {})\n",
        swift_string_literal(&props.center_lat),
        swift_string_literal(&props.center_lng),
        props.zoom,
        swift_string_literal(&props.height),
        swift_string_literal(&props.width),
        props.show_controls,
        props.show_scale,
        props.show_location_control,
        props.interactive,
        swift_map_markers(markers, context),
        swift_map_waypoints(waypoints),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_optional_component_action(props.on_location.as_deref(), None, context),
        swift_optional_component_action(props.on_location_error.as_deref(), None, context),
        swift_optional_component_action(props.on_route.as_deref(), None, context),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}
