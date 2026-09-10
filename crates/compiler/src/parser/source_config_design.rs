fn parse_fonts(node: &SourceNode) -> DoweResult<FontConfig> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`fonts` only accepts `default` and `install` props",
        ));
    }
    reject_unknown_props(node, &["default", "install"])?;

    let default_family = match node.prop("default") {
        Some(prop) => parse_font_token(prop, "fonts.default")?,
        None => FontFamily::Inter,
    };
    let install = match node.prop("install") {
        Some(prop) => parse_install_fonts(prop)?,
        None => Vec::new(),
    };

    Ok(FontConfig {
        default_family,
        install,
    })
}

fn parse_install_fonts(prop: &SourceProp) -> DoweResult<Vec<FontFamily>> {
    let SourceValue::Array(values) = &prop.value else {
        return Err(prop_error(
            prop,
            "`fonts.install` must be an array of font tokens",
        ));
    };
    let mut seen = BTreeSet::new();
    let mut fonts = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let token = required_static_string_value(prop, value)?;
        let family = FontFamily::from_name(&token).ok_or_else(|| {
            prop_error(
                prop,
                format!("unknown font token `{token}` in `fonts.install[{index}]`"),
            )
        })?;
        if !seen.insert(family) {
            return Err(prop_error(
                prop,
                format!("duplicate font token `{token}` in `fonts.install`"),
            ));
        }
        fonts.push(family);
    }
    Ok(fonts)
}

fn parse_font_token(prop: &SourceProp, field: &str) -> DoweResult<FontFamily> {
    let token = required_static_string_prop(prop)?;
    FontFamily::from_name(&token)
        .ok_or_else(|| prop_error(prop, format!("unknown font token `{token}` in `{field}`")))
}

fn parse_design(node: &SourceNode) -> DoweResult<DesignConfig> {
    if !node.args.is_empty() {
        return Err(node_error(node, "`design` does not accept args"));
    }
    reject_unknown_props(node, &["defaultTheme"])?;
    let default_theme = match node.prop("defaultTheme") {
        Some(prop) => parse_theme_name_prop(prop, "design.defaultTheme")?,
        None => "light".to_string(),
    };

    let mut themes = HashMap::new();
    let mut configured_defaults = DesignDefaults::empty();
    for child in &node.children {
        match child.name.as_str() {
            "theme" => {
                let theme = parse_raw_theme(child)?;
                if themes.insert(theme.name.clone(), theme.clone()).is_some() {
                    return Err(node_error(
                        child,
                        format!("duplicate theme `{}`", theme.name),
                    ));
                }
            }
            "Card" | "Button" | "IconButton" | "Drawer" | "Toast" | "Section" | "Accordion"
            | "Checkbox" | "Input" | "Date" | "DateRange" | "Color" | "Textarea" | "Password"
            | "Select" | "Pin" | "AppBar" | "Footer" | "Modal" | "Dropdown" | "Tooltip"
            | "Tabs" | "Chip" | "SideNav" | "Sidebar" | "NavMenu" | "Avatar" | "Ui" | "Text"
            | "Title" => parse_component_defaults(child, &mut configured_defaults)?,
            _ => {
                return Err(node_error(
                    child,
                    format!("`{}` is not valid inside `design`", child.name),
                ));
            }
        }
    }

    if !themes.contains_key(&default_theme) {
        return Err(node_error(
            node,
            format!("default theme `{default_theme}` is not declared"),
        ));
    }

    let mut resolving = HashSet::new();
    let mut resolved = HashMap::new();
    let mut names = themes.keys().cloned().collect::<Vec<_>>();
    names.sort();
    let mut output = Vec::new();
    for name in names {
        output.push(resolve_theme(
            &name,
            &themes,
            &mut resolving,
            &mut resolved,
        )?);
    }

    let default_custom_colors = output
        .iter()
        .find(|theme| theme.name == default_theme)
        .map(|theme| {
            theme
                .colors
                .iter()
                .filter(|(token, _)| !token.is_builtin())
                .map(|(token, value)| (*token, value.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for theme in &mut output {
        if theme.name == default_theme {
            continue;
        }
        for (token, value) in &default_custom_colors {
            theme.colors.entry(*token).or_insert_with(|| value.clone());
        }
    }

    let resolved_default = output
        .iter()
        .find(|theme| theme.name == default_theme)
        .expect("resolved default theme");
    let defaults = DesignDefaults::with_builtin_overrides(configured_defaults);
    validate_design_custom_color_defaults(node, &defaults, resolved_default)?;

    Ok(DesignConfig {
        default_theme,
        themes: output,
        defaults,
    })
}

fn validate_design_custom_color_defaults(
    node: &SourceNode,
    defaults: &DesignDefaults,
    theme: &DesignTheme,
) -> DoweResult<()> {
    for child in &node.children {
        let Some(slot) = DesignComponentSlot::from_name(&child.name) else {
            continue;
        };
        for (prop_name, families) in [
            ("scheme", &defaults.scheme),
            ("borderColor", &defaults.border_color),
            ("shadowColor", &defaults.shadow_color),
        ] {
            let Some(prop) = child.prop(prop_name) else {
                continue;
            };
            let Some(family) = families.get(&slot).copied() else {
                continue;
            };
            if family.is_builtin() {
                continue;
            }
            validate_design_custom_color_default(prop, theme, family)?;
        }
    }
    Ok(())
}

fn validate_design_custom_color_default(
    prop: &SourceProp,
    theme: &DesignTheme,
    family: ColorFamily,
) -> DoweResult<()> {
    if theme.contains_color_family(family) {
        return Ok(());
    }
    let name = family.theme_name();
    Err(prop_error(
        prop,
        format!(
            "`{}` requires complete `{name}` color, text and title roles in default theme `{}`",
            prop.name, theme.name
        ),
    ))
}

fn parse_component_defaults(node: &SourceNode, defaults: &mut DesignDefaults) -> DoweResult<()> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(
            node,
            format!("`{}` only accepts static default props", node.name),
        ));
    }
    let slot = DesignComponentSlot::from_name(&node.name)
        .ok_or_else(|| node_error(node, format!("unknown theme component `{}`", node.name)))?;
    for prop in &node.props {
        if matches!(slot, DesignComponentSlot::Text | DesignComponentSlot::Title) {
            if prop.name != "font" {
                return Err(prop_error(
                    prop,
                    format!(
                        "`{}` is not a theme default prop for `{}`",
                        prop.name, node.name
                    ),
                ));
            }
            let value = parse_font_token(prop, &format!("{}.font", node.name))?;
            set_component_default(prop, &mut defaults.font, slot, value)?;
            continue;
        }
        match prop.name.as_str() {
            "radius" | "rounded" => {
                let value = required_design_string(prop, "radius")?;
                let value = RoundedSize::from_name(&value)
                    .ok_or_else(|| prop_error(prop, "radius must be xs, sm, md, lg, xl or full"))?;
                set_component_default(prop, &mut defaults.radius, slot, value)?;
            }
            "shadow" => {
                let value = required_design_string(prop, "shadow")?;
                let value = ShadowSize::from_name(&value)
                    .ok_or_else(|| prop_error(prop, "shadow must be xs, sm, md, lg or xl"))?;
                set_component_default(prop, &mut defaults.shadow, slot, value)?;
            }
            "shadowColor" => {
                let value = parse_component_family_default(prop, "shadowColor")?;
                set_component_default(prop, &mut defaults.shadow_color, slot, value)?;
            }
            "border" => {
                let value = required_design_string(prop, "border")?;
                let value = value
                    .parse::<u8>()
                    .ok()
                    .filter(|value| (1..=4).contains(value))
                    .map(BorderWidth)
                    .ok_or_else(|| prop_error(prop, "border must be an integer from 1 to 4"))?;
                set_component_default(prop, &mut defaults.border, slot, value)?;
            }
            "borderColor" => {
                let value = parse_component_family_default(prop, "borderColor")?;
                set_component_default(prop, &mut defaults.border_color, slot, value)?;
            }
            "scheme" => {
                let value = parse_component_family_default(prop, "scheme")?;
                set_component_default(prop, &mut defaults.scheme, slot, value)?;
            }
            "variant" => {
                let value = required_design_string(prop, "variant")?;
                if slot == DesignComponentSlot::Tabs {
                    let value = TabsVariant::from_name(&value).ok_or_else(|| {
                        prop_error(
                            prop,
                            "variant must be solid, outlined, line, ghost or pills",
                        )
                    })?;
                    set_component_default(prop, &mut defaults.tabs_variant, slot, value)?;
                } else {
                    let value = ComponentVariant::from_name(&value).ok_or_else(|| {
                        prop_error(
                            prop,
                            "variant must be solid, outline, outlined, line or ghost",
                        )
                    })?;
                    set_component_default(prop, &mut defaults.variant, slot, value)?;
                }
            }
            "size" => {
                let value = required_design_string(prop, "size")?;
                let value = ButtonSize::from_name(&value)
                    .ok_or_else(|| prop_error(prop, "size must be xs, sm, md, lg or xl"))?;
                set_component_default(prop, &mut defaults.size, slot, value)?;
            }
            "labelFloating"
                if matches!(
                    slot,
                    DesignComponentSlot::Input
                        | DesignComponentSlot::Password
                        | DesignComponentSlot::Select
                        | DesignComponentSlot::Date
                        | DesignComponentSlot::DateRange
                        | DesignComponentSlot::Color
                        | DesignComponentSlot::Textarea
                ) =>
            {
                let value = parse_boolean_prop(prop, "labelFloating")?;
                set_component_default(prop, &mut defaults.label_floating, slot, value)?;
            }
            _ => {
                return Err(prop_error(
                    prop,
                    format!(
                        "`{}` is not a theme default prop for `{}`",
                        prop.name, node.name
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn required_design_string(prop: &SourceProp, name: &str) -> DoweResult<String> {
    prop.value
        .as_string_like()
        .ok_or_else(|| prop_error(prop, format!("{name} must be a static value")))
}

fn parse_component_family_default(prop: &SourceProp, name: &str) -> DoweResult<ColorFamily> {
    let value = required_design_string(prop, name)?;
    ColorFamily::from_name(&value)
        .ok_or_else(|| prop_error(prop, format!("{name} uses an unknown scheme")))
}

fn set_component_default<T: Copy>(
    prop: &SourceProp,
    target: &mut BTreeMap<DesignComponentSlot, T>,
    slot: DesignComponentSlot,
    value: T,
) -> DoweResult<()> {
    if target.insert(slot, value).is_some() {
        return Err(prop_error(
            prop,
            format!("duplicate `{}` default for `{}`", prop.name, slot.as_str()),
        ));
    }
    Ok(())
}

