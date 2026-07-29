struct DevReactiveRoute {
    initial: Vec<String>,
    metadata: Vec<String>,
    actions: Vec<String>,
    init: Vec<String>,
    autoload: Vec<String>,
}

fn dev_reactive_route(tree: &ViewNode) -> DevReactiveRoute {
    let mut initial = Vec::new();
    let mut metadata = Vec::new();
    let mut actions = Vec::new();
    let mut init = Vec::new();
    let mut autoload = Vec::new();
    collect_dev_reactive(
        tree,
        &ComposeReactiveContext::default(),
        &mut initial,
        &mut metadata,
        &mut actions,
        &mut init,
        &mut autoload,
    );
    DevReactiveRoute {
        initial,
        metadata,
        actions,
        init,
        autoload,
    }
}

fn collect_dev_reactive(
    node: &ViewNode,
    context: &ComposeReactiveContext,
    initial: &mut Vec<String>,
    metadata: &mut Vec<String>,
    actions: &mut Vec<String>,
    init: &mut Vec<String>,
    autoload: &mut Vec<String>,
) {
    match node {
        ViewNode::Scope {
            constants,
            signals,
            actions: scope_actions,
            children,
        } => {
            let context = context.with_scope(constants, signals, scope_actions);
            initial.extend(constants.iter().map(|constant| {
                format!(
                    "dowePutInitial(\"{}\", {});",
                    escape_java(&constant.id),
                    java_signal_value(&constant.value)
                )
            }));
            initial.extend(signals.iter().map(|signal| {
                format!(
                    "dowePutInitial(\"{}\", {});",
                    escape_java(&signal.id),
                    java_signal_value(&signal.initial)
                )
            }));
            metadata.extend(signals.iter().map(|signal| {
                format!(
                    "dowePutSignalMetadata(\"{}\", \"{}\", \"{}\", \"{}\");",
                    escape_java(&signal.id),
                    escape_java(&signal.storage_key),
                    signal.scope.as_str(),
                    signal.storage.as_str()
                )
            }));
            for action in scope_actions {
                actions.push(format!(
                    "doweActions.put(\"{}\", {});",
                    escape_java(&action.id),
                    java_action_value(action, &context)
                ));
                if action.is_init() {
                    init.push(action.id.clone());
                } else if action_autoloads(action) {
                    autoload.push(action.id.clone());
                }
            }
            for child in children {
                collect_dev_reactive(child, &context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter().chain(children) {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
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
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
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
        } => {
            for child in start.iter().chain(center).chain(end) {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::BottomBar { .. } => {}
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &tab.children {
                    collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
                }
            }
        }
        ViewNode::Marquee { children, .. } | ViewNode::Collapsible { children, .. } => {
            for child in children {
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let dowe_components::NavMenuItem::Megamenu { content, .. } = item {
                    for child in content {
                        collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
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
                collect_dev_reactive(child, context, initial, metadata, actions, init, autoload);
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
        | ViewNode::SelectTheme { .. }
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
        ViewActionKind::Sequence(statements) => format!(
            "DoweAction.sequence(new DoweStep[] {{{}}})",
            statements
                .iter()
                .map(|statement| java_function_statement(statement, context))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ViewActionKind::Request(request) => java_request_value(request, context),
        ViewActionKind::Assign(assign) => java_assign_value(assign, context),
        ViewActionKind::Reset(reset) => format!(
            "DoweAction.reset(\"{}\")",
            escape_java(&context.signal_path(&reset.target))
        ),
    }
}

fn java_function_statement(
    statement: &dowe_components::ViewFunctionStatement,
    context: &ComposeReactiveContext,
) -> String {
    match statement {
        dowe_components::ViewFunctionStatement::Request { result, action } => format!(
            "DoweStep.request(\"{}\", {})",
            escape_java(result),
            java_request_value(action, context)
        ),
        dowe_components::ViewFunctionStatement::If { result, success, error } => format!(
            "DoweStep.branch(\"{}\", new DoweStep[] {{{}}}, new DoweStep[] {{{}}})",
            escape_java(result),
            success.iter().map(|step| java_function_statement(step, context)).collect::<Vec<_>>().join(", "),
            error.iter().map(|step| java_function_statement(step, context)).collect::<Vec<_>>().join(", ")
        ),
        dowe_components::ViewFunctionStatement::Assign(assign) => format!(
            "DoweStep.assign(\"{}\", \"{}\", {}, {}, {})",
            escape_java(&context.signal_path(&assign.target)),
            escape_java(&context.signal_path(&assign.source)),
            assign.literal.as_ref().map(java_signal_value).unwrap_or_else(|| "null".to_string()),
            assign.literal.is_some(),
            assign.call.as_ref().map(|call| format!("DoweAction.assignCall(\"\", \"\", \"{}\", \"{}\", new Object[][] {{{}}})", escape_java(&call.namespace), escape_java(&call.function), call.args.iter().map(|arg| format!("new Object[] {{\"{}\", {}}}", escape_java(&arg.name), java_stdlib_value(&arg.value, context))).collect::<Vec<_>>().join(", "))).unwrap_or_else(|| "null".to_string())
        ),
        dowe_components::ViewFunctionStatement::Reset(reset) => format!(
            "DoweStep.reset(\"{}\")",
            escape_java(&context.signal_path(&reset.target))
        ),
        dowe_components::ViewFunctionStatement::Toast(toast) => format!(
            "DoweStep.toast(\"{}\", \"{}\", \"{}\", {})",
            escape_java(&toast.kind),
            escape_java(&toast.title),
            escape_java(&toast.message),
            toast.duration.map(|duration| duration.to_string()).unwrap_or_else(|| "null".to_string())
        ),
    }
}

fn java_assign_value(assign: &dowe_components::ViewAssignAction, context: &ComposeReactiveContext) -> String {
    let target = escape_java(&context.signal_path(&assign.target));
    let source = escape_java(&context.signal_path(&assign.source));
    if let Some(call) = &assign.call {
        format!(
            "DoweAction.assignCall(\"{}\", \"{}\", \"{}\", \"{}\", new Object[][] {{{}}})",
            target,
            source,
            escape_java(&call.namespace),
            escape_java(&call.function),
            call.args.iter().map(|arg| format!("new Object[] {{\"{}\", {}}}", escape_java(&arg.name), java_stdlib_value(&arg.value, context))).collect::<Vec<_>>().join(", ")
        )
    } else {
        format!("DoweAction.assign(\"{}\", \"{}\")", target, source)
    }
}

fn java_stdlib_value(
    value: &dowe_components::StdlibValue,
    context: &ComposeReactiveContext,
) -> String {
    match value {
        dowe_components::StdlibValue::Null => "new Object[] {\"null\", null}".to_string(),
        dowe_components::StdlibValue::Bool(value) => {
            format!("new Object[] {{\"bool\", {value}}}")
        }
        dowe_components::StdlibValue::Number(value) => format!(
            "new Object[] {{\"number\", \"{}\"}}",
            escape_java(value)
        ),
        dowe_components::StdlibValue::String(value) => format!(
            "new Object[] {{\"string\", \"{}\"}}",
            escape_java(value)
        ),
        dowe_components::StdlibValue::Reference(value) => format!(
            "new Object[] {{\"reference\", \"{}\"}}",
            escape_java(&context.signal_path(value))
        ),
        dowe_components::StdlibValue::Array(values) => format!(
            "new Object[] {{\"array\", new Object[] {{{}}}}}",
            values
                .iter()
                .map(|value| java_stdlib_value(value, context))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        dowe_components::StdlibValue::Object(entries) => format!(
            "new Object[] {{\"object\", new Object[] {{{}}}}}",
            entries
                .iter()
                .map(|(key, value)| format!(
                    "new Object[] {{\"{}\", {}}}",
                    escape_java(key),
                    java_stdlib_value(value, context)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn java_request_value(action: &ViewRequestAction, context: &ComposeReactiveContext) -> String {
    let base = action
        .base_env
        .as_ref()
        .map(|name| format!("DoweEnvironment.{name}"))
        .unwrap_or_else(|| "\"\"".to_string());
    let headers = java_request_headers(action, context);
    format!(
        "DoweAction.request(\"{}\", \"{}\", {}, {}, {}, {}, {}, {}, {}, {}, {})",
        action.method.as_str(),
        escape_java(&action.path),
        base,
        headers,
        java_optional_path(action.body.as_deref(), context),
        java_optional_path(action.update.as_deref(), context),
        java_optional_path(action.reset.as_deref(), context),
        java_optional_path(action.success_alert.as_deref(), context),
        java_optional_string(action.success_message.as_deref()),
        java_optional_path(action.error_alert.as_deref(), context),
        java_optional_string(action.error_message.as_deref())
    )
}

fn java_request_headers(action: &ViewRequestAction, context: &ComposeReactiveContext) -> String {
    format!(
        "new Object[][] {{{}}}",
        action
            .headers
            .iter()
            .map(|header| match &header.value {
                dowe_components::ViewRequestHeaderValue::Static(value) => format!(
                    "new Object[] {{\"{}\", \"static\", \"{}\"}}",
                    escape_java(&header.name),
                    escape_java(value)
                ),
                dowe_components::ViewRequestHeaderValue::Signal(value) => format!(
                    "new Object[] {{\"{}\", \"signal\", \"{}\"}}",
                    escape_java(&header.name),
                    escape_java(&context.signal_path(value))
                ),
            })
            .collect::<Vec<_>>()
            .join(", ")
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
