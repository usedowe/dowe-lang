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
    require_solid_variant(BuiltinComponent::Fab, style.variant)?;
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

