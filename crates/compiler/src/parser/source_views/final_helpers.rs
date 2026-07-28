fn is_dynamic_reference(value: &str) -> bool {
    value.contains('.')
        && value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || value == '_' || value == '.')
}

fn validate_navigation(pages: &[ViewPage]) -> DoweResult<()> {
    let mut sections_by_path = HashMap::new();
    for page in pages {
        sections_by_path.insert(
            page.route_path.clone(),
            page.sections
                .iter()
                .map(|section| section.id.clone())
                .collect::<HashSet<_>>(),
        );
    }
    for page in pages {
        for action in &page.navigation_actions {
            validate_navigation_action(page, action, &sections_by_path)?;
        }
    }
    Ok(())
}

fn validate_navigation_action(
    page: &ViewPage,
    action: &ViewNavigationAction,
    sections_by_path: &HashMap<String, HashSet<String>>,
) -> DoweResult<()> {
    match &action.action {
        dowe_components::NavigationAction::Internal { path, fragment, .. } => {
            let Some(sections) = sections_by_path.get(path) else {
                return Err(DoweError::at_path(
                    &page.source_path,
                    format!("unknown navigation route `{path}`"),
                ));
            };
            if let Some(fragment) = fragment
                && !sections.contains(fragment)
            {
                return Err(DoweError::at_path(
                    &page.source_path,
                    format!("unknown section `#{fragment}` for route `{path}`"),
                ));
            }
        }
        dowe_components::NavigationAction::Section { fragment, .. } => {
            let sections = sections_by_path
                .get(&page.route_path)
                .expect("current route sections");
            if !sections.contains(fragment) {
                return Err(DoweError::at_path(
                    &page.source_path,
                    format!(
                        "unknown section `#{fragment}` for route `{}`",
                        page.route_path
                    ),
                ));
            }
        }
        dowe_components::NavigationAction::External { .. }
        | dowe_components::NavigationAction::Back => {}
    }
    Ok(())
}

fn normalize_route_path(parent: &str, child: &str) -> String {
    let raw = if child.starts_with('/') {
        child.to_string()
    } else if child.is_empty() {
        parent.to_string()
    } else if parent == "/" {
        format!("/{child}")
    } else {
        format!("{}/{}", parent.trim_end_matches('/'), child)
    };
    let parts = raw
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

fn combine_layout_stack(layouts: &[RouteLayout]) -> ViewNode {
    let mut tree = ViewNode::Children;
    for layout in layouts.iter().rev() {
        tree = compose_tree(&layout.tree, &tree);
    }
    tree
}

fn strip_web_prefix(path: &Path) -> String {
    path.strip_prefix("web")
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn route_id(path: &str) -> String {
    let name = path
        .trim_matches('/')
        .replace(|value: char| !value.is_ascii_alphanumeric(), "-");
    if name.is_empty() {
        "index".to_string()
    } else {
        name
    }
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}

fn prop_error(prop: &SourceProp, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &prop.location.path,
        format!(
            "{}:{}: {}",
            prop.location.line,
            prop.location.column,
            message.as_ref()
        ),
    )
}

fn quoted_static_string_error(prop: &SourceProp) -> DoweError {
    prop_error(
        prop,
        ComponentError::invalid_prop(&prop.name, "quoted static string literal").to_string(),
    )
}

fn component_error(node: &SourceNode, error: ComponentError) -> DoweError {
    let message = error.to_string();
    if let Some(name) = first_backtick_value(&message)
        && let Some(prop) = node.prop(name)
    {
        return prop_error(prop, message);
    }
    node_error(node, message)
}

fn first_backtick_value(message: &str) -> Option<&str> {
    let (_, after_open) = message.split_once('`')?;
    let (value, _) = after_open.split_once('`')?;
    if value.is_empty() { None } else { Some(value) }
}
