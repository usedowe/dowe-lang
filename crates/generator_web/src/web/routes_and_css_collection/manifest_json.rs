fn navigation_action_json(action: &ViewNavigationAction) -> String {
    match &action.action {
        NavigationAction::Internal {
            path,
            fragment,
            operation,
        } => format!(
            r#"{{"id":"{}","kind":"internal","operation":"{}","path":"{}","fragment":{}}}"#,
            escape_json(&action.id),
            operation.as_str(),
            escape_json(path),
            json_optional_string(fragment.as_deref())
        ),
        NavigationAction::Section {
            fragment,
            operation,
        } => format!(
            r#"{{"id":"{}","kind":"section","operation":"{}","fragment":"{}"}}"#,
            escape_json(&action.id),
            operation.as_str(),
            escape_json(fragment)
        ),
        NavigationAction::External {
            url,
            web_target,
            native_external_mode,
        } => format!(
            r#"{{"id":"{}","kind":"external","operation":"external","url":"{}","webTarget":"{}","nativeExternalMode":"{}"}}"#,
            escape_json(&action.id),
            escape_json(url),
            web_target.as_str(),
            native_external_mode.as_str()
        ),
        NavigationAction::Back => format!(
            r#"{{"id":"{}","kind":"history","operation":"back"}}"#,
            escape_json(&action.id)
        ),
    }
}

fn json_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!(r#""{}""#, escape_json(value)))
        .unwrap_or_else(|| "null".to_string())
}
