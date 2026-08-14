fn validate_side_nav_actions(
    path: &Path,
    items: &[dowe_components::SideNavItem],
    actions: &HashSet<String>,
) -> DoweResult<()> {
    for item in items {
        match item {
            dowe_components::SideNavItem::Header(props)
            | dowe_components::SideNavItem::Item(props) => {
                validate_side_nav_item_action(path, props, actions)?;
            }
            dowe_components::SideNavItem::Submenu { props, items, .. } => {
                validate_side_nav_item_action(path, props, actions)?;
                for props in items {
                    validate_side_nav_item_action(path, props, actions)?;
                }
            }
            dowe_components::SideNavItem::Divider => {}
        }
    }
    Ok(())
}

fn validate_side_nav_item_action(
    path: &Path,
    props: &dowe_components::SideNavItemProps,
    actions: &HashSet<String>,
) -> DoweResult<()> {
    if let Some(action) = props.on_click.as_ref()
        && !actions.contains(action)
    {
        return Err(DoweError::at_path(path, format!("unknown fn `{action}`")));
    }
    Ok(())
}

fn validate_nav_menu_actions(
    path: &Path,
    items: &[dowe_components::NavMenuItem],
    actions: &HashSet<String>,
) -> DoweResult<()> {
    for item in items {
        match item {
            dowe_components::NavMenuItem::Item(props) => {
                validate_nav_menu_item_action(path, props, actions)?;
            }
            dowe_components::NavMenuItem::Submenu { props, items } => {
                validate_nav_menu_item_action(path, props, actions)?;
                for props in items {
                    validate_nav_menu_item_action(path, props, actions)?;
                }
            }
            dowe_components::NavMenuItem::Megamenu { props, .. } => {
                validate_nav_menu_item_action(path, props, actions)?;
            }
        }
    }
    Ok(())
}

fn validate_nav_menu_item_action(
    path: &Path,
    props: &dowe_components::NavMenuItemProps,
    actions: &HashSet<String>,
) -> DoweResult<()> {
    if let Some(action) = props.on_click.as_ref()
        && !actions.contains(action)
    {
        return Err(DoweError::at_path(path, format!("unknown fn `{action}`")));
    }
    Ok(())
}

fn validate_optional_signal_name(
    path: &Path,
    signals: &HashSet<String>,
    value: Option<&str>,
    label: &str,
) -> DoweResult<()> {
    if let Some(value) = value {
        validate_signal_name(path, signals, value, label)?;
    }
    Ok(())
}

fn validate_optional_body_name(
    path: &Path,
    signals: &HashSet<String>,
    value: Option<&str>,
) -> DoweResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    let root = path_root(value);
    if signals.contains(root) || root == "item" {
        Ok(())
    } else {
        Err(DoweError::at_path(
            path,
            format!("unknown request body `{value}`"),
        ))
    }
}

fn validate_signal_name(
    path: &Path,
    signals: &HashSet<String>,
    value: &str,
    label: &str,
) -> DoweResult<()> {
    if signals.contains(value) {
        Ok(())
    } else {
        Err(DoweError::at_path(
            path,
            format!("unknown signal `{value}` in `{label}`"),
        ))
    }
}

fn validate_signal_path(
    path: &Path,
    signals: &HashSet<String>,
    value: &str,
    label: &str,
) -> DoweResult<()> {
    validate_signal_name(path, signals, path_root(value), label)
}

#[derive(Clone, Copy)]
enum ViewPathExpectation {
    Any,
    String,
    Bool,
    Number,
}

fn validate_typed_path(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    value: &str,
    label: &str,
    expectation: ViewPathExpectation,
) -> DoweResult<()> {
    let root = path_root(value);
    if signals.contains_key(root)
        && (value == format!("{root}.isValid")
            || value == format!("{root}.isInvalid"))
    {
        return if matches!(expectation, ViewPathExpectation::Bool | ViewPathExpectation::Any) {
            Ok(())
        } else {
            Err(DoweError::at_path(
                path,
                format!("invalid signal path `{value}` in `{label}`: expected bool"),
            ))
        };
    }
    if signals.contains_key(root)
        && (value.starts_with(&format!("{root}.errors."))
            || value.starts_with(&format!("{root}.touched.")))
        && matches!(expectation, ViewPathExpectation::Any | ViewPathExpectation::String | ViewPathExpectation::Bool)
    {
        return Ok(());
    }
    let mut resolved = if let Some(value) = signals.get(root) {
        Some(value.clone())
    } else if let Some(value) = locals.get(root) {
        value.clone()
    } else {
        return Err(DoweError::at_path(
            path,
            format!("unknown signal path `{value}` in `{label}`"),
        ));
    };
    let Some(mut resolved_value) = resolved.take() else {
        return Ok(());
    };
    for field in value.split('.').skip(1) {
        let ViewSignalValue::Object(fields) = resolved_value else {
            return Err(DoweError::at_path(
                path,
                format!("unknown signal path `{value}` in `{label}`"),
            ));
        };
        let Some((_, next)) = fields.into_iter().find(|(name, _)| name == field) else {
            return Err(DoweError::at_path(
                path,
                format!("unknown signal path `{value}` in `{label}`"),
            ));
        };
        resolved_value = next;
    }
    let valid = match expectation {
        ViewPathExpectation::Any => true,
        ViewPathExpectation::String => matches!(resolved_value, ViewSignalValue::String(_)),
        ViewPathExpectation::Bool => matches!(resolved_value, ViewSignalValue::Bool(_)),
        ViewPathExpectation::Number => matches!(resolved_value, ViewSignalValue::Number(_)),
    };
    if valid {
        Ok(())
    } else {
        let expected = match expectation {
            ViewPathExpectation::Any => unreachable!(),
            ViewPathExpectation::String => "string",
            ViewPathExpectation::Bool => "bool",
            ViewPathExpectation::Number => "number",
        };
        Err(DoweError::at_path(
            path,
            format!("invalid signal path `{value}` in `{label}`: expected {expected}"),
        ))
    }
}

fn path_root(value: &str) -> &str {
    value.split('.').next().unwrap_or(value)
}

fn reactive_id(
    namespace: &str,
    scope_kind: &str,
    scope_name: &str,
    node: &SourceNode,
    name: &str,
) -> String {
    let source = format!(
        "{scope_kind}:{scope_name}:{}:{}:{name}",
        node.location.line, node.location.column
    );
    stable_reactive_id(namespace, &source)
}

fn synthetic_reactive_id(
    namespace: &str,
    scope_kind: &str,
    scope_name: &str,
    line: usize,
    column: usize,
    name: &str,
) -> String {
    stable_reactive_id(
        namespace,
        &format!("{scope_kind}:{scope_name}:{line}:{column}:{name}"),
    )
}

fn stable_reactive_id(namespace: &str, source: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in namespace.bytes().chain([0]).chain(source.bytes()) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let alphabet = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut value = hash;
    let mut id = String::with_capacity(8);
    for index in 0..8 {
        let digit = (value % 36) as usize;
        id.push(alphabet[digit] as char);
        value /= 36;
        if value == 0 {
            value = hash.rotate_left((index + 1) as u32);
        }
    }
    id
}
