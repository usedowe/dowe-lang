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
        ViewActionKind::Invoke(invoke) => compose_invoke_value(invoke, context, action),
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
        ViewActionKind::Invoke(invoke) => invoke.autoload,
        ViewActionKind::Sequence(statements) => matches!(
            statements.first(),
            Some(dowe_components::ViewFunctionStatement::Request { action, .. }) if action.autoload
        ) || matches!(
            statements.first(),
            Some(dowe_components::ViewFunctionStatement::Invoke { action, .. }) if action.autoload
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
        dowe_components::ViewFunctionStatement::Invoke { result, action } => format!(
            "DoweStep.Invoke(\"{}\", {})",
            escape_kotlin(result),
            compose_invoke_action_value(action, context)
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

fn compose_invoke_value(
    action: &dowe_components::ViewInvokeAction,
    context: &ComposeReactiveContext,
    view_action: &ViewAction,
) -> String {
    format!(
        "DoweAction.Invoke({}, {})",
        compose_invoke_action_value(action, context),
        compose_function_metadata(view_action)
    )
}

fn compose_invoke_action_value(
    action: &dowe_components::ViewInvokeAction,
    context: &ComposeReactiveContext,
) -> String {
    format!(
        "DoweInvokeAction(\"{}\", listOf({}), {}, {}, {}, {}, {}, {})",
        escape_kotlin(&action.function),
        action.args.iter().map(|arg| format!("DoweStdlibArg(\"{}\", {})", escape_kotlin(&arg.name), compose_stdlib_value(&arg.value, context))).collect::<Vec<_>>().join(", "),
        compose_optional_path(action.update.as_deref(), context),
        compose_optional_path(action.reset.as_deref(), context),
        compose_optional_path(action.success_alert.as_deref(), context),
        compose_optional_string(action.success_message.as_deref()),
        compose_optional_path(action.error_alert.as_deref(), context),
        compose_optional_string(action.error_message.as_deref())
    )
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
