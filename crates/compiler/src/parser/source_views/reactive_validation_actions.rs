fn validate_action_references(
    path: &Path,
    action: &ViewAction,
    signals: &HashSet<String>,
    readable_values: &HashSet<String>,
    readable_types: &HashMap<String, ViewSignalValue>,
    environment: &EnvironmentConfig,
) -> DoweResult<()> {
    validate_function_signature(path, action, signals, readable_types)?;
    match &action.kind {
        ViewActionKind::Sequence(statements) => {
            let mut results = HashSet::new();
            validate_function_statements(
                path,
                statements,
                signals,
                readable_values,
                readable_types,
                environment,
                &mut results,
            )?;
        }
        ViewActionKind::Request(request) => {
            validate_request_base_env(path, environment, request.base_env.as_deref())?;
            validate_optional_body_name(path, readable_values, request.body.as_deref())?;
            validate_optional_signal_name(path, signals, request.update.as_deref(), "update")?;
            validate_optional_signal_name(path, signals, request.reset.as_deref(), "reset")?;
            validate_optional_signal_name(
                path,
                signals,
                request.success_alert.as_deref(),
                "successAlert",
            )?;
            validate_optional_signal_name(
                path,
                signals,
                request.error_alert.as_deref(),
                "errorAlert",
            )?;
            for header in &request.headers {
                if let ViewRequestHeaderValue::Signal(value) = &header.value {
                    let source_root = path_root(value);
                    if !readable_values.contains(source_root) {
                        return Err(DoweError::at_path(
                            path,
                            format!("unknown request header source `{value}`"),
                        ));
                    }
                }
            }
        }
        ViewActionKind::Invoke(invoke) => {
            validate_invoke_action(path, readable_values, signals, invoke)?;
        }
        ViewActionKind::Assign(assign) => {
            validate_signal_path(path, signals, &assign.target, "target")?;
            if assign.target.contains('.') {
                let locals = HashMap::new();
                signal_path_value(path, readable_types, &locals, &assign.target, "set target")?;
            }
            let locals = HashMap::new();
            match assign.source.as_str() {
                "$dowe:onClick:add" => validate_typed_path(
                    path,
                    readable_types,
                    &locals,
                    &assign.target,
                    "onClick add target",
                    ViewPathExpectation::Number,
                )?,
                "$dowe:onClick:append" => validate_typed_path(
                    path,
                    readable_types,
                    &locals,
                    &assign.target,
                    "onClick append target",
                    ViewPathExpectation::String,
                )?,
                _ => {}
            }
            if let Some(call) = &assign.call {
                for reference in dowe_stdlib::reference_paths(call) {
                    let source_root = path_root(&reference);
                    if !readable_values.contains(source_root) && source_root != "item" {
                        return Err(DoweError::at_path(
                            path,
                            format!("unknown stdlib argument source `{reference}`"),
                        ));
                    }
                }
                return Ok(());
            }
            if assign.source.starts_with("$dowe:") {
                return Ok(());
            }
            let source = assign.source.strip_prefix('!').unwrap_or(&assign.source);
            let source_root = path_root(source);
            if !readable_values.contains(source_root) && source_root != "item" {
                return Err(DoweError::at_path(
                    path,
                    format!("unknown set value `{}`", assign.source),
                ));
            }
            if assign.source.starts_with('!') {
                let locals = HashMap::new();
                let value = signal_path_value(path, readable_types, &locals, source, "set value")?;
                if !matches!(value, Some(ViewSignalValue::Bool(_))) {
                    return Err(DoweError::at_path(
                        path,
                        format!("`set value:{}` must reference a boolean", assign.source),
                    ));
                }
            }
        }
        ViewActionKind::Reset(reset) => {
            validate_signal_name(path, signals, &reset.target, "target")?;
        }
    }
    Ok(())
}

fn validate_function_statements(
    path: &Path,
    statements: &[ViewFunctionStatement],
    signals: &HashSet<String>,
    readable_values: &HashSet<String>,
    readable_types: &HashMap<String, ViewSignalValue>,
    environment: &EnvironmentConfig,
    results: &mut HashSet<String>,
) -> DoweResult<()> {
    for statement in statements {
        match statement {
            ViewFunctionStatement::Validate { target } => {
                validate_signal_name(path, signals, target, "validate")?;
            }
            ViewFunctionStatement::Request { result, action } => {
                if !results.insert(result.clone()) {
                    return Err(DoweError::at_path(
                        path,
                        format!("duplicate request result `{result}`"),
                    ));
                }
                validate_request_base_env(path, environment, action.base_env.as_deref())?;
                validate_optional_body_name(path, readable_values, action.body.as_deref())?;
            }
            ViewFunctionStatement::Invoke { result, action } => {
                if !results.insert(result.clone()) {
                    return Err(DoweError::at_path(
                        path,
                        format!("duplicate request result `{result}`"),
                    ));
                }
                validate_invoke_action(path, readable_values, signals, action)?;
            }
            ViewFunctionStatement::If {
                result,
                success,
                error,
            } => {
                if !results.contains(result) {
                    return Err(DoweError::at_path(
                        path,
                        format!("unknown request result `{result}`"),
                    ));
                }
                validate_function_statements(
                    path,
                    success,
                    signals,
                    readable_values,
                    readable_types,
                    environment,
                    results,
                )?;
                validate_function_statements(
                    path,
                    error,
                    signals,
                    readable_values,
                    readable_types,
                    environment,
                    results,
                )?;
            }
            ViewFunctionStatement::Assign(assign) => {
                validate_signal_path(path, signals, &assign.target, "target")?;
                if assign.call.is_some()
                    || assign.literal.is_some()
                    || assign.source.starts_with("$dowe:")
                {
                    continue;
                }
                let source = assign.source.strip_prefix('!').unwrap_or(&assign.source);
                let source_root = path_root(source);
                if !readable_values.contains(source_root)
                    && source_root != "item"
                    && !results.contains(source_root)
                {
                    return Err(DoweError::at_path(
                        path,
                        format!("unknown set value `{}`", assign.source),
                    ));
                }
            }
            ViewFunctionStatement::Reset(reset) => {
                validate_optional_signal_name(path, signals, Some(&reset.target), "reset")?
            }
            ViewFunctionStatement::Toast(_) => {}
            ViewFunctionStatement::Redirect { .. } => {}
        }
    }
    Ok(())
}

