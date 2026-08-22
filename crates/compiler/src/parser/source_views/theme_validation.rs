fn validate_view_theme_references(node: &SourceNode, design: &DesignConfig) -> DoweResult<()> {
    if let Some(prop) = node.prop("scheme")
        && let SourceValue::String(value) = &prop.value
        && let Some(family) = ColorFamily::from_name(value)
        && !family.is_builtin()
    {
        validate_custom_view_family(prop, design, family, false)?;
        if effective_source_variant_is_soft(node, design) {
            validate_custom_view_family(prop, design, family, true)?;
        }
    }
    for prop in &node.props {
        if matches!(prop.name.as_str(), "bg" | "color" | "fill" | "stroke") {
            validate_custom_view_color_value(prop, &prop.value, design)?;
        }
        if matches!(prop.name.as_str(), "borderColor" | "shadowColor")
            && let SourceValue::String(value) = &prop.value
            && let Some(family) = ColorFamily::from_name(value)
            && !family.is_builtin()
        {
            validate_custom_view_family(prop, design, family, false)?;
        }
    }
    for child in &node.children {
        validate_view_theme_references(child, design)?;
    }
    Ok(())
}

fn validate_custom_view_color_value(
    prop: &SourceProp,
    value: &SourceValue,
    design: &DesignConfig,
) -> DoweResult<()> {
    match value {
        SourceValue::String(value) if !matches!(value.as_str(), "currentColor" | "none") => {
            if let Some(token) = ColorToken::from_name(value)
                && !token.is_builtin()
                && !design.default_theme().contains_color_token(token)
            {
                return Err(prop_error(
                    prop,
                    format!(
                        "invalid value for prop `{}`: color token `{value}` is not declared in default theme `{}`",
                        prop.name,
                        design.default_theme
                    ),
                ));
            }
        }
        SourceValue::Array(values) => {
            for value in values {
                validate_custom_view_color_value(prop, value, design)?;
            }
        }
        SourceValue::Object(entries) => {
            for entry in entries {
                if let SourceObjectEntry::KeyValue { value, .. } = entry {
                    validate_custom_view_color_value(prop, value, design)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_custom_view_family(
    prop: &SourceProp,
    design: &DesignConfig,
    family: ColorFamily,
    _soft: bool,
) -> DoweResult<()> {
    if design.default_theme().contains_color_family(family, false) {
        return Ok(());
    }
    let name = family.theme_name(false);
    Err(prop_error(
        prop,
        format!(
            "scheme `{}` requires complete `{name}` color, text and title roles in default theme `{}`",
            family.as_str(),
            design.default_theme
        ),
    ))
}

fn effective_source_variant_is_soft(node: &SourceNode, design: &DesignConfig) -> bool {
    if node.name == "Tabs"
        && let Some(prop) = node.prop("variant")
        && let SourceValue::String(value) = &prop.value
    {
        return matches!(value.as_str(), "pills");
    }
    if let Some(prop) = node.prop("variant")
        && let SourceValue::String(value) = &prop.value
        && let Some(variant) = ComponentVariant::from_name(value)
    {
        return variant == ComponentVariant::Soft;
    }
    let slot = match node.name.as_str() {
        "Card" => DesignComponentSlot::Card,
        "Button" => DesignComponentSlot::Button,
        "IconButton" => DesignComponentSlot::IconButton,
        "Drawer" => DesignComponentSlot::Drawer,
        "Toast" => DesignComponentSlot::Toast,
        "Section" => DesignComponentSlot::Section,
        "Accordion" => DesignComponentSlot::Accordion,
        "Checkbox" => DesignComponentSlot::Checkbox,
        "Input" => DesignComponentSlot::Input,
        "Date" => DesignComponentSlot::Date,
        "DateRange" => DesignComponentSlot::DateRange,
        "Color" => DesignComponentSlot::Color,
        "Textarea" => DesignComponentSlot::Textarea,
        "Password" => DesignComponentSlot::Password,
        "Select" => DesignComponentSlot::Select,
        "Pin" => DesignComponentSlot::Pin,
        "AppBar" => DesignComponentSlot::AppBar,
        "Footer" => DesignComponentSlot::Footer,
        "Modal" => DesignComponentSlot::Modal,
        "Dropdown" => DesignComponentSlot::Dropdown,
        "Tooltip" => DesignComponentSlot::Tooltip,
        "Tabs" => DesignComponentSlot::Tabs,
        "Chip" => DesignComponentSlot::Chip,
        "Avatar" | "AvatarGroup" => DesignComponentSlot::Avatar,
        "Text" => DesignComponentSlot::Text,
        "Title" => DesignComponentSlot::Title,
        _ => DesignComponentSlot::Ui,
    };
    if slot == DesignComponentSlot::Tabs {
        return design
            .defaults
            .tabs_variant
            .get(&slot)
            .copied()
            .is_some_and(|variant| matches!(variant, TabsVariant::Pills));
    }
    design
        .defaults
        .variant
        .get(&slot)
        .copied()
        .is_some_and(|variant| variant == ComponentVariant::Soft)
}
