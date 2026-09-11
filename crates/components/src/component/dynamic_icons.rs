#[derive(Clone)]
struct IconBindingValue {
    value: ViewSignalValue,
    is_constant: bool,
}

fn resolve_icon_binding(
    path: &str,
    values: &std::collections::HashMap<String, IconBindingValue>,
    locals: &std::collections::HashMap<String, IconBindingValue>,
) -> Option<IconBindingValue> {
    let root = path.split('.').next().unwrap_or(path);
    let mut value = values.get(root).or_else(|| locals.get(root))?.clone();
    for field in path.split('.').skip(1) {
        let ViewSignalValue::Object(fields) = value.value else {
            return None;
        };
        value.value = fields
            .into_iter()
            .find_map(|(name, value)| (name == field).then_some(value))?;
    }
    Some(value)
}

fn unresolved_icon_demand(binding: &str) -> ComponentError {
    ComponentError::new(format!(
        "computed Icon name `{binding}` must resolve to known names from a `const` or an `each` over a constant collection; mutable or unresolved values cannot determine which icons to generate"
    ))
}

pub fn dynamic_icon_names(node: &ViewNode) -> ComponentResult<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    if !tree_has_dynamic_icon(node) {
        return Ok(names);
    }
    static VALID_NAMES: std::sync::OnceLock<BTreeSet<String>> = std::sync::OnceLock::new();
    let valid_names = VALID_NAMES.get_or_init(|| all_icon_names().into_iter().collect());
    let values = std::collections::HashMap::new();
    let locals = std::collections::HashMap::new();
    let mut pending = vec![(node, values, locals)];
    while let Some((node, values, locals)) = pending.pop() {
        match node {
            ViewNode::Scope {
                constants,
                signals,
                children,
                ..
            } => {
                let mut scoped = values;
                scoped.extend(constants.iter().map(|constant| {
                    (
                        constant.name.clone(),
                        IconBindingValue {
                            value: constant.value.clone(),
                            is_constant: true,
                        },
                    )
                }));
                scoped.extend(signals.iter().map(|signal| {
                    (
                        signal.name.clone(),
                        IconBindingValue {
                            value: ViewSignalValue::Null,
                            is_constant: false,
                        },
                    )
                }));
                pending.extend(
                    children
                        .iter()
                        .map(|child| (child, scoped.clone(), locals.clone())),
                );
            }
            ViewNode::Each {
                item,
                collection,
                children,
                ..
            } => {
                if !children.iter().any(tree_has_dynamic_icon) {
                    continue;
                }
                let items = match resolve_icon_binding(collection, &values, &locals) {
                    Some(IconBindingValue {
                        value: ViewSignalValue::Array(items),
                        is_constant: true,
                    }) => items
                        .into_iter()
                        .map(|value| IconBindingValue {
                            value,
                            is_constant: true,
                        })
                        .collect::<Vec<_>>(),
                    Some(IconBindingValue {
                        is_constant: true, ..
                    }) => return Err(unresolved_icon_demand(collection)),
                    _ => vec![IconBindingValue {
                        value: ViewSignalValue::Null,
                        is_constant: false,
                    }],
                };
                for item_value in items {
                    let mut scoped = locals.clone();
                    scoped.insert(item.clone(), item_value);
                    pending.extend(
                        children
                            .iter()
                            .map(|child| (child, values.clone(), scoped.clone())),
                    );
                }
            }
            ViewNode::Svg { props, .. }
            | ViewNode::Avatar {
                icon: Some(SideNavIcon { props, .. }),
                ..
            } => {
                let Some(binding) = props.icon_name.as_deref() else {
                    continue;
                };
                let Some(IconBindingValue {
                    value,
                    is_constant: true,
                }) = resolve_icon_binding(binding, &values, &locals)
                else {
                    return Err(unresolved_icon_demand(binding));
                };
                let ViewSignalValue::String(name) = value else {
                    return Err(ComponentError::new(format!(
                        "computed Icon name `{binding}` must be a string for every possible value"
                    )));
                };
                if !valid_names.contains(&name) {
                    return Err(ComponentError::new(format!(
                        "unknown icon name `{name}` from computed Icon name `{binding}`"
                    )));
                }
                names.insert(name);
            }
            _ => {
                pending.extend(
                    node_child_groups(node)
                        .into_iter()
                        .flatten()
                        .map(|child| (child, values.clone(), locals.clone())),
                );
            }
        }
    }
    Ok(names)
}

pub fn dynamic_icon_catalog_for_trees<'a>(
    trees: impl IntoIterator<Item = &'a ViewNode>,
) -> ComponentResult<Vec<(String, String)>> {
    let mut names = BTreeSet::new();
    for tree in trees {
        names.extend(dynamic_icon_names(tree)?);
    }
    runtime_icon_catalog_for_names(names)
}
