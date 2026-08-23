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

pub fn password_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
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
            "color" => return Err(scheme_prop_error(BuiltinComponent::Password)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Password, &style_props)?;
    style.size = Some(size);
    Ok(ViewNode::Password {
        props: PasswordProps {
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

pub fn phone_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
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
            "color" => return Err(scheme_prop_error(BuiltinComponent::Phone)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Phone, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Outlined);
    style.color.get_or_insert(ColorFamily::Primary);
    style.size = Some(size);
    style
        .placeholder
        .get_or_insert_with(|| "Enter phone number".to_string());
    Ok(ViewNode::Phone {
        props: PhoneProps {
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

pub fn pin_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut value = None;
    let mut length = 6;
    let mut kind = PinKind::Text;
    let mut name = None;
    let mut help_text = None;
    let mut error_text = None;
    let mut size = ButtonSize::Md;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(parse_required_string(&prop.name, &prop.value)?),
            "length" => length = parse_u8_in_range(&prop.name, &prop.value, 1, 12)?,
            "type" => kind = parse_pin_kind(&prop.name, &prop.value)?,
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "helpText" => help_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "errorText" => error_text = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Pin)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Pin, &style_props)?;
    style.size = Some(size);
    Ok(ViewNode::Pin {
        props: PinProps {
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

