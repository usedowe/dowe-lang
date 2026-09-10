fn ios_navigation_node_references_layout_bindings(node: &ViewNode, bindings: &IosLayoutBindings) -> bool {
    match node {
        ViewNode::Divider { props } => ios_style_references_layout_bindings(&props.style, bindings),
        ViewNode::Alert { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .visible
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .on_close
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }
        ViewNode::Svg { props, .. } => ios_style_references_layout_bindings(&props.style, bindings),
        ViewNode::SideNav { props, items } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || ios_side_nav_items_reference_layout_bindings(items, bindings)
        }
        ViewNode::RailNav { props, items } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || items.iter().any(|item| match item {
                    RailNavItem::Item(props) => props
                        .on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value)),
                    RailNavItem::Divider => false,
                })
        }
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || header
                    .iter()
                    .chain(body)
                    .chain(footer)
                    .any(|child| ios_node_references_layout_bindings(child, bindings))
        }
        ViewNode::AppBar {
            props,
            top,
            start,
            center,
            end,
            bottom,
            ..
        }
        | ViewNode::Footer {
            props,
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(top, bindings)
                || ios_children_reference_layout_bindings(start, bindings)
                || ios_children_reference_layout_bindings(center, bindings)
                || ios_children_reference_layout_bindings(end, bindings)
                || ios_children_reference_layout_bindings(bottom, bindings)
        }
        ViewNode::BottomBar { props, .. } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
        } => {
            ios_style_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(app_bar, bindings)
                || ios_children_reference_layout_bindings(start, bindings)
                || ios_children_reference_layout_bindings(main, bindings)
                || ios_children_reference_layout_bindings(end, bindings)
                || ios_children_reference_layout_bindings(bottom_bar, bindings)
                || ios_children_reference_layout_bindings(overlays, bindings)
        }
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.open)
                || ios_children_reference_layout_bindings(header, bindings)
                || ios_children_reference_layout_bindings(body, bindings)
                || ios_children_reference_layout_bindings(footer, bindings)
        }
        ViewNode::Badge { props, children } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Chip { props, .. } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .on_close
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }
        ViewNode::Skeleton { props } => ios_style_references_layout_bindings(&props.style, bindings),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.open)
                || props
                    .on_close
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
                || ios_children_reference_layout_bindings(header, bindings)
                || ios_children_reference_layout_bindings(body, bindings)
                || ios_children_reference_layout_bindings(footer, bindings)
        }
        ViewNode::AlertDialog { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.open)
                || props
                    .on_confirm
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
                || props
                    .on_cancel
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }
        ViewNode::Tooltip { props, children } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Toast { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .source
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Dropdown {
            props,
            trigger,
            entries,
            header,
            footer,
        } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(trigger, bindings)
                || ios_overlay_entries_reference_layout_bindings(entries, bindings)
                || ios_children_reference_layout_bindings(header, bindings)
                || ios_children_reference_layout_bindings(footer, bindings)
        }
        ViewNode::Command { props, entries } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .open
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || ios_command_entries_reference_layout_bindings(entries, bindings)
        }
        ViewNode::AvatarGroup { props, items } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .items
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || items.iter().any(|item| {
                    item.on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value))
                })
        }
        _ => false
    }
}
