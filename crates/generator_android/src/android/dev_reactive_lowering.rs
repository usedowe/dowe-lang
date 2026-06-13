struct DevReactiveRoute {
    initial: Vec<String>,
    actions: Vec<String>,
    autoload: Vec<String>,
}

fn dev_reactive_route(tree: &ViewNode) -> DevReactiveRoute {
    let mut initial = Vec::new();
    let mut actions = Vec::new();
    let mut autoload = Vec::new();
    collect_dev_reactive(
        tree,
        &ComposeReactiveContext::default(),
        &mut initial,
        &mut actions,
        &mut autoload,
    );
    DevReactiveRoute {
        initial,
        actions,
        autoload,
    }
}

fn collect_dev_reactive(
    node: &ViewNode,
    context: &ComposeReactiveContext,
    initial: &mut Vec<String>,
    actions: &mut Vec<String>,
    autoload: &mut Vec<String>,
) {
    match node {
        ViewNode::Scope {
            signals,
            actions: scope_actions,
            children,
        } => {
            let context = context.with_scope(signals, scope_actions);
            initial.extend(signals.iter().map(|signal| {
                format!(
                    "dowePutInitial(\"{}\", {});",
                    escape_java(&signal.id),
                    java_signal_value(&signal.initial)
                )
            }));
            for action in scope_actions {
                actions.push(format!(
                    "doweActions.put(\"{}\", {});",
                    escape_java(&action.id),
                    java_action_value(action, &context)
                ));
                if matches!(&action.kind, ViewActionKind::Request(request) if request.autoload) {
                    autoload.push(action.id.clone());
                }
            }
            for child in children {
                collect_dev_reactive(child, &context, initial, actions, autoload);
            }
        }
        ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Drawer { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Button { children, .. }
        | ViewNode::Each { children, .. } => {
            for child in children {
                collect_dev_reactive(child, context, initial, actions, autoload);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_dev_reactive(child, context, initial, actions, autoload);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_dev_reactive(child, context, initial, actions, autoload);
            }
        }
        ViewNode::AppBar {
            start,
            center,
            end,
            ..
        }
        | ViewNode::Footer {
            start,
            center,
            end,
            ..
        }
        | ViewNode::BottomBar {
            start,
            center,
            end,
            ..
        } => {
            for child in start.iter().chain(center).chain(end) {
                collect_dev_reactive(child, context, initial, actions, autoload);
            }
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &tab.children {
                    collect_dev_reactive(child, context, initial, actions, autoload);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    collect_dev_reactive(child, context, initial, actions, autoload);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    collect_dev_reactive(child, context, initial, actions, autoload);
                }
            }
        }
        ViewNode::Marquee { children, .. } | ViewNode::Collapsible { children, .. } => {
            for child in children {
                collect_dev_reactive(child, context, initial, actions, autoload);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let dowe_components::NavMenuItem::Megamenu { content, .. } = item {
                    for child in content {
                        collect_dev_reactive(child, context, initial, actions, autoload);
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
            ..
        } => {
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
            {
                collect_dev_reactive(child, context, initial, actions, autoload);
            }
        }
        ViewNode::Input { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::Table { .. }
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
        | ViewNode::Sidebar { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::Children => {}
    }
}

fn java_signal_value(value: &ViewSignalValue) -> String {
    match value {
        ViewSignalValue::Null => "null".to_string(),
        ViewSignalValue::Bool(value) => value.to_string(),
        ViewSignalValue::Number(value) => value.clone(),
        ViewSignalValue::String(value) => format!("\"{}\"", escape_java(value)),
        ViewSignalValue::Array(values) => format!(
            "doweArray({})",
            values
                .iter()
                .map(java_signal_value)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ViewSignalValue::Object(values) => format!(
            "doweObject({})",
            values
                .iter()
                .map(|(key, value)| {
                    format!("\"{}\", {}", escape_java(key), java_signal_value(value))
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn java_action_value(action: &ViewAction, context: &ComposeReactiveContext) -> String {
    match &action.kind {
        ViewActionKind::Request(request) => java_request_value(request, context),
        ViewActionKind::Assign(assign) => format!(
            "DoweAction.assign(\"{}\", \"{}\")",
            escape_java(&context.signal_path(&assign.target)),
            escape_java(&context.signal_path(&assign.source))
        ),
        ViewActionKind::Reset(reset) => format!(
            "DoweAction.reset(\"{}\")",
            escape_java(&context.signal_path(&reset.target))
        ),
    }
}

fn java_request_value(action: &ViewRequestAction, context: &ComposeReactiveContext) -> String {
    let base = action
        .base_env
        .as_ref()
        .map(|name| format!("DoweEnvironment.{name}"))
        .unwrap_or_else(|| "\"\"".to_string());
    format!(
        "DoweAction.request(\"{}\", \"{}\", {}, {}, {}, {}, {}, {}, {}, {})",
        action.method.as_str(),
        escape_java(&action.path),
        base,
        java_optional_path(action.body.as_deref(), context),
        java_optional_path(action.update.as_deref(), context),
        java_optional_path(action.reset.as_deref(), context),
        java_optional_path(action.success_alert.as_deref(), context),
        java_optional_string(action.success_message.as_deref()),
        java_optional_path(action.error_alert.as_deref(), context),
        java_optional_string(action.error_message.as_deref())
    )
}

fn java_optional_path(value: Option<&str>, context: &ComposeReactiveContext) -> String {
    value
        .map(|value| format!("\"{}\"", escape_java(&context.signal_path(value))))
        .unwrap_or_else(|| "null".to_string())
}

fn java_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_java(value)))
        .unwrap_or_else(|| "null".to_string())
}
