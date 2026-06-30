struct ComposeReactiveRoute {
    initial: String,
    actions: String,
    autoload: Vec<String>,
}

#[derive(Clone, Default)]
struct ComposeReactiveContext {
    signals: Vec<(String, String)>,
    actions: Vec<(String, String)>,
    items: Vec<(String, String)>,
}

impl ComposeReactiveContext {
    fn with_scope(&self, signals: &[ViewSignal], actions: &[ViewAction]) -> Self {
        let mut next = self.clone();
        next.signals.extend(
            signals
                .iter()
                .map(|signal| (signal.name.clone(), signal.id.clone())),
        );
        next.actions.extend(
            actions
                .iter()
                .map(|action| (action.name.clone(), action.id.clone())),
        );
        next
    }

    fn with_item(&self, name: &str, value: String) -> Self {
        let mut next = self.clone();
        next.items.push((name.to_string(), value));
        next
    }

    fn signal_path(&self, path: &str) -> String {
        let (root, suffix) = path
            .split_once('.')
            .map(|(root, suffix)| (root, format!(".{suffix}")))
            .unwrap_or((path, String::new()));
        self.signals
            .iter()
            .rev()
            .find(|(name, _)| name == root)
            .map(|(_, id)| format!("{id}{suffix}"))
            .unwrap_or_else(|| path.to_string())
    }

    fn action_id(&self, name: &str) -> Option<&str> {
        self.actions
            .iter()
            .rev()
            .find(|(action_name, _)| action_name == name)
            .map(|(_, id)| id.as_str())
    }

    fn item_value(&self, path: &str) -> Option<&str> {
        let root = path.split('.').next().unwrap_or(path);
        self.items
            .iter()
            .rev()
            .find(|(name, _)| name == root)
            .map(|(_, value)| value.as_str())
    }

    fn item_path(&self, path: &str) -> Option<String> {
        let (root, suffix) = path
            .split_once('.')
            .map(|(root, suffix)| (root, Some(suffix)))
            .unwrap_or((path, None));
        self.items
            .iter()
            .rev()
            .find(|(name, _)| name == root)
            .map(|_| {
                suffix
                    .map(|suffix| format!("item.{suffix}"))
                    .unwrap_or_else(|| "item".to_string())
            })
    }

    fn active_item(&self) -> Option<&str> {
        self.items.last().map(|(_, value)| value.as_str())
    }

    fn dynamic_path(&self, path: &str) -> Option<String> {
        if let Some(path) = self.item_path(path) {
            return Some(path);
        }
        let resolved = self.signal_path(path);
        (resolved != path).then_some(resolved)
    }
}

fn compose_reactive_route(tree: &ViewNode) -> ComposeReactiveRoute {
    let mut signals = Vec::new();
    let mut actions = Vec::new();
    let mut autoload = Vec::new();
    collect_compose_reactive(
        tree,
        &ComposeReactiveContext::default(),
        &mut signals,
        &mut actions,
        &mut autoload,
    );
    ComposeReactiveRoute {
        initial: format!("mapOf<String, Any?>({})", signals.join(", ")),
        actions: format!("mapOf({})", actions.join(", ")),
        autoload,
    }
}

fn collect_compose_reactive(
    node: &ViewNode,
    context: &ComposeReactiveContext,
    signals: &mut Vec<String>,
    actions: &mut Vec<String>,
    autoload: &mut Vec<String>,
) {
    match node {
        ViewNode::Scope {
            signals: scope_signals,
            actions: scope_actions,
            children,
        } => {
            let context = context.with_scope(scope_signals, scope_actions);
            signals.extend(scope_signals.iter().map(|signal| {
                format!(
                    "\"{}\" to {}",
                    escape_kotlin(&signal.id),
                    compose_signal_value(&signal.initial)
                )
            }));
            for action in scope_actions {
                actions.push(format!(
                    "\"{}\" to {}",
                    escape_kotlin(&action.id),
                    compose_action_value(action, &context)
                ));
                if matches!(&action.kind, ViewActionKind::Request(request) if request.autoload) {
                    autoload.push(action.id.clone());
                }
            }
            for child in children {
                collect_compose_reactive(child, &context, signals, actions, autoload);
            }
        }
        ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Button { children, .. }
        | ViewNode::Each { children, .. } => {
            for child in children {
                collect_compose_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_compose_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_compose_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_compose_reactive(child, context, signals, actions, autoload);
            }
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
            for child in start.iter().chain(center).chain(end) {
                collect_compose_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &tab.children {
                    collect_compose_reactive(child, context, signals, actions, autoload);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    collect_compose_reactive(child, context, signals, actions, autoload);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    collect_compose_reactive(child, context, signals, actions, autoload);
                }
            }
        }
        ViewNode::Marquee { children, .. } | ViewNode::Collapsible { children, .. } => {
            for child in children {
                collect_compose_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let dowe_components::NavMenuItem::Megamenu { content, .. } = item {
                    for child in content {
                        collect_compose_reactive(child, context, signals, actions, autoload);
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
                collect_compose_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::Input { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::PasswordField { .. }
        | ViewNode::PhoneField { .. }
        | ViewNode::PinField { .. }
        | ViewNode::Textarea { .. }
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

fn compose_signal_value(value: &ViewSignalValue) -> String {
    match value {
        ViewSignalValue::Null => "null".to_string(),
        ViewSignalValue::Bool(value) => value.to_string(),
        ViewSignalValue::Number(value) => value.clone(),
        ViewSignalValue::String(value) => format!("\"{}\"", escape_kotlin(value)),
        ViewSignalValue::Array(values) => {
            if values.is_empty() {
                "listOf<Any?>()".to_string()
            } else {
                format!(
                    "listOf<Any?>({})",
                    values
                        .iter()
                        .map(compose_signal_value)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        ViewSignalValue::Object(values) => format!(
            "mapOf<String, Any?>({})",
            values
                .iter()
                .map(|(key, value)| {
                    format!(
                        "\"{}\" to {}",
                        escape_kotlin(key),
                        compose_signal_value(value)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn compose_action_value(action: &ViewAction, context: &ComposeReactiveContext) -> String {
    match &action.kind {
        ViewActionKind::Request(request) => compose_request_value(request, context),
        ViewActionKind::Assign(assign) => {
            let target = escape_kotlin(&context.signal_path(&assign.target));
            let source = escape_kotlin(&context.signal_path(&assign.source));
            if let Some(call) = &assign.call {
                format!(
                    "DoweAction.Assign(\"{}\", \"{}\", {})",
                    target,
                    source,
                    compose_stdlib_call_value(call, context)
                )
            } else {
                format!("DoweAction.Assign(\"{}\", \"{}\")", target, source)
            }
        }
        ViewActionKind::Reset(reset) => format!(
            "DoweAction.Reset(\"{}\")",
            escape_kotlin(&context.signal_path(&reset.target))
        ),
    }
}

fn compose_stdlib_call_value(
    call: &dowe_components::StdlibCall,
    context: &ComposeReactiveContext,
) -> String {
    format!(
        "DoweStdlibCall(\"{}\", \"{}\", listOf({}))",
        escape_kotlin(&call.namespace),
        escape_kotlin(&call.function),
        call.args
            .iter()
            .map(|arg| format!(
                "DoweStdlibArg(\"{}\", {})",
                escape_kotlin(&arg.name),
                compose_stdlib_value(&arg.value, context)
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn compose_stdlib_value(
    value: &dowe_components::StdlibValue,
    context: &ComposeReactiveContext,
) -> String {
    match value {
        dowe_components::StdlibValue::Null => "DoweStdlibValue(\"null\", null)".to_string(),
        dowe_components::StdlibValue::Bool(value) => {
            format!("DoweStdlibValue(\"bool\", {value})")
        }
        dowe_components::StdlibValue::Number(value) => format!(
            "DoweStdlibValue(\"number\", \"{}\")",
            escape_kotlin(value)
        ),
        dowe_components::StdlibValue::String(value) => format!(
            "DoweStdlibValue(\"string\", \"{}\")",
            escape_kotlin(value)
        ),
        dowe_components::StdlibValue::Reference(value) => format!(
            "DoweStdlibValue(\"reference\", \"{}\")",
            escape_kotlin(&context.signal_path(value))
        ),
        dowe_components::StdlibValue::Array(values) => format!(
            "DoweStdlibValue(\"array\", listOf({}))",
            values
                .iter()
                .map(|value| compose_stdlib_value(value, context))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        dowe_components::StdlibValue::Object(entries) => format!(
            "DoweStdlibValue(\"object\", listOf({}))",
            entries
                .iter()
                .map(|(key, value)| format!(
                    "\"{}\" to {}",
                    escape_kotlin(key),
                    compose_stdlib_value(value, context)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn compose_request_value(action: &ViewRequestAction, context: &ComposeReactiveContext) -> String {
    let base = action
        .base_env
        .as_ref()
        .map(|name| format!("DoweEnvironment.{name}"))
        .unwrap_or_else(|| "\"\"".to_string());
    format!(
        "DoweAction.Request(DoweRequestAction(\"{}\", \"{}\", {}, {}, {}, {}, {}, {}, {}, {}))",
        action.method.as_str(),
        escape_kotlin(&action.path),
        base,
        compose_optional_path(action.body.as_deref(), context),
        compose_optional_path(action.update.as_deref(), context),
        compose_optional_path(action.reset.as_deref(), context),
        compose_optional_path(action.success_alert.as_deref(), context),
        compose_optional_string(action.success_message.as_deref()),
        compose_optional_path(action.error_alert.as_deref(), context),
        compose_optional_string(action.error_message.as_deref())
    )
}

fn compose_optional_path(value: Option<&str>, context: &ComposeReactiveContext) -> String {
    value
        .map(|value| format!("\"{}\"", escape_kotlin(&context.signal_path(value))))
        .unwrap_or_else(|| "null".to_string())
}

fn compose_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_kotlin(value)))
        .unwrap_or_else(|| "null".to_string())
}
