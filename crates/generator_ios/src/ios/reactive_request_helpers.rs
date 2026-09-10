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
