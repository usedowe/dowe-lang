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
        ViewActionKind::Invoke(invoke) => swift_invoke_value(invoke, context, action),
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
        dowe_components::ViewFunctionStatement::Invoke { result, action } => format!(
            ".invoke(\"{}\", {})",
            escape_swift(result),
            swift_invoke_action_value(action, context)
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

fn swift_invoke_value(
    action: &dowe_components::ViewInvokeAction,
    context: &SwiftReactiveContext,
    view_action: &ViewAction,
) -> String {
    format!(
        ".invoke({}, {})",
        swift_invoke_action_value(action, context),
        swift_function_metadata(view_action)
    )
}

fn swift_invoke_action_value(
    action: &dowe_components::ViewInvokeAction,
    context: &SwiftReactiveContext,
) -> String {
    format!(
        "DoweInvokeAction(function: \"{}\", args: [{}], update: {}, reset: {}, successAlert: {}, successMessage: {}, errorAlert: {}, errorMessage: {})",
        escape_swift(&action.function),
        action.args.iter().map(|arg| format!("DoweStdlibArg(name: \"{}\", value: {})", escape_swift(&arg.name), swift_stdlib_value(&arg.value, context))).collect::<Vec<_>>().join(", "),
        swift_optional_path(action.update.as_deref(), context),
        swift_optional_path(action.reset.as_deref(), context),
        swift_optional_path(action.success_alert.as_deref(), context),
        swift_optional_string(action.success_message.as_deref()),
        swift_optional_path(action.error_alert.as_deref(), context),
        swift_optional_string(action.error_message.as_deref())
    )
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

