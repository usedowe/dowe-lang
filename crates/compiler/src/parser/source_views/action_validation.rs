fn validate_optional_action(
    path: &Path,
    actions: &HashSet<String>,
    action: Option<&str>,
) -> DoweResult<()> {
    if let Some(action) = action
        && !actions.contains(action)
    {
        return Err(DoweError::at_path(path, format!("unknown fn `{action}`")));
    }
    Ok(())
}

fn validate_optional_typed_path(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    value: Option<&str>,
    label: &str,
    expectation: ViewPathExpectation,
) -> DoweResult<()> {
    if let Some(value) = value {
        validate_typed_path(path, signals, locals, value, label, expectation)?;
    }
    Ok(())
}

fn validate_avatar_group_items(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    source: &str,
) -> DoweResult<()> {
    let Some(value) = signal_path_value(path, signals, locals, source, "items")? else {
        return Ok(());
    };
    let ViewSignalValue::Array(items) = value else {
        return Err(DoweError::at_path(
            path,
            format!("invalid signal path `{source}` in `items`: expected array"),
        ));
    };
    for item in items {
        let ViewSignalValue::Object(fields) = item else {
            return Err(DoweError::at_path(
                path,
                "AvatarGroup items must be objects",
            ));
        };
        for field in ["src", "name", "alt", "href", "onClick"] {
            if let Some(value) = object_field(&fields, field)
                && !matches!(value, ViewSignalValue::String(_))
            {
                return Err(DoweError::at_path(
                    path,
                    format!("AvatarGroup item field `{field}` must be string"),
                ));
            }
        }
    }
    Ok(())
}

fn validate_chat_box_messages(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    source: &str,
) -> DoweResult<()> {
    let Some(value) = signal_path_value(path, signals, locals, source, "messages")? else {
        return Ok(());
    };
    let ViewSignalValue::Array(items) = value else {
        return Err(DoweError::at_path(
            path,
            format!("invalid signal path `{source}` in `messages`: expected array"),
        ));
    };
    for item in items {
        let ViewSignalValue::Object(fields) = item else {
            return Err(DoweError::at_path(path, "ChatBox messages must be objects"));
        };
        for field in [
            "id", "userId", "name", "avatar", "message", "text", "type", "status",
        ] {
            if let Some(value) = object_field(&fields, field)
                && !matches!(value, ViewSignalValue::String(_))
            {
                return Err(DoweError::at_path(
                    path,
                    format!("ChatBox message field `{field}` must be string"),
                ));
            }
        }
        for field in ["own", "isOwn", "streaming"] {
            if let Some(value) = object_field(&fields, field)
                && !matches!(value, ViewSignalValue::Bool(_))
            {
                return Err(DoweError::at_path(
                    path,
                    format!("ChatBox message field `{field}` must be bool"),
                ));
            }
        }
        if let Some(value) = object_field(&fields, "createdAt")
            && !matches!(
                value,
                ViewSignalValue::String(_) | ViewSignalValue::Number(_)
            )
        {
            return Err(DoweError::at_path(
                path,
                "ChatBox message field `createdAt` must be string or number",
            ));
        }
    }
    Ok(())
}

fn validate_avatar_group_actions(
    path: &Path,
    items: &[dowe_components::AvatarGroupItem],
    actions: &HashSet<String>,
) -> DoweResult<()> {
    for item in items {
        validate_optional_action(path, actions, item.on_click.as_deref())?;
    }
    Ok(())
}

fn validate_toast_source(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    source: &str,
) -> DoweResult<()> {
    let Some(value) = signal_path_value(path, signals, locals, source, "source")? else {
        return Ok(());
    };
    let ViewSignalValue::Object(fields) = value else {
        return Err(DoweError::at_path(
            path,
            format!("invalid signal path `{source}` in `source`: expected object"),
        ));
    };
    let visible = object_field(&fields, "visible")
        .ok_or_else(|| DoweError::at_path(path, "Toast source must include `visible`"))?;
    if !matches!(visible, ViewSignalValue::Bool(_)) {
        return Err(DoweError::at_path(
            path,
            "Toast source field `visible` must be bool",
        ));
    }
    let message = object_field(&fields, "message")
        .ok_or_else(|| DoweError::at_path(path, "Toast source must include `message`"))?;
    if !matches!(message, ViewSignalValue::String(_)) {
        return Err(DoweError::at_path(
            path,
            "Toast source field `message` must be string",
        ));
    }
    for field in ["title", "type"] {
        if let Some(value) = object_field(&fields, field)
            && !matches!(value, ViewSignalValue::String(_))
        {
            return Err(DoweError::at_path(
                path,
                format!("Toast source field `{field}` must be string"),
            ));
        }
    }
    Ok(())
}

fn signal_path_value(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    value: &str,
    label: &str,
) -> DoweResult<Option<ViewSignalValue>> {
    let root = path_root(value);
    let mut resolved = if let Some(value) = signals.get(root) {
        Some(value.clone())
    } else if let Some(value) = locals.get(root) {
        value.clone()
    } else {
        return Err(DoweError::at_path(
            path,
            format!("unknown signal path `{value}` in `{label}`"),
        ));
    };
    let Some(mut resolved_value) = resolved.take() else {
        return Ok(None);
    };
    for field in value.split('.').skip(1) {
        let ViewSignalValue::Object(fields) = resolved_value else {
            return Err(DoweError::at_path(
                path,
                format!("unknown signal path `{value}` in `{label}`"),
            ));
        };
        let Some((_, next)) = fields.into_iter().find(|(name, _)| name == field) else {
            return Err(DoweError::at_path(
                path,
                format!("unknown signal path `{value}` in `{label}`"),
            ));
        };
        resolved_value = next;
    }
    Ok(Some(resolved_value))
}

fn object_field<'a>(
    fields: &'a [(String, ViewSignalValue)],
    name: &str,
) -> Option<&'a ViewSignalValue> {
    fields
        .iter()
        .find_map(|(field, value)| (field == name).then_some(value))
}

fn validate_overlay_entry_actions(
    path: &Path,
    entries: &[dowe_components::OverlayEntry],
    actions: &HashSet<String>,
) -> DoweResult<()> {
    for entry in entries {
        if let dowe_components::OverlayEntry::Item(props) = entry {
            validate_overlay_item_action(path, props, actions)?;
        }
    }
    Ok(())
}

fn validate_command_entry_actions(
    path: &Path,
    entries: &[dowe_components::CommandEntry],
    actions: &HashSet<String>,
) -> DoweResult<()> {
    for entry in entries {
        match entry {
            dowe_components::CommandEntry::Item(props) => {
                validate_overlay_item_action(path, props, actions)?
            }
            dowe_components::CommandEntry::Group { items, .. } => {
                for props in items {
                    validate_overlay_item_action(path, props, actions)?;
                }
            }
        }
    }
    Ok(())
}

fn validate_overlay_item_action(
    path: &Path,
    props: &dowe_components::OverlayItemProps,
    actions: &HashSet<String>,
) -> DoweResult<()> {
    validate_optional_action(path, actions, props.on_click.as_deref())
}

fn validate_candlestick_data(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    data: &str,
) -> DoweResult<()> {
    let root = path_root(data);
    let Some(collection_type) = signals.get(root) else {
        return Err(DoweError::at_path(
            path,
            format!("unknown signal `{root}` in `data`"),
        ));
    };
    let ViewSignalValue::Array(items) = collection_type else {
        return Err(DoweError::at_path(
            path,
            format!("signal `{root}` in `data` must be an array"),
        ));
    };
    for item in items {
        validate_candlestick_item(path, item)?;
    }
    Ok(())
}

fn validate_canvas_scene(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    scene: &str,
) -> DoweResult<()> {
    let root = path_root(scene);
    let Some(value) = signals.get(root) else {
        return Err(DoweError::at_path(
            path,
            format!("unknown signal `{root}` in `scene`"),
        ));
    };
    if !matches!(value, ViewSignalValue::Array(_)) {
        return Err(DoweError::at_path(
            path,
            format!("signal `{root}` in `scene` must be an array"),
        ));
    }
    Ok(())
}

fn validate_category_chart_common(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    props: &dowe_components::ChartCommonProps,
    component: &str,
) -> DoweResult<()> {
    let Some(data) = props.data.as_deref() else {
        return Err(DoweError::at_path(
            path,
            format!("{component} requires `data`"),
        ));
    };
    validate_chart_data(
        path,
        signals,
        data,
        "data",
        component,
        ChartDataShape::Category,
    )
}

fn validate_category_or_series_chart_common(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    props: &dowe_components::ChartCommonProps,
    component: &str,
) -> DoweResult<()> {
    if let Some(data) = props.data.as_deref() {
        validate_chart_data(
            path,
            signals,
            data,
            "data",
            component,
            ChartDataShape::Category,
        )?;
    }
    if let Some(series) = props.series.as_deref() {
        validate_chart_data(
            path,
            signals,
            series,
            "series",
            component,
            ChartDataShape::CategorySeries,
        )?;
    }
    Ok(())
}

fn validate_point_or_series_chart_common(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    props: &dowe_components::ChartCommonProps,
    component: &str,
) -> DoweResult<()> {
    if let Some(data) = props.data.as_deref() {
        validate_chart_data(
            path,
            signals,
            data,
            "data",
            component,
            ChartDataShape::Point,
        )?;
    }
    if let Some(series) = props.series.as_deref() {
        validate_chart_data(
            path,
            signals,
            series,
            "series",
            component,
            ChartDataShape::PointSeries,
        )?;
    }
    Ok(())
}
