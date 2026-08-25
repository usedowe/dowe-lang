pub fn checkbox_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut checked = false;
    let mut disabled = false;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "checked" => checked = parse_static_bool(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Checkbox)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Checkbox, &style_props)?;
    if help_text.is_some() || error_text.is_some() {
        let validation = style.element.form_validation_mut();
        validation.help_text = help_text;
        validation.error_text = error_text;
    }
    Ok(ViewNode::Checkbox {
        props: CheckboxProps {
            style,
            checked,
            disabled,
            name,
        },
    })
}

pub fn color_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut value = "#3b82f6".to_string();
    let mut size = ButtonSize::Md;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut show_hex = false;
    let mut show_rgb = false;
    let mut show_cmyk = false;
    let mut show_oklch = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => value = parse_hex_color_prop(&prop.name, &prop.value)?,
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "showHex" => show_hex = parse_static_bool(&prop.name, &prop.value)?,
            "showRgb" => show_rgb = parse_static_bool(&prop.name, &prop.value)?,
            "showCmyk" => show_cmyk = parse_static_bool(&prop.name, &prop.value)?,
            "showOklch" => show_oklch = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Color)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Color, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Outlined);
    style.color.get_or_insert(ColorFamily::Primary);
    style
        .placeholder
        .get_or_insert_with(|| "Select color".to_string());
    Ok(ViewNode::Color {
        props: ColorProps {
            style,
            value,
            size,
            name,
            help_text,
            error_text,
            show_hex,
            show_rgb,
            show_cmyk,
            show_oklch,
        },
    })
}

pub fn date_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut value = None;
    let mut size = ButtonSize::Md;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut min = None;
    let mut max = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(parse_date_literal(&prop.name, &prop.value)?),
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "min" => min = Some(parse_date_literal(&prop.name, &prop.value)?),
            "max" => max = Some(parse_date_literal(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Date)),
            _ => style_props.push(prop),
        }
    }
    validate_date_bounds(min.as_deref(), max.as_deref())?;
    let mut style = parse_variant_props(BuiltinComponent::Date, &style_props)?;
    style
        .placeholder
        .get_or_insert_with(|| "Select date".to_string());
    Ok(ViewNode::Date {
        props: DateProps {
            style,
            value,
            size,
            name,
            help_text,
            error_text,
            min,
            max,
        },
    })
}

pub fn date_range_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut start = None;
    let mut end = None;
    let mut start_value = None;
    let mut end_value = None;
    let mut size = ButtonSize::Md;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut min = None;
    let mut max = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "start" => {
                start = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal string path",
                )?)
            }
            "end" => {
                end = Some(parse_signal_path(
                    &prop.name,
                    &prop.value,
                    "signal string path",
                )?)
            }
            "startValue" => start_value = Some(parse_date_literal(&prop.name, &prop.value)?),
            "endValue" => end_value = Some(parse_date_literal(&prop.name, &prop.value)?),
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "min" => min = Some(parse_date_literal(&prop.name, &prop.value)?),
            "max" => max = Some(parse_date_literal(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::DateRange)),
            _ => style_props.push(prop),
        }
    }
    validate_date_bounds(min.as_deref(), max.as_deref())?;
    let mut style = parse_variant_props(BuiltinComponent::DateRange, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Outlined);
    style.color.get_or_insert(ColorFamily::Primary);
    style
        .placeholder
        .get_or_insert_with(|| "Select date range".to_string());
    Ok(ViewNode::DateRange {
        props: DateRangeProps {
            style,
            start,
            end,
            start_value,
            end_value,
            size,
            name,
            help_text,
            error_text,
            min,
            max,
        },
    })
}

pub fn radio_group_component_node(
    props: Vec<ComponentProp>,
    options: Vec<RadioOption>,
) -> ComponentResult<ViewNode> {
    if options.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "RadioGroup requires at least one item",
        ));
    }
    let mut seen = BTreeSet::new();
    for option in &options {
        if !seen.insert(option.value.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate RadioGroup item value `{}`",
                option.value
            )));
        }
    }
    let mut size = ButtonSize::Md;
    let mut orientation = RadioGroupOrientation::Vertical;
    let mut name = None;
    let mut info = None;
    let mut error = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "orientation" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                orientation = RadioGroupOrientation::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("orientation", "vertical or horizontal")
                })?;
            }
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "info" => info = Some(parse_required_string(&prop.name, &prop.value)?),
            "error" => error = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::RadioGroup)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::RadioGroup, &style_props)?;
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::RadioGroup {
        props: RadioGroupProps {
            style,
            size,
            orientation,
            name,
            info,
            error,
        },
        options,
    })
}

pub fn radio_option_component(props: Vec<ComponentProp>) -> ComponentResult<RadioOption> {
    let mut value = None;
    let mut label = None;
    let mut disabled = false;
    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(parse_static_string_or_number(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::RadioGroup,
                    &prop.name,
                ));
            }
        }
    }
    Ok(RadioOption {
        value: value
            .ok_or_else(|| ComponentError::invalid_prop("value", "static string or number"))?,
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        disabled,
    })
}

pub fn toggle_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut checked = false;
    let mut disabled = false;
    let mut name = None;
    let mut label_left = None;
    let mut label_right = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "checked" => checked = parse_static_bool(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "labelLeft" => label_left = Some(parse_required_string(&prop.name, &prop.value)?),
            "labelRight" => label_right = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Toggle)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Toggle, &style_props)?;
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Toggle {
        props: ToggleProps {
            style,
            checked,
            disabled,
            name,
            label_left,
            label_right,
        },
    })
}

pub fn theme_toggle_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut light_label = "Switch to light mode".to_string();
    let mut dark_label = "Switch to dark mode".to_string();
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "lightLabel" => light_label = parse_required_string(&prop.name, &prop.value)?,
            "darkLabel" => dark_label = parse_required_string(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::ToggleTheme)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::ToggleTheme, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    let size = *style.size.get_or_insert(ButtonSize::Md);
    apply_icon_button_size_defaults(&mut style.style, size);
    let icon_size = ResponsiveValue::scalar(SizeValue::Scale(size.icon_button_icon_size()));
    let mut light_icon = solar_control_icon("sun")?;
    light_icon.props.style.sizing.w = Some(icon_size.clone());
    light_icon.props.style.sizing.h = Some(icon_size.clone());
    let mut dark_icon = solar_control_icon("moon")?;
    dark_icon.props.style.sizing.w = Some(icon_size.clone());
    dark_icon.props.style.sizing.h = Some(icon_size);
    Ok(ViewNode::ToggleTheme {
        props: ThemeToggleProps {
            style,
            light_label,
            dark_label,
            light_icon,
            dark_icon,
        },
    })
}

pub fn theme_select_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut label = "Theme".to_string();
    let mut placeholder = "Choose a theme".to_string();
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "label" => label = parse_required_string(&prop.name, &prop.value)?,
            "placeholder" => placeholder = parse_required_string(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::SelectTheme)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::SelectTheme, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Outlined);
    style.color.get_or_insert(ColorFamily::Surface);
    Ok(ViewNode::SelectTheme {
        props: ThemeSelectProps {
            style,
            label,
            placeholder,
            themes: Vec::new(),
            default_theme: "light".to_string(),
        },
    })
}

