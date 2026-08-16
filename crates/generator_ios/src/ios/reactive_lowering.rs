struct SwiftReactiveRoute {
    constants: String,
    initial: String,
    signals: String,
    actions: String,
    forms: String,
    init: Vec<String>,
    autoload: Vec<String>,
}

#[derive(Clone, Default)]
struct SwiftReactiveContext {
    constants: Vec<(String, String)>,
    signals: Vec<(String, String)>,
    actions: Vec<(String, String)>,
    items: Vec<(String, String)>,
    children_expression: Option<String>,
    node_expressions: BTreeMap<usize, String>,
}

impl SwiftReactiveContext {
    fn with_scope(
        &self,
        constants: &[ViewConstant],
        signals: &[ViewSignal],
        actions: &[ViewAction],
    ) -> Self {
        let mut next = self.clone();
        next.constants.extend(
            constants
                .iter()
                .map(|constant| (constant.name.clone(), constant.id.clone())),
        );
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
        if let Some(path) = path.strip_prefix('!') {
            return format!("!{}", self.signal_path(path));
        }
        let (root, suffix) = path
            .split_once('.')
            .map(|(root, suffix)| (root, format!(".{suffix}")))
            .unwrap_or((path, String::new()));
        self.signals
            .iter()
            .rev()
            .chain(self.constants.iter().rev())
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

fn swift_reactive_context_for_node(
    root: &ViewNode,
    target: &ViewNode,
) -> Option<SwiftReactiveContext> {
    fn find(
        node: &ViewNode,
        target: &ViewNode,
        context: &SwiftReactiveContext,
    ) -> Option<SwiftReactiveContext> {
        if std::ptr::eq(node, target) {
            return Some(context.clone());
        }
        let next = match node {
            ViewNode::Scope {
                constants,
                signals,
                actions,
                ..
            } => context.with_scope(constants, signals, actions),
            ViewNode::Each { item, .. } => context.with_item(item, "row.value".to_string()),
            _ => context.clone(),
        };
        node_child_groups(node)
            .into_iter()
            .flat_map(|group| group.iter())
            .find_map(|child| find(child, target, &next))
    }

    find(root, target, &SwiftReactiveContext::default())
}

fn swift_node_key(node: &ViewNode) -> usize {
    node as *const ViewNode as usize
}

fn swift_reactive_route(tree: &ViewNode) -> SwiftReactiveRoute {
    let mut constants = Vec::new();
    let mut signals = Vec::new();
    let mut metadata = Vec::new();
    let mut actions = Vec::new();
    let mut init = Vec::new();
    let mut autoload = Vec::new();
    collect_swift_reactive(
        tree,
        &SwiftReactiveContext::default(),
        &mut constants,
        &mut signals,
        &mut metadata,
        &mut actions,
        &mut init,
        &mut autoload,
    );
    let form_ids = swift_form_signal_ids(tree);
    let forms = collect_view_forms(tree)
        .iter()
        .map(|form| swift_form_value(form, &form_ids))
        .collect::<Vec<_>>();
    SwiftReactiveRoute {
        constants: swift_dictionary(&constants),
        initial: swift_dictionary(&signals),
        signals: swift_dictionary(&metadata),
        actions: swift_dictionary(&actions),
        forms: swift_dictionary(&forms),
        init,
        autoload,
    }
}

fn swift_form_signal_ids(tree: &ViewNode) -> BTreeMap<String, String> {
    fn collect(node: &ViewNode, output: &mut BTreeMap<String, String>) {
        match node {
            ViewNode::Scope { signals, children, .. } => {
                output.extend(signals.iter().map(|signal| (signal.name.clone(), signal.id.clone())));
                for child in children { collect(child, output); }
            }
            _ => for child in node_child_groups(node).into_iter().flat_map(|group| group.iter()) { collect(child, output); },
        }
    }
    let mut output = BTreeMap::new();
    collect(tree, &mut output);
    output
}

fn swift_form_value(form: &ViewForm, signal_ids: &BTreeMap<String, String>) -> String {
    let signal = signal_ids.get(&form.signal).cloned().unwrap_or_else(|| form.signal.clone());
    let fields = form.fields.iter().map(|field| {
        let rules = field.rules.iter().map(|rule| {
            let argument = match &rule.kind {
                FormValidationRuleKind::Matches(path) => {
                    let (root, suffix) = path.split_once('.').unwrap_or((path.as_str(), ""));
                    let resolved = signal_ids.get(root).cloned().unwrap_or_else(|| root.to_string());
                    let resolved = if suffix.is_empty() { resolved } else { format!("{resolved}.{suffix}") };
                    swift_optional_string(Some(resolved.as_str()))
                }
                _ => swift_optional_string(rule.kind.argument().as_deref()),
            };
            format!("DoweValidationRule(kind: \"{}\", argument: {}, message: \"{}\")", escape_swift(rule.kind.name()), argument, escape_swift(&rule.message))
        }).collect::<Vec<_>>().join(", ");
        format!("DoweFormFieldMetadata(path: \"{}\", kind: \"{}\", rules: [{}])", escape_swift(&field.path), match field.kind { ViewFormFieldKind::Boolean => "boolean", ViewFormFieldKind::String => "string" }, rules)
    }).collect::<Vec<_>>().join(", ");
    format!("\"{}\": [ {} ]", escape_swift(&signal), fields)
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
        | ViewNode::RailNav { .. }
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
        ViewActionKind::Sequence(statements) => format!(
            ".sequence([{}], {})",
            statements
                .iter()
                .map(|statement| swift_function_statement(statement, context))
                .collect::<Vec<_>>()
                .join(", "),
            swift_function_metadata(action)
        ),
        ViewActionKind::Request(request) => swift_request_value(request, context, action),
        ViewActionKind::Assign(assign) => swift_assign_value(assign, context, action),
        ViewActionKind::Reset(reset) => format!(
            ".reset(\"{}\", {})",
            escape_swift(&context.signal_path(&reset.target)),
            swift_function_metadata(action)
        ),
    }
}

fn action_autoloads(action: &ViewAction) -> bool {
    match &action.kind {
        ViewActionKind::Request(request) => request.autoload,
        ViewActionKind::Sequence(statements) => matches!(
            statements.first(),
            Some(dowe_components::ViewFunctionStatement::Request { action, .. }) if action.autoload
        ),
        ViewActionKind::Assign(_) | ViewActionKind::Reset(_) => false,
    }
}

fn swift_function_statement(
    statement: &dowe_components::ViewFunctionStatement,
    context: &SwiftReactiveContext,
) -> String {
    match statement {
        dowe_components::ViewFunctionStatement::Validate { target } => format!(
            ".validate(\"{}\")",
            escape_swift(&context.signal_path(target))
        ),
        dowe_components::ViewFunctionStatement::Request { result, action } => format!(
            ".request(\"{}\", {})",
            escape_swift(result),
            swift_request_action_value(action, context)
        ),
        dowe_components::ViewFunctionStatement::If {
            result,
            success,
            error,
        } => format!(
            ".branch(\"{}\", [{}], [{}])",
            escape_swift(result),
            success
                .iter()
                .map(|step| swift_function_statement(step, context))
                .collect::<Vec<_>>()
                .join(", "),
            error
                .iter()
                .map(|step| swift_function_statement(step, context))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        dowe_components::ViewFunctionStatement::Assign(assign) => format!(
            ".assign(\"{}\", \"{}\", {}, {}, {})",
            escape_swift(&context.signal_path(&assign.target)),
            escape_swift(&context.signal_path(&assign.source)),
            assign
                .literal
                .as_ref()
                .map(swift_signal_value)
                .unwrap_or_else(|| "nil".to_string()),
            assign.literal.is_some(),
            assign
                .call
                .as_ref()
                .map(|call| swift_stdlib_call_value(call, context))
                .unwrap_or_else(|| "nil".to_string())
        ),
        dowe_components::ViewFunctionStatement::Reset(reset) => format!(
            ".reset(\"{}\")",
            escape_swift(&context.signal_path(&reset.target))
        ),
        dowe_components::ViewFunctionStatement::Toast(toast) => format!(
            ".toast(\"{}\", \"{}\", \"{}\", {}, {}, {}, {})",
            escape_swift(&toast.kind),
            escape_swift(&toast.title),
            escape_swift(&toast.message),
            toast
                .duration
                .map(|value| value.to_string())
                .unwrap_or_else(|| "nil".to_string()),
            swift_optional_string(toast.scheme.as_deref()),
            swift_optional_string(toast.variant.as_deref()),
            swift_optional_string(toast.position.as_deref())
        ),
        dowe_components::ViewFunctionStatement::Redirect { path } => format!(
            ".redirect(\"{}\")",
            escape_swift(path)
        ),
    }
}

fn swift_assign_value(assign: &dowe_components::ViewAssignAction, context: &SwiftReactiveContext, view_action: &ViewAction) -> String {
    let target = escape_swift(&context.signal_path(&assign.target));
    let source = escape_swift(&context.signal_path(&assign.source));
    let call = assign.call.as_ref().map(|call| swift_stdlib_call_value(call, context)).unwrap_or_else(|| "nil".to_string());
    format!(".assign(\"{}\", \"{}\", {}, {})", target, source, call, swift_function_metadata(view_action))
}

fn swift_stdlib_call_value(
    call: &dowe_components::StdlibCall,
    context: &SwiftReactiveContext,
) -> String {
    format!(
        "DoweStdlibCall(namespace: \"{}\", function: \"{}\", args: [{}])",
        escape_swift(&call.namespace),
        escape_swift(&call.function),
        call.args
            .iter()
            .map(|arg| format!(
                "DoweStdlibArg(name: \"{}\", value: {})",
                escape_swift(&arg.name),
                swift_stdlib_value(&arg.value, context)
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn swift_stdlib_value(
    value: &dowe_components::StdlibValue,
    context: &SwiftReactiveContext,
) -> String {
    match value {
        dowe_components::StdlibValue::Null => {
            "DoweStdlibValue(kind: \"null\", value: nil)".to_string()
        }
        dowe_components::StdlibValue::Bool(value) => {
            format!("DoweStdlibValue(kind: \"bool\", value: {value})")
        }
        dowe_components::StdlibValue::Number(value) => format!(
            "DoweStdlibValue(kind: \"number\", value: \"{}\")",
            escape_swift(value)
        ),
        dowe_components::StdlibValue::String(value) => format!(
            "DoweStdlibValue(kind: \"string\", value: \"{}\")",
            escape_swift(value)
        ),
        dowe_components::StdlibValue::Reference(value) => format!(
            "DoweStdlibValue(kind: \"reference\", value: \"{}\")",
            escape_swift(&context.signal_path(value))
        ),
        dowe_components::StdlibValue::Array(values) => format!(
            "DoweStdlibValue(kind: \"array\", value: [{}])",
            values
                .iter()
                .map(|value| swift_stdlib_value(value, context))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        dowe_components::StdlibValue::Object(entries) => format!(
            "DoweStdlibValue(kind: \"object\", value: [{}])",
            entries
                .iter()
                .map(|(key, value)| format!(
                    "(\"{}\", {})",
                    escape_swift(key),
                    swift_stdlib_value(value, context)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn swift_request_value(
    action: &ViewRequestAction,
    context: &SwiftReactiveContext,
    view_action: &ViewAction,
) -> String {
    format!(
        ".request({}, {})",
        swift_request_action_value(action, context),
        swift_function_metadata(view_action)
    )
}

fn swift_request_action_value(
    action: &ViewRequestAction,
    context: &SwiftReactiveContext,
) -> String {
    let base = action
        .base_env
        .as_ref()
        .map(|name| format!("DoweEnvironment.{name}"))
        .unwrap_or_else(|| "\"\"".to_string());
    let headers = swift_request_headers(action, context);
    format!(
        "DoweRequestAction(method: \"{}\", path: \"{}\", base: {}, headers: {}, body: {}, update: {}, reset: {}, successAlert: {}, successMessage: {}, errorAlert: {}, errorMessage: {})",
        action.method.as_str(),
        escape_swift(&action.path),
        base,
        headers,
        swift_optional_path(action.body.as_deref(), context),
        swift_optional_path(action.update.as_deref(), context),
        swift_optional_path(action.reset.as_deref(), context),
        swift_optional_path(action.success_alert.as_deref(), context),
        swift_optional_string(action.success_message.as_deref()),
        swift_optional_path(action.error_alert.as_deref(), context),
        swift_optional_string(action.error_message.as_deref())
    )
}

fn swift_function_metadata(action: &ViewAction) -> String {
    let params = if action.params.is_empty() {
        "[:]".to_string()
    } else {
        format!(
            "[{}]",
            action
                .params
                .iter()
                .map(|parameter| format!(
                    "\"{}\": \"{}\"",
                    escape_swift(&parameter.name),
                    escape_swift(&parameter.type_name)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    format!(
        "DoweActionMetadata(params: {}, returnType: {})",
        params,
        swift_optional_string(action.return_type.as_ref().map(|value| value.type_name.as_str()))
    )
}

fn swift_request_headers(action: &ViewRequestAction, context: &SwiftReactiveContext) -> String {
    format!(
        "[{}]",
        action
            .headers
            .iter()
            .map(|header| match &header.value {
                dowe_components::ViewRequestHeaderValue::Static(value) => format!(
                    "(\"{}\", \"static\", \"{}\")",
                    escape_swift(&header.name),
                    escape_swift(value)
                ),
                dowe_components::ViewRequestHeaderValue::Signal(value) => format!(
                    "(\"{}\", \"signal\", \"{}\")",
                    escape_swift(&header.name),
                    escape_swift(&context.signal_path(value))
                ),
            })
            .collect::<Vec<_>>()
            .join(", ")
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
