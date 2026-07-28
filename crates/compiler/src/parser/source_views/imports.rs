fn view_imports(root: &Path, file: &SourceFile) -> DoweResult<HashMap<String, ViewImport>> {
    let mut imports = HashMap::new();
    for import in &file.imports {
        let path = resolve_import(root, &file.path, import)?;
        if is_shared_type_path(root, &path) {
            continue;
        }
        if read_view_store_file(root, &path)?.is_some() {
            continue;
        }
        if path.starts_with(root.join("config")) {
            return Err(DoweError::at_path(
                &import.location.path,
                "server config modules can only be imported by server source",
            ));
        }
        if imports
            .insert(import.local.clone(), ViewImport { path })
            .is_some()
        {
            return Err(DoweError::at_path(
                &file.path,
                format!("duplicate import `{}`", import.local),
            ));
        }
    }
    Ok(imports)
}

fn view_store_imports(root: &Path, file: &SourceFile) -> DoweResult<Vec<ImportedViewStore>> {
    let mut stores = Vec::new();
    let mut names = HashSet::new();
    for import in &file.imports {
        let path = resolve_import(root, &file.path, import)?;
        let Some(store_file) = read_view_store_file(root, &path)? else {
            continue;
        };
        if !names.insert(import.local.clone()) {
            return Err(DoweError::at_path(
                &import.location.path,
                format!("duplicate View Store import `{}`", import.local),
            ));
        }
        stores.push(parse_view_store_module(root, &store_file, &import.local)?);
    }
    Ok(stores)
}

fn read_view_store_file(root: &Path, path: &Path) -> DoweResult<Option<SourceFile>> {
    let source =
        fs::read_to_string(path).map_err(|error| DoweError::at_path(path, error.to_string()))?;
    let file = parse_source_file(root, path, source)?;
    Ok(file
        .nodes
        .iter()
        .any(|node| node.name == "store")
        .then_some(file))
}

fn parse_view_store_module(
    root: &Path,
    file: &SourceFile,
    imported_name: &str,
) -> DoweResult<ImportedViewStore> {
    for import in &file.imports {
        let path = resolve_import(root, &file.path, import)?;
        if !is_shared_type_path(root, &path) {
            return Err(DoweError::at_path(
                &import.location.path,
                "View Store modules can only import shared type modules",
            ));
        }
    }
    let exports = file
        .nodes
        .iter()
        .filter(|node| node.name != "type")
        .collect::<Vec<_>>();
    let [node] = exports.as_slice() else {
        return Err(DoweError::at_path(
            &file.path,
            "View Store modules must export exactly one `store` declaration",
        ));
    };
    if node.name != "store" {
        return Err(node_error(
            node,
            "View Store modules must export exactly one `store` declaration",
        ));
    }
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`store` must declare one name and no children",
        ));
    }
    let name = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`store` must declare a name"))?;
    if name != imported_name {
        return Err(node_error(
            node,
            format!("View Store export `{name}` does not match import `{imported_name}`"),
        ));
    }
    for prop in &node.props {
        if !matches!(prop.name.as_str(), "type" | "persistent" | "value") {
            return Err(prop_error(
                prop,
                format!("unknown prop `{}` on `store`", prop.name),
            ));
        }
    }
    let value = node
        .prop("value")
        .ok_or_else(|| node_error(node, "`store` requires `value`"))?;
    let initial = signal_value(&value.value, node)?;
    let persistent = match node.prop("persistent").map(|prop| &prop.value) {
        None | Some(SourceValue::Boolean(false)) => false,
        Some(SourceValue::Boolean(true)) => true,
        Some(_) => return Err(node_error(node, "`store persistent` must be a boolean")),
    };
    let types = TypeRegistry::parse_file(root, file)?;
    let schema = optional_prop_string(node, "type")?
        .map(|name| {
            let schema = types.resolve(node, &name)?;
            validate_source_value_type(node, &value.value, &schema, "store value")?;
            Ok::<ViewSignalValue, DoweError>(view_schema_value(&schema))
        })
        .transpose()?;
    let relative = file
        .path
        .strip_prefix(root)
        .unwrap_or(&file.path)
        .with_extension("")
        .to_string_lossy()
        .replace('\\', "/");
    Ok(ImportedViewStore {
        name: name.clone(),
        storage_key: format!("{relative}:{name}"),
        storage: if persistent {
            ViewSignalStorage::Local
        } else {
            ViewSignalStorage::None
        },
        initial,
        schema,
    })
}

pub(crate) fn validate_view_store_source(root: &Path, file: &SourceFile) -> DoweResult<()> {
    let name = file
        .nodes
        .iter()
        .find(|node| node.name == "store")
        .and_then(|node| node.args.first())
        .and_then(SourceValue::as_required_string)
        .ok_or_else(|| {
            DoweError::at_path(
                &file.path,
                "View Store modules must export exactly one `store` declaration",
            )
        })?;
    parse_view_store_module(root, file, &name).map(|_| ())
}

fn reject_unused_imports(
    path: &Path,
    imports: &HashMap<String, ViewImport>,
    used: &HashSet<String>,
) -> DoweResult<()> {
    for local in imports.keys() {
        if !used.contains(local) {
            return Err(DoweError::at_path(
                path,
                format!("import `{local}` is not used by the view module"),
            ));
        }
    }
    Ok(())
}

fn reject_component_usage_shape(node: &SourceNode) -> DoweResult<()> {
    if !node.args.is_empty() || !node.props.is_empty() || !node.children.is_empty() {
        return Err(node_error(
            node,
            format!(
                "component `{}` cannot declare args, props or children",
                node.name
            ),
        ));
    }
    Ok(())
}

fn reject_component_state_nodes(node: &SourceNode) -> DoweResult<()> {
    for child in &node.children {
        if matches!(child.name.as_str(), "signal" | "fn" | "request") {
            return Err(node_error(
                child,
                "component exports cannot declare signal, fn or request",
            ));
        }
        reject_component_state_nodes(child)?;
    }
    Ok(())
}

fn view_declarations(file: &SourceFile) -> DoweResult<Vec<ViewDeclaration>> {
    view_declarations_named(file, None)
}

fn view_declarations_named(
    file: &SourceFile,
    expected_name: Option<&str>,
) -> DoweResult<Vec<ViewDeclaration>> {
    let views = file
        .nodes
        .iter()
        .filter(|node| node.name == "views")
        .collect::<Vec<_>>();
    if views.len() != 1 {
        return Err(DoweError::at_path(
            &file.path,
            "views modules must declare one `views` block",
        ));
    }
    if let Some(expected_name) = expected_name {
        let actual = views[0]
            .args
            .first()
            .and_then(SourceValue::as_required_string)
            .ok_or_else(|| node_error(views[0], "`views` must declare an export name"))?;
        if actual != expected_name {
            return Err(node_error(
                views[0],
                format!("views export `{actual}` does not match import `{expected_name}`"),
            ));
        }
    }
    let declarations = views[0]
        .children
        .iter()
        .map(|node| parse_route_node(node, false))
        .collect::<DoweResult<Vec<_>>>()?;
    non_empty_view_declarations(file, declarations)
}

fn non_empty_view_declarations(
    file: &SourceFile,
    declarations: Vec<ViewDeclaration>,
) -> DoweResult<Vec<ViewDeclaration>> {
    if declarations.is_empty() {
        return Err(DoweError::at_path(
            &file.path,
            "views must declare at least one route",
        ));
    }
    Ok(declarations)
}

fn single_main(file: &SourceFile) -> DoweResult<&SourceNode> {
    let roots = file
        .nodes
        .iter()
        .filter(|node| node.name == "main")
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(DoweError::at_path(
            &file.path,
            "`main.dowe` must declare one `main` block",
        ));
    }
    Ok(roots[0])
}

fn views_references(node: &SourceNode) -> DoweResult<Option<Vec<String>>> {
    let value = node
        .prop("views")
        .map(|prop| &prop.value)
        .or_else(|| node.args.first());
    let Some(value) = value else {
        return Ok(None);
    };
    let values =
        match value {
            SourceValue::Array(values) => {
                if values.is_empty() {
                    return Err(node_error(
                        node,
                        "`views` route module list must not be empty",
                    ));
                }
                values
                    .iter()
                    .map(|value| {
                        let SourceValue::Bareword(value) = value else {
                            return Err(node_error(
                                node,
                                "`views` list values must be imported symbols",
                            ));
                        };
                        (!value.is_empty()).then(|| value.clone()).ok_or_else(|| {
                            node_error(node, "`views` list values must be imported symbols")
                        })
                    })
                    .collect::<DoweResult<Vec<_>>>()?
            }
            value => vec![value.as_required_string().ok_or_else(|| {
                node_error(node, "`views` must reference an imported route graph")
            })?],
        };
    let mut seen = HashSet::new();
    for reference in &values {
        if !seen.insert(reference.clone()) {
            return Err(node_error(
                node,
                format!("duplicate views reference `{reference}`"),
            ));
        }
    }
    Ok(Some(values))
}

fn parse_route_node(node: &SourceNode, inside_group: bool) -> DoweResult<ViewDeclaration> {
    match node.name.as_str() {
        "group" => {
            if inside_group {
                return Err(node_error(
                    node,
                    "view route groups cannot contain another `group`; use sibling groups or direct `route` children",
                ));
            }
            reject_unknown_route_props(node, &["path", "layout", "platform"])?;
            Ok(ViewDeclaration {
                path: required_path_prop(node)?,
                component: required_prop_string(node, "layout")?,
                platforms: optional_platforms_prop(node)?,
                children: node
                    .children
                    .iter()
                    .map(|child| parse_route_node(child, true))
                    .collect::<DoweResult<Vec<_>>>()?,
            })
        }
        "route" => {
            reject_unknown_route_props(node, &["path", "page", "platform"])?;
            Ok(ViewDeclaration {
                path: required_path_prop(node)?,
                component: required_prop_string(node, "page")?,
                platforms: optional_platforms_prop(node)?,
                children: Vec::new(),
            })
        }
        _ => Err(node_error(
            node,
            "route graph only accepts `group` and `route`",
        )),
    }
}

fn reject_unknown_route_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.contains(&prop.name.as_str()) {
            return Err(prop_error(
                prop,
                format!("`{}` does not support `{}`", node.name, prop.name),
            ));
        }
    }
    Ok(())
}

fn optional_platforms_prop(node: &SourceNode) -> DoweResult<Option<Vec<ViewPlatform>>> {
    let Some(prop) = node.prop("platform") else {
        return Ok(None);
    };
    let values = match &prop.value {
        SourceValue::String(value) => vec![platform_from_string(prop, value)?],
        SourceValue::Array(values) => {
            if values.is_empty() {
                return Err(prop_error(
                    prop,
                    "`platform` must include at least one value",
                ));
            }
            values
                .iter()
                .map(|value| match value {
                    SourceValue::String(value) => platform_from_string(prop, value),
                    _ => Err(quoted_static_string_error(prop)),
                })
                .collect::<DoweResult<Vec<_>>>()?
        }
        _ => return Err(quoted_static_string_error(prop)),
    };
    let mut seen = BTreeSet::new();
    let mut platforms = Vec::new();
    for platform in values {
        if !seen.insert(platform) {
            return Err(prop_error(
                prop,
                format!("duplicate platform `{}`", platform.as_str()),
            ));
        }
        platforms.push(platform);
    }
    Ok(Some(
        ViewPlatform::all()
            .iter()
            .copied()
            .filter(|platform| platforms.contains(platform))
            .collect(),
    ))
}

fn platform_from_string(prop: &SourceProp, value: &str) -> DoweResult<ViewPlatform> {
    ViewPlatform::from_name(value).ok_or_else(|| {
        prop_error(
            prop,
            format!(
                "`platform` must be one of \"web\", \"desktop\", \"android\" or \"ios\", got `{value}`"
            ),
        )
    })
}

fn export_tree(
    node: &SourceNode,
    allow_children: bool,
    environment: &EnvironmentConfig,
    types: &TypeRegistry,
) -> DoweResult<ViewNode> {
    export_tree_with_stores(node, allow_children, environment, types, &[])
}

fn export_tree_with_stores(
    node: &SourceNode,
    allow_children: bool,
    environment: &EnvironmentConfig,
    types: &TypeRegistry,
    stores: &[ImportedViewStore],
) -> DoweResult<ViewNode> {
    for store in stores {
        if !node_uses_reference(node, &store.name) {
            return Err(DoweError::at_path(
                &node.location.path,
                format!(
                    "View Store import `{}` is not used by the view module",
                    store.name
                ),
            ));
        }
    }
    let tree = lower_export_tree_with_stores(node, allow_children, types, stores)?;
    validate_view_tree(&tree).map_err(|error| node_error(node, error.to_string()))?;
    validate_reactive_view_tree(&node.location.path, &tree, environment)?;
    Ok(tree)
}

fn node_uses_reference(node: &SourceNode, name: &str) -> bool {
    value_uses_reference(&SourceValue::Bareword(node.name.clone()), name)
        || node
            .args
            .iter()
            .any(|value| value_uses_reference(value, name))
        || node
            .props
            .iter()
            .any(|prop| value_uses_reference(&prop.value, name))
        || node
            .children
            .iter()
            .any(|child| node_uses_reference(child, name))
}

fn value_uses_reference(value: &SourceValue, name: &str) -> bool {
    match value {
        SourceValue::Bareword(value) => value == name || value.starts_with(&format!("{name}.")),
        SourceValue::Array(values) => values.iter().any(|value| value_uses_reference(value, name)),
        SourceValue::Object(entries) => entries.iter().any(|entry| match entry {
            SourceObjectEntry::KeyValue { value, .. } => value_uses_reference(value, name),
            SourceObjectEntry::Spread(value) => {
                value == name || value.starts_with(&format!("{name}."))
            }
        }),
        SourceValue::String(_)
        | SourceValue::Number(_)
        | SourceValue::Boolean(_)
        | SourceValue::Null => false,
    }
}

