fn compose_scale_literal(value: ScaleValue) -> String {
    format!("{}.dp", value.native_units())
}

fn compose_variant_border(props: &VariantProps) -> String {
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        variant_content(props).to_string()
    } else {
        "null".to_string()
    }
}

fn compose_show_condition(show: &VisibilityCondition, context: &ComposeReactiveContext) -> String {
    match show {
        VisibilityCondition::Static(value) => {
            format!("{} ?: true", compose_bool_value(value))
        }
        VisibilityCondition::Signal(path) => {
            if let Some(item) = context.item_value(path) {
                let path = context.item_path(path).unwrap_or_else(|| path.to_string());
                format!("state.bool(\"{}\", {item})", escape_kotlin(&path))
            } else {
                format!(
                    "state.bool(\"{}\")",
                    escape_kotlin(&context.signal_path(path))
                )
            }
        }
    }
}

fn compose_string_literal(value: &str) -> String {
    format!("\"{}\"", escape_kotlin(value))
}

fn compose_optional_u16(value: Option<u16>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}

fn compose_scheme_color(props: &VariantProps) -> &'static str {
    color_ref(family_color(props.color.unwrap_or(ColorFamily::Primary)))
}

fn compose_text_value_and_change(
    props: &VariantProps,
    fallback: &str,
    context: &ComposeReactiveContext,
) -> (String, String) {
    compose_optional_text_path_and_change(props.element.bind.as_deref(), fallback, context)
}

fn compose_optional_text_path_and_change(
    path: Option<&str>,
    fallback: &str,
    context: &ComposeReactiveContext,
) -> (String, String) {
    path.map(|path| {
        let path = escape_kotlin(&context.signal_path(path));
        (
            format!("state.text(\"{path}\")"),
            format!("{{ state.write(\"{path}\", it) }}"),
        )
    })
    .unwrap_or_else(|| (compose_string_literal(fallback), "{}".to_string()))
}

fn compose_bool_value_and_change(
    props: &VariantProps,
    fallback: bool,
    context: &ComposeReactiveContext,
) -> (String, String) {
    props
        .element
        .bind
        .as_deref()
        .map(|path| {
            let path = escape_kotlin(&context.signal_path(path));
            (
                format!("state.bool(\"{path}\")"),
                format!("{{ state.write(\"{path}\", it) }}"),
            )
        })
        .unwrap_or_else(|| (fallback.to_string(), "{}".to_string()))
}

fn compose_radio_options(options: &[RadioOption]) -> String {
    let values = options
        .iter()
        .map(|option| {
            format!(
                "DoweRadioOption(value = {}, label = {}, disabled = {})",
                compose_string_literal(&option.value),
                compose_string_literal(&option.label),
                option.disabled
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_table_columns(columns: &[TableColumn]) -> String {
    let values = columns
        .iter()
        .map(|column| {
            format!(
                "DoweTableColumn(field = {}, label = {}, align = {}, width = {})",
                compose_string_literal(&column.field),
                compose_string_literal(&column.label),
                compose_table_align(column.align),
                compose_optional_string(column.width.as_deref())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_avatar_group_items_value(
    props: &AvatarGroupProps,
    items: &[AvatarGroupItem],
    context: &ComposeReactiveContext,
) -> String {
    let fallback = compose_avatar_group_items(items, context);
    props
        .items
        .as_deref()
        .map(|path| {
            format!(
                "doweAvatarGroupItems(state.rows(\"{}\").map {{ it.value }}, {fallback})",
                escape_kotlin(&context.signal_path(path))
            )
        })
        .unwrap_or(fallback)
}

fn compose_avatar_group_items(
    items: &[AvatarGroupItem],
    context: &ComposeReactiveContext,
) -> String {
    if items.is_empty() {
        return "emptyList()".to_string();
    }
    let values = items
        .iter()
        .map(|item| {
            format!(
                "DoweAvatarGroupItem(source = {}, name = {}, alt = {}, onClick = {})",
                compose_optional_string(item.src.as_deref()),
                compose_optional_string(item.name.as_deref()),
                compose_optional_string(item.alt.as_deref()),
                compose_optional_component_action(
                    item.on_click.as_deref(),
                    item.navigation.as_ref(),
                    context,
                )
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_optional_bool_signal(path: Option<&str>, context: &ComposeReactiveContext) -> String {
    path.map(|path| {
        format!(
            "state.bool(\"{}\")",
            escape_kotlin(&context.signal_path(path))
        )
    })
    .unwrap_or_else(|| "false".to_string())
}

fn compose_chat_send_action(action: Option<&str>, context: &ComposeReactiveContext) -> String {
    action
        .and_then(|name| context.action_id(name))
        .map(|id| {
            format!(
                "{{ _: String -> actionScope.launch {{ state.run(\"{}\") }} }}",
                escape_kotlin(id)
            )
        })
        .unwrap_or_else(|| "null".to_string())
}

fn compose_type_writer_items(items: &[TypeWriterItem]) -> String {
    if items.is_empty() {
        return "emptyList()".to_string();
    }
    let values = items
        .iter()
        .map(|item| compose_string_literal(&item.text))
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_rich_text_marks(marks: &[RichTextMark]) -> String {
    if marks.is_empty() {
        return "emptyList()".to_string();
    }
    let values = marks
        .iter()
        .map(|mark| {
            format!(
                "DoweRichTextMark(text = {}, style = {}, color = {})",
                compose_string_literal(&mark.text),
                compose_string_literal(mark.style.as_str()),
                color_ref(family_color(mark.color))
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_toggle_group_items(items: &[ToggleGroupItem]) -> String {
    if items.is_empty() {
        return "emptyList()".to_string();
    }
    let values = items
        .iter()
        .map(|item| {
            format!(
                "DoweToggleGroupItem(id = {}, label = {}, icon = {})",
                compose_string_literal(&item.id),
                compose_string_literal(&item.label),
                compose_optional_string(item.icon.map(|icon| icon.as_str()))
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_map_markers(markers: &[MapMarker], context: &ComposeReactiveContext) -> String {
    if markers.is_empty() {
        return "emptyList()".to_string();
    }
    let values = markers
        .iter()
        .map(|marker| {
            format!(
                "DoweMapMarker(id = {}, lat = {}, lng = {}, label = {}, popup = {}, icon = {}, onClick = {})",
                compose_string_literal(&marker.id),
                compose_string_literal(&marker.lat),
                compose_string_literal(&marker.lng),
                compose_optional_string(marker.label.as_deref()),
                compose_optional_string(marker.popup.as_deref()),
                compose_string_literal(marker.icon.as_str()),
                compose_optional_component_action(marker.on_click.as_deref(), None, context)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_map_waypoints(waypoints: &[MapWaypoint]) -> String {
    if waypoints.is_empty() {
        return "emptyList()".to_string();
    }
    let values = waypoints
        .iter()
        .map(|waypoint| {
            format!(
                "DoweMapWaypoint(lat = {}, lng = {})",
                compose_string_literal(&waypoint.lat),
                compose_string_literal(&waypoint.lng)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_table_align(value: TableColumnAlign) -> &'static str {
    match value {
        TableColumnAlign::Start => "DoweTableColumnAlign.Start",
        TableColumnAlign::Center => "DoweTableColumnAlign.Center",
        TableColumnAlign::End => "DoweTableColumnAlign.End",
    }
}

fn compose_table_size(value: TableSize) -> &'static str {
    match value {
        TableSize::Sm => "DoweTableSize.Sm",
        TableSize::Md => "DoweTableSize.Md",
        TableSize::Lg => "DoweTableSize.Lg",
    }
}

fn compose_select_options(options: &[SelectOption]) -> String {
    let values = options
        .iter()
        .map(|option| {
            format!(
                "DoweSelectOption(value = {}, label = {}, description = {})",
                compose_string_literal(&option.value),
                compose_string_literal(&option.label),
                compose_optional_string(option.description.as_deref())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_tabs_items(tabs: &[TabItem]) -> String {
    let values = tabs
        .iter()
        .map(|tab| {
            format!(
                "DoweTabItem(id = {}, label = {})",
                compose_string_literal(&tab.id),
                compose_string_literal(&tab.label)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}
