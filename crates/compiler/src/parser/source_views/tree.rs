fn lower_export_tree_with_stores(
    node: &SourceNode,
    allow_children: bool,
    types: &TypeRegistry,
    stores: &[ImportedViewStore],
) -> DoweResult<ViewNode> {
    let scope_name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .unwrap_or_default();
    let mut constants = Vec::new();
    let mut signals = stores
        .iter()
        .map(|store| ViewSignal {
            id: reactive_id("store", &node.name, &scope_name, node, &store.name),
            name: store.name.clone(),
            storage_key: store.storage_key.clone(),
            scope: ViewSignalScope::Global,
            storage: store.storage,
            initial: store.initial.clone(),
            schema: store.schema.clone(),
        })
        .collect::<Vec<_>>();
    let mut actions = Vec::new();
    let mut visual_nodes = Vec::new();
    let mut init_action = None;
    let mut splash_node = None;

    for child in &node.children {
        match child.name.as_str() {
            "const" => constants.push(parse_constant(child, &node.name, &scope_name)?),
            "signal" => signals.push(parse_signal(child, &node.name, &scope_name, types)?),
            "init" => {
                if init_action.is_some() {
                    return Err(node_error(
                        child,
                        "a layout or page accepts only one `init` hook",
                    ));
                }
                init_action = Some(parse_view_init(child, &node.name, &scope_name)?);
            }
            "fn" => actions.push(parse_view_function(child, &node.name, &scope_name, types)?),
            "action" => {
                return Err(node_error(
                    child,
                    "`action` was replaced by `fn <name>` in views",
                ));
            }
            "Splash" => {
                if splash_node.is_some() {
                    return Err(node_error(
                        child,
                        "a layout or page accepts only one root `Splash`",
                    ));
                }
                splash_node = Some(child.clone());
            }
            _ => visual_nodes.push(child.clone()),
        }
    }

    if let Some(init_action) = init_action {
        actions.insert(0, init_action);
    }

    actions.extend(lower_inline_on_click_actions(
        &mut visual_nodes,
        &node.name,
        &scope_name,
    )?);
    if let Some(splash) = splash_node.as_mut() {
        actions.extend(lower_inline_on_click_actions(
            &mut splash.children,
            &node.name,
            &scope_name,
        )?);
    }

    let normal_children = lower_node_sequence(&visual_nodes, allow_children)?;
    if normal_children.is_empty() {
        return Err(node_error(node, "view exports must contain a visual node"));
    }
    if node.name == "layout" && normal_children.len() != 1 {
        return Err(node_error(
            node,
            "layout exports must contain one root view node",
        ));
    }
    let mut children = if let Some(splash) = splash_node {
        vec![lower_splash_boundary(&splash, normal_children, &signals)?]
    } else {
        normal_children
    };
    if constants.is_empty() && signals.is_empty() && actions.is_empty() && children.len() == 1 {
        Ok(children.remove(0))
    } else {
        Ok(ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
        })
    }
}

fn lower_splash_boundary(
    node: &SourceNode,
    content: Vec<ViewNode>,
    signals: &[ViewSignal],
) -> DoweResult<ViewNode> {
    if !node.args.is_empty() {
        return Err(node_error(node, "`Splash` does not accept arguments"));
    }
    for prop in &node.props {
        if prop.name != "bind" {
            return Err(prop_error(
                prop,
                format!("unknown prop `{}` on `Splash`", prop.name),
            ));
        }
    }
    let binding = required_prop_bareword(node, "bind")?;
    if binding.starts_with('!') {
        return Err(node_error(
            node,
            "`Splash bind` must reference a boolean Signal or View Store",
        ));
    }
    let initial = splash_binding_initial(node, &binding, signals)?;
    let children = lower_node_sequence(&node.children, false)?;
    Ok(ViewNode::Splash {
        binding,
        initial,
        content,
        children,
    })
}

fn splash_binding_initial(
    node: &SourceNode,
    binding: &str,
    signals: &[ViewSignal],
) -> DoweResult<bool> {
    let mut parts = binding.split('.');
    let root = parts.next().unwrap_or_default();
    let Some(signal) = signals.iter().find(|signal| signal.name == root) else {
        return Err(node_error(
            node,
            "`Splash bind` must reference a boolean Signal or View Store",
        ));
    };
    let mut value = &signal.initial;
    for part in parts {
        let ViewSignalValue::Object(fields) = value else {
            return Err(node_error(
                node,
                "`Splash bind` must reference a boolean Signal or View Store",
            ));
        };
        let Some((_, next)) = fields.iter().find(|(name, _)| name == part) else {
            return Err(node_error(
                node,
                "`Splash bind` must reference a boolean Signal or View Store",
            ));
        };
        value = next;
    }
    let ViewSignalValue::Bool(initial) = value else {
        return Err(node_error(
            node,
            "`Splash bind` must reference a boolean Signal or View Store",
        ));
    };
    Ok(*initial)
}

fn lower_inline_on_click_actions(
    nodes: &mut [SourceNode],
    scope_kind: &str,
    scope_name: &str,
) -> DoweResult<Vec<ViewAction>> {
    let mut actions = Vec::new();
    for node in nodes {
        let node_line = node.location.line;
        let node_column = node.location.column;
        for prop in &mut node.props {
            if prop.name != "onClick" || !matches!(prop.value, SourceValue::Object(_)) {
                continue;
            }
            let action =
                parse_inline_on_click_action(prop, scope_kind, scope_name, node_line, node_column)?;
            prop.value = SourceValue::Bareword(action.name.clone());
            actions.push(action);
        }
        actions.extend(lower_inline_on_click_actions(
            &mut node.children,
            scope_kind,
            scope_name,
        )?);
    }
    Ok(actions)
}

