fn parse_raw_theme(node: &SourceNode) -> DoweResult<RawTheme> {
    if !node.args.is_empty() {
        return Err(node_error(node, "`theme` does not accept args"));
    }
    reject_unknown_props(node, &["name", "extends"])?;
    let name_prop = node
        .prop("name")
        .ok_or_else(|| node_error(node, "`theme` requires `name`"))?;
    let name = parse_theme_name_prop(name_prop, "theme.name")?;
    let extends = match node.prop("extends") {
        Some(prop) => Some(parse_theme_name_prop(prop, "theme.extends")?),
        None => None,
    };
    let mut colors = BTreeMap::new();
    let mut color_families = HashSet::new();

    for child in &node.children {
        match child.name.as_str() {
            "colors" => parse_colors(child, &mut colors, &mut color_families)?,
            _ => {
                return Err(node_error(
                    child,
                    format!("`{}` is not valid inside `theme`", child.name),
                ));
            }
        }
    }

    Ok(RawTheme {
        node: node.clone(),
        name,
        extends,
        colors,
    })
}

fn parse_theme_color_family_name(value: &str) -> Option<ColorFamily> {
    ColorFamily::from_theme_name(value).map(|(family, _)| family)
}

fn parse_colors(
    node: &SourceNode,
    colors: &mut BTreeMap<ColorToken, String>,
    color_families: &mut HashSet<String>,
) -> DoweResult<()> {
    if !node.args.is_empty() {
        return Err(node_error(node, "`colors` does not accept args"));
    }
    if let Some(prop) = node.props.first() {
        return Err(prop_error(
            prop,
            "`colors` accepts grouped color families such as `primary color:\"#1F3A5F\" text:\"#FFFFFF\" title:\"#FFFFFE\"`",
        ));
    }
    for family_node in &node.children {
        let Some(family) = parse_theme_color_family_name(&family_node.name) else {
            return Err(node_error(
                family_node,
                format!("unknown color family `{}`", family_node.name),
            ));
        };
        if !color_families.insert(family_node.name.clone()) {
            return Err(node_error(
                family_node,
                format!("duplicate color family `{}`", family_node.name),
            ));
        }
        if !family_node.args.is_empty() || !family_node.children.is_empty() {
            return Err(node_error(
                family_node,
                format!("color family `{}` only accepts props", family_node.name),
            ));
        }
        reject_unknown_props(family_node, &["color", "text", "title"])?;
        if family_node.props.is_empty() {
            return Err(node_error(
                family_node,
                format!(
                    "color family `{}` requires at least one of `color`, `text`, or `title`",
                    family_node.name
                ),
            ));
        }
        let tokens = [
            family.color_token(),
            family.text_token(),
            family.title_token(),
        ];
        for (role, token) in ["color", "text", "title"].into_iter().zip(tokens) {
            let Some(prop) = family_node.prop(role) else {
                continue;
            };
            if colors.contains_key(&token) {
                return Err(prop_error(
                    prop,
                    format!("duplicate `{role}` role for `{}`", family_node.name),
                ));
            }
            let value = required_static_string_prop(prop)?;
            colors.insert(token, normalize_hex_color(prop, &value)?);
        }
    }
    Ok(())
}

fn resolve_theme(
    name: &str,
    raw: &HashMap<String, RawTheme>,
    resolving: &mut HashSet<String>,
    resolved: &mut HashMap<String, DesignTheme>,
) -> DoweResult<DesignTheme> {
    if let Some(theme) = resolved.get(name) {
        return Ok(theme.clone());
    }
    let theme = raw
        .get(name)
        .ok_or_else(|| DoweError::new(format!("theme `{name}` is not declared")))?;
    if !resolving.insert(name.to_string()) {
        return Err(node_error(
            &theme.node,
            format!("theme `{name}` has cyclic inheritance"),
        ));
    }

    let base = match theme.extends.as_deref() {
        Some(parent) => {
            if raw.contains_key(parent) {
                Some(resolve_theme(parent, raw, resolving, resolved)?)
            } else {
                integrated_design_theme(parent)
                    .ok_or_else(|| {
                        node_error(
                            &theme.node,
                            format!("theme `{name}` extends unknown theme `{parent}`"),
                        )
                    })?
                    .into()
            }
        }
        None => integrated_design_theme(&theme.name),
    };

    let mut colors = base
        .as_ref()
        .map(|theme| theme.colors.clone())
        .unwrap_or_default();
    for (token, value) in &theme.colors {
        colors.insert(*token, value.clone());
    }

    let radius = base.as_ref().map(|value| value.radius).unwrap_or(8);

    for token in ColorToken::all() {
        if !colors.contains_key(token) {
            return Err(node_error(
                &theme.node,
                format!("theme `{name}` is missing `{}`", token.as_str()),
            ));
        }
    }

    resolving.remove(name);
    let resolved_theme = DesignTheme {
        name: name.to_string(),
        colors,
        radius,
    };
    resolved.insert(name.to_string(), resolved_theme.clone());
    Ok(resolved_theme)
}

fn parse_theme_name_prop(prop: &SourceProp, field: &str) -> DoweResult<String> {
    let value = required_static_string_prop(prop)?;
    if is_valid_theme_name(&value) {
        Ok(value)
    } else {
        Err(prop_error(
            prop,
            format!("`{field}` must use lowercase letters, numbers, and hyphens"),
        ))
    }
}

fn parse_app_name_prop(prop: &SourceProp) -> DoweResult<String> {
    let value = required_static_string_prop(prop)?;
    if !value.trim().is_empty() && !value.contains('\n') && !value.contains('\r') {
        Ok(value)
    } else {
        Err(prop_error(
            prop,
            "`app.name` must be a non-empty single-line string",
        ))
    }
}

fn parse_app_bundle_prop(prop: &SourceProp) -> DoweResult<String> {
    let value = required_static_string_prop(prop)?;
    if is_valid_app_bundle(&value) {
        Ok(value)
    } else {
        Err(prop_error(
            prop,
            "`app.bundle` must be a reverse-DNS identifier such as `com.example.app`",
        ))
    }
}

fn is_valid_theme_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && chars.all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == '-')
}

fn is_valid_app_bundle(value: &str) -> bool {
    let segments = value.split('.').collect::<Vec<_>>();
    segments.len() >= 2
        && segments
            .iter()
            .all(|segment| is_valid_bundle_segment(segment))
}

fn is_valid_bundle_segment(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}

fn normalize_hex_color(prop: &SourceProp, value: &str) -> DoweResult<String> {
    let Some(raw) = value.strip_prefix('#') else {
        return Err(prop_error(
            prop,
            format!("color token `{}` must be a hex color", prop.name),
        ));
    };
    if !matches!(raw.len(), 3 | 6 | 8) || !raw.chars().all(|value| value.is_ascii_hexdigit()) {
        return Err(prop_error(
            prop,
            format!(
                "color token `{}` must be `#RGB`, `#RRGGBB`, or `#RRGGBBAA`",
                prop.name
            ),
        ));
    }
    let output = if raw.len() == 3 {
        raw.chars()
            .flat_map(|value| [value.to_ascii_lowercase(), value.to_ascii_lowercase()])
            .collect::<String>()
    } else {
        raw.to_ascii_lowercase()
    };
    Ok(format!("#{output}"))
}

