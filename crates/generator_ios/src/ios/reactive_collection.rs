fn collect_swift_reactive(
    node: &ViewNode,
    context: &SwiftReactiveContext,
    constants: &mut Vec<String>,
    signals: &mut Vec<String>,
    metadata: &mut Vec<String>,
    actions: &mut Vec<String>,
    init: &mut Vec<String>,
    autoload: &mut Vec<String>,
) {
    match node {
        ViewNode::Scope {
            constants: scope_constants,
            signals: scope_signals,
            actions: scope_actions,
            children,
        } => {
            let context = context.with_scope(scope_constants, scope_signals, scope_actions);
            constants.extend(scope_constants.iter().map(|constant| {
                format!(
                    "\"{}\": {}",
                    escape_swift(&constant.id),
                    swift_signal_value(&constant.value)
                )
            }));
            signals.extend(scope_signals.iter().map(|signal| {
                format!(
                    "\"{}\": {}",
                    escape_swift(&signal.id),
                    swift_signal_value(&signal.initial)
                )
            }));
            metadata.extend(scope_signals.iter().map(|signal| {
                format!(
                    "\"{}\": DoweSignalMetadata(name: \"{}\", scope: \"{}\", storage: \"{}\")",
                    escape_swift(&signal.id),
                    escape_swift(&signal.storage_key),
                    signal.scope.as_str(),
                    signal.storage.as_str()
                )
            }));
            for action in scope_actions {
                actions.push(format!(
                    "\"{}\": {}",
                    escape_swift(&action.id),
                    swift_action_value(action, &context)
                ));
                if action.is_init() {
                    init.push(action.id.clone());
                } else if action_autoloads(action) {
                    autoload.push(action.id.clone());
                }
            }
            for child in children {
                collect_swift_reactive(child, &context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter().chain(children) {
                collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. }
        | ViewNode::Button { children, .. }
        | ViewNode::Each { children, .. } => {
            for child in children {
                collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::AppBar {
            top, start, center, end, bottom, ..
        }
        | ViewNode::Footer {
            top, start, center, end, bottom, ..
        } => {
            for child in top.iter().chain(start).chain(center).chain(end).chain(bottom) {
                collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::BottomBar { .. } => {}
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &tab.children {
                    collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Marquee { children, .. } => {
            for child in children {
                collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::Collapsible { children, .. } => {
            for child in children {
                collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let dowe_components::NavMenuItem::Megamenu { content, .. } = item {
                    for child in content {
                        collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
                    }
                }
            }
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
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
                .chain(overlays)
            {
                collect_swift_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::Input { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::Password { .. }
        | ViewNode::Phone { .. }
        | ViewNode::Pin { .. }
        | ViewNode::Textarea { .. }
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
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
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
        | ViewNode::Avatar { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Command { .. }
        | ViewNode::Svg { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::RailNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::Children => {}
    }
}

