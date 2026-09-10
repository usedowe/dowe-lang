fn render_compose_record(
    props: &RecordProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweRecord(name = {}, url = {}, disabled = {}, maxDuration = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, onStart = {}, onPause = {}, onResume = {}, onStop = {}, onDiscard = {}, onConfirm = {}, modifier = {})\n",
        compose_string_literal(&props.name),
        compose_optional_string(props.url.as_deref()),
        props.disabled,
        props.max_duration.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_optional_component_action(props.on_start.as_deref(), None, context),
        compose_optional_component_action(props.on_pause.as_deref(), None, context),
        compose_optional_component_action(props.on_resume.as_deref(), None, context),
        compose_optional_component_action(props.on_stop.as_deref(), None, context),
        compose_optional_component_action(props.on_discard.as_deref(), None, context),
        compose_optional_component_action(props.on_confirm.as_deref(), None, context),
        modifier_for_style(&props.style.style),
    ));
}

fn render_compose_toggle_group(
    props: &ToggleGroupProps,
    items: &[ToggleGroupItem],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    if props.kind == ToggleGroupKind::Pagination {
        render_compose_pagination(props, items, indent, output, context);
        return;
    }
    let pad = " ".repeat(indent);
    let value = props
        .value
        .as_deref()
        .map(|path| {
            format!(
                "state.text(\"{}\")",
                escape_kotlin(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| compose_string_literal(&props.selected));
    let change = props
        .value
        .as_deref()
        .map(|path| {
            format!(
                "{{ value -> state.write(\"{}\", value) }}",
                escape_kotlin(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| "{ _ -> }".to_string());
    output.push_str(&format!(
        "{pad}DoweToggleGroup(value = {value}, onValueChange = {change}, items = {}, size = {}, wide = {}, vertical = {}, disabled = {}, ariaLabel = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, onChange = {}, modifier = {})\n",
        compose_toggle_group_items(items),
        compose_string_literal(props.size.as_str()),
        props.wide,
        props.vertical,
        props.disabled,
        compose_optional_string(props.aria_label.as_deref()),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_optional_component_action(props.on_change.as_deref(), None, context),
        modifier_for_style(&props.style.style),
    ));
}

fn render_compose_pagination(
    props: &ToggleGroupProps,
    items: &[ToggleGroupItem],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let value = props
        .value
        .as_deref()
        .map(|path| {
            format!(
                "state.text(\"{}\")",
                escape_kotlin(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| compose_string_literal(&props.selected));
    let change = props
        .value
        .as_deref()
        .map(|path| {
            format!(
                "{{ value -> state.write(\"{}\", value) }}",
                escape_kotlin(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| "{ _ -> }".to_string());
    let page_count = props
        .pagination
        .as_ref()
        .map(|pagination| match &pagination.total {
            dowe_components::PaginationTotal::Static(total) => {
                total.div_ceil(pagination.page_size).max(1).to_string()
            }
            dowe_components::PaginationTotal::Signal(total) => {
                let path = escape_kotlin(&context.signal_path(total));
                let offset = pagination.page_size - 1;
                format!(
                    "maxOf(1, minOf(25, ((state.text(\"{path}\").toIntOrNull() ?: 0).coerceAtLeast(0) + {offset}) / {}))",
                    pagination.page_size
                )
            }
        })
        .unwrap_or_else(|| items.len().max(1).to_string());
    let previous = solar_control_icon("arrow-left").expect("bundled Pagination previous icon");
    let next = solar_control_icon("arrow-right").expect("bundled Pagination next icon");
    output.push_str(&format!(
        "{pad}DowePagination(value = {value}, onValueChange = {change}, pageCount = {page_count}, size = {}, disabled = {}, ariaLabel = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, onChange = {}, previousIcon = ",
        compose_string_literal(props.size.as_str()),
        props.disabled,
        compose_optional_string(props.aria_label.as_deref()),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_optional_component_action(props.on_change.as_deref(), None, context),
    ));
    output.push_str("{\n");
    render_compose_side_icon(&previous, indent + 4, output);
    output.push_str(&format!("{pad}}}, nextIcon = {{\n"));
    render_compose_side_icon(&next, indent + 4, output);
    output.push_str(&format!(
        "{pad}}}, modifier = {})\n",
        modifier_for_style(&props.style.style)
    ));
}

fn render_compose_collapsible(
    props: &CollapsibleProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let arrow = solar_control_icon("alt-arrow-down").expect("bundled Collapsible arrow icon");
    output.push_str(&format!(
        "{pad}DoweCollapsible(label = {}, defaultOpen = {}, disabled = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, radius = {}, modifier = {}, arrowIcon = {{\n",
        compose_string_literal(&props.label),
        props.default_open,
        props.disabled,
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_card_radius(&props.style.style),
        modifier_for_style(&props.style.style),
    ));
    render_compose_side_icon(&arrow, indent + 4, output);
    output.push_str(&format!("{pad}}}) {{\n"));
    for child in children {
        render_compose_node_in_flow(
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
}

