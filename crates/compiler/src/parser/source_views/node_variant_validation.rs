fn validate_node_variant_references(
    path: &Path,
    node: &ViewNode,
    signals: &HashMap<String, ViewSignalValue>,
    writable_signals: &HashSet<String>,
    actions: &HashSet<String>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
) -> DoweResult<()> {
    match node {
        ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } => {
            validate_typed_path(
                path,
                signals,
                locals,
                binding,
                "Splash bind",
                ViewPathExpectation::Bool,
            )?;
            for child in content.iter().chain(children) {
                validate_node_references(path, child, signals, writable_signals, actions, locals)?;
            }
        }
        ViewNode::Scope { children, .. } => {
            for child in children {
                validate_node_references(path, child, signals, writable_signals, actions, locals)?;
            }
        }
        ViewNode::Each {
            item,
            collection,
            key,
            children,
        } => {
            let Some(collection_type) = signals.get(collection) else {
                return Err(DoweError::at_path(
                    path,
                    format!("unknown view value `{collection}` in `collection`"),
                ));
            };
            let ViewSignalValue::Array(items) = collection_type else {
                return Err(DoweError::at_path(
                    path,
                    format!("view value `{collection}` in `collection` must be an array"),
                ));
            };
            if path_root(key) != item {
                return Err(DoweError::at_path(
                    path,
                    format!("`each` key `{key}` must start with `{item}`"),
                ));
            }
            let mut scoped = locals.clone();
            scoped.insert(item.clone(), items.first().cloned());
            validate_typed_path(path, signals, &scoped, key, "key", ViewPathExpectation::Any)?;
            for child in children {
                validate_node_references(path, child, signals, writable_signals, actions, &scoped)?;
            }
        }
        ViewNode::Select {
            option_each: Some(option_each),
            ..
        } => {
            let Some(ViewSignalValue::Array(items)) = signals.get(&option_each.collection) else {
                return Err(DoweError::at_path(
                    path,
                    format!(
                        "view value `{}` in Select `each` must be an array",
                        option_each.collection
                    ),
                ));
            };
            if path_root(&option_each.key) != option_each.item {
                return Err(DoweError::at_path(
                    path,
                    format!(
                        "`each` key `{}` must start with `{}`",
                        option_each.key, option_each.item
                    ),
                ));
            }
            let mut scoped = locals.clone();
            scoped.insert(option_each.item.clone(), items.first().cloned());
            for (name, value) in [
                ("key", option_each.key.as_str()),
                ("value", option_each.value.as_str()),
                ("label", option_each.label.as_str()),
            ] {
                validate_typed_path(
                    path,
                    signals,
                    &scoped,
                    value,
                    name,
                    if name == "key" {
                        ViewPathExpectation::Any
                    } else {
                        ViewPathExpectation::String
                    },
                )?;
            }
            if let Some(description) = option_each.description.as_deref() {
                validate_typed_path(
                    path,
                    signals,
                    &scoped,
                    description,
                    "description",
                    ViewPathExpectation::String,
                )?;
            }
        }
        ViewNode::Title { props, value } | ViewNode::Text { props, value } => {
            let binding = text_binding_path(value);
            if props.i18n.is_some() && binding.is_some() {
                return Err(DoweError::at_path(
                    path,
                    "`i18n` requires a static fallback text child",
                ));
            }
            if let Some(binding) = binding {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    binding,
                    "text",
                    ViewPathExpectation::String,
                )?;
            }
        }
        ViewNode::Alert { props } => {
            if is_dynamic_reference(&props.message) {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    &props.message,
                    "message",
                    ViewPathExpectation::String,
                )?;
            }
            if let Some(visible) = props.visible.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    visible,
                    "visible",
                    ViewPathExpectation::Bool,
                )?;
            }
            if let Some(action) = props.on_close.as_ref()
                && !actions.contains(action)
            {
                return Err(DoweError::at_path(path, format!("unknown fn `{action}`")));
            }
        }
        ViewNode::Svg { props, .. } => {
            if let Some(data) = props.data.as_deref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    data,
                    "Svg data",
                    ViewPathExpectation::Any,
                )?;
            }
            if let Some(name) = props.icon_name.as_deref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    name,
                    "Icon name",
                    ViewPathExpectation::String,
                )?;
                if let Some(ViewSignalValue::String(value)) =
                    signal_path_value(path, signals, locals, name, "Icon name")?
                {
                    if !dowe_components::all_icon_names()
                        .iter()
                        .any(|name| name == &value)
                    {
                        return Err(DoweError::at_path(
                            path,
                            format!("invalid initial icon name `{value}` for `Icon name`"),
                        ));
                    }
                }
            }
        }
        ViewNode::ToggleGroup { props, .. } => {
            if let Some(value) = props.value.as_deref() {
                if signals.contains_key(path_root(value))
                    && !writable_signals.contains(path_root(value))
                {
                    return Err(DoweError::at_path(
                        path,
                        format!("constant path `{value}` cannot be used in `bind`"),
                    ));
                }
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    value,
                    if props.kind == ToggleGroupKind::Pagination {
                        "Pagination bind"
                    } else {
                        "ToggleGroup value"
                    },
                    if props.kind == ToggleGroupKind::Pagination {
                        ViewPathExpectation::Any
                    } else {
                        ViewPathExpectation::String
                    },
                )?;
            }
            if let Some(dowe_components::PaginationProps {
                total: dowe_components::PaginationTotal::Signal(total),
                ..
            }) = props.pagination.as_ref()
            {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    total,
                    "Pagination total",
                    ViewPathExpectation::Number,
                )?;
            }
            validate_optional_action(path, actions, props.on_change.as_deref())?;
        }
        ViewNode::Candlestick { props } => {
            validate_candlestick_data(path, signals, &props.data)?;
        }
        ViewNode::Canvas { props } => {
            validate_canvas_scene(path, signals, &props.scene)?;
            validate_optional_action(path, actions, props.on_pointer.as_deref())?;
            validate_optional_action(path, actions, props.on_key.as_deref())?;
            validate_optional_action(path, actions, props.on_motion.as_deref())?;
        }
        ViewNode::Camera { props } => {
            validate_optional_action(path, actions, props.on_start.as_deref())?;
            validate_optional_action(path, actions, props.on_capture.as_deref())?;
            validate_optional_action(path, actions, props.on_error.as_deref())?;
        }
        ViewNode::Microphone { props } => {
            validate_optional_action(path, actions, props.on_start.as_deref())?;
            validate_optional_action(path, actions, props.on_stop.as_deref())?;
            validate_optional_action(path, actions, props.on_error.as_deref())?;
        }
        ViewNode::Image { props } => {
            if let Some(src) = props.reactive_src.as_deref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    src,
                    "src",
                    ViewPathExpectation::String,
                )?;
            }
        }
        ViewNode::ArcChart { props } => {
            validate_category_chart_common(path, signals, &props.common, "ArcChart")?;
        }
        ViewNode::PieChart { props } => {
            validate_category_chart_common(path, signals, &props.common, "PieChart")?;
        }
        ViewNode::BarChart { props } => {
            validate_category_or_series_chart_common(path, signals, &props.common, "BarChart")?;
        }
        ViewNode::AreaChart { props } => {
            validate_point_or_series_chart_common(path, signals, &props.common, "AreaChart")?;
        }
        ViewNode::LineChart { props } => {
            validate_point_or_series_chart_common(path, signals, &props.common, "LineChart")?;
        }
        ViewNode::Table { props } => {
            validate_table_data(path, signals, &props.data, &props.columns)?;
        }
        ViewNode::SideNav { items, .. } => {
            validate_side_nav_actions(path, items, actions)?;
        }
        ViewNode::RailNav { items, .. } => {
            for item in items {
                if let dowe_components::RailNavItem::Item(props) = item
                    && let Some(action) = props.on_click.as_ref()
                    && !actions.contains(action)
                {
                    return Err(DoweError::at_path(path, format!("unknown fn `{action}`")));
                }
            }
        }
        ViewNode::NavMenu { props, items } => {
            for (name, binding) in [
                ("variant", props.style.reactive.variant.as_deref()),
                ("scheme", props.style.reactive.scheme.as_deref()),
            ] {
                if let Some(binding) = binding {
                    validate_typed_path(path, signals, locals, binding, name, ViewPathExpectation::String)?;
                    if let Some(ViewSignalValue::String(value)) = signal_path_value(path, signals, locals, binding, name)? {
                        let allowed: &[&str] = if name == "variant" {
                            &["solid", "outlined", "ghost"]
                        } else {
                            &["primary", "secondary", "tertiary", "success", "info", "warning", "danger"]
                        };
                        if !allowed.contains(&value.as_str()) {
                            return Err(DoweError::at_path(path, format!("invalid initial value `{value}` for reactive NavMenu prop `{name}`")));
                        }
                    }
                }
            }
            validate_nav_menu_actions(path, items, actions)?;
            for group in node_child_groups(node) {
                for child in group {
                    validate_node_references(
                        path,
                        child,
                        signals,
                        writable_signals,
                        actions,
                        locals,
                    )?;
                }
            }
        }
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            validate_typed_path(
                path,
                signals,
                locals,
                &props.open,
                "open",
                ViewPathExpectation::Bool,
            )?;
            for child in header.iter().chain(body).chain(footer) {
                validate_node_references(path, child, signals, writable_signals, actions, locals)?;
            }
        }
        ViewNode::Chip { props, .. } => {
            validate_optional_action(path, actions, props.on_close.as_deref())?;
        }
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            validate_typed_path(
                path,
                signals,
                locals,
                &props.open,
                "open",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_action(path, actions, props.on_close.as_deref())?;
            for child in header.iter().chain(body).chain(footer) {
                validate_node_references(path, child, signals, writable_signals, actions, locals)?;
            }
        }
        ViewNode::AlertDialog { props } => {
            validate_typed_path(
                path,
                signals,
                locals,
                &props.open,
                "open",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_action(path, actions, props.on_confirm.as_deref())?;
            validate_optional_action(path, actions, props.on_cancel.as_deref())?;
        }
        ViewNode::Toast { props } => {
            if let Some(source) = props.source.as_ref() {
                validate_toast_source(path, signals, locals, source)?;
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            entries,
            footer,
            ..
        } => {
            validate_overlay_entry_actions(path, entries, actions)?;
            for child in trigger.iter().chain(header).chain(footer) {
                validate_node_references(path, child, signals, writable_signals, actions, locals)?;
            }
        }
        ViewNode::Command { props, entries } => {
            if let Some(open) = props.open.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    open,
                    "open",
                    ViewPathExpectation::Bool,
                )?;
            }
            validate_command_entry_actions(path, entries, actions)?;
        }
        ViewNode::AvatarGroup { props, items } => {
            if let Some(source) = props.items.as_ref() {
                validate_avatar_group_items(path, signals, locals, source)?;
            }
            validate_avatar_group_actions(path, items, actions)?;
        }
        ViewNode::ChatBox { props } => {
            validate_chat_box_messages(path, signals, locals, &props.messages)?;
            validate_optional_typed_path(
                path,
                signals,
                locals,
                props.loading.as_deref(),
                "loading",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_typed_path(
                path,
                signals,
                locals,
                props.sending.as_deref(),
                "sending",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_typed_path(
                path,
                signals,
                locals,
                props.streaming.as_deref(),
                "streaming",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_typed_path(
                path,
                signals,
                locals,
                props.has_more.as_deref(),
                "hasMore",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_action(path, actions, props.on_send.as_deref())?;
            validate_optional_action(path, actions, props.on_load_more.as_deref())?;
            validate_optional_action(path, actions, props.on_stop.as_deref())?;
            validate_optional_action(path, actions, props.on_voice_note.as_deref())?;
            validate_optional_action(path, actions, props.on_file_attach.as_deref())?;
            validate_optional_action(path, actions, props.on_camera_capture.as_deref())?;
        }
        ViewNode::DateRange { props } => {
            if let Some(start) = props.start.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    start,
                    "start",
                    ViewPathExpectation::String,
                )?;
            }
            if let Some(end) = props.end.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    end,
                    "end",
                    ViewPathExpectation::String,
                )?;
            }
        }
        ViewNode::Fab {
            actions: fab_actions,
            ..
        } => {
            for action in fab_actions {
                validate_optional_action(path, actions, action.on_click.as_deref())?;
            }
        }
        _ => {
            for group in node_child_groups(node) {
                for child in group {
                    validate_node_references(
                        path,
                        child,
                        signals,
                        writable_signals,
                        actions,
                        locals,
                    )?;
                }
            }
        }
    }
    Ok(())
}
