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
        ViewActionKind::Invoke(invoke) => java_invoke_value(invoke, context),
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
        dowe_components::ViewFunctionStatement::Validate { target } => format!(
            "DoweStep.validate(\"{}\")",
            escape_java(&context.signal_path(target))
        ),
        dowe_components::ViewFunctionStatement::Request { result, action } => format!(
            "DoweStep.request(\"{}\", {})",
            escape_java(result),
            java_request_value(action, context)
        ),
        dowe_components::ViewFunctionStatement::Invoke { result, action } => format!(
            "DoweStep.invoke(\"{}\", {})",
            escape_java(result),
            java_invoke_value(action, context)
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
            "DoweStep.toast(\"{}\", \"{}\", \"{}\", {}, {}, {}, {})",
            escape_java(&toast.kind),
            escape_java(&toast.title),
            escape_java(&toast.message),
            toast.duration.map(|duration| duration.to_string()).unwrap_or_else(|| "null".to_string()),
            toast.scheme.as_deref().map(|value| format!("\"{}\"", escape_java(value))).unwrap_or_else(|| "null".to_string()),
            toast.variant.as_deref().map(|value| format!("\"{}\"", escape_java(value))).unwrap_or_else(|| "null".to_string()),
            toast.position.as_deref().map(|value| format!("\"{}\"", escape_java(value))).unwrap_or_else(|| "null".to_string())
        ),
        dowe_components::ViewFunctionStatement::Redirect { path } => format!(
            "DoweStep.redirect(\"{}\")",
            escape_java(path)
        ),
    }
}

fn java_invoke_value(action: &dowe_components::ViewInvokeAction, context: &ComposeReactiveContext) -> String {
    format!(
        "DoweAction.invoke(\"{}\", new Object[][] {{{}}}, {}, {}, {}, {}, {}, {})",
        escape_java(&action.function),
        action.args.iter().map(|arg| format!("new Object[] {{\"{}\", {}}}", escape_java(&arg.name), java_stdlib_value(&arg.value, context))).collect::<Vec<_>>().join(", "),
        java_optional_path(action.update.as_deref(), context),
        java_optional_path(action.reset.as_deref(), context),
        java_optional_path(action.success_alert.as_deref(), context),
        action.success_message.as_deref().map(|value| format!("\"{}\"", escape_java(value))).unwrap_or_else(|| "null".to_string()),
        java_optional_path(action.error_alert.as_deref(), context),
        action.error_message.as_deref().map(|value| format!("\"{}\"", escape_java(value))).unwrap_or_else(|| "null".to_string())
    )
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

