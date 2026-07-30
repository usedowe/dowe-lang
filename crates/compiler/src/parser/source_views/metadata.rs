fn parse_view_metadata(node: &SourceNode) -> DoweResult<Vec<ViewMetadata>> {
    let mut metadata = Vec::new();
    let mut names = HashSet::new();
    for child in node.children.iter().filter(|child| child.name == "meta") {
        if !child.args.is_empty() || !child.children.is_empty() {
            return Err(node_error(
                child,
                "`meta` accepts only `name` and `content` props and no children",
            ));
        }
        let mut prop_names = HashSet::new();
        for prop in &child.props {
            if !matches!(prop.name.as_str(), "name" | "content") {
                return Err(prop_error(
                    prop,
                    format!("unknown prop `{}` on `meta`", prop.name),
                ));
            }
            if !prop_names.insert(prop.name.as_str()) {
                return Err(prop_error(
                    prop,
                    format!("duplicate prop `{}` on `meta`", prop.name),
                ));
            }
        }
        let name = static_meta_prop(child, "name")?;
        let content = static_meta_prop(child, "content")?;
        if !VIEW_META_NAMES.contains(&name.as_str()) {
            return Err(node_error(
                child,
                format!("unsupported meta name `{name}`"),
            ));
        }
        if !names.insert(name.clone()) {
            return Err(node_error(
                child,
                format!("duplicate meta name `{name}`"),
            ));
        }
        metadata.push(ViewMetadata { name, content });
    }
    Ok(metadata)
}

fn static_meta_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("missing `{name}` on `meta`")))?;
    let SourceValue::String(value) = &prop.value else {
        return Err(quoted_static_string_error(prop));
    };
    if value.trim().is_empty() {
        return Err(prop_error(prop, format!("`{name}` must not be empty")));
    }
    Ok(value.clone())
}
