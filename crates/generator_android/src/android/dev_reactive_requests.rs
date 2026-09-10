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
