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

