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
        | ViewNode::Tree { .. }
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
