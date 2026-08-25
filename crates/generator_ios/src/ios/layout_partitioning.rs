fn reusable_ios_layouts(routes: &[ViewRoute]) -> (Vec<&ViewNode>, Vec<Option<usize>>) {
    let mut layouts = Vec::new();
    let mut route_layouts = Vec::new();
    for route in routes {
        let can_split = ios_layout_can_split(&route.layout_tree);
        let references_layout =
            ios_page_references_layout_bindings(&route.layout_tree, &route.page_tree);
        if !can_split || references_layout {
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

fn ios_page_references_layout_bindings(layout: &ViewNode, page: &ViewNode) -> bool {
    let mut bindings = IosLayoutBindings::default();
    ios_collect_scope_bindings(layout, &mut bindings);
    if bindings.is_empty() {
        return false;
    }
    ios_node_references_layout_bindings(page, &bindings)
}

#[derive(Clone, Default)]
struct IosLayoutBindings {
    signal_names: std::collections::BTreeSet<String>,
    signal_ids: std::collections::BTreeSet<String>,
    action_names: std::collections::BTreeSet<String>,
    action_ids: std::collections::BTreeSet<String>,
}

impl IosLayoutBindings {
    fn is_empty(&self) -> bool {
        self.signal_names.is_empty()
            && self.signal_ids.is_empty()
            && self.action_names.is_empty()
            && self.action_ids.is_empty()
    }

    fn with_scope_shadowed(
        &self,
        constants: &[ViewConstant],
        signals: &[ViewSignal],
        actions: &[ViewAction],
    ) -> Self {
        let mut next = self.clone();
        for constant in constants {
            next.signal_names.remove(&constant.name);
            next.signal_ids.remove(&constant.id);
        }
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

fn ios_collect_scope_bindings(node: &ViewNode, bindings: &mut IosLayoutBindings) {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => {
            ios_collect_scope_bindings_from_children(content, bindings);
            ios_collect_scope_bindings_from_children(children, bindings);
        }
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
            ..
        } => {
            for constant in constants {
                bindings.signal_ids.insert(constant.id.clone());
                bindings.signal_names.insert(constant.name.clone());
            }
            for signal in signals {
                bindings.signal_ids.insert(signal.id.clone());
                bindings.signal_names.insert(signal.name.clone());
            }
            for action in actions {
                bindings.action_ids.insert(action.id.clone());
                bindings.action_names.insert(action.name.clone());
            }
            ios_collect_scope_bindings_from_children(children, bindings);
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
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. }
        | ViewNode::Button { children, .. } => {
            ios_collect_scope_bindings_from_children(children, bindings);
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => {
            ios_collect_scope_bindings_from_children(header, bindings);
            ios_collect_scope_bindings_from_children(body, bindings);
            ios_collect_scope_bindings_from_children(footer, bindings);
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            ..
        } => {
            ios_collect_scope_bindings_from_children(app_bar, bindings);
            ios_collect_scope_bindings_from_children(start, bindings);
            ios_collect_scope_bindings_from_children(main, bindings);
            ios_collect_scope_bindings_from_children(end, bindings);
            ios_collect_scope_bindings_from_children(bottom_bar, bindings);
            ios_collect_scope_bindings_from_children(overlays, bindings);
        }
        ViewNode::AppBar {
            top, start, center, end, bottom, ..
        }
        | ViewNode::Footer {
            top, start, center, end, bottom, ..
        } => {
            ios_collect_scope_bindings_from_children(top, bindings);
            ios_collect_scope_bindings_from_children(start, bindings);
            ios_collect_scope_bindings_from_children(center, bindings);
            ios_collect_scope_bindings_from_children(end, bindings);
            ios_collect_scope_bindings_from_children(bottom, bindings);
        }
        ViewNode::BottomBar { .. } => {}
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            ios_collect_scope_bindings_from_children(header, bindings);
            ios_collect_scope_bindings_from_children(body, bindings);
            ios_collect_scope_bindings_from_children(footer, bindings);
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            ios_collect_scope_bindings_from_children(trigger, bindings);
            ios_collect_scope_bindings_from_children(header, bindings);
            ios_collect_scope_bindings_from_children(footer, bindings);
        }
        ViewNode::Tooltip { children, .. } => {
            ios_collect_scope_bindings_from_children(children, bindings);
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                ios_collect_scope_bindings_from_children(&tab.children, bindings);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let NavMenuItem::Megamenu { content, .. } = item {
                    ios_collect_scope_bindings_from_children(content, bindings);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                ios_collect_scope_bindings_from_children(&item.children, bindings);
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                ios_collect_scope_bindings_from_children(&slide.children, bindings);
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
        | ViewNode::Password { .. }
        | ViewNode::Phone { .. }
        | ViewNode::Pin { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Iframe { .. }
        | ViewNode::Device { .. }
        | ViewNode::Canvas { .. }
        | ViewNode::Diagram { .. }
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
        | ViewNode::RailNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::Avatar { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::SelectTheme { .. }
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

fn ios_collect_scope_bindings_from_children(
    children: &[ViewNode],
    bindings: &mut IosLayoutBindings,
) {
    for child in children {
        ios_collect_scope_bindings(child, bindings);
    }
}

fn ios_node_references_layout_bindings(node: &ViewNode, bindings: &IosLayoutBindings) -> bool {
    match node {
        ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } => {
            bindings.references_signal(binding)
                || ios_children_reference_layout_bindings(content, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
            ..
        } => {
            let bindings = bindings.with_scope_shadowed(constants, signals, actions);
            actions
                .iter()
                .any(|action| ios_action_references_layout_bindings(action, &bindings))
                || ios_children_reference_layout_bindings(children, &bindings)
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            ios_style_references_layout_bindings(props, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Flex { props, children } => {
            ios_layout_references_layout_bindings(props, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Grid { props, children } => {
            ios_grid_references_layout_bindings(props, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Brand { props, children } => {
            ios_style_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Banner { props, children } => {
            ios_style_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Card { props, children } | ViewNode::Button { props, children } => {
            ios_variant_references_layout_bindings(props, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Text { props, value } | ViewNode::Title { props, value } => {
            ios_text_references_layout_bindings(props, value, bindings)
        }
        ViewNode::Input { props } | ViewNode::Select { props, .. } => {
            ios_variant_references_layout_bindings(props, bindings)
        }
        ViewNode::Audio { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Image { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .reactive_src
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Camera { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Microphone { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Code { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Video { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Iframe { props } => ios_style_references_layout_bindings(&props.style, bindings),
        ViewNode::Device { props, .. } => ios_style_references_layout_bindings(&props.style, bindings),
        ViewNode::Canvas { props } => {
            ios_style_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.scene)
                || props
                    .on_pointer
                    .iter()
                    .chain(&props.on_key)
                    .chain(&props.on_motion)
                    .any(|value| bindings.references_action(value))
        }
        ViewNode::Diagram { props } => {
            ios_style_references_layout_bindings(&props.style.style, bindings)
                || bindings.references_signal(&props.nodes)
                || bindings.references_signal(&props.edges)
                || props
                    .on_node_click
                    .iter()
                    .chain(&props.on_node_drag)
                    .chain(&props.on_connect)
                    .any(|value| bindings.references_action(value))
        }
        ViewNode::Checkbox { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::RadioGroup { props, .. } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Toggle { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::ToggleTheme { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::SelectTheme { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Empty { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::ComboBox { props, .. } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::CsvField { props, .. } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::DragDrop { props, .. } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Editor { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::ImageCropper { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .src
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Password { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Phone { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Pin { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Textarea { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Slider { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.value)
        }
        ViewNode::Dropzone { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Color { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.value)
        }
        ViewNode::Date { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::DateRange { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
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
            ios_variant_references_layout_bindings(&props.style, bindings)
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
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.data)
        }
        ViewNode::ArcChart { props } => ios_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::AreaChart { props } => ios_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::BarChart { props } => ios_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::LineChart { props } => ios_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::PieChart { props } => ios_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::Table { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.data)
        }
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
        ViewNode::ChatBox { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
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
            ios_style_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::TypeWriter { props, .. } => {
            ios_style_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::RichText { props, .. } => {
            ios_style_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Record { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
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
            ios_variant_references_layout_bindings(&props.style, bindings)
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
            ios_variant_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Countdown { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
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
            ios_variant_references_layout_bindings(&props.style, bindings)
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
            ios_variant_references_layout_bindings(&props.style, bindings)
                || items.iter().any(|item| {
                    ios_children_reference_layout_bindings(&item.children, bindings)
                })
        }
        ViewNode::Carousel { props, slides } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || slides.iter().any(|slide| {
                    ios_children_reference_layout_bindings(&slide.children, bindings)
                })
        }
        ViewNode::Tabs { props, tabs } => {
            ios_style_references_layout_bindings(&props.style, bindings)
                || tabs.iter().any(|tab| {
                    ios_children_reference_layout_bindings(&tab.children, bindings)
                })
        }
        ViewNode::NavMenu { props, items } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || ios_nav_menu_items_reference_layout_bindings(items, bindings)
        }
        ViewNode::Each {
            children,
            collection,
            ..
        } => {
            bindings.references_signal(collection)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Avatar { props, .. } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Children => false,
    }
}

fn ios_children_reference_layout_bindings(
    children: &[ViewNode],
    bindings: &IosLayoutBindings,
) -> bool {
    children
        .iter()
        .any(|child| ios_node_references_layout_bindings(child, bindings))
}

fn ios_action_references_layout_bindings(
    action: &ViewAction,
    bindings: &IosLayoutBindings,
) -> bool {
    match &action.kind {
        ViewActionKind::Sequence(statements) => statements.iter().any(|statement| match statement {
            dowe_components::ViewFunctionStatement::Request { action, .. } => action.body.as_deref().is_some_and(|value| bindings.references_signal(value)),
            dowe_components::ViewFunctionStatement::Assign(assign) => bindings.references_signal(&assign.target) || bindings.references_signal(&assign.source),
            dowe_components::ViewFunctionStatement::Reset(reset) => bindings.references_signal(&reset.target),
            dowe_components::ViewFunctionStatement::If { success, error, .. } => success.iter().chain(error).any(|step| matches!(step, dowe_components::ViewFunctionStatement::Assign(assign) if bindings.references_signal(&assign.target) || bindings.references_signal(&assign.source))),
            dowe_components::ViewFunctionStatement::Toast(_) => false,
            dowe_components::ViewFunctionStatement::Redirect { .. } => false,
            dowe_components::ViewFunctionStatement::Validate { .. } => false,
        }),
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

fn ios_element_references_layout_bindings(
    props: &ElementProps,
    bindings: &IosLayoutBindings,
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
            .is_some_and(|value| ios_visibility_references_layout_bindings(value, bindings))
}

fn ios_visibility_references_layout_bindings(
    value: &VisibilityCondition,
    bindings: &IosLayoutBindings,
) -> bool {
    match value {
        VisibilityCondition::Static(_) => false,
        VisibilityCondition::Signal(path) => bindings.references_signal(path),
        VisibilityCondition::NumberComparison { path, .. }
        | VisibilityCondition::StringEquality { path, .. } => bindings.references_signal(path),
    }
}

fn ios_style_references_layout_bindings(
    props: &StyleProps,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_element_references_layout_bindings(&props.element, bindings)
}

fn ios_layout_references_layout_bindings(
    props: &LayoutProps,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_style_references_layout_bindings(&props.style, bindings)
}

fn ios_grid_references_layout_bindings(
    props: &GridProps,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_style_references_layout_bindings(&props.style, bindings)
}

fn ios_variant_references_layout_bindings(
    props: &VariantProps,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_element_references_layout_bindings(&props.element, bindings)
        || ios_style_references_layout_bindings(&props.style, bindings)
}

fn ios_text_references_layout_bindings(
    props: &TextProps,
    value: &str,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_style_references_layout_bindings(&props.style, bindings)
        || (props.i18n.is_none()
            && text_template_bindings(value).any(|path| bindings.references_signal(&path)))
}

fn ios_chart_references_layout_bindings(
    props: &ChartCommonProps,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_variant_references_layout_bindings(&props.style, bindings)
        || props
            .data
            .as_deref()
            .is_some_and(|value| bindings.references_signal(value))
        || props
            .series
            .as_deref()
            .is_some_and(|value| bindings.references_signal(value))
}

fn ios_side_nav_items_reference_layout_bindings(
    items: &[SideNavItem],
    bindings: &IosLayoutBindings,
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

fn ios_nav_menu_items_reference_layout_bindings(
    items: &[NavMenuItem],
    bindings: &IosLayoutBindings,
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
                || ios_children_reference_layout_bindings(content, bindings)
        }
    })
}

fn ios_overlay_entries_reference_layout_bindings(
    entries: &[OverlayEntry],
    bindings: &IosLayoutBindings,
) -> bool {
    entries.iter().any(|entry| match entry {
        OverlayEntry::Item(props) => props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value)),
        OverlayEntry::Divider => false,
    })
}

fn ios_command_entries_reference_layout_bindings(
    entries: &[CommandEntry],
    bindings: &IosLayoutBindings,
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

fn ios_layout_can_split(layout: &ViewNode) -> bool {
    let boundary = ios_children_boundary(layout, true, false);
    boundary.count == 1 && boundary.valid
}

struct IosChildrenBoundary {
    count: usize,
    valid: bool,
}

impl Default for IosChildrenBoundary {
    fn default() -> Self {
        Self {
            count: 0,
            valid: true,
        }
    }
}

fn ios_children_boundary(
    node: &ViewNode,
    neutral_context: bool,
    parent_horizontal: bool,
) -> IosChildrenBoundary {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => ios_merge_children_boundaries([
            ios_children_boundaries(content, neutral_context, parent_horizontal),
            ios_children_boundaries(children, neutral_context, parent_horizontal),
        ]),
        ViewNode::Children => IosChildrenBoundary {
            count: 1,
            valid: neutral_context && !parent_horizontal,
        },
        ViewNode::Scope { children, .. } => {
            ios_children_boundaries(children, neutral_context, parent_horizontal)
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            ios_children_boundaries(
                children,
                neutral_context && props.font.is_none() && props.text.is_none(),
                false,
            )
        }
        ViewNode::Grid { props, children } => ios_children_boundaries(
            children,
            neutral_context && props.style.font.is_none() && props.style.text.is_none(),
            false,
        ),
        ViewNode::Flex { children, .. } => ios_children_boundaries(children, neutral_context, true),
        ViewNode::Brand { props, children } => ios_children_boundaries(
            children,
            neutral_context && props.style.font.is_none() && props.style.text.is_none(),
            true,
        ),
        ViewNode::Banner { props, children } => ios_children_boundaries(
            children,
            neutral_context && props.style.font.is_none() && props.style.text.is_none(),
            false,
        ),
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
        } => {
            let neutral =
                neutral_context && props.style.font.is_none() && props.style.text.is_none();
            ios_merge_children_boundaries([
                ios_children_boundaries(app_bar, neutral, false),
                ios_children_boundaries(start, neutral, true),
                ios_children_boundaries(main, neutral, false),
                ios_children_boundaries(end, neutral, true),
                ios_children_boundaries(bottom_bar, neutral, false),
                ios_children_boundaries(overlays, neutral, false),
            ])
        }
        ViewNode::Tooltip { props, children } => ios_children_boundaries(
            children,
            neutral_context && props.style.style.font.is_none(),
            false,
        ),
        ViewNode::AppBar {
            top, start, center, end, bottom, ..
        }
        | ViewNode::Footer {
            top, start, center, end, bottom, ..
        } => ios_merge_children_boundaries([
            ios_children_boundaries(top, false, false),
            ios_children_boundaries(start, false, true),
            ios_children_boundaries(center, false, true),
            ios_children_boundaries(end, false, true),
            ios_children_boundaries(bottom, false, false),
        ]),
        ViewNode::Card { children, .. } | ViewNode::Badge { children, .. } => {
            ios_children_boundaries(children, false, false)
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => ios_merge_children_boundaries([
            ios_children_boundaries(header, false, false),
            ios_children_boundaries(body, false, false),
            ios_children_boundaries(footer, false, false),
        ]),
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => ios_merge_children_boundaries([
            ios_children_boundaries(header, false, false),
            ios_children_boundaries(body, false, false),
            ios_children_boundaries(footer, false, false),
        ]),
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => ios_merge_children_boundaries([
            ios_children_boundaries(trigger, false, false),
            ios_children_boundaries(header, false, false),
            ios_children_boundaries(footer, false, false),
        ]),
        ViewNode::Tabs { tabs, .. } => ios_merge_children_boundaries(
            tabs.iter()
                .map(|tab| ios_children_boundaries(&tab.children, false, false)),
        ),
        ViewNode::NavMenu { items, .. } => {
            ios_merge_children_boundaries(items.iter().filter_map(|item| match item {
                NavMenuItem::Megamenu { content, .. } => {
                    Some(ios_children_boundaries(content, false, false))
                }
                _ => None,
            }))
        }
        ViewNode::Accordion { items, .. } => ios_merge_children_boundaries(
            items
                .iter()
                .map(|item| ios_children_boundaries(&item.children, false, false)),
        ),
        ViewNode::Carousel { slides, .. } => ios_merge_children_boundaries(
            slides
                .iter()
                .map(|slide| ios_children_boundaries(&slide.children, false, false)),
        ),
        ViewNode::Marquee { children, .. } => ios_children_boundaries(children, false, false),
        ViewNode::Collapsible { children, .. } => ios_children_boundaries(children, false, false),
        ViewNode::Each { children, .. } => ios_children_boundaries(children, false, false),
        ViewNode::Button { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::SelectTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Input { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::Password { .. }
        | ViewNode::Phone { .. }
        | ViewNode::Pin { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Iframe { .. }
        | ViewNode::Device { .. }
        | ViewNode::Canvas { .. }
        | ViewNode::Diagram { .. }
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
        | ViewNode::RailNav { .. }
        | ViewNode::BottomBar { .. }
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
        | ViewNode::TypeWriter { .. } => IosChildrenBoundary::default(),
    }
}

fn ios_children_boundaries(
    children: &[ViewNode],
    neutral_context: bool,
    parent_horizontal: bool,
) -> IosChildrenBoundary {
    ios_merge_children_boundaries(
        children
            .iter()
            .map(|child| ios_children_boundary(child, neutral_context, parent_horizontal)),
    )
}

fn ios_merge_children_boundaries(
    boundaries: impl IntoIterator<Item = IosChildrenBoundary>,
) -> IosChildrenBoundary {
    boundaries.into_iter().fold(
        IosChildrenBoundary {
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
