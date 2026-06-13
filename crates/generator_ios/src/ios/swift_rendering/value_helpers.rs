fn swift_text_expression(
    value: &str,
    i18n: Option<&str>,
    context: &SwiftReactiveContext,
) -> String {
    if let Some(key) = i18n {
        return format!("String(localized: \"{}\")", escape_swift(key));
    }
    match context.dynamic_path(value) {
        Some(path) => context
            .item_value(value)
            .map(|item| format!("state.text(\"{}\", item: {item})", escape_swift(&path)))
            .unwrap_or_else(|| format!("state.text(\"{}\")", escape_swift(&path))),
        None => format!("\"{}\"", escape_swift(value)),
    }
}

fn swift_string_literal(value: &str) -> String {
    format!("\"{}\"", escape_swift(value))
}

fn swift_optional_u16(value: Option<u16>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_scheme_color(props: &VariantProps) -> &'static str {
    color_ref(family_color(props.color.unwrap_or(ColorFamily::Primary)))
}

fn swift_string_binding(
    props: &VariantProps,
    fallback: &str,
    context: &SwiftReactiveContext,
) -> String {
    swift_optional_string_binding(props.element.bind.as_deref(), fallback, context)
}

fn swift_optional_string_binding(
    path: Option<&str>,
    fallback: &str,
    context: &SwiftReactiveContext,
) -> String {
    path.map(|path| {
        format!(
            "state.binding(\"{}\")",
            escape_swift(&context.signal_path(path))
        )
    })
    .unwrap_or_else(|| format!(".constant({})", swift_string_literal(fallback)))
}

fn swift_bool_binding(
    props: &VariantProps,
    fallback: bool,
    context: &SwiftReactiveContext,
) -> String {
    props
        .element
        .bind
        .as_deref()
        .map(|path| {
            format!(
                "state.boolBinding(\"{}\")",
                escape_swift(&context.signal_path(path))
            )
        })
        .unwrap_or_else(|| format!(".constant({fallback})"))
}

fn swift_radio_options(options: &[RadioOption]) -> String {
    let values = options
        .iter()
        .map(|option| {
            format!(
                "DoweRadioOption(value: {}, label: {}, disabled: {})",
                swift_string_literal(&option.value),
                swift_string_literal(&option.label),
                option.disabled
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_table_columns(columns: &[TableColumn]) -> String {
    let values = columns
        .iter()
        .map(|column| {
            format!(
                "DoweTableColumn(field: {}, label: {}, align: {}, width: {})",
                swift_string_literal(&column.field),
                swift_string_literal(&column.label),
                swift_table_align(column.align),
                swift_optional_literal(column.width.as_deref())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_avatar_group_items_value(
    props: &AvatarGroupProps,
    items: &[AvatarGroupItem],
    context: &SwiftReactiveContext,
) -> String {
    let fallback = swift_avatar_group_items(items, context);
    props
        .items
        .as_deref()
        .map(|path| {
            format!(
                "doweAvatarGroupItems(state.rows({}).map {{ $0.value }}, fallback: {fallback})",
                swift_string_literal(&context.signal_path(path))
            )
        })
        .unwrap_or(fallback)
}

fn swift_avatar_group_items(items: &[AvatarGroupItem], context: &SwiftReactiveContext) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }
    let values = items
        .iter()
        .map(|item| {
            format!(
                "DoweAvatarGroupItem(source: {}, name: {}, alt: {}, action: {})",
                swift_optional_literal(item.src.as_deref()),
                swift_optional_literal(item.name.as_deref()),
                swift_optional_literal(item.alt.as_deref()),
                swift_optional_component_action(
                    item.on_click.as_deref(),
                    item.navigation.as_ref(),
                    context,
                )
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_optional_bool_signal(path: Option<&str>, context: &SwiftReactiveContext) -> String {
    path.map(|path| {
        format!(
            "state.bool({})",
            swift_string_literal(&context.signal_path(path))
        )
    })
    .unwrap_or_else(|| "false".to_string())
}

fn swift_chat_send_action(action: Option<&str>, context: &SwiftReactiveContext) -> String {
    action
        .and_then(|name| context.action_id(name))
        .map(|id| format!("{{ _ in state.run(\"{}\") }}", escape_swift(id)))
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_type_writer_items(items: &[TypeWriterItem]) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }
    let values = items
        .iter()
        .map(|item| swift_string_literal(&item.text))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_rich_text_marks(marks: &[RichTextMark]) -> String {
    let values = marks
        .iter()
        .map(|mark| {
            format!(
                "DoweRichTextMark(text: {}, style: {}, color: {})",
                swift_string_literal(&mark.text),
                swift_string_literal(mark.style.as_str()),
                color_ref(family_color(mark.color))
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_toggle_group_items(items: &[ToggleGroupItem]) -> String {
    let values = items
        .iter()
        .map(|item| {
            format!(
                "DoweToggleGroupItem(id: {}, label: {}, icon: {})",
                swift_string_literal(&item.id),
                swift_string_literal(&item.label),
                swift_optional_literal(item.icon.map(|icon| icon.as_str()))
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_map_markers(markers: &[MapMarker], context: &SwiftReactiveContext) -> String {
    let values = markers
        .iter()
        .map(|marker| {
            format!(
                "DoweMapMarker(id: {}, lat: {}, lng: {}, label: {}, popup: {}, icon: {}, action: {})",
                swift_string_literal(&marker.id),
                swift_string_literal(&marker.lat),
                swift_string_literal(&marker.lng),
                swift_optional_literal(marker.label.as_deref()),
                swift_optional_literal(marker.popup.as_deref()),
                swift_string_literal(marker.icon.as_str()),
                swift_optional_component_action(marker.on_click.as_deref(), None, context),
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_map_waypoints(waypoints: &[MapWaypoint]) -> String {
    let values = waypoints
        .iter()
        .map(|waypoint| {
            format!(
                "DoweMapWaypoint(lat: {}, lng: {})",
                swift_string_literal(&waypoint.lat),
                swift_string_literal(&waypoint.lng),
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_table_align(value: TableColumnAlign) -> &'static str {
    match value {
        TableColumnAlign::Start => ".start",
        TableColumnAlign::Center => ".center",
        TableColumnAlign::End => ".end",
    }
}

fn swift_table_size(value: TableSize) -> &'static str {
    match value {
        TableSize::Sm => ".sm",
        TableSize::Md => ".md",
        TableSize::Lg => ".lg",
    }
}

fn swift_optional_literal(value: Option<&str>) -> String {
    value
        .map(swift_string_literal)
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_select_options(options: &[SelectOption]) -> String {
    let values = options
        .iter()
        .map(|option| {
            format!(
                "DoweSelectOption(value: {}, label: {}, description: {})",
                swift_string_literal(&option.value),
                swift_string_literal(&option.label),
                swift_optional_literal(option.description.as_deref())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_tabs_items(tabs: &[TabItem]) -> String {
    let values = tabs
        .iter()
        .map(|tab| {
            format!(
                "DoweTabItem(id: {}, label: {})",
                swift_string_literal(&tab.id),
                swift_string_literal(&tab.label)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}
