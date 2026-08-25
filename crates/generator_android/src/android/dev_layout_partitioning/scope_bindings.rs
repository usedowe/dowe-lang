fn dev_collect_scope_bindings(node: &ViewNode, bindings: &mut DevLayoutBindings) {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => {
            dev_collect_scope_bindings_from_children(content, bindings);
            dev_collect_scope_bindings_from_children(children, bindings);
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
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. }
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
            overlays,
            ..
        } => {
            dev_collect_scope_bindings_from_children(app_bar, bindings);
            dev_collect_scope_bindings_from_children(start, bindings);
            dev_collect_scope_bindings_from_children(main, bindings);
            dev_collect_scope_bindings_from_children(end, bindings);
            dev_collect_scope_bindings_from_children(bottom_bar, bindings);
            dev_collect_scope_bindings_from_children(overlays, bindings);
        }
        ViewNode::AppBar {
            top, start, center, end, bottom, ..
        }
        | ViewNode::Footer {
            top, start, center, end, bottom, ..
        } => {
            dev_collect_scope_bindings_from_children(top, bindings);
            dev_collect_scope_bindings_from_children(start, bindings);
            dev_collect_scope_bindings_from_children(center, bindings);
            dev_collect_scope_bindings_from_children(end, bindings);
            dev_collect_scope_bindings_from_children(bottom, bindings);
        }
        ViewNode::BottomBar { .. } => {}
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

fn dev_collect_scope_bindings_from_children(
    children: &[ViewNode],
    bindings: &mut DevLayoutBindings,
) {
    for child in children {
        dev_collect_scope_bindings(child, bindings);
    }
}

