struct ComposeReactiveRoute {
    constants: String,
    initial: String,
    signals: String,
    actions: String,
    forms: String,
    init: Vec<String>,
    autoload: Vec<String>,
}

#[derive(Clone, Default)]
struct ComposeReactiveContext {
    constants: Vec<(String, String)>,
    signals: Vec<(String, String)>,
    actions: Vec<(String, String)>,
    items: Vec<(String, String)>,
}

impl ComposeReactiveContext {
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

fn compose_reactive_context_for_node(
    root: &ViewNode,
    target: &ViewNode,
) -> Option<ComposeReactiveContext> {
    fn find(
        node: &ViewNode,
        target: &ViewNode,
        context: &ComposeReactiveContext,
    ) -> Option<ComposeReactiveContext> {
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

    find(root, target, &ComposeReactiveContext::default())
}

fn compose_reactive_route(tree: &ViewNode) -> ComposeReactiveRoute {
    let mut constants = Vec::new();
    let mut signals = Vec::new();
    let mut metadata = Vec::new();
    let mut actions = Vec::new();
    let mut init = Vec::new();
    let mut autoload = Vec::new();
    collect_compose_reactive(
        tree,
        &ComposeReactiveContext::default(),
        &mut constants,
        &mut signals,
        &mut metadata,
        &mut actions,
        &mut init,
        &mut autoload,
    );
    let form_ids = compose_form_signal_ids(tree);
    let forms = collect_view_forms(tree)
        .iter()
        .map(|form| compose_form_value(form, &form_ids))
        .collect::<Vec<_>>();
    ComposeReactiveRoute {
        constants: format!("mapOf<String, Any?>({})", constants.join(", ")),
        initial: format!("mapOf<String, Any?>({})", signals.join(", ")),
        signals: format!("mapOf({})", metadata.join(", ")),
        actions: format!("mapOf({})", actions.join(", ")),
        forms: format!("mapOf({})", forms.join(", ")),
        init,
        autoload,
    }
}

fn compose_form_signal_ids(tree: &ViewNode) -> BTreeMap<String, String> {
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

fn compose_form_value(form: &ViewForm, signal_ids: &BTreeMap<String, String>) -> String {
    let signal = signal_ids.get(&form.signal).cloned().unwrap_or_else(|| form.signal.clone());
    let fields = form.fields.iter().map(|field| {
        let rules = field.rules.iter().map(|rule| {
            let argument = match &rule.kind {
                FormValidationRuleKind::Matches(path) => {
                    let (root, suffix) = path.split_once('.').unwrap_or((path.as_str(), ""));
                    let resolved = signal_ids.get(root).cloned().unwrap_or_else(|| root.to_string());
                    let resolved = if suffix.is_empty() { resolved } else { format!("{resolved}.{suffix}") };
                    compose_optional_string(Some(resolved.as_str()))
                }
                _ => compose_optional_string(rule.kind.argument().as_deref()),
            };
            format!("DoweValidationRule(\"{}\", {}, \"{}\")", escape_kotlin(rule.kind.name()), argument, escape_kotlin(&rule.message))
        }).collect::<Vec<_>>().join(", ");
        format!("DoweFormFieldMetadata(\"{}\", \"{}\", listOf({}))", escape_kotlin(&field.path), match field.kind { ViewFormFieldKind::Boolean => "boolean", ViewFormFieldKind::String => "string" }, rules)
    }).collect::<Vec<_>>().join(", ");
    format!("\"{}\" to listOf({})", escape_kotlin(&signal), fields)
}

fn collect_compose_reactive(
    node: &ViewNode,
    context: &ComposeReactiveContext,
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
                    "\"{}\" to {}",
                    escape_kotlin(&constant.id),
                    compose_signal_value(&constant.value)
                )
            }));
            signals.extend(scope_signals.iter().map(|signal| {
                format!(
                    "\"{}\" to {}",
                    escape_kotlin(&signal.id),
                    compose_signal_value(&signal.initial)
                )
            }));
            metadata.extend(scope_signals.iter().map(|signal| {
                format!(
                    "\"{}\" to DoweSignalMetadata(\"{}\", \"{}\", \"{}\")",
                    escape_kotlin(&signal.id),
                    escape_kotlin(&signal.storage_key),
                    signal.scope.as_str(),
                    signal.storage.as_str()
                )
            }));
            for action in scope_actions {
                actions.push(format!(
                    "\"{}\" to {}",
                    escape_kotlin(&action.id),
                    compose_action_value(action, &context)
                ));
                if action.is_init() {
                    init.push(action.id.clone());
                } else if action_autoloads(action) {
                    autoload.push(action.id.clone());
                }
            }
            for child in children {
                collect_compose_reactive(child, &context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter().chain(children) {
                collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
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
                collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::AppBar {
            top, start, center, end, bottom, ..
        }
        | ViewNode::Footer {
            top, start, center, end, bottom, ..
        } => {
            for child in top.iter().chain(start).chain(center).chain(end).chain(bottom) {
                collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::BottomBar { .. } => {}
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &tab.children {
                    collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Marquee { children, .. } | ViewNode::Collapsible { children, .. } => {
            for child in children {
                collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let dowe_components::NavMenuItem::Megamenu { content, .. } = item {
                    for child in content {
                        collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
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
                collect_compose_reactive(child, context, constants, signals, metadata, actions, init, autoload);
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
        ViewActionKind::Sequence(statements) => format!(
            "DoweAction.Sequence(listOf({}), {})",
            statements
                .iter()
                .map(|statement| compose_function_statement(statement, context))
                .collect::<Vec<_>>()
                .join(", "),
            compose_function_metadata(action)
        ),
        ViewActionKind::Request(request) => compose_request_value(request, context, action),
        ViewActionKind::Assign(assign) => compose_assign_value(assign, context, action),
        ViewActionKind::Reset(reset) => format!(
            "DoweAction.Reset(\"{}\", {})",
            escape_kotlin(&context.signal_path(&reset.target)),
            compose_function_metadata(action)
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

fn compose_function_statement(
    statement: &dowe_components::ViewFunctionStatement,
    context: &ComposeReactiveContext,
) -> String {
    match statement {
        dowe_components::ViewFunctionStatement::Validate { target } => format!(
            "DoweStep.Validate(\"{}\")",
            escape_kotlin(&context.signal_path(target))
        ),
        dowe_components::ViewFunctionStatement::Request { result, action } => format!(
            "DoweStep.Request(\"{}\", {})",
            escape_kotlin(result),
            compose_request_action_value(action, context)
        ),
        dowe_components::ViewFunctionStatement::If { result, success, error } => format!(
            "DoweStep.Branch(\"{}\", listOf({}), listOf({}))",
            escape_kotlin(result),
            success.iter().map(|step| compose_function_statement(step, context)).collect::<Vec<_>>().join(", "),
            error.iter().map(|step| compose_function_statement(step, context)).collect::<Vec<_>>().join(", ")
        ),
        dowe_components::ViewFunctionStatement::Assign(assign) => format!(
            "DoweStep.Assign(\"{}\", \"{}\", {}, {}, {})",
            escape_kotlin(&context.signal_path(&assign.target)),
            escape_kotlin(&context.signal_path(&assign.source)),
            assign.literal.as_ref().map(compose_signal_value).unwrap_or_else(|| "null".to_string()),
            assign.literal.is_some(),
            assign.call.as_ref().map(|call| compose_stdlib_call_value(call, context)).unwrap_or_else(|| "null".to_string())
        ),
        dowe_components::ViewFunctionStatement::Reset(reset) => format!(
            "DoweStep.Reset(\"{}\")",
            escape_kotlin(&context.signal_path(&reset.target))
        ),
        dowe_components::ViewFunctionStatement::Toast(toast) => format!(
            "DoweStep.Toast(\"{}\", \"{}\", \"{}\", {}, {}, {}, {})",
            escape_kotlin(&toast.kind),
            escape_kotlin(&toast.title),
            escape_kotlin(&toast.message),
            toast.duration.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()),
            compose_optional_string(toast.scheme.as_deref()),
            compose_optional_string(toast.variant.as_deref()),
            compose_optional_string(toast.position.as_deref())
        ),
        dowe_components::ViewFunctionStatement::Redirect { path } => format!(
            "DoweStep.Redirect(\"{}\")",
            escape_kotlin(path)
        ),
    }
}

fn compose_assign_value(action: &dowe_components::ViewAssignAction, context: &ComposeReactiveContext, view_action: &ViewAction) -> String {
    let target = escape_kotlin(&context.signal_path(&action.target));
    let source = escape_kotlin(&context.signal_path(&action.source));
    let metadata = compose_function_metadata(view_action);
    if let Some(call) = &action.call {
        format!(
            "DoweAction.Assign(\"{}\", \"{}\", {}, {})",
            target,
            source,
            compose_stdlib_call_value(call, context),
            metadata
        )
    } else {
        format!("DoweAction.Assign(\"{}\", \"{}\", null, {})", target, source, metadata)
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

fn compose_request_value(
    action: &ViewRequestAction,
    context: &ComposeReactiveContext,
    view_action: &ViewAction,
) -> String {
    format!(
        "DoweAction.Request({}, {})",
        compose_request_action_value(action, context),
        compose_function_metadata(view_action)
    )
}

fn compose_request_action_value(
    action: &ViewRequestAction,
    context: &ComposeReactiveContext,
) -> String {
    let base = action
        .base_env
        .as_ref()
        .map(|name| format!("DoweEnvironment.{name}"))
        .unwrap_or_else(|| "\"\"".to_string());
    let headers = compose_request_headers(action, context);
    format!(
        "DoweRequestAction(\"{}\", \"{}\", {}, {}, {}, {}, {}, {}, {}, {}, {})",
        action.method.as_str(),
        escape_kotlin(&action.path),
        base,
        headers,
        compose_optional_path(action.body.as_deref(), context),
        compose_optional_path(action.update.as_deref(), context),
        compose_optional_path(action.reset.as_deref(), context),
        compose_optional_path(action.success_alert.as_deref(), context),
        compose_optional_string(action.success_message.as_deref()),
        compose_optional_path(action.error_alert.as_deref(), context),
        compose_optional_string(action.error_message.as_deref())
    )
}

fn compose_function_metadata(action: &ViewAction) -> String {
    format!(
        "DoweActionMetadata(mapOf({}), {})",
        action
            .params
            .iter()
            .map(|parameter| format!(
                "\"{}\" to \"{}\"",
                escape_kotlin(&parameter.name),
                escape_kotlin(&parameter.type_name)
            ))
            .collect::<Vec<_>>()
            .join(", "),
        compose_optional_string(action.return_type.as_ref().map(|value| value.type_name.as_str()))
    )
}

fn compose_request_headers(action: &ViewRequestAction, context: &ComposeReactiveContext) -> String {
    format!(
        "listOf({})",
        action
            .headers
            .iter()
            .map(|header| match &header.value {
                dowe_components::ViewRequestHeaderValue::Static(value) => format!(
                    "Triple(\"{}\", \"static\", \"{}\")",
                    escape_kotlin(&header.name),
                    escape_kotlin(value)
                ),
                dowe_components::ViewRequestHeaderValue::Signal(value) => format!(
                    "Triple(\"{}\", \"signal\", \"{}\")",
                    escape_kotlin(&header.name),
                    escape_kotlin(&context.signal_path(value))
                ),
            })
            .collect::<Vec<_>>()
            .join(", ")
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
