pub fn checkbox_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut checked = false;
    let mut disabled = false;
    let mut name = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "checked" => checked = parse_static_bool(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Checkbox)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Checkbox, &style_props)?;
    style.color.get_or_insert(ColorFamily::Primary);
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
    style.variant.get_or_insert(ComponentVariant::Outlined);
    style.color.get_or_insert(ColorFamily::Primary);
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
    let mut name = None;
    let mut info = None;
    let mut error = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
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
    Ok(ViewNode::ToggleTheme {
        props: ThemeToggleProps {
            style,
            light_label,
            dark_label,
        },
    })
}

pub fn fab_component_node(
    props: Vec<ComponentProp>,
    actions: Vec<FabAction>,
) -> ComponentResult<ViewNode> {
    let mut position = OverlayCornerPosition::BottomRight;
    let mut fixed = true;
    let mut offset_x = ScaleValue::from_half_steps(8);
    let mut offset_y = ScaleValue::from_half_steps(8);
    let mut icon = ViewIcon::Plus;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "position" => position = parse_overlay_corner_position(&prop.name, &prop.value)?,
            "fixed" => fixed = parse_static_bool(&prop.name, &prop.value)?,
            "offsetX" => offset_x = parse_static_scale(&prop.name, &prop.value)?,
            "offsetY" => offset_y = parse_static_scale(&prop.name, &prop.value)?,
            "icon" => icon = parse_view_icon(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Fab)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Fab, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Fab, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    let size = *style.size.get_or_insert(ButtonSize::Lg);
    apply_icon_button_size_defaults(&mut style.style, size);
    let label = style
        .label
        .clone()
        .unwrap_or_else(|| "Open actions".to_string());
    Ok(ViewNode::Fab {
        props: FabProps {
            style,
            position,
            fixed,
            offset_x,
            offset_y,
            icon,
            label,
        },
        actions,
    })
}

pub fn fab_action_component(props: Vec<ComponentProp>) -> ComponentResult<FabAction> {
    let mut label = None;
    let mut icon = ViewIcon::Plus;
    let mut color = ColorFamily::Muted;
    let mut href = None;
    let mut target = None;
    let mut navigate = None;
    let mut on_click = None;
    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "icon" => icon = parse_view_icon(&prop.name, &prop.value)?,
            "scheme" => {
                color = parse_family_prop(BuiltinComponent::FabAction, &prop.name, &prop.value)?
            }
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "target" => target = Some(parse_web_target(&prop.name, &prop.value)?),
            "navigate" => navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?),
            "onClick" => on_click = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::FabAction)),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::FabAction,
                    &prop.name,
                ));
            }
        }
    }
    let navigation = parse_navigation_props(
        BuiltinComponent::FabAction,
        href,
        navigate,
        None,
        target,
        None,
    )?;
    if navigation.is_none() && on_click.is_none() {
        return Err(ComponentError::invalid_prop_combination(
            "fabAction requires `href` or `onClick`",
        ));
    }
    Ok(FabAction {
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        icon,
        color,
        on_click,
        navigation,
    })
}

pub fn slider_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut value = "0".to_string();
    let mut min = "0".to_string();
    let mut max = "100".to_string();
    let mut step = None;
    let mut size = ButtonSize::Md;
    let mut name = None;
    let mut hide_label = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => value = parse_number_literal(&prop.name, &prop.value)?,
            "min" => min = parse_number_literal(&prop.name, &prop.value)?,
            "max" => max = parse_number_literal(&prop.name, &prop.value)?,
            "step" => step = Some(parse_positive_number_literal(&prop.name, &prop.value)?),
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "hideLabel" => hide_label = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Slider)),
            _ => style_props.push(prop),
        }
    }
    validate_slider_range(&min, &max, &value)?;
    let mut style = parse_variant_props(BuiltinComponent::Slider, &style_props)?;
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Slider {
        props: SliderProps {
            style,
            value,
            min,
            max,
            step,
            size,
            name,
            hide_label,
        },
    })
}

pub fn dropzone_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut accept = None;
    let mut multiple = true;
    let mut max_size = None;
    let mut size = ButtonSize::Md;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut disabled = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "accept" => accept = Some(parse_required_string(&prop.name, &prop.value)?),
            "multiple" => multiple = parse_static_bool(&prop.name, &prop.value)?,
            "maxSize" => max_size = Some(parse_positive_u64(&prop.name, &prop.value)?),
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Dropzone)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Dropzone, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    style
        .placeholder
        .get_or_insert_with(|| "Drag & drop files here or click to select".to_string());
    Ok(ViewNode::Dropzone {
        props: DropzoneProps {
            style,
            accept,
            multiple,
            max_size,
            size,
            name,
            help_text,
            error_text,
            disabled,
        },
    })
}

pub fn combo_box_component_node(
    props: Vec<ComponentProp>,
    options: Vec<ComboOption>,
) -> ComponentResult<ViewNode> {
    if options.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "ComboBox requires at least one comboOption child",
        ));
    }
    let mut seen = BTreeSet::new();
    for option in &options {
        if !seen.insert(option.value.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate ComboBox option value `{}`",
                option.value
            )));
        }
    }
    let mut value = None;
    let mut search_placeholder = "Search...".to_string();
    let mut empty_text = "No options found".to_string();
    let mut loading_text = "Loading...".to_string();
    let mut loading_more_text = "Loading more...".to_string();
    let mut clearable = false;
    let mut disabled = false;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(parse_static_string_or_number(&prop.name, &prop.value)?),
            "searchPlaceholder" => {
                search_placeholder = parse_required_string(&prop.name, &prop.value)?
            }
            "emptyText" => empty_text = parse_required_string(&prop.name, &prop.value)?,
            "loadingText" => loading_text = parse_required_string(&prop.name, &prop.value)?,
            "loadingMoreText" => {
                loading_more_text = parse_required_string(&prop.name, &prop.value)?
            }
            "clearable" => clearable = parse_static_bool(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::ComboBox)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::ComboBox, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Outlined);
    style.color.get_or_insert(ColorFamily::Primary);
    style.size = Some(size);
    style
        .placeholder
        .get_or_insert_with(|| "Select an option".to_string());
    Ok(ViewNode::ComboBox {
        props: ComboBoxProps {
            style,
            value,
            search_placeholder,
            empty_text,
            loading_text,
            loading_more_text,
            clearable,
            disabled,
            name,
            help_text,
            error_text,
        },
        options,
    })
}

pub fn combo_option_component(props: Vec<ComponentProp>) -> ComponentResult<ComboOption> {
    let mut value = None;
    let mut label = None;
    let mut description = None;
    let mut src = None;
    let mut icon = None;
    let mut disabled = false;
    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(parse_static_string_or_number(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "description" => description = Some(parse_required_string(&prop.name, &prop.value)?),
            "src" => src = Some(parse_required_string(&prop.name, &prop.value)?),
            "icon" => icon = Some(parse_view_icon(&prop.name, &prop.value)?),
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::ComboOption,
                    &prop.name,
                ));
            }
        }
    }
    let value = value.ok_or_else(|| ComponentError::invalid_prop("value", "static scalar"))?;
    if value.is_empty() {
        return Err(ComponentError::invalid_prop("value", "non-empty scalar"));
    }
    let label = label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?;
    Ok(ComboOption {
        value,
        label,
        description,
        src,
        icon,
        disabled,
    })
}

pub fn csv_field_component_node(
    props: Vec<ComponentProp>,
    columns: Vec<CsvColumn>,
) -> ComponentResult<ViewNode> {
    let mut button_text = "Select CSV file".to_string();
    let mut modal_title = "Map CSV Columns".to_string();
    let mut instructions =
        "Map each required column to the corresponding column from the CSV file:".to_string();
    let mut cancel_text = "Cancel".to_string();
    let mut confirm_text = "Confirm mapping".to_string();
    let mut clear_text = "Clear".to_string();
    let mut preview_title = "Imported Data".to_string();
    let mut multiple = false;
    let mut show_preview = true;
    let mut preview_rows = 5;
    let mut preview_page_size = 10;
    let mut error_text = None;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "buttonText" => button_text = parse_required_string(&prop.name, &prop.value)?,
            "modalTitle" => modal_title = parse_required_string(&prop.name, &prop.value)?,
            "instructions" => instructions = parse_required_string(&prop.name, &prop.value)?,
            "cancelText" => cancel_text = parse_required_string(&prop.name, &prop.value)?,
            "confirmText" => confirm_text = parse_required_string(&prop.name, &prop.value)?,
            "clearText" => clear_text = parse_required_string(&prop.name, &prop.value)?,
            "previewTitle" => preview_title = parse_required_string(&prop.name, &prop.value)?,
            "multiple" => multiple = parse_static_bool(&prop.name, &prop.value)?,
            "showPreview" => show_preview = parse_static_bool(&prop.name, &prop.value)?,
            "previewRows" => preview_rows = parse_u16_in_range(&prop.name, &prop.value, 1, 100)?,
            "previewPageSize" => {
                preview_page_size = parse_u16_in_range(&prop.name, &prop.value, 1, 500)?
            }
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => size = parse_button_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::CsvField)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::CsvField, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    style.size = Some(size);
    Ok(ViewNode::CsvField {
        props: CsvFieldProps {
            style,
            button_text,
            modal_title,
            instructions,
            cancel_text,
            confirm_text,
            clear_text,
            preview_title,
            multiple,
            show_preview,
            preview_rows,
            preview_page_size,
            error_text,
        },
        columns,
    })
}

pub fn csv_column_component(props: Vec<ComponentProp>) -> ComponentResult<CsvColumn> {
    let mut name = None;
    let mut label = None;
    for prop in props {
        match prop.name.as_str() {
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::CsvColumn,
                    &prop.name,
                ));
            }
        }
    }
    Ok(CsvColumn {
        name: name.ok_or_else(|| ComponentError::invalid_prop("name", "non-empty string"))?,
        label,
    })
}

pub fn drag_drop_component_node(
    props: Vec<ComponentProp>,
    items: Vec<DragItem>,
    groups: Vec<DragGroup>,
) -> ComponentResult<ViewNode> {
    if !items.is_empty() && !groups.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "DragDrop cannot mix dragItem and dragGroup children",
        ));
    }
    let mut seen = BTreeSet::new();
    for item in items
        .iter()
        .chain(groups.iter().flat_map(|group| group.items.iter()))
    {
        if !seen.insert(item.id.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate DragDrop item id `{}`",
                item.id
            )));
        }
    }
    let mut empty_text = "Drop items here".to_string();
    let mut direction = DragDropDirection::Vertical;
    let mut allow_group_transfer = true;
    let mut disabled = false;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "emptyText" => empty_text = parse_required_string(&prop.name, &prop.value)?,
            "direction" => direction = parse_drag_drop_direction(&prop.name, &prop.value)?,
            "allowGroupTransfer" => {
                allow_group_transfer = parse_static_bool(&prop.name, &prop.value)?
            }
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::DragDrop)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::DragDrop, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::DragDrop {
        props: DragDropProps {
            style,
            empty_text,
            direction,
            allow_group_transfer,
            disabled,
            size,
        },
        items,
        groups,
    })
}

pub fn drag_group_component(
    props: Vec<ComponentProp>,
    items: Vec<DragItem>,
) -> ComponentResult<DragGroup> {
    let mut id = None;
    let mut title = None;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_required_string(&prop.name, &prop.value)?),
            "title" => title = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::DragGroup,
                    &prop.name,
                ));
            }
        }
    }
    Ok(DragGroup {
        id: id.ok_or_else(|| ComponentError::invalid_prop("id", "non-empty string"))?,
        title,
        items,
    })
}

pub fn drag_item_component(props: Vec<ComponentProp>) -> ComponentResult<DragItem> {
    let mut id = None;
    let mut label = None;
    let mut description = None;
    let mut disabled = false;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_required_string(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "description" => description = Some(parse_required_string(&prop.name, &prop.value)?),
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::DragItem,
                    &prop.name,
                ));
            }
        }
    }
    Ok(DragItem {
        id: id.ok_or_else(|| ComponentError::invalid_prop("id", "non-empty string"))?,
        label,
        description,
        disabled,
    })
}

pub fn editor_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut value = None;
    let mut min_height = 200;
    let mut hide_toolbar = false;
    let mut disabled = false;
    let mut readonly = false;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(parse_required_string(&prop.name, &prop.value)?),
            "minHeight" => min_height = parse_u16_in_range(&prop.name, &prop.value, 80, 2000)?,
            "hideToolbar" => hide_toolbar = parse_static_bool(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "readonly" => readonly = parse_static_bool(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Editor)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Editor, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Outlined);
    style.color.get_or_insert(ColorFamily::Primary);
    style.size = Some(size);
    Ok(ViewNode::Editor {
        props: EditorProps {
            style,
            value,
            min_height,
            hide_toolbar,
            disabled,
            readonly,
            name,
            help_text,
            error_text,
        },
    })
}

pub fn image_cropper_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut src = None;
    let mut alt = "Avatar".to_string();
    let mut accept = "image/*".to_string();
    let mut aspect_ratio = None;
    let mut min_width = 50;
    let mut min_height = 50;
    let mut max_width = None;
    let mut max_height = None;
    let mut shape = ImageCropperShape::Circle;
    let mut disabled = false;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "src" => src = Some(parse_required_string(&prop.name, &prop.value)?),
            "alt" => alt = parse_required_string(&prop.name, &prop.value)?,
            "accept" => accept = parse_required_string(&prop.name, &prop.value)?,
            "aspectRatio" => {
                aspect_ratio = Some(parse_positive_number_literal(&prop.name, &prop.value)?)
            }
            "minWidth" => min_width = parse_u16_in_range(&prop.name, &prop.value, 1, 4000)?,
            "minHeight" => min_height = parse_u16_in_range(&prop.name, &prop.value, 1, 4000)?,
            "maxWidth" => max_width = Some(parse_u16_in_range(&prop.name, &prop.value, 1, 8000)?),
            "maxHeight" => {
                max_height = Some(parse_u16_in_range(&prop.name, &prop.value, 1, 8000)?)
            }
            "shape" => shape = parse_image_cropper_shape(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => size = parse_button_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::ImageCropper)),
            _ => style_props.push(prop),
        }
    }
    validate_dimensions(min_width, min_height, max_width, max_height)?;
    let mut style = parse_variant_props(BuiltinComponent::ImageCropper, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    style.size = Some(size);
    style
        .placeholder
        .get_or_insert_with(|| "Upload".to_string());
    Ok(ViewNode::ImageCropper {
        props: ImageCropperProps {
            style,
            src,
            alt,
            accept,
            aspect_ratio,
            min_width,
            min_height,
            max_width,
            max_height,
            shape,
            disabled,
            name,
            help_text,
            error_text,
        },
    })
}

pub fn password_field_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut value = None;
    let mut hide_strength = false;
    let mut weak_label = "Weak".to_string();
    let mut medium_label = "Medium".to_string();
    let mut strong_label = "Strong".to_string();
    let mut disabled = false;
    let mut readonly = false;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(parse_required_string(&prop.name, &prop.value)?),
            "hideStrength" => hide_strength = parse_static_bool(&prop.name, &prop.value)?,
            "weakLabel" => weak_label = parse_required_string(&prop.name, &prop.value)?,
            "mediumLabel" => medium_label = parse_required_string(&prop.name, &prop.value)?,
            "strongLabel" => strong_label = parse_required_string(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "readonly" => readonly = parse_static_bool(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::PasswordField)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::PasswordField, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Outlined);
    style.color.get_or_insert(ColorFamily::Primary);
    style.size = Some(size);
    Ok(ViewNode::PasswordField {
        props: PasswordFieldProps {
            style,
            value,
            hide_strength,
            weak_label,
            medium_label,
            strong_label,
            disabled,
            readonly,
            name,
            help_text,
            error_text,
        },
    })
}

pub fn phone_field_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut value = None;
    let mut country = None;
    let mut dial_code_name = "dialCode".to_string();
    let mut search_placeholder = "Search country...".to_string();
    let mut empty_text = "No countries found".to_string();
    let mut loading_text = "Loading...".to_string();
    let mut priority_countries = Vec::new();
    let mut disabled = false;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(parse_required_string(&prop.name, &prop.value)?),
            "country" => country = Some(parse_country_code(&prop.name, &prop.value)?),
            "dialCodeName" => dial_code_name = parse_required_string(&prop.name, &prop.value)?,
            "searchPlaceholder" => {
                search_placeholder = parse_required_string(&prop.name, &prop.value)?
            }
            "emptyText" => empty_text = parse_required_string(&prop.name, &prop.value)?,
            "loadingText" => loading_text = parse_required_string(&prop.name, &prop.value)?,
            "priorityCountries" => {
                priority_countries = parse_country_code_list(&prop.name, &prop.value)?
            }
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::PhoneField)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::PhoneField, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Outlined);
    style.color.get_or_insert(ColorFamily::Primary);
    style.size = Some(size);
    style
        .placeholder
        .get_or_insert_with(|| "Enter phone number".to_string());
    Ok(ViewNode::PhoneField {
        props: PhoneFieldProps {
            style,
            value,
            country,
            dial_code_name,
            search_placeholder,
            empty_text,
            loading_text,
            priority_countries,
            disabled,
            name,
            help_text,
            error_text,
        },
    })
}

pub fn pin_field_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut value = None;
    let mut length = 6;
    let mut kind = PinFieldKind::Text;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(parse_required_string(&prop.name, &prop.value)?),
            "length" => length = parse_u8_in_range(&prop.name, &prop.value, 1, 12)?,
            "type" => kind = parse_pin_field_kind(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::PinField)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::PinField, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Outlined);
    style.color.get_or_insert(ColorFamily::Primary);
    style.size = Some(size);
    Ok(ViewNode::PinField {
        props: PinFieldProps {
            style,
            value,
            length,
            kind,
            name,
            help_text,
            error_text,
        },
    })
}

pub fn textarea_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut value = None;
    let mut rows = 4;
    let mut cols = None;
    let mut max_length = None;
    let mut resize = false;
    let mut disabled = false;
    let mut readonly = false;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(parse_required_string(&prop.name, &prop.value)?),
            "rows" => rows = parse_u16_in_range(&prop.name, &prop.value, 1, 100)?,
            "cols" => cols = Some(parse_u16_in_range(&prop.name, &prop.value, 1, 300)?),
            "maxLength" => {
                max_length = Some(parse_u16_in_range(&prop.name, &prop.value, 1, 65535)?)
            }
            "resize" => resize = parse_static_bool(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "readonly" => readonly = parse_static_bool(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Textarea)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Textarea, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Outlined);
    style.color.get_or_insert(ColorFamily::Primary);
    style.size = Some(size);
    Ok(ViewNode::Textarea {
        props: TextareaProps {
            style,
            value,
            rows,
            cols,
            max_length,
            resize,
            disabled,
            readonly,
            name,
            help_text,
            error_text,
        },
    })
}

fn parse_drag_drop_direction(name: &str, value: &PropValue) -> ComponentResult<DragDropDirection> {
    let value = parse_required_string(name, value)?;
    DragDropDirection::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "horizontal or vertical"))
}

fn parse_image_cropper_shape(name: &str, value: &PropValue) -> ComponentResult<ImageCropperShape> {
    let value = parse_required_string(name, value)?;
    ImageCropperShape::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "circle or square"))
}

fn parse_pin_field_kind(name: &str, value: &PropValue) -> ComponentResult<PinFieldKind> {
    let value = parse_required_string(name, value)?;
    PinFieldKind::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "text, password or number"))
}

fn parse_u16_in_range(
    name: &str,
    value: &PropValue,
    min: u16,
    max: u16,
) -> ComponentResult<u16> {
    let value = parse_positive_u64(name, value)?;
    if (u64::from(min)..=u64::from(max)).contains(&value) {
        Ok(value as u16)
    } else {
        Err(ComponentError::invalid_prop_combination(format!(
            "`{name}` must be an integer from {min} to {max}"
        )))
    }
}

fn parse_u8_in_range(name: &str, value: &PropValue, min: u8, max: u8) -> ComponentResult<u8> {
    let value = parse_positive_u64(name, value)?;
    if (u64::from(min)..=u64::from(max)).contains(&value) {
        Ok(value as u8)
    } else {
        Err(ComponentError::invalid_prop_combination(format!(
            "`{name}` must be an integer from {min} to {max}"
        )))
    }
}

fn parse_country_code(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if valid_country_code(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "uppercase ISO alpha-2 code"))
    }
}

fn parse_country_code_list(name: &str, value: &PropValue) -> ComponentResult<Vec<String>> {
    let value = parse_required_string(name, value)?;
    let mut countries = Vec::new();
    for country in value.split(',').map(str::trim).filter(|value| !value.is_empty()) {
        if !valid_country_code(country) {
            return Err(ComponentError::invalid_prop(
                name,
                "comma-separated uppercase ISO alpha-2 codes",
            ));
        }
        countries.push(country.to_string());
    }
    Ok(countries)
}

fn valid_country_code(value: &str) -> bool {
    value.len() == 2 && value.chars().all(|value| value.is_ascii_uppercase())
}

fn validate_dimensions(
    min_width: u16,
    min_height: u16,
    max_width: Option<u16>,
    max_height: Option<u16>,
) -> ComponentResult<()> {
    if max_width.is_some_and(|max_width| max_width < min_width) {
        return Err(ComponentError::invalid_prop_combination(
            "`maxWidth` cannot be smaller than `minWidth`",
        ));
    }
    if max_height.is_some_and(|max_height| max_height < min_height) {
        return Err(ComponentError::invalid_prop_combination(
            "`maxHeight` cannot be smaller than `minHeight`",
        ));
    }
    Ok(())
}
