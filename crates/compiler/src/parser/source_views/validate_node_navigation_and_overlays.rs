fn validate_node_navigation_and_overlays(
    path: &Path,
    node: &ViewNode,
    signals: &HashMap<String, ViewSignalValue>,
    writable_signals: &HashSet<String>,
    actions: &HashSet<String>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
) -> Option<DoweResult<()>> {
    match node {
        ViewNode::SideNav { items, .. } => Some((|| -> DoweResult<()> {
            validate_side_nav_actions(path, items, actions)?;
            Ok(())
        })()),
        ViewNode::RailNav { items, .. } => Some((|| -> DoweResult<()> {
            for item in items {
                if let dowe_components::RailNavItem::Item(props) = item
                    && let Some(action) = props.on_click.as_ref()
                    && !actions.contains(action)
                {
                    return Err(DoweError::at_path(path, format!("unknown fn `{action}`")));
                }
            }
            Ok(())
        })()),
        ViewNode::NavMenu { props, items } => Some((|| -> DoweResult<()> {
            for (name, binding) in [
                ("variant", props.style.reactive.variant.as_deref()),
                ("scheme", props.style.reactive.scheme.as_deref()),
            ] {
                if let Some(binding) = binding {
                    validate_typed_path(
                        path,
                        signals,
                        locals,
                        binding,
                        name,
                        ViewPathExpectation::String,
                    )?;
                    if let Some(ViewSignalValue::String(value)) =
                        signal_path_value(path, signals, locals, binding, name)?
                    {
                        let allowed: &[&str] = if name == "variant" {
                            &["solid", "outlined", "ghost"]
                        } else {
                            &[
                                "primary",
                                "secondary",
                                "accent",
                                "success",
                                "info",
                                "warning",
                                "danger",
                            ]
                        };
                        if !allowed.contains(&value.as_str()) {
                            return Err(DoweError::at_path(
                                path,
                                format!(
                                    "invalid initial value `{value}` for reactive NavMenu prop `{name}`"
                                ),
                            ));
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
            Ok(())
        })()),
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => Some((|| -> DoweResult<()> {
            validate_typed_path(
                path,
                signals,
                locals,
                &props.open,
                "bind",
                ViewPathExpectation::Bool,
            )?;
            for child in header.iter().chain(body).chain(footer) {
                validate_node_references(path, child, signals, writable_signals, actions, locals)?;
            }
            Ok(())
        })()),
        ViewNode::Chip { props, .. } => Some((|| -> DoweResult<()> {
            validate_optional_action(path, actions, props.on_close.as_deref())?;
            Ok(())
        })()),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => Some((|| -> DoweResult<()> {
            validate_typed_path(
                path,
                signals,
                locals,
                &props.open,
                "bind",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_action(path, actions, props.on_close.as_deref())?;
            for child in header.iter().chain(body).chain(footer) {
                validate_node_references(path, child, signals, writable_signals, actions, locals)?;
            }
            Ok(())
        })()),
        ViewNode::AlertDialog { props } => Some((|| -> DoweResult<()> {
            validate_typed_path(
                path,
                signals,
                locals,
                &props.open,
                "bind",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_action(path, actions, props.on_confirm.as_deref())?;
            validate_optional_action(path, actions, props.on_cancel.as_deref())?;
            Ok(())
        })()),
        ViewNode::Toast { props } => Some((|| -> DoweResult<()> {
            if let Some(source) = props.source.as_ref() {
                validate_toast_source(path, signals, locals, source)?;
            }
            Ok(())
        })()),
        ViewNode::Dropdown {
            trigger,
            header,
            entries,
            footer,
            ..
        } => Some((|| -> DoweResult<()> {
            validate_overlay_entry_actions(path, entries, actions)?;
            for child in trigger.iter().chain(header).chain(footer) {
                validate_node_references(path, child, signals, writable_signals, actions, locals)?;
            }
            Ok(())
        })()),
        ViewNode::Command { props, entries } => Some((|| -> DoweResult<()> {
            if let Some(open) = props.open.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    open,
                    "bind",
                    ViewPathExpectation::Bool,
                )?;
            }
            validate_command_entry_actions(path, entries, actions)?;
            Ok(())
        })()),
        ViewNode::AvatarGroup { props, items } => Some((|| -> DoweResult<()> {
            if let Some(source) = props.items.as_ref() {
                validate_avatar_group_items(path, signals, locals, source)?;
            }
            validate_avatar_group_actions(path, items, actions)?;
            Ok(())
        })()),
        ViewNode::ChatBox { props } => Some((|| -> DoweResult<()> {
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
            Ok(())
        })()),
        ViewNode::DateRange { props } => Some((|| -> DoweResult<()> {
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
            Ok(())
        })()),
        ViewNode::Fab {
            actions: fab_actions,
            ..
        } => Some((|| -> DoweResult<()> {
            for action in fab_actions {
                validate_optional_action(path, actions, action.on_click.as_deref())?;
            }
            Ok(())
        })()),
        _ => None,
    }
}
