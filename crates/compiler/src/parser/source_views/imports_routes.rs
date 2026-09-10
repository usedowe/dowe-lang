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

