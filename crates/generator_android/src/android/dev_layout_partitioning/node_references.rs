fn dev_node_references_layout_bindings(node: &ViewNode, bindings: &DevLayoutBindings) -> bool {
    match node {
        ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } => {
            bindings.references_signal(binding)
                || dev_children_reference_layout_bindings(content, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
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
        ViewNode::Brand { props, children } => {
            dev_style_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Banner { props, children } => {
            dev_style_references_layout_bindings(&props.style, bindings)
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
        ViewNode::Image { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .reactive_src
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Camera { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Diagram { props } => {
            dev_style_references_layout_bindings(&props.style.style, bindings)
                || bindings.references_signal(&props.nodes)
                || bindings.references_signal(&props.edges)
                || props
                    .on_node_click
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Microphone { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Code { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Video { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Iframe { props } => dev_style_references_layout_bindings(&props.style, bindings),
        ViewNode::Device { props, .. } => dev_style_references_layout_bindings(&props.style, bindings),
        ViewNode::Canvas { props } => {
            dev_style_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.scene)
                || props
                    .on_pointer
                    .iter()
                    .chain(&props.on_key)
                    .chain(&props.on_motion)
                    .any(|value| bindings.references_action(value))
        }
        ViewNode::Checkbox { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::RadioGroup { props, .. } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Toggle { props } => dev_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::ToggleTheme { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::SelectTheme { props } => {
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
        ViewNode::Password { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Phone { props } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Pin { props } => {
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
        ViewNode::RailNav { props, items } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
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
            dev_variant_references_layout_bindings(&props.style, bindings)
                || header
                    .iter()
                    .chain(body)
                    .chain(footer)
                    .any(|child| dev_node_references_layout_bindings(child, bindings))
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
            dev_variant_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(top, bindings)
                || dev_children_reference_layout_bindings(start, bindings)
                || dev_children_reference_layout_bindings(center, bindings)
                || dev_children_reference_layout_bindings(end, bindings)
                || dev_children_reference_layout_bindings(bottom, bindings)
        }
        ViewNode::BottomBar { props, .. } => {
            dev_variant_references_layout_bindings(&props.style, bindings)
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
            dev_style_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(app_bar, bindings)
                || dev_children_reference_layout_bindings(start, bindings)
                || dev_children_reference_layout_bindings(main, bindings)
                || dev_children_reference_layout_bindings(end, bindings)
                || dev_children_reference_layout_bindings(bottom_bar, bindings)
                || dev_children_reference_layout_bindings(overlays, bindings)
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

