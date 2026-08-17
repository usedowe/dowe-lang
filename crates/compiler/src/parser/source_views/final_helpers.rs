fn is_dynamic_reference(value: &str) -> bool {
    value.contains('.')
        && value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || value == '_' || value == '.')
}

fn resolve_dynamic_icon_fallbacks(tree: &mut ViewNode) {
    fn resolve_value(
        path: &str,
        values: &HashMap<String, ViewSignalValue>,
        locals: &HashMap<String, Option<ViewSignalValue>>,
    ) -> Option<ViewSignalValue> {
        let root = path.split('.').next().unwrap_or(path);
        let mut value = values
            .get(root)
            .cloned()
            .or_else(|| locals.get(root).and_then(Clone::clone))?;
        for field in path.split('.').skip(1) {
            let ViewSignalValue::Object(fields) = value else {
                return None;
            };
            value = fields
                .into_iter()
                .find_map(|(name, value)| (name == field).then_some(value))?;
        }
        Some(value)
    }

    fn visit(
        node: &mut ViewNode,
        values: &HashMap<String, ViewSignalValue>,
        locals: &HashMap<String, Option<ViewSignalValue>>,
    ) {
        match node {
            ViewNode::Scope {
                constants,
                signals,
                children,
                ..
            } => {
                let mut scoped = values.clone();
                scoped.extend(
                    constants
                        .iter()
                        .map(|constant| (constant.name.clone(), constant.value.clone())),
                );
                scoped.extend(
                    signals
                        .iter()
                        .map(|signal| (signal.name.clone(), signal.initial.clone())),
                );
                for child in children {
                    visit(child, &scoped, locals);
                }
            }
            ViewNode::Each {
                collection,
                item,
                children,
                ..
            } => {
                let mut scoped = locals.clone();
                let item_value =
                    resolve_value(collection, values, locals).and_then(|value| match value {
                        ViewSignalValue::Array(items) => items.first().cloned(),
                        _ => None,
                    });
                scoped.insert(item.clone(), item_value);
                for child in children {
                    visit(child, values, &scoped);
                }
            }
            ViewNode::Svg { props, paths } => {
                let Some(binding) = props.icon_name.as_deref() else {
                    return;
                };
                let Some(ViewSignalValue::String(name)) = resolve_value(binding, values, locals)
                else {
                    return;
                };
                if !dowe_components::all_icon_names()
                    .iter()
                    .any(|value| value == &name)
                {
                    return;
                }
                props.icon_fallback = Some(name.clone());
                let mut icon_props = vec![ComponentProp {
                    name: "name".to_string(),
                    value: PropValue::String(name),
                }];
                if let Some(fill) = props.icon_fill {
                    icon_props.push(ComponentProp {
                        name: "fill".to_string(),
                        value: PropValue::String(fill.as_str().to_string()),
                    });
                }
                if let Some(stroke) = props.icon_stroke {
                    icon_props.push(ComponentProp {
                        name: "stroke".to_string(),
                        value: PropValue::String(stroke.as_str().to_string()),
                    });
                }
                if let Ok(ViewNode::Svg {
                    props: fallback_props,
                    paths: fallback_paths,
                }) = icon_component_node(icon_props)
                {
                    props.view_box = fallback_props.view_box;
                    *paths = fallback_paths;
                }
            }
            _ => {
                for group in node_child_groups_mut(node) {
                    for child in group {
                        visit(child, values, locals);
                    }
                }
            }
        }
    }

    visit(tree, &HashMap::new(), &HashMap::new());
}

fn validate_navigation(pages: &[Arc<ViewPage>]) -> DoweResult<()> {
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

fn compose_route_metadata(
    layouts: &[RouteLayout],
    page_metadata: &[ViewMetadata],
) -> Vec<ViewMetadata> {
    let mut metadata = Vec::<ViewMetadata>::new();
    for entry in layouts
        .iter()
        .flat_map(|layout| layout.metadata.iter())
        .chain(page_metadata)
    {
        if let Some(existing) = metadata.iter_mut().find(|item| item.name == entry.name) {
            existing.content = entry.content.clone();
        } else {
            metadata.push(entry.clone());
        }
    }
    metadata
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
