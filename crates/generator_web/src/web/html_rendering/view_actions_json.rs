fn page_definition_json(tree: &ViewNode) -> String {
    match tree {
        ViewNode::Scope {
            constants, signals, actions, ..
        } => {
            let context = ReactiveRenderContext::default().with_scope(
                constants.as_slice(),
                signals.as_slice(),
                actions.as_slice(),
            );
            format!(
                r#"{{"constants":[{}],"signals":[{}],"actions":[{}],"forms":[{}]}}"#,
                constants
                    .iter()
                    .map(constant_json)
                    .collect::<Vec<_>>()
                    .join(","),
                signals
                    .iter()
                    .map(signal_json)
                    .collect::<Vec<_>>()
                    .join(","),
                actions
                    .iter()
                    .map(|action| action_json(action, &context))
                    .collect::<Vec<_>>()
                    .join(","),
                collect_view_forms(tree)
                    .iter()
                    .map(|form| form_json(form, &context))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        _ => r#"{"signals":[],"actions":[],"forms":[]}"#.to_string(),
    }
}

fn form_json(form: &ViewForm, context: &ReactiveRenderContext) -> String {
    format!(
        r#"{{"signal":"{}","fields":[{}]}}"#,
        escape_json(&context.signal_path(&form.signal)),
        form.fields
            .iter()
            .map(|field| {
                let rules = field
                    .rules
                    .iter()
                    .map(|rule| {
                        let argument = match &rule.kind {
                            FormValidationRuleKind::Matches(path) => {
                                Some(context.signal_path(path))
                            }
                            _ => rule.kind.argument(),
                        };
                        format!(
                            r#"{{"path":"{}","kind":"{}","argument":{},"message":"{}"}}"#,
                            escape_json(&field.path),
                            rule.kind.name(),
                            argument
                                .as_deref()
                                .map(|value| format!(r#""{}""#, escape_json(value)))
                                .unwrap_or_else(|| "null".to_string()),
                            escape_json(&rule.message)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    r#"{{"path":"{}","kind":"{}","rules":[{}]}}"#,
                    escape_json(&field.path),
                    match field.kind {
                        ViewFormFieldKind::Boolean => "boolean",
                        ViewFormFieldKind::String => "string",
                    },
                    rules
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn constant_json(constant: &ViewConstant) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","value":{}}}"#,
        escape_json(&constant.id),
        escape_json(&constant.name),
        signal_value_json(&constant.value)
    )
}

fn signal_json(signal: &ViewSignal) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","storageKey":"{}","scope":"{}","storage":"{}","initial":{}}}"#,
        escape_json(&signal.id),
        escape_json(&signal.name),
        escape_json(&signal.storage_key),
        signal.scope.as_str(),
        signal.storage.as_str(),
        signal_value_json(&signal.initial)
    )
}

fn signal_value_json(value: &ViewSignalValue) -> String {
    match value {
        ViewSignalValue::Null => "null".to_string(),
        ViewSignalValue::Bool(value) => value.to_string(),
        ViewSignalValue::Number(value) => value.clone(),
        ViewSignalValue::String(value) => format!(r#""{}""#, escape_json(value)),
        ViewSignalValue::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(signal_value_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        ViewSignalValue::Object(entries) => format!(
            "{{{}}}",
            entries
                .iter()
                .map(|(key, value)| format!(
                    r#""{}":{}"#,
                    escape_json(key),
                    signal_value_json(value)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn action_json(action: &ViewAction, context: &ReactiveRenderContext) -> String {
    match &action.kind {
        ViewActionKind::Sequence(statements) => format!(
            r#"{{"id":"{}","name":"{}","params":{},"returnType":{},"kind":"sequence","steps":[{}],"autoload":{},"init":{}}}"#,
            escape_json(&action.id),
            escape_json(&action.name),
            function_params_json(action),
            function_return_json(action),
            statements.iter().map(|statement| statement_json(statement, context)).collect::<Vec<_>>().join(","),
            action.is_init() || statements.iter().any(|statement| matches!(statement, dowe_components::ViewFunctionStatement::Request { action, .. } if action.autoload)),
            action.is_init()
        ),
        ViewActionKind::Request(request) => request_action_json(action, request, context),
        ViewActionKind::Invoke(invoke) => invoke_action_json(action, invoke, context),
        ViewActionKind::Assign(assign) => assign_action_json(action, assign, context),
        ViewActionKind::Reset(reset) => reset_action_json(action, reset, context),
    }
}

fn statement_json(statement: &dowe_components::ViewFunctionStatement, context: &ReactiveRenderContext) -> String {
    match statement {
        dowe_components::ViewFunctionStatement::Validate { target } => format!(
            r#"{{"kind":"validate","target":"{}"}}"#,
            escape_json(&context.signal_path(target))
        ),
        dowe_components::ViewFunctionStatement::Request { result, action } => format!(
            r#"{{"kind":"request","result":"{}","method":"{}","path":"{}","baseEnv":{},"headers":{},"body":{}}}"#,
            escape_json(result), action.method.as_str(), escape_json(&action.path), json_optional_string(action.base_env.as_deref()), request_headers_json(action, context), json_optional_path(action.body.as_deref(), context)
        ),
        dowe_components::ViewFunctionStatement::Invoke { result, action } => format!(
            r#"{{"kind":"invoke","result":"{}","function":"{}","args":{},"update":{},"reset":{},"successAlert":{},"successMessage":{},"errorAlert":{},"errorMessage":{}}}"#,
            escape_json(result),
            escape_json(&action.function),
            invoke_args_json(&action.args, context),
            json_optional_path(action.update.as_deref(), context),
            json_optional_path(action.reset.as_deref(), context),
            json_optional_path(action.success_alert.as_deref(), context),
            json_optional_string(action.success_message.as_deref()),
            json_optional_path(action.error_alert.as_deref(), context),
            json_optional_string(action.error_message.as_deref())
        ),
        dowe_components::ViewFunctionStatement::If { result, success, error } => format!(
            r#"{{"kind":"if","result":"{}","success":[{}],"error":[{}]}}"#,
            escape_json(result),
            success.iter().map(|step| statement_json(step, context)).collect::<Vec<_>>().join(","),
            error.iter().map(|step| statement_json(step, context)).collect::<Vec<_>>().join(",")
        ),
        dowe_components::ViewFunctionStatement::Assign(assign) => format!(
            r#"{{"kind":"assign","target":"{}","source":"{}","literal":{},"call":{}}}"#,
            escape_json(&context.signal_path(&assign.target)),
            escape_json(&context.signal_path(&assign.source)),
            assign.literal.as_ref().map(signal_value_json).unwrap_or_else(|| "null".to_string()),
            assign.call.as_ref().map(|call| stdlib_call_json(call, context)).unwrap_or_else(|| "null".to_string())
        ),
        dowe_components::ViewFunctionStatement::Reset(reset) => format!(r#"{{"kind":"reset","target":"{}"}}"#, escape_json(&context.signal_path(&reset.target))),
        dowe_components::ViewFunctionStatement::Toast(toast) => format!(
            r#"{{"kind":"toast","type":"{}","title":"{}","message":"{}","duration":{},"scheme":{},"variant":{},"position":{}}}"#,
            escape_json(&toast.kind), escape_json(&toast.title), escape_json(&toast.message), toast.duration.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()), json_optional_string(toast.scheme.as_deref()), json_optional_string(toast.variant.as_deref()), json_optional_string(toast.position.as_deref())
        ),
        dowe_components::ViewFunctionStatement::Redirect { path } => format!(
            r#"{{"kind":"redirect","path":"{}"}}"#,
            escape_json(path)
        ),
    }
}

fn invoke_action_json(
    view_action: &ViewAction,
    action: &dowe_components::ViewInvokeAction,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","params":{},"returnType":{},"kind":"invoke","function":"{}","args":{},"update":{},"reset":{},"successAlert":{},"successMessage":{},"errorAlert":{},"errorMessage":{},"autoload":{}}}"#,
        escape_json(&view_action.id),
        escape_json(&view_action.name),
        function_params_json(view_action),
        function_return_json(view_action),
        escape_json(&action.function),
        invoke_args_json(&action.args, context),
        json_optional_path(action.update.as_deref(), context),
        json_optional_path(action.reset.as_deref(), context),
        json_optional_path(action.success_alert.as_deref(), context),
        json_optional_string(action.success_message.as_deref()),
        json_optional_path(action.error_alert.as_deref(), context),
        json_optional_string(action.error_message.as_deref()),
        action.autoload
    )
}

fn invoke_args_json(args: &[dowe_components::StdlibArgument], context: &ReactiveRenderContext) -> String {
    format!(
        "[{}]",
        args.iter()
            .map(|arg| format!(r#"{{"name":"{}","value":{}}}"#, escape_json(&arg.name), stdlib_value_json(&arg.value, context)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn request_action_json(
    view_action: &ViewAction,
    action: &ViewRequestAction,
    context: &ReactiveRenderContext,
) -> String {
    let headers = request_headers_json(action, context);
    format!(
        r#"{{"id":"{}","name":"{}","params":{},"returnType":{},"kind":"request","method":"{}","path":"{}","baseEnv":{},"headers":{},"body":{},"update":{},"reset":{},"successAlert":{},"successMessage":{},"errorAlert":{},"errorMessage":{},"autoload":{}}}"#,
        escape_json(&view_action.id),
        escape_json(&view_action.name),
        function_params_json(view_action),
        function_return_json(view_action),
        action.method.as_str(),
        escape_json(&action.path),
        json_optional_string(action.base_env.as_deref()),
        headers,
        json_optional_path(action.body.as_deref(), context),
        json_optional_path(action.update.as_deref(), context),
        json_optional_path(action.reset.as_deref(), context),
        json_optional_path(action.success_alert.as_deref(), context),
        json_optional_string(action.success_message.as_deref()),
        json_optional_path(action.error_alert.as_deref(), context),
        json_optional_string(action.error_message.as_deref()),
        action.autoload
    )
}

fn request_headers_json(action: &ViewRequestAction, context: &ReactiveRenderContext) -> String {
    format!(
        "[{}]",
        action
            .headers
            .iter()
            .map(|header| match &header.value {
                dowe_components::ViewRequestHeaderValue::Static(value) => format!(
                    r#"{{"name":"{}","kind":"static","value":"{}"}}"#,
                    escape_json(&header.name),
                    escape_json(value)
                ),
                dowe_components::ViewRequestHeaderValue::Signal(value) => format!(
                    r#"{{"name":"{}","kind":"signal","value":"{}"}}"#,
                    escape_json(&header.name),
                    escape_json(&context.signal_path(value))
                ),
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn assign_action_json(
    view_action: &ViewAction,
    action: &ViewAssignAction,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","params":{},"returnType":{},"kind":"assign","target":"{}","source":"{}","literal":{},"call":{}}}"#,
        escape_json(&view_action.id),
        escape_json(&view_action.name),
        function_params_json(view_action),
        function_return_json(view_action),
        escape_json(&context.signal_path(&action.target)),
        escape_json(&context.signal_path(&action.source)),
        action.literal.as_ref().map(signal_value_json).unwrap_or_else(|| "null".to_string()),
        action
            .call
            .as_ref()
            .map(|call| stdlib_call_json(call, context))
            .unwrap_or_else(|| "null".to_string())
    )
}

fn stdlib_call_json(
    call: &dowe_components::StdlibCall,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"{{"namespace":"{}","function":"{}","args":[{}]}}"#,
        escape_json(&call.namespace),
        escape_json(&call.function),
        call.args
            .iter()
            .map(|arg| format!(
                r#"{{"name":"{}","value":{}}}"#,
                escape_json(&arg.name),
                stdlib_value_json(&arg.value, context)
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn stdlib_value_json(
    value: &dowe_components::StdlibValue,
    context: &ReactiveRenderContext,
) -> String {
    match value {
        dowe_components::StdlibValue::Null => r#"{"kind":"null","value":null}"#.to_string(),
        dowe_components::StdlibValue::Bool(value) => {
            format!(r#"{{"kind":"bool","value":{value}}}"#)
        }
        dowe_components::StdlibValue::Number(value) => {
            format!(r#"{{"kind":"number","value":"{}"}}"#, escape_json(value))
        }
        dowe_components::StdlibValue::String(value) => {
            format!(r#"{{"kind":"string","value":"{}"}}"#, escape_json(value))
        }
        dowe_components::StdlibValue::Reference(value) => format!(
            r#"{{"kind":"reference","value":"{}"}}"#,
            escape_json(&context.signal_path(value))
        ),
        dowe_components::StdlibValue::Array(values) => format!(
            r#"{{"kind":"array","value":[{}]}}"#,
            values
                .iter()
                .map(|value| stdlib_value_json(value, context))
                .collect::<Vec<_>>()
                .join(",")
        ),
        dowe_components::StdlibValue::Object(entries) => format!(
            r#"{{"kind":"object","value":[{}]}}"#,
            entries
                .iter()
                .map(|(key, value)| format!(
                    r#"["{}",{}]"#,
                    escape_json(key),
                    stdlib_value_json(value, context)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn reset_action_json(
    view_action: &ViewAction,
    action: &ViewResetAction,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","params":{},"returnType":{},"kind":"reset","target":"{}"}}"#,
        escape_json(&view_action.id),
        escape_json(&view_action.name),
        function_params_json(view_action),
        function_return_json(view_action),
        escape_json(&context.signal_path(&action.target))
    )
}

fn function_params_json(action: &ViewAction) -> String {
    format!(
        "[{}]",
        action
            .params
            .iter()
            .map(|parameter| format!(
                r#"{{"name":"{}","type":"{}"}}"#,
                escape_json(&parameter.name),
                escape_json(&parameter.type_name)
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn function_return_json(action: &ViewAction) -> String {
    json_optional_string(action.return_type.as_ref().map(|value| value.type_name.as_str()))
}

fn json_optional_path(value: Option<&str>, context: &ReactiveRenderContext) -> String {
    value
        .map(|value| format!(r#""{}""#, escape_json(&context.signal_path(value))))
        .unwrap_or_else(|| "null".to_string())
}
