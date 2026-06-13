struct SwiftReactiveRoute {
    initial: String,
    actions: String,
    autoload: Vec<String>,
}

#[derive(Clone, Default)]
struct SwiftReactiveContext {
    signals: Vec<(String, String)>,
    actions: Vec<(String, String)>,
    items: Vec<(String, String)>,
    children_expression: Option<String>,
    node_expressions: BTreeMap<usize, String>,
}

impl SwiftReactiveContext {
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

    fn with_children_expression(&self, expression: &str) -> Self {
        let mut next = self.clone();
        next.children_expression = Some(expression.to_string());
        next
    }

    fn with_node_expressions(&self, expressions: BTreeMap<usize, String>) -> Self {
        let mut next = self.clone();
        next.node_expressions = expressions;
        next
    }

    fn without_node_expression(&self, node: &ViewNode) -> Self {
        let mut next = self.clone();
        next.node_expressions.remove(&swift_node_key(node));
        next
    }

    fn node_expression(&self, node: &ViewNode) -> Option<&str> {
        self.node_expressions
            .get(&swift_node_key(node))
            .map(String::as_str)
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

fn swift_node_key(node: &ViewNode) -> usize {
    node as *const ViewNode as usize
}

fn swift_reactive_route(tree: &ViewNode) -> SwiftReactiveRoute {
    let mut signals = Vec::new();
    let mut actions = Vec::new();
    let mut autoload = Vec::new();
    collect_swift_reactive(
        tree,
        &SwiftReactiveContext::default(),
        &mut signals,
        &mut actions,
        &mut autoload,
    );
    SwiftReactiveRoute {
        initial: swift_dictionary(&signals),
        actions: swift_dictionary(&actions),
        autoload,
    }
}

fn swift_dictionary(values: &[String]) -> String {
    if values.is_empty() {
        "[:]".to_string()
    } else {
        format!("[{}]", values.join(", "))
    }
}

fn collect_swift_reactive(
    node: &ViewNode,
    context: &SwiftReactiveContext,
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
                    "\"{}\": {}",
                    escape_swift(&signal.id),
                    swift_signal_value(&signal.initial)
                )
            }));
            for action in scope_actions {
                actions.push(format!(
                    "\"{}\": {}",
                    escape_swift(&action.id),
                    swift_action_value(action, &context)
                ));
                if matches!(&action.kind, ViewActionKind::Request(request) if request.autoload) {
                    autoload.push(action.id.clone());
                }
            }
            for child in children {
                collect_swift_reactive(child, &context, signals, actions, autoload);
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
                collect_swift_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_swift_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_swift_reactive(child, context, signals, actions, autoload);
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
            for child in start.iter().chain(center.iter()).chain(end.iter()) {
                collect_swift_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &tab.children {
                    collect_swift_reactive(child, context, signals, actions, autoload);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    collect_swift_reactive(child, context, signals, actions, autoload);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    collect_swift_reactive(child, context, signals, actions, autoload);
                }
            }
        }
        ViewNode::Marquee { children, .. } => {
            for child in children {
                collect_swift_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::Collapsible { children, .. } => {
            for child in children {
                collect_swift_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let dowe_components::NavMenuItem::Megamenu { content, .. } = item {
                    for child in content {
                        collect_swift_reactive(child, context, signals, actions, autoload);
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
                collect_swift_reactive(child, context, signals, actions, autoload);
            }
        }
        ViewNode::Input { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
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
        | ViewNode::Children => {}
    }
}

fn swift_signal_value(value: &ViewSignalValue) -> String {
    match value {
        ViewSignalValue::Null => "NSNull()".to_string(),
        ViewSignalValue::Bool(value) => value.to_string(),
        ViewSignalValue::Number(value) => value.clone(),
        ViewSignalValue::String(value) => format!("\"{}\"", escape_swift(value)),
        ViewSignalValue::Array(values) if values.is_empty() => "[Any]()".to_string(),
        ViewSignalValue::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(swift_signal_value)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ViewSignalValue::Object(values) => format!(
            "[{}]",
            values
                .iter()
                .map(|(key, value)| {
                    format!("\"{}\": {}", escape_swift(key), swift_signal_value(value))
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn swift_action_value(action: &ViewAction, context: &SwiftReactiveContext) -> String {
    match &action.kind {
        ViewActionKind::Request(request) => swift_request_value(request, context),
        ViewActionKind::Assign(assign) => format!(
            ".assign(\"{}\", \"{}\")",
            escape_swift(&context.signal_path(&assign.target)),
            escape_swift(&context.signal_path(&assign.source))
        ),
        ViewActionKind::Reset(reset) => format!(
            ".reset(\"{}\")",
            escape_swift(&context.signal_path(&reset.target))
        ),
    }
}

fn swift_request_value(action: &ViewRequestAction, context: &SwiftReactiveContext) -> String {
    let base = action
        .base_env
        .as_ref()
        .map(|name| format!("DoweEnvironment.{name}"))
        .unwrap_or_else(|| "\"\"".to_string());
    format!(
        ".request(DoweRequestAction(method: \"{}\", path: \"{}\", base: {}, body: {}, update: {}, reset: {}, successAlert: {}, successMessage: {}, errorAlert: {}, errorMessage: {}))",
        action.method.as_str(),
        escape_swift(&action.path),
        base,
        swift_optional_path(action.body.as_deref(), context),
        swift_optional_path(action.update.as_deref(), context),
        swift_optional_path(action.reset.as_deref(), context),
        swift_optional_path(action.success_alert.as_deref(), context),
        swift_optional_string(action.success_message.as_deref()),
        swift_optional_path(action.error_alert.as_deref(), context),
        swift_optional_string(action.error_message.as_deref())
    )
}

fn swift_optional_path(value: Option<&str>, context: &SwiftReactiveContext) -> String {
    value
        .map(|value| format!("\"{}\"", escape_swift(&context.signal_path(value))))
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_swift(value)))
        .unwrap_or_else(|| "nil".to_string())
}
