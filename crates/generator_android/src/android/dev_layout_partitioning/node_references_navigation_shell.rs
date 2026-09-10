fn dev_node_references_navigation_shell(
    node: &ViewNode,
    bindings: &DevLayoutBindings,
) -> Option<bool> {
    match node {
        ViewNode::Editor { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::ImageCropper { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .src
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::Password { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::Phone { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::Pin { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::Textarea { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::Slider { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.value)
        }),
        ViewNode::Dropzone { props } => Some(dev_variant_references_layout_bindings(&props.style, bindings)),
        ViewNode::Color { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.value)
        }),
        ViewNode::Date { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::DateRange { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .start
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .end
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::Fab { props, actions } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .style
                    .element
                    .on_click
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
                || actions.iter().any(|action| {
                    action
                        .on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value))
                })
        }),
        ViewNode::Candlestick { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.data)
        }),
        ViewNode::ArcChart { props } => Some(dev_chart_references_layout_bindings(&props.common, bindings)),
        ViewNode::AreaChart { props } => Some(dev_chart_references_layout_bindings(&props.common, bindings)),
        ViewNode::BarChart { props } => Some(dev_chart_references_layout_bindings(&props.common, bindings)),
        ViewNode::LineChart { props } => Some(dev_chart_references_layout_bindings(&props.common, bindings)),
        ViewNode::PieChart { props } => Some(dev_chart_references_layout_bindings(&props.common, bindings)),
        ViewNode::Table { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.data)
        }),
        ViewNode::Tree { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.data)
                || props.bind.as_deref().is_some_and(|value| bindings.references_signal(value))
                || props.on_select.as_deref().is_some_and(|value| bindings.references_action(value))
        }),
        ViewNode::Divider { props } => Some(dev_style_references_layout_bindings(&props.style, bindings)),
        ViewNode::Alert { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .visible
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .on_close
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }),
        ViewNode::Svg { props, .. } => Some(dev_style_references_layout_bindings(&props.style, bindings)),
        ViewNode::SideNav { props, items } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_side_nav_items_reference_layout_bindings(items, bindings)
        }),
        ViewNode::RailNav { props, items } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || items.iter().any(|item| match item {
                    RailNavItem::Item(props) => props
                        .on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value)),
                    RailNavItem::Divider => false,
                })
        }),
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || header
                    .iter()
                    .chain(body)
                    .chain(footer)
                    .any(|child| dev_node_references_layout_bindings(child, bindings))
        }),
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
        } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(top, bindings)
                || dev_children_reference_layout_bindings(start, bindings)
                || dev_children_reference_layout_bindings(center, bindings)
                || dev_children_reference_layout_bindings(end, bindings)
                || dev_children_reference_layout_bindings(bottom, bindings)
        }),
        ViewNode::BottomBar { props, .. } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
        }),
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
        } => Some({
            dev_style_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(app_bar, bindings)
                || dev_children_reference_layout_bindings(start, bindings)
                || dev_children_reference_layout_bindings(main, bindings)
                || dev_children_reference_layout_bindings(end, bindings)
                || dev_children_reference_layout_bindings(bottom_bar, bindings)
                || dev_children_reference_layout_bindings(overlays, bindings)
        }),
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.open)
                || dev_children_reference_layout_bindings(header, bindings)
                || dev_children_reference_layout_bindings(body, bindings)
                || dev_children_reference_layout_bindings(footer, bindings)
        }),
        ViewNode::Badge { props, children } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::Chip { props, .. } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .on_close
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }),
        ViewNode::Skeleton { props } => Some(dev_style_references_layout_bindings(&props.style, bindings)),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.open)
                || props
                    .on_close
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
                || dev_children_reference_layout_bindings(header, bindings)
                || dev_children_reference_layout_bindings(body, bindings)
                || dev_children_reference_layout_bindings(footer, bindings)
        }),
        ViewNode::AlertDialog { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.open)
                || props
                    .on_confirm
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
                || props
                    .on_cancel
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }),
        ViewNode::Tooltip { props, children } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::Toast { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .source
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::Dropdown {
            props,
            trigger,
            entries,
            header,
            footer,
        } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(trigger, bindings)
                || dev_overlay_entries_reference_layout_bindings(entries, bindings)
                || dev_children_reference_layout_bindings(header, bindings)
                || dev_children_reference_layout_bindings(footer, bindings)
        }),
        ViewNode::Command { props, entries } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .open
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || dev_command_entries_reference_layout_bindings(entries, bindings)
        }),
        ViewNode::AvatarGroup { props, items } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .items
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || items.iter().any(|item| {
                    item.on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value))
                })
        }),
        ViewNode::ChatBox { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.messages)
                || props
                    .loading
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .sending
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .streaming
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .has_more
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || [
                    props.on_send.as_deref(),
                    props.on_load_more.as_deref(),
                    props.on_stop.as_deref(),
                    props.on_voice_note.as_deref(),
                    props.on_file_attach.as_deref(),
                    props.on_camera_capture.as_deref(),
                ]
                .into_iter()
                .flatten()
                .any(|value| bindings.references_action(value))
        }),
        ViewNode::Marquee { props, children } => Some({
            dev_style_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::TypeWriter { props, .. } => Some({
            dev_style_references_layout_bindings(&props.style, bindings)
        }),
        ViewNode::RichText { props, .. } => Some({
            dev_style_references_layout_bindings(&props.style, bindings)
        }),
        ViewNode::Record { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || [
                    props.on_start.as_deref(),
                    props.on_pause.as_deref(),
                    props.on_resume.as_deref(),
                    props.on_stop.as_deref(),
                    props.on_discard.as_deref(),
                    props.on_confirm.as_deref(),
                ]
                .into_iter()
                .flatten()
                .any(|value| bindings.references_action(value))
        }),
        ViewNode::ToggleGroup { props, .. } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .on_change
                    .as_deref()
                .is_some_and(|value| bindings.references_action(value))
        }),
        ViewNode::Collapsible { props, children } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::Countdown { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .on_complete
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }),
        ViewNode::Map {
            props,
            markers,
            waypoints: _,
        } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || [
                    props.on_location.as_deref(),
                    props.on_location_error.as_deref(),
                    props.on_route.as_deref(),
                ]
                .into_iter()
                .flatten()
                .any(|value| bindings.references_action(value))
                || markers.iter().any(|marker| {
                    marker
                        .on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value))
                })
        }),
        ViewNode::Accordion { props, items } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || items.iter().any(|item| {
                    dev_children_reference_layout_bindings(&item.children, bindings)
                })
        }),
        ViewNode::Carousel { props, slides } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || slides.iter().any(|slide| {
                    dev_children_reference_layout_bindings(&slide.children, bindings)
                })
        }),
        ViewNode::Tabs { props, tabs } => Some({
            dev_style_references_layout_bindings(&props.style, bindings)
                || tabs.iter().any(|tab| {
                    dev_children_reference_layout_bindings(&tab.children, bindings)
                })
        }),
        _ => None,
    }
}
