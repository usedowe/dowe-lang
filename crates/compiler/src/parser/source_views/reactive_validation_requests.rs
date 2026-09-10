fn validate_invoke_action(
    path: &Path,
    readable_values: &HashSet<String>,
    signals: &HashSet<String>,
    action: &ViewInvokeAction,
) -> DoweResult<()> {
    validate_optional_signal_name(path, signals, action.update.as_deref(), "update")?;
    validate_optional_signal_name(path, signals, action.reset.as_deref(), "reset")?;
    validate_optional_signal_name(path, signals, action.success_alert.as_deref(), "successAlert")?;
    validate_optional_signal_name(path, signals, action.error_alert.as_deref(), "errorAlert")?;
    let mut references = Vec::new();
    for arg in &action.args {
        collect_stdlib_value_references(&arg.value, &mut references);
    }
    for reference in references {
        let source_root = path_root(&reference);
        if !readable_values.contains(source_root) && source_root != "item" {
            return Err(DoweError::at_path(
                path,
                format!("unknown invoke argument source `{reference}`"),
            ));
        }
    }
    Ok(())
}

fn collect_stdlib_value_references(value: &StdlibValue, output: &mut Vec<String>) {
    match value {
        StdlibValue::Reference(value) => output.push(value.clone()),
        StdlibValue::Array(values) => {
            for value in values {
                collect_stdlib_value_references(value, output);
            }
        }
        StdlibValue::Object(entries) => {
            for (_, value) in entries {
                collect_stdlib_value_references(value, output);
            }
        }
        StdlibValue::Null | StdlibValue::Bool(_) | StdlibValue::Number(_) | StdlibValue::String(_) => {}
    }
}

fn validate_function_signature(
    path: &Path,
    action: &ViewAction,
    signals: &HashSet<String>,
    readable_types: &HashMap<String, ViewSignalValue>,
) -> DoweResult<()> {
    for parameter in &action.params {
        if !signals.contains(&parameter.name) {
            return Err(DoweError::at_path(
                path,
                format!("unknown fn parameter source `{}`", parameter.name),
            ));
        }
        let actual = readable_types
            .get(&parameter.name)
            .expect("function parameter signal type");
        if !view_value_assignable(actual, &parameter.schema) {
            return Err(DoweError::at_path(
                path,
                format!(
                    "fn parameter `{}` does not match declared type `{}`",
                    parameter.name, parameter.type_name
                ),
            ));
        }
    }
    let Some(return_type) = &action.return_type else {
        return Ok(());
    };
    let actual = match &action.kind {
        ViewActionKind::Sequence(statements) => {
            if statements
                .iter()
                .any(|statement| matches!(statement, ViewFunctionStatement::Request { .. } | ViewFunctionStatement::Invoke { .. }))
            {
                ViewSignalValue::Bool(false)
            } else {
                ViewSignalValue::Null
            }
        }
        ViewActionKind::Request(_) | ViewActionKind::Invoke(_) => ViewSignalValue::Bool(false),
        ViewActionKind::Assign(assign) => signal_path_value(
            path,
            readable_types,
            &HashMap::new(),
            &assign.target,
            "set target",
        )?
        .unwrap_or(ViewSignalValue::Null),
        ViewActionKind::Reset(reset) => readable_types
            .get(&reset.target)
            .cloned()
            .unwrap_or(ViewSignalValue::Null),
    };
    if view_value_assignable(&actual, &return_type.schema) {
        Ok(())
    } else {
        Err(DoweError::at_path(
            path,
            format!(
                "fn return type `{}` does not match its operation result",
                return_type.type_name
            ),
        ))
    }
}

fn view_value_assignable(actual: &ViewSignalValue, expected: &ViewSignalValue) -> bool {
    match (actual, expected) {
        (_, ViewSignalValue::Null) => matches!(actual, ViewSignalValue::Null),
        (ViewSignalValue::Bool(_), ViewSignalValue::Bool(_))
        | (ViewSignalValue::Number(_), ViewSignalValue::Number(_))
        | (ViewSignalValue::String(_), ViewSignalValue::String(_)) => true,
        (ViewSignalValue::Array(actual), ViewSignalValue::Array(expected)) => {
            match (actual.first(), expected.first()) {
                (_, None) => true,
                (Some(actual), Some(expected)) => view_value_assignable(actual, expected),
                (None, Some(_)) => true,
            }
        }
        (ViewSignalValue::Object(actual), ViewSignalValue::Object(expected)) => {
            expected.iter().all(|(name, expected)| {
                actual
                    .iter()
                    .find(|(candidate, _)| candidate == name)
                    .is_some_and(|(_, actual)| view_value_assignable(actual, expected))
            })
        }
        _ => false,
    }
}

fn validate_request_base_env(
    path: &Path,
    environment: &EnvironmentConfig,
    name: Option<&str>,
) -> DoweResult<()> {
    let Some(name) = name else {
        return Ok(());
    };
    let variable = environment.variable(name).ok_or_else(|| {
        DoweError::at_path(path, format!("unknown environment variable `{name}`"))
    })?;
    if let Some(value) = variable.resolved_value.as_deref()
        && !value.is_empty()
        && !valid_request_base_url(value)
    {
        return Err(DoweError::at_path(
            path,
            format!("environment variable `{name}` must resolve to an http or https URL"),
        ));
    }
    Ok(())
}

fn valid_request_base_url(value: &str) -> bool {
    if value.contains('?') || value.contains('#') || value.chars().any(char::is_whitespace) {
        return false;
    }
    let Some(rest) = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
    else {
        return false;
    };
    !rest.is_empty() && !rest.starts_with('/') && !rest.starts_with('?') && !rest.starts_with('#')
}
