fn validate_reactive_view_tree(
    path: &Path,
    tree: &ViewNode,
    environment: &EnvironmentConfig,
) -> DoweResult<()> {
    match tree {
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
        } => {
            let signal_names = unique_names(
                path,
                signals.iter().map(|signal| signal.name.as_str()),
                "signal",
            )?;
            let constant_names = unique_names(
                path,
                constants.iter().map(|constant| constant.name.as_str()),
                "constant",
            )?;
            if let Some(name) = signal_names.intersection(&constant_names).next() {
                return Err(DoweError::at_path(
                    path,
                    format!("duplicate view value `{name}`"),
                ));
            }
            let action_names = unique_names(
                path,
                actions.iter().map(|action| action.name.as_str()),
                "fn",
            )?;
            let readable_names = signal_names
                .union(&constant_names)
                .cloned()
                .collect::<HashSet<_>>();
            let mut readable_types = constants
                .iter()
                .map(|constant| (constant.name.clone(), constant.value.clone()))
                .collect::<HashMap<_, _>>();
            readable_types.extend(
                signals
                    .iter()
                    .map(|signal| {
                        (
                            signal.name.clone(),
                            signal
                                .schema
                                .clone()
                                .unwrap_or_else(|| signal.initial.clone()),
                        )
                    })
                    .collect::<HashMap<_, _>>(),
            );
            for action in actions {
                validate_action_references(
                    path,
                    action,
                    &signal_names,
                    &readable_names,
                    &readable_types,
                    environment,
                )?;
                validate_form_validate_targets(path, action, &dowe_components::collect_view_forms(tree))?;
            }
            let locals = HashMap::new();
            for child in children {
                validate_node_references(
                    path,
                    child,
                    &readable_types,
                    &signal_names,
                    &action_names,
                    &locals,
                )?;
            }
            validate_derived_button_paths(path, tree)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_form_validate_targets(
    path: &Path,
    action: &ViewAction,
    forms: &[dowe_components::ViewForm],
) -> DoweResult<()> {
    let form_names = forms
        .iter()
        .map(|form| form.signal.as_str())
        .collect::<HashSet<_>>();
    fn visit(
        path: &Path,
        statements: &[ViewFunctionStatement],
        forms: &HashSet<&str>,
    ) -> DoweResult<()> {
        for statement in statements {
            match statement {
                ViewFunctionStatement::Validate { target } if !forms.contains(target.as_str()) => {
                    return Err(DoweError::at_path(
                        path,
                        format!("`validate {target}` requires a Signal with registered validate fields"),
                    ));
                }
                ViewFunctionStatement::If { success, error, .. } => {
                    visit(path, success, forms)?;
                    visit(path, error, forms)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    if let ViewActionKind::Sequence(statements) = &action.kind {
        visit(path, statements, &form_names)?;
    }
    Ok(())
}

fn validate_derived_button_paths(path: &Path, tree: &ViewNode) -> DoweResult<()> {
    let forms = dowe_components::collect_view_forms(tree)
        .into_iter()
        .map(|form| form.signal)
        .collect::<HashSet<_>>();
    fn visit(path: &Path, node: &ViewNode, forms: &HashSet<String>) -> DoweResult<()> {
        if let ViewNode::Button { props, .. } = node
            && let Some(binding) = props.reactive.disabled.as_deref()
        {
            let root = path_root(binding).to_string();
            if (binding == format!("{root}.isValid")
                || binding == format!("{root}.isInvalid")
                || binding.starts_with(&format!("{root}.errors."))
                || binding.starts_with(&format!("{root}.touched.")))
                && !forms.contains(&root)
            {
                return Err(DoweError::at_path(
                    path,
                    format!("unknown derived form state `{binding}` in `disabled`"),
                ));
            }
        }
        for child in dowe_components::node_children(node) {
            visit(path, child, forms)?;
        }
        Ok(())
    }
    visit(path, tree, &forms)
}

fn unique_names<'a>(
    path: &Path,
    names: impl Iterator<Item = &'a str>,
    kind: &str,
) -> DoweResult<HashSet<String>> {
    let mut output = HashSet::new();
    for name in names {
        if !output.insert(name.to_string()) {
            return Err(DoweError::at_path(
                path,
                format!("duplicate {kind} `{name}`"),
            ));
        }
    }
    Ok(output)
}

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
                .any(|statement| matches!(statement, ViewFunctionStatement::Request { .. }))
            {
                ViewSignalValue::Bool(false)
            } else {
                ViewSignalValue::Null
            }
        }
        ViewActionKind::Request(_) => ViewSignalValue::Bool(false),
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
