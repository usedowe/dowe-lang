fn reusable_dev_layouts(routes: &[ViewRoute]) -> (Vec<&ViewNode>, Vec<Option<usize>>) {
    let mut layouts = Vec::new();
    let mut route_layouts = Vec::new();
    for route in routes {
        if !dev_layout_can_split(&route.layout_tree)
            || dev_page_references_layout_bindings(&route.layout_tree, &route.page_tree)
        {
            route_layouts.push(None);
            continue;
        }
        let index = layouts
            .iter()
            .position(|layout| *layout == &route.layout_tree)
            .unwrap_or_else(|| {
                layouts.push(&route.layout_tree);
                layouts.len() - 1
            });
        route_layouts.push(Some(index));
    }
    (layouts, route_layouts)
}

fn dev_page_references_layout_bindings(layout: &ViewNode, page: &ViewNode) -> bool {
    let mut bindings = DevLayoutBindings::default();
    dev_collect_scope_bindings(layout, &mut bindings);
    if bindings.is_empty() {
        return false;
    }
    dev_node_references_layout_bindings(page, &bindings)
}

#[derive(Clone, Default)]
struct DevLayoutBindings {
    signal_names: std::collections::BTreeSet<String>,
    signal_ids: std::collections::BTreeSet<String>,
    action_names: std::collections::BTreeSet<String>,
    action_ids: std::collections::BTreeSet<String>,
}

impl DevLayoutBindings {
    fn is_empty(&self) -> bool {
        self.signal_names.is_empty()
            && self.signal_ids.is_empty()
            && self.action_names.is_empty()
            && self.action_ids.is_empty()
    }

    fn with_scope_shadowed(&self, signals: &[ViewSignal], actions: &[ViewAction]) -> Self {
        let mut next = self.clone();
        for signal in signals {
            next.signal_names.remove(&signal.name);
            next.signal_ids.remove(&signal.id);
        }
        for action in actions {
            next.action_names.remove(&action.name);
            next.action_ids.remove(&action.id);
        }
        next
    }

    fn references_signal(&self, path: &str) -> bool {
        let root = path.split_once('.').map(|(root, _)| root).unwrap_or(path);
        self.signal_names.contains(root) || self.signal_ids.contains(root)
    }

    fn references_action(&self, name: &str) -> bool {
        self.action_names.contains(name) || self.action_ids.contains(name)
    }
}

fn dev_collect_scope_bindings(node: &ViewNode, bindings: &mut DevLayoutBindings) {
    match node {
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => {
            for signal in signals {
                bindings.signal_ids.insert(signal.id.clone());
                bindings.signal_names.insert(signal.name.clone());
            }
            for action in actions {
                bindings.action_ids.insert(action.id.clone());
                bindings.action_names.insert(action.name.clone());
            }
            dev_collect_scope_bindings_from_children(children, bindings);
        }
        ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Collapsible { children, .. }
        | ViewNode::Each { children, .. }
        | ViewNode::Button { children, .. } => {
            dev_collect_scope_bindings_from_children(children, bindings);
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => {
            dev_collect_scope_bindings_from_children(header, bindings);
            dev_collect_scope_bindings_from_children(body, bindings);
            dev_collect_scope_bindings_from_children(footer, bindings);
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            ..
        } => {
            dev_collect_scope_bindings_from_children(app_bar, bindings);
            dev_collect_scope_bindings_from_children(start, bindings);
            dev_collect_scope_bindings_from_children(main, bindings);
            dev_collect_scope_bindings_from_children(end, bindings);
            dev_collect_scope_bindings_from_children(bottom_bar, bindings);
        }
        ViewNode::AppBar {
            start, center, end, ..
        }
        | ViewNode::Footer {
            start, center, end, ..
        }
        | ViewNode::BottomBar {
            start, center, end, ..
        } => {
            dev_collect_scope_bindings_from_children(start, bindings);
            dev_collect_scope_bindings_from_children(center, bindings);
            dev_collect_scope_bindings_from_children(end, bindings);
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            dev_collect_scope_bindings_from_children(header, bindings);
            dev_collect_scope_bindings_from_children(body, bindings);
            dev_collect_scope_bindings_from_children(footer, bindings);
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            dev_collect_scope_bindings_from_children(trigger, bindings);
            dev_collect_scope_bindings_from_children(header, bindings);
            dev_collect_scope_bindings_from_children(footer, bindings);
        }
        ViewNode::Tooltip { children, .. } => {
            dev_collect_scope_bindings_from_children(children, bindings);
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                dev_collect_scope_bindings_from_children(&tab.children, bindings);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let NavMenuItem::Megamenu { content, .. } = item {
                    dev_collect_scope_bindings_from_children(content, bindings);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                dev_collect_scope_bindings_from_children(&item.children, bindings);
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                dev_collect_scope_bindings_from_children(&slide.children, bindings);
            }
        }
        ViewNode::Fab { .. }
        | ViewNode::Input { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::PasswordField { .. }
        | ViewNode::PhoneField { .. }
        | ViewNode::PinField { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Table { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Svg { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::Avatar { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Command { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::Children => {}
    }
}

fn dev_collect_scope_bindings_from_children(
    children: &[ViewNode],
    bindings: &mut DevLayoutBindings,
) {
    for child in children {
        dev_collect_scope_bindings(child, bindings);
    }
}

fn dev_node_references_layout_bindings(node: &ViewNode, bindings: &DevLayoutBindings) -> bool {
    match node {
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => {
            let bindings = bindings.with_scope_shadowed(signals, actions);
            actions
                .iter()
                .any(|action| dev_action_references_layout_bindings(action, &bindings))
                || dev_children_reference_layout_bindings(children, &bindings)
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            dev_style_references_layout_bindings(props, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Flex { props, children } => {
            dev_layout_references_layout_bindings(props, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Grid { props, children } => {
            dev_grid_references_layout_bindings(props, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Card { props, children } | ViewNode::Button { props, children } => {
            dev_variant_references_layout_bindings(props, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Text { props, value } | ViewNode::Title { props, value } => {
            dev_text_references_layout_bindings(props, value, bindings)
        }
        ViewNode::Input { props } | ViewNode::Select { props, .. } => {
            dev_variant_references_layout_bindings(props, bindings)
        }
        ViewNode::Audio { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Image { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Code { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Video { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Checkbox { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::RadioGroup { props, .. } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Toggle { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::ToggleTheme { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Empty { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::ComboBox { props, .. } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::CsvField { props, .. } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::DragDrop { props, .. } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Editor { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::ImageCropper { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .src
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::PasswordField { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::PhoneField { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::PinField { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Textarea { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Slider { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.value)
        }
        ViewNode::Dropzone { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Color { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.value)
        }
        ViewNode::Date { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::DateRange { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .start
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .end
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Fab { props, actions } => {
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
        }
        ViewNode::Candlestick { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.data)
        }
        ViewNode::ArcChart { props } => dev_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::AreaChart { props } => dev_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::BarChart { props } => dev_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::LineChart { props } => dev_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::PieChart { props } => dev_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::Table { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.data)
        }
        ViewNode::Divider { props } => dev_style_references_layout_bindings(&props.style, bindings),
        ViewNode::Alert { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .visible
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .on_close
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }
        ViewNode::Svg { props, .. } => dev_style_references_layout_bindings(&props.style, bindings),
        ViewNode::SideNav { props, items } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_side_nav_items_reference_layout_bindings(items, bindings)
        }
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || header
                    .iter()
                    .chain(body)
                    .chain(footer)
                    .any(|child| dev_node_references_layout_bindings(child, bindings))
        }
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        }
        | ViewNode::Footer {
            props,
            start,
            center,
            end,
        }
        | ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(start, bindings)
                || dev_children_reference_layout_bindings(center, bindings)
                || dev_children_reference_layout_bindings(end, bindings)
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            dev_style_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(app_bar, bindings)
                || dev_children_reference_layout_bindings(start, bindings)
                || dev_children_reference_layout_bindings(main, bindings)
                || dev_children_reference_layout_bindings(end, bindings)
                || dev_children_reference_layout_bindings(bottom_bar, bindings)
        }
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.open)
                || dev_children_reference_layout_bindings(header, bindings)
                || dev_children_reference_layout_bindings(body, bindings)
                || dev_children_reference_layout_bindings(footer, bindings)
        }
        ViewNode::Badge { props, children } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Chip { props, .. } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .on_close
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }
        ViewNode::Skeleton { props } => dev_style_references_layout_bindings(&props.style, bindings),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.open)
                || props
                    .on_close
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
                || dev_children_reference_layout_bindings(header, bindings)
                || dev_children_reference_layout_bindings(body, bindings)
                || dev_children_reference_layout_bindings(footer, bindings)
        }
        ViewNode::AlertDialog { props } => {
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
        }
        ViewNode::Tooltip { props, children } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Toast { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
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
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(trigger, bindings)
                || dev_overlay_entries_reference_layout_bindings(entries, bindings)
                || dev_children_reference_layout_bindings(header, bindings)
                || dev_children_reference_layout_bindings(footer, bindings)
        }
        ViewNode::Command { props, entries } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .open
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || dev_command_entries_reference_layout_bindings(entries, bindings)
        }
        ViewNode::AvatarGroup { props, items } => {
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
        }
        ViewNode::ChatBox { props } => {
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
        }
        ViewNode::Marquee { props, children } => {
            dev_style_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::TypeWriter { props, .. } => {
            dev_style_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::RichText { props, .. } => {
            dev_style_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Record { props } => {
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
        }
        ViewNode::ToggleGroup { props, .. } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .on_change
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }
        ViewNode::Collapsible { props, children } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Countdown { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .on_complete
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }
        ViewNode::Map {
            props,
            markers,
            waypoints: _,
        } => {
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
        }
        ViewNode::Accordion { props, items } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || items.iter().any(|item| {
                    dev_children_reference_layout_bindings(&item.children, bindings)
                })
        }
        ViewNode::Carousel { props, slides } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || slides.iter().any(|slide| {
                    dev_children_reference_layout_bindings(&slide.children, bindings)
                })
        }
        ViewNode::Tabs { props, tabs } => {
            dev_style_references_layout_bindings(&props.style, bindings)
                || tabs.iter().any(|tab| {
                    dev_children_reference_layout_bindings(&tab.children, bindings)
                })
        }
        ViewNode::NavMenu { props, items } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_nav_menu_items_reference_layout_bindings(items, bindings)
        }
        ViewNode::Each {
            children,
            collection,
            ..
        } => {
            bindings.references_signal(collection)
                || dev_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Avatar { props, .. } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Children => false,
    }
}

fn dev_children_reference_layout_bindings(
    children: &[ViewNode],
    bindings: &DevLayoutBindings,
) -> bool {
    children
        .iter()
        .any(|child| dev_node_references_layout_bindings(child, bindings))
}

fn dev_action_references_layout_bindings(
    action: &ViewAction,
    bindings: &DevLayoutBindings,
) -> bool {
    match &action.kind {
        ViewActionKind::Request(request) => [
            request.body.as_deref(),
            request.update.as_deref(),
            request.reset.as_deref(),
            request.success_alert.as_deref(),
            request.error_alert.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|value| bindings.references_signal(value)),
        ViewActionKind::Assign(assign) => {
            bindings.references_signal(&assign.target) || bindings.references_signal(&assign.source)
        }
        ViewActionKind::Reset(reset) => bindings.references_signal(&reset.target),
    }
}

fn dev_element_references_layout_bindings(
    props: &ElementProps,
    bindings: &DevLayoutBindings,
) -> bool {
    props
        .bind
        .as_deref()
        .is_some_and(|value| bindings.references_signal(value))
        || props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value))
        || props
            .show
            .as_ref()
            .is_some_and(|value| dev_visibility_references_layout_bindings(value, bindings))
}

fn dev_visibility_references_layout_bindings(
    value: &VisibilityCondition,
    bindings: &DevLayoutBindings,
) -> bool {
    match value {
        VisibilityCondition::Static(_) => false,
        VisibilityCondition::Signal(path) => bindings.references_signal(path),
    }
}

fn dev_style_references_layout_bindings(
    props: &StyleProps,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_element_references_layout_bindings(&props.element, bindings)
}

fn dev_layout_references_layout_bindings(
    props: &LayoutProps,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_style_references_layout_bindings(&props.style, bindings)
}

fn dev_grid_references_layout_bindings(
    props: &GridProps,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_style_references_layout_bindings(&props.style, bindings)
}

fn dev_variant_references_layout_bindings(
    props: &VariantProps,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_element_references_layout_bindings(&props.element, bindings)
        || dev_style_references_layout_bindings(&props.style, bindings)
}

fn dev_text_references_layout_bindings(
    props: &TextProps,
    value: &str,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_style_references_layout_bindings(&props.style, bindings)
        || (props.i18n.is_none() && bindings.references_signal(value))
}

fn dev_chart_references_layout_bindings(
    props: &ChartCommonProps,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_variant_references_layout_bindings(&props.style, bindings)
        || props
            .data
            .as_deref()
            .is_some_and(|value| bindings.references_signal(value))
        || props
            .series
            .as_deref()
            .is_some_and(|value| bindings.references_signal(value))
}

fn dev_side_nav_items_reference_layout_bindings(
    items: &[SideNavItem],
    bindings: &DevLayoutBindings,
) -> bool {
    items.iter().any(|item| match item {
        SideNavItem::Header(props) | SideNavItem::Item(props) => props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value)),
        SideNavItem::Submenu { props, items, .. } => {
            props
                .on_click
                .as_deref()
                .is_some_and(|value| bindings.references_action(value))
                || items.iter().any(|item| {
                    item.on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value))
                })
        }
        SideNavItem::Divider => false,
    })
}

fn dev_nav_menu_items_reference_layout_bindings(
    items: &[NavMenuItem],
    bindings: &DevLayoutBindings,
) -> bool {
    items.iter().any(|item| match item {
        NavMenuItem::Item(props) => props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value)),
        NavMenuItem::Submenu { props, items } => {
            props
                .on_click
                .as_deref()
                .is_some_and(|value| bindings.references_action(value))
                || items.iter().any(|item| {
                    item.on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value))
                })
        }
        NavMenuItem::Megamenu { props, content } => {
            props
                .on_click
                .as_deref()
                .is_some_and(|value| bindings.references_action(value))
                || dev_children_reference_layout_bindings(content, bindings)
        }
    })
}

fn dev_overlay_entries_reference_layout_bindings(
    entries: &[OverlayEntry],
    bindings: &DevLayoutBindings,
) -> bool {
    entries.iter().any(|entry| match entry {
        OverlayEntry::Item(props) => props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value)),
        OverlayEntry::Divider => false,
    })
}

fn dev_command_entries_reference_layout_bindings(
    entries: &[CommandEntry],
    bindings: &DevLayoutBindings,
) -> bool {
    entries.iter().any(|entry| match entry {
        CommandEntry::Item(props) => props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value)),
        CommandEntry::Group { items, .. } => items.iter().any(|item| {
            item.on_click
                .as_deref()
                .is_some_and(|value| bindings.references_action(value))
        }),
    })
}

fn dev_layout_can_split(layout: &ViewNode) -> bool {
    let boundary = dev_children_boundary(layout, true, false);
    boundary.count == 1 && boundary.valid
}

struct DevChildrenBoundary {
    count: usize,
    valid: bool,
}

impl Default for DevChildrenBoundary {
    fn default() -> Self {
        Self {
            count: 0,
            valid: true,
        }
    }
}

fn dev_children_boundary(
    node: &ViewNode,
    neutral_context: bool,
    parent_horizontal: bool,
) -> DevChildrenBoundary {
    match node {
        ViewNode::Children => DevChildrenBoundary {
            count: 1,
            valid: neutral_context && !parent_horizontal,
        },
        ViewNode::Scope { children, .. } => {
            dev_children_boundaries(children, neutral_context, parent_horizontal)
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            dev_children_boundaries(
                children,
                neutral_context && props.font.is_none() && props.text.is_none(),
                false,
            )
        }
        ViewNode::Grid { props, children } => dev_children_boundaries(
            children,
            neutral_context && props.style.font.is_none() && props.style.text.is_none(),
            false,
        ),
        ViewNode::Flex { children, .. } => dev_children_boundaries(children, neutral_context, true),
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            let neutral =
                neutral_context && props.style.font.is_none() && props.style.text.is_none();
            dev_merge_children_boundaries([
                dev_children_boundaries(app_bar, neutral, false),
                dev_children_boundaries(start, neutral, true),
                dev_children_boundaries(main, neutral, false),
                dev_children_boundaries(end, neutral, true),
                dev_children_boundaries(bottom_bar, neutral, false),
            ])
        }
        ViewNode::Tooltip { props, children } => dev_children_boundaries(
            children,
            neutral_context && props.style.style.font.is_none(),
            false,
        ),
        ViewNode::AppBar {
            start, center, end, ..
        }
        | ViewNode::Footer {
            start, center, end, ..
        }
        | ViewNode::BottomBar {
            start, center, end, ..
        } => dev_merge_children_boundaries([
            dev_children_boundaries(start, false, true),
            dev_children_boundaries(center, false, true),
            dev_children_boundaries(end, false, true),
        ]),
        ViewNode::Card { children, .. } | ViewNode::Badge { children, .. } => {
            dev_children_boundaries(children, false, false)
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => dev_merge_children_boundaries([
            dev_children_boundaries(header, false, false),
            dev_children_boundaries(body, false, false),
            dev_children_boundaries(footer, false, false),
        ]),
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => dev_merge_children_boundaries([
            dev_children_boundaries(header, false, false),
            dev_children_boundaries(body, false, false),
            dev_children_boundaries(footer, false, false),
        ]),
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => dev_merge_children_boundaries([
            dev_children_boundaries(trigger, false, false),
            dev_children_boundaries(header, false, false),
            dev_children_boundaries(footer, false, false),
        ]),
        ViewNode::Tabs { tabs, .. } => dev_merge_children_boundaries(
            tabs.iter()
                .map(|tab| dev_children_boundaries(&tab.children, false, false)),
        ),
        ViewNode::NavMenu { items, .. } => {
            dev_merge_children_boundaries(items.iter().filter_map(|item| match item {
                NavMenuItem::Megamenu { content, .. } => {
                    Some(dev_children_boundaries(content, false, false))
                }
                _ => None,
            }))
        }
        ViewNode::Accordion { items, .. } => dev_merge_children_boundaries(
            items
                .iter()
                .map(|item| dev_children_boundaries(&item.children, false, false)),
        ),
        ViewNode::Carousel { slides, .. } => dev_merge_children_boundaries(
            slides
                .iter()
                .map(|slide| dev_children_boundaries(&slide.children, false, false)),
        ),
        ViewNode::Marquee { children, .. } | ViewNode::Collapsible { children, .. } => {
            dev_children_boundaries(children, false, false)
        }
        ViewNode::Each { children, .. } => dev_children_boundaries(children, false, false),
        ViewNode::Button { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Input { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::PasswordField { .. }
        | ViewNode::PhoneField { .. }
        | ViewNode::PinField { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Table { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Svg { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::Avatar { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Command { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::TypeWriter { .. } => DevChildrenBoundary::default(),
    }
}

fn dev_children_boundaries(
    children: &[ViewNode],
    neutral_context: bool,
    parent_horizontal: bool,
) -> DevChildrenBoundary {
    dev_merge_children_boundaries(
        children
            .iter()
            .map(|child| dev_children_boundary(child, neutral_context, parent_horizontal)),
    )
}

fn dev_merge_children_boundaries(
    boundaries: impl IntoIterator<Item = DevChildrenBoundary>,
) -> DevChildrenBoundary {
    boundaries.into_iter().fold(
        DevChildrenBoundary {
            count: 0,
            valid: true,
        },
        |mut combined, boundary| {
            combined.count += boundary.count;
            combined.valid &= boundary.valid;
            combined
        },
    )
}
