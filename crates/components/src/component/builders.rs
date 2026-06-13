pub fn box_node(children: Vec<ViewNode>) -> ComponentResult<ViewNode> {
    container_node(
        BuiltinComponent::Box,
        Vec::new(),
        children,
        true,
        StyleProps::default(),
    )
}

pub fn text_node(value: impl AsRef<str>) -> ComponentResult<ViewNode> {
    let value = static_text(value, BuiltinComponent::Text)?;
    Ok(ViewNode::Text {
        props: TextProps::default(),
        value,
    })
}

pub fn text_component_node(
    component: BuiltinComponent,
    props: Vec<ComponentProp>,
    value: impl AsRef<str>,
) -> ComponentResult<ViewNode> {
    let value = static_text(value, component)?;
    let text_props = parse_text_props(component, &props)?;
    match component {
        BuiltinComponent::Title => Ok(ViewNode::Title {
            props: text_props,
            value,
        }),
        BuiltinComponent::Text => Ok(ViewNode::Text {
            props: text_props,
            value,
        }),
        _ => Err(ComponentError::invalid_prop("component", "text component")),
    }
}

pub fn input_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let props = parse_variant_props(BuiltinComponent::Input, &props)?;
    Ok(ViewNode::Input { props })
}

pub fn select_node(
    props: Vec<ComponentProp>,
    options: Vec<SelectOption>,
) -> ComponentResult<ViewNode> {
    if options.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Select requires at least one Option child",
        ));
    }
    let mut seen = BTreeSet::new();
    for option in &options {
        if !seen.insert(option.value.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate Select option value `{}`",
                option.value
            )));
        }
    }
    let props = parse_variant_props(BuiltinComponent::Select, &props)?;
    Ok(ViewNode::Select { props, options })
}

pub fn code_node(props: Vec<ComponentProp>, lines: Vec<String>) -> ComponentResult<ViewNode> {
    if lines.is_empty() {
        return Err(ComponentError::invalid_prop(
            "lines",
            "non-empty string array",
        ));
    }
    let mut language = CodeLanguage::Dowe;
    let mut copy_label = "Copy".to_string();
    let mut copied_label = "Copied".to_string();
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "language" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                language = CodeLanguage::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("language", "dowe, typescript, go or rust")
                })?;
            }
            "copyLabel" => copy_label = parse_required_string(&prop.name, &prop.value)?,
            "copiedLabel" => copied_label = parse_required_string(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Code, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Soft);
    style.color.get_or_insert(ColorFamily::Surface);
    let source = lines.join("\n");
    let tokens = highlight_code(language, &source);
    Ok(ViewNode::Code {
        props: CodeProps {
            style,
            language,
            source,
            tokens,
            copy_label,
            copied_label,
        },
    })
}

pub fn video_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut src = None;
    let mut poster = None;
    let mut autoplay = false;
    let mut aspect = VideoAspect::Horizontal;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "src" => src = Some(parse_video_src(&prop.name, &prop.value)?),
            "poster" => poster = Some(parse_video_poster(&prop.name, &prop.value)?),
            "autoplay" => autoplay = parse_static_bool(&prop.name, &prop.value)?,
            "aspect" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                aspect = VideoAspect::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("aspect", "horizontal, vertical or square")
                })?;
            }
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Video, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    Ok(ViewNode::Video {
        props: VideoProps {
            style,
            src: src.ok_or_else(|| ComponentError::invalid_prop("src", "https URL"))?,
            poster,
            autoplay,
            aspect,
        },
    })
}

pub fn audio_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut src = None;
    let mut subtitle = None;
    let mut avatar_src = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "src" => src = Some(parse_media_source(&prop.name, &prop.value)?),
            "subtitle" => subtitle = Some(parse_required_string(&prop.name, &prop.value)?),
            "avatarSrc" => avatar_src = Some(parse_media_source(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Audio)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Audio, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Audio, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Audio {
        props: AudioProps {
            style,
            src: src.ok_or_else(|| {
                ComponentError::invalid_prop("src", "asset path or https URL")
            })?,
            subtitle,
            avatar_src,
        },
    })
}

pub fn image_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut src = None;
    let mut alt = String::new();
    let mut aspect = ImageAspect::Auto;
    let mut object_fit = ImageObjectFit::Cover;
    let mut loading = ImageLoading::Lazy;
    let mut hide_controls = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "src" => src = Some(parse_media_source(&prop.name, &prop.value)?),
            "alt" => alt = parse_static_string(&prop.name, &prop.value)?,
            "aspect" => aspect = parse_image_aspect(&prop.name, &prop.value)?,
            "objectFit" => object_fit = parse_image_object_fit(&prop.name, &prop.value)?,
            "loading" => loading = parse_image_loading(&prop.name, &prop.value)?,
            "hideControls" => hide_controls = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Image)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Image, &style_props)?;
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Image {
        props: ImageProps {
            style,
            src: src.ok_or_else(|| {
                ComponentError::invalid_prop("src", "asset path or https URL")
            })?,
            alt,
            aspect,
            object_fit,
            loading,
            hide_controls,
        },
    })
}

pub fn accordion_component_node(
    props: Vec<ComponentProp>,
    items: Vec<AccordionItem>,
) -> ComponentResult<ViewNode> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Accordion requires at least one item",
        ));
    }
    let mut seen = BTreeSet::new();
    for item in &items {
        if !seen.insert(item.id.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate Accordion item id `{}`",
                item.id
            )));
        }
    }
    let mut multiple = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "multiple" => multiple = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Accordion)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Accordion, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Background);
    Ok(ViewNode::Accordion {
        props: AccordionProps { style, multiple },
        items,
    })
}

pub fn accordion_item_component(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
) -> ComponentResult<AccordionItem> {
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Accordion item requires body children",
        ));
    }
    let mut id = None;
    let mut label = None;
    let mut disabled = false;
    let mut default_open = false;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_static_string_or_number(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "defaultOpen" => default_open = parse_static_bool(&prop.name, &prop.value)?,
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::Accordion, &prop.name)),
        }
    }
    Ok(AccordionItem {
        id: id.ok_or_else(|| ComponentError::invalid_prop("id", "static string or number"))?,
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        disabled,
        default_open,
        children,
    })
}

pub fn carousel_component_node(
    props: Vec<ComponentProp>,
    slides: Vec<CarouselSlide>,
) -> ComponentResult<ViewNode> {
    if slides.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Carousel requires at least one slide",
        ));
    }
    let mut seen = BTreeSet::new();
    for slide in &slides {
        if !seen.insert(slide.id.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate Carousel slide id `{}`",
                slide.id
            )));
        }
    }
    let mut autoplay = false;
    let mut autoplay_interval = 3000;
    let mut disable_loop = false;
    let mut hide_controls = false;
    let mut hide_indicators = false;
    let mut show_navigation = false;
    let mut show_counter = false;
    let mut orientation = CarouselOrientation::Horizontal;
    let mut size = ButtonSize::Md;
    let mut indicator_type = CarouselIndicatorType::Bar;
    let mut title = None;
    let mut slide_width = None;
    let mut slide_height = None;
    let mut slides_per_view = 1;
    let mut gap = 0;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "autoplay" => autoplay = parse_static_bool(&prop.name, &prop.value)?,
            "autoplayInterval" => autoplay_interval = parse_positive_u16(&prop.name, &prop.value)?,
            "disableLoop" => disable_loop = parse_static_bool(&prop.name, &prop.value)?,
            "hideControls" => hide_controls = parse_static_bool(&prop.name, &prop.value)?,
            "hideIndicators" => hide_indicators = parse_static_bool(&prop.name, &prop.value)?,
            "showNavigation" => show_navigation = parse_static_bool(&prop.name, &prop.value)?,
            "showCounter" => show_counter = parse_static_bool(&prop.name, &prop.value)?,
            "orientation" => orientation = parse_carousel_orientation(&prop.name, &prop.value)?,
            "size" => size = parse_control_size_prop(&prop.name, &prop.value)?,
            "indicatorType" => indicator_type = parse_carousel_indicator(&prop.name, &prop.value)?,
            "title" => title = Some(parse_static_string(&prop.name, &prop.value)?),
            "slideWidth" => slide_width = Some(parse_positive_u16(&prop.name, &prop.value)?),
            "slideHeight" => slide_height = Some(parse_positive_u16(&prop.name, &prop.value)?),
            "slidesPerView" => slides_per_view = parse_positive_u16(&prop.name, &prop.value)?,
            "gap" => gap = parse_non_negative_u16(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Carousel)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Carousel, &style_props)?;
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Carousel {
        props: CarouselProps {
            style,
            autoplay,
            autoplay_interval,
            disable_loop,
            hide_controls,
            hide_indicators,
            show_navigation,
            show_counter,
            orientation,
            size,
            indicator_type,
            title,
            slide_width,
            slide_height,
            slides_per_view,
            gap,
        },
        slides,
    })
}

pub fn carousel_slide_component(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
) -> ComponentResult<CarouselSlide> {
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Carousel slide requires children",
        ));
    }
    let mut id = None;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_static_string_or_number(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::Carousel, &prop.name)),
        }
    }
    Ok(CarouselSlide {
        id: id.ok_or_else(|| ComponentError::invalid_prop("id", "static string or number"))?,
        children,
    })
}

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
    style.placeholder
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
    style.placeholder
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
            "start" => start = Some(parse_signal_path(&prop.name, &prop.value, "signal string path")?),
            "end" => end = Some(parse_signal_path(&prop.name, &prop.value, "signal string path")?),
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
    style.placeholder
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
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::RadioGroup, &prop.name)),
        }
    }
    Ok(RadioOption {
        value: value.ok_or_else(|| ComponentError::invalid_prop("value", "static string or number"))?,
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
            "scheme" => color = parse_family_prop(BuiltinComponent::FabAction, &prop.name, &prop.value)?,
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "target" => target = Some(parse_web_target(&prop.name, &prop.value)?),
            "navigate" => navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?),
            "onClick" => on_click = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::FabAction)),
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::FabAction, &prop.name)),
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

pub fn candlestick_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut data = None;
    let mut stream = None;
    let mut up_color = ColorToken::Success;
    let mut down_color = ColorToken::Danger;
    let mut empty_label = "No candle data".to_string();
    let mut max_points = 240;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "data" => data = Some(parse_reference_path(&prop.name, &prop.value)?),
            "stream" => stream = Some(parse_candlestick_stream(&prop.name, &prop.value)?),
            "upColor" => up_color = parse_candlestick_color(&prop.name, &prop.value)?,
            "downColor" => down_color = parse_candlestick_color(&prop.name, &prop.value)?,
            "emptyLabel" => empty_label = parse_required_string(&prop.name, &prop.value)?,
            "maxPoints" => max_points = parse_positive_u16(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Candlestick, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    if style.style.sizing.h.is_none() {
        style.style.sizing.h = Some(ResponsiveValue::scalar(SizeValue::Scale(
            ScaleValue::from_half_steps(128),
        )));
    }
    Ok(ViewNode::Candlestick {
        props: CandlestickProps {
            style,
            data: data.ok_or_else(|| ComponentError::invalid_prop("data", "signal array path"))?,
            stream,
            up_color,
            down_color,
            empty_label,
            max_points,
        },
    })
}

pub fn table_node(
    props: Vec<ComponentProp>,
    columns: Vec<TableColumn>,
) -> ComponentResult<ViewNode> {
    if columns.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Table requires at least one column",
        ));
    }
    let mut data = None;
    let mut size = TableSize::Md;
    let mut striped = false;
    let mut bordered = false;
    let mut dividers = true;
    let mut empty_title = "No data".to_string();
    let mut empty_description = "There are no records to display".to_string();
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "data" => data = Some(parse_reference_path(&prop.name, &prop.value)?),
            "size" => size = parse_table_size_prop(&prop.name, &prop.value)?,
            "striped" => striped = parse_static_bool(&prop.name, &prop.value)?,
            "bordered" => bordered = parse_static_bool(&prop.name, &prop.value)?,
            "dividers" => dividers = parse_static_bool(&prop.name, &prop.value)?,
            "emptyTitle" => empty_title = parse_required_string(&prop.name, &prop.value)?,
            "emptyDescription" => {
                empty_description = parse_required_string(&prop.name, &prop.value)?
            }
            "color" => {
                return Err(ComponentError::new(
                    "unknown prop `color` on `Table`; use `scheme` for visual family",
                ));
            }
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Table, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    Ok(ViewNode::Table {
        props: TableProps {
            style,
            data: data.ok_or_else(|| ComponentError::invalid_prop("data", "signal array path"))?,
            columns,
            size,
            striped,
            bordered,
            dividers,
            empty_title,
            empty_description,
        },
    })
}

pub fn table_column_component(props: Vec<ComponentProp>) -> ComponentResult<TableColumn> {
    let mut field = None;
    let mut label = None;
    let mut align = TableColumnAlign::Start;
    let mut width = None;
    for prop in props {
        match prop.name.as_str() {
            "field" => field = Some(parse_table_field(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "align" => align = parse_table_column_align_prop(&prop.name, &prop.value)?,
            "width" => width = Some(parse_table_column_width(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Table,
                    &prop.name,
                ));
            }
        }
    }
    Ok(TableColumn {
        field: field.ok_or_else(|| ComponentError::invalid_prop("field", "relative field path"))?,
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        align,
        width,
    })
}

pub fn tabs_component_node(
    props: Vec<ComponentProp>,
    tabs: Vec<TabItem>,
) -> ComponentResult<ViewNode> {
    if tabs.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Tabs requires at least one tab",
        ));
    }
    let mut seen = BTreeSet::new();
    for tab in &tabs {
        if !seen.insert(tab.id.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate Tabs tab id `{}`",
                tab.id
            )));
        }
        if tab.children.is_empty() {
            return Err(ComponentError::invalid_prop_combination(format!(
                "Tabs tab `{}` requires at least one child",
                tab.id
            )));
        }
    }
    let props = parse_tabs_props(BuiltinComponent::Tabs, &props)?;
    Ok(ViewNode::Tabs { props, tabs })
}

pub fn tabs_tab_component(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
) -> ComponentResult<TabItem> {
    let mut id = None;
    let mut label = None;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_id_prop(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Tab,
                    &prop.name,
                ));
            }
        }
    }
    let id = id.ok_or_else(|| ComponentError::invalid_prop("id", "portable tab id"))?;
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "Tabs tab `{id}` requires at least one child"
        )));
    }
    Ok(TabItem {
        id,
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        children,
    })
}

pub fn divider_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut orientation = DividerOrientation::Horizontal;
    let mut color = ColorFamily::Muted;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "orientation" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                orientation = DividerOrientation::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("orientation", "horizontal or vertical")
                })?;
            }
            "scheme" => {
                color = parse_family_prop(BuiltinComponent::Divider, &prop.name, &prop.value)?
            }
            _ => style_props.push(prop),
        }
    }
    let style = parse_style_props(
        BuiltinComponent::Divider,
        &style_props,
        StylePropMode::Variant,
    )?;
    Ok(ViewNode::Divider {
        props: DividerProps {
            style,
            orientation,
            color,
        },
    })
}

fn parse_video_src(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if value.starts_with("https://") && parse_cover_source(&value).is_some() {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "https URL"))
    }
}

fn parse_video_poster(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    parse_cover_source(&value)
        .map(|source| source.0)
        .ok_or_else(|| ComponentError::invalid_prop(name, "asset path or https URL"))
}

fn parse_reference_path(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if is_reference_path(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "signal array path"))
    }
}

fn parse_candlestick_stream(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if valid_candlestick_stream(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(
            name,
            "absolute path or https URL",
        ))
    }
}

fn valid_candlestick_stream(value: &str) -> bool {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return false;
    }
    if value.starts_with("https://") {
        return parse_cover_source(value).is_some();
    }
    value.starts_with('/') && !value.starts_with("//") && !value.contains("://")
}

fn parse_candlestick_color(name: &str, value: &PropValue) -> ComponentResult<ColorToken> {
    let value = parse_required_string(name, value)?;
    ColorToken::from_name(&value).ok_or_else(|| ComponentError::invalid_prop(name, "color token"))
}

fn parse_table_field(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if is_reference_path(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "relative field path"))
    }
}

fn parse_table_column_width(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if valid_table_width(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "portable table width"))
    }
}

fn valid_table_width(value: &str) -> bool {
    matches!(value, "auto" | "min-content" | "max-content")
        || valid_positive_width_suffix(value, "px")
        || valid_positive_width_suffix(value, "rem")
        || valid_positive_width_suffix(value, "%")
        || valid_positive_width_suffix(value, "fr")
}

fn valid_positive_width_suffix(value: &str, suffix: &str) -> bool {
    let Some(number) = value.strip_suffix(suffix) else {
        return false;
    };
    !number.is_empty()
        && number
            .parse::<f32>()
            .ok()
            .filter(|value| *value > 0.0)
            .is_some()
}

fn parse_positive_u16(name: &str, value: &PropValue) -> ComponentResult<u16> {
    match value {
        PropValue::Number(value) => value
            .parse::<u16>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| ComponentError::invalid_prop(name, "positive integer")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "positive integer"))
        }
    }
}

pub fn select_option_component(props: Vec<ComponentProp>) -> ComponentResult<SelectOption> {
    let mut value = None;
    let mut label = None;
    let mut description = None;

    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(prop_value_string(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "description" => description = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Option,
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
    Ok(SelectOption {
        value,
        label,
        description,
    })
}

pub fn alert_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut kind = None;
    let mut message = None;
    let mut visible = None;
    let mut on_close = None;
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "type" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                kind = Some(AlertKind::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("type", "success, error, info or warning")
                })?);
            }
            "message" => message = Some(parse_required_string(&prop.name, &prop.value)?),
            "visible" => visible = Some(prop_value_string(&prop.name, &prop.value)?),
            "onClose" => on_close = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => style_props.push(prop),
        }
    }

    let kind = kind
        .ok_or_else(|| ComponentError::invalid_prop("type", "success, error, info or warning"))?;
    let message = message
        .ok_or_else(|| ComponentError::invalid_prop("message", "static string or signal path"))?;
    let mut style = parse_variant_props(BuiltinComponent::Alert, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Soft);
    style.color.get_or_insert(kind.color());

    Ok(ViewNode::Alert {
        props: AlertProps {
            style,
            kind,
            message,
            visible,
            on_close,
        },
    })
}

pub fn svg_component_node(
    props: Vec<ComponentProp>,
    paths: Vec<SvgPath>,
) -> ComponentResult<ViewNode> {
    let props = parse_svg_props(BuiltinComponent::Svg, &props)?;
    if paths.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Svg requires at least one Path child",
        ));
    }
    Ok(ViewNode::Svg { props, paths })
}

pub fn svg_path_component(props: Vec<ComponentProp>) -> ComponentResult<SvgPath> {
    parse_svg_path_props(BuiltinComponent::Path, &props)
}

pub fn bar_component_node(
    component: BuiltinComponent,
    props: Vec<ComponentProp>,
    start: Vec<ViewNode>,
    center: Vec<ViewNode>,
    end: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if start.is_empty() && center.is_empty() && end.is_empty() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "{} requires at least one region with content",
            component.as_str()
        )));
    }
    if !allow_children
        && (contains_children(&start) || contains_children(&center) || contains_children(&end))
    {
        return Err(ComponentError::children_outside_layout());
    }
    let props = parse_bar_props(component, &props)?;
    match component {
        BuiltinComponent::AppBar => Ok(ViewNode::AppBar {
            props,
            start,
            center,
            end,
        }),
        BuiltinComponent::Footer => Ok(ViewNode::Footer {
            props,
            start,
            center,
            end,
        }),
        BuiltinComponent::BottomBar => Ok(ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        }),
        _ => Err(ComponentError::invalid_prop("component", "bar component")),
    }
}

pub fn side_nav_component_node(
    props: Vec<ComponentProp>,
    items: Vec<SideNavItem>,
) -> ComponentResult<ViewNode> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "SideNav requires at least one entry",
        ));
    }
    let mut size = None;
    let mut wide = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "size" => {
                size = Some(parse_side_nav_size_prop(&prop.name, &prop.value)?);
            }
            "wide" => wide = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::SideNav, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Ghost);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::SideNav {
        props: SideNavProps {
            style,
            size: size.unwrap_or(SideNavSize::Md),
            wide,
        },
        items,
    })
}

pub fn sidebar_component_node(
    props: Vec<ComponentProp>,
    items: Vec<SideNavItem>,
) -> ComponentResult<ViewNode> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Sidebar requires at least one entry",
        ));
    }
    let mut size = None;
    let mut wide = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "size" => {
                size = Some(parse_side_nav_size_prop(&prop.name, &prop.value)?);
            }
            "wide" => wide = parse_static_bool(&prop.name, &prop.value)?,
            "color" => {
                return Err(ComponentError::new(
                    "unknown prop `color` on `Sidebar`; use `scheme` for visual family",
                ));
            }
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Sidebar, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Ghost);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::Sidebar {
        props: SideNavProps {
            style,
            size: size.unwrap_or(SideNavSize::Md),
            wide,
        },
        items,
    })
}

pub fn nav_menu_component_node(
    props: Vec<ComponentProp>,
    items: Vec<NavMenuItem>,
) -> ComponentResult<ViewNode> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "NavMenu requires at least one entry",
        ));
    }
    let mut size = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "size" => size = Some(parse_side_nav_size_prop(&prop.name, &prop.value)?),
            "color" => {
                return Err(ComponentError::new(
                    "unknown prop `color` on `NavMenu`; use `scheme` for visual family",
                ));
            }
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::NavMenu, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Ghost);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::NavMenu {
        props: NavMenuProps {
            style,
            size: size.unwrap_or(SideNavSize::Md),
        },
        items,
    })
}

pub fn scaffold_component_node(
    props: Vec<ComponentProp>,
    app_bar: Vec<ViewNode>,
    start: Vec<ViewNode>,
    main: Vec<ViewNode>,
    end: Vec<ViewNode>,
    bottom_bar: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if main.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Scaffold requires a main region with content",
        ));
    }
    if !allow_children
        && (contains_children(&app_bar)
            || contains_children(&start)
            || contains_children(&main)
            || contains_children(&end)
            || contains_children(&bottom_bar))
    {
        return Err(ComponentError::children_outside_layout());
    }
    let mut boxed = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "boxed" => boxed = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    Ok(ViewNode::Scaffold {
        props: ScaffoldProps {
            style: parse_style_props(BuiltinComponent::Scaffold, &style_props, StylePropMode::Variant)?,
            boxed,
        },
        app_bar,
        start,
        main,
        end,
        bottom_bar,
    })
}

pub fn drawer_component_node(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Drawer requires at least one child",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Drawer, &children, allow_children)?;
    let mut open = None;
    let mut position = DrawerPosition::Start;
    let mut disable_overlay_close = false;
    let mut hide_close_button = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "open" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                if !is_reference_path(&value) {
                    return Err(ComponentError::invalid_prop("open", "signal bool path"));
                }
                open = Some(value);
            }
            "position" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                position = DrawerPosition::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("position", "start, end, top or bottom")
                })?;
            }
            "disableOverlayClose" => {
                disable_overlay_close = parse_static_bool(&prop.name, &prop.value)?
            }
            "hideCloseButton" => hide_close_button = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Drawer, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    Ok(ViewNode::Drawer {
        props: DrawerProps {
            style,
            open: open.ok_or_else(|| ComponentError::invalid_prop("open", "signal bool path"))?,
            position,
            disable_overlay_close,
            hide_close_button,
        },
        children,
    })
}

pub fn avatar_component_node(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
) -> ComponentResult<ViewNode> {
    let mut src = None;
    let mut name = None;
    let mut alt = "Avatar".to_string();
    let mut size = ButtonSize::Md;
    let mut status = None;
    let mut bordered = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "src" => src = Some(parse_avatar_src(&prop.name, &prop.value)?),
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "alt" => alt = parse_required_string(&prop.name, &prop.value)?,
            "size" => size = parse_button_size_prop(&prop.name, &prop.value)?,
            "status" => status = Some(parse_avatar_status(&prop.name, &prop.value)?),
            "bordered" => bordered = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Avatar)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Avatar, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Avatar, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Avatar {
        props: AvatarProps {
            style,
            src,
            name,
            alt,
            size,
            status,
            bordered,
        },
        icon,
    })
}

pub fn badge_component_node(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Badge requires at least one child",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Badge, &children, allow_children)?;
    let mut text = None;
    let mut position = OverlayCornerPosition::TopRight;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "text" => text = Some(parse_required_string(&prop.name, &prop.value)?),
            "position" => position = parse_overlay_corner_position(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Badge)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Badge, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Badge, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Badge {
        props: BadgeProps {
            style,
            text: text.ok_or_else(|| ComponentError::invalid_prop("text", "non-empty string"))?,
            position,
        },
        children,
    })
}

pub fn chip_component_node(
    props: Vec<ComponentProp>,
    value: impl AsRef<str>,
    start: Option<SideNavIcon>,
    end: Option<SideNavIcon>,
) -> ComponentResult<ViewNode> {
    let value = static_text(value, BuiltinComponent::Chip)?;
    let mut on_close = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "onClose" => on_close = Some(parse_required_string(&prop.name, &prop.value)?),
            "size" => style_props.push(prop),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Chip)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Chip, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Soft);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::Chip {
        props: ChipProps { style, on_close },
        value,
        start,
        end,
    })
}

pub fn skeleton_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut variant = SkeletonVariant::Text;
    let mut animation = SkeletonAnimation::Wave;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "variant" => variant = parse_skeleton_variant(&prop.name, &prop.value)?,
            "animation" => animation = parse_skeleton_animation(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    Ok(ViewNode::Skeleton {
        props: SkeletonProps {
            style: parse_style_props(
                BuiltinComponent::Skeleton,
                &style_props,
                StylePropMode::Variant,
            )?,
            variant,
            animation,
        },
    })
}

pub fn modal_component_node(
    props: Vec<ComponentProp>,
    header: Vec<ViewNode>,
    body: Vec<ViewNode>,
    footer: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if body.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Modal requires body children",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Modal, &header, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Modal, &body, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Modal, &footer, allow_children)?;
    let mut open = None;
    let mut on_close = None;
    let mut disable_overlay_close = false;
    let mut hide_close_button = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "open" => open = Some(parse_signal_path(&prop.name, &prop.value, "signal bool path")?),
            "onClose" => on_close = Some(parse_required_string(&prop.name, &prop.value)?),
            "disableOverlayClose" => {
                disable_overlay_close = parse_static_bool(&prop.name, &prop.value)?
            }
            "hideCloseButton" => hide_close_button = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Modal)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Modal, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    Ok(ViewNode::Modal {
        props: ModalProps {
            style,
            open: open.ok_or_else(|| ComponentError::invalid_prop("open", "signal bool path"))?,
            on_close,
            disable_overlay_close,
            hide_close_button,
        },
        header,
        body,
        footer,
    })
}

pub fn alert_dialog_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut open = None;
    let mut title = "Are you sure?".to_string();
    let mut description = "This action cannot be undone.".to_string();
    let mut confirm_text = "Confirm".to_string();
    let mut cancel_text = "Cancel".to_string();
    let mut on_confirm = None;
    let mut on_cancel = None;
    let mut loading = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "open" => open = Some(parse_signal_path(&prop.name, &prop.value, "signal bool path")?),
            "title" => title = parse_required_string(&prop.name, &prop.value)?,
            "description" => description = parse_required_string(&prop.name, &prop.value)?,
            "confirmText" => confirm_text = parse_required_string(&prop.name, &prop.value)?,
            "cancelText" => cancel_text = parse_required_string(&prop.name, &prop.value)?,
            "onConfirm" => on_confirm = Some(parse_required_string(&prop.name, &prop.value)?),
            "onCancel" => on_cancel = Some(parse_required_string(&prop.name, &prop.value)?),
            "loading" => loading = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::AlertDialog)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::AlertDialog, &style_props)?;
    style.color.get_or_insert(ColorFamily::Danger);
    style.variant.get_or_insert(ComponentVariant::Solid);
    Ok(ViewNode::AlertDialog {
        props: AlertDialogProps {
            style,
            open: open.ok_or_else(|| ComponentError::invalid_prop("open", "signal bool path"))?,
            title,
            description,
            confirm_text,
            cancel_text,
            on_confirm,
            on_cancel,
            loading,
        },
    })
}

pub fn tooltip_component_node(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Tooltip requires at least one child",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Tooltip, &children, allow_children)?;
    let mut label = None;
    let mut position = OverlayPosition::Top;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "position" => position = parse_overlay_position(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Tooltip)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Tooltip, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Tooltip, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::Tooltip {
        props: TooltipProps {
            style,
            label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
            position,
        },
        children,
    })
}

pub fn toast_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut source = None;
    let mut kind = ToastKind::Info;
    let mut title = None;
    let mut description = String::new();
    let mut position = OverlayCornerPosition::BottomLeft;
    let mut show_icon = false;
    let mut style_props = Vec::new();
    let mut explicit_scheme = false;
    for prop in props {
        match prop.name.as_str() {
            "source" => source = Some(parse_signal_path(&prop.name, &prop.value, "signal object path")?),
            "type" => kind = parse_toast_kind(&prop.name, &prop.value)?,
            "title" => title = Some(parse_required_string(&prop.name, &prop.value)?),
            "description" => description = parse_static_string(&prop.name, &prop.value)?,
            "position" => position = parse_overlay_corner_position(&prop.name, &prop.value)?,
            "showIcon" => show_icon = parse_static_bool(&prop.name, &prop.value)?,
            "scheme" => {
                explicit_scheme = true;
                style_props.push(prop);
            }
            "color" => return Err(scheme_prop_error(BuiltinComponent::Toast)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Toast, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Toast, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    if !explicit_scheme {
        style.color.get_or_insert(kind.color());
    }
    Ok(ViewNode::Toast {
        props: ToastProps {
            style,
            source,
            kind,
            title,
            description,
            position,
            show_icon,
        },
    })
}

pub fn dropdown_component_node(
    props: Vec<ComponentProp>,
    trigger: Vec<ViewNode>,
    header: Vec<ViewNode>,
    entries: Vec<OverlayEntry>,
    footer: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if trigger.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Dropdown requires a trigger region",
        ));
    }
    if entries.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Dropdown requires at least one item",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Dropdown, &trigger, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Dropdown, &header, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Dropdown, &footer, allow_children)?;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "variant" => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Dropdown,
                    &prop.name,
                ));
            }
            "color" => return Err(scheme_prop_error(BuiltinComponent::Dropdown)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Dropdown, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Dropdown {
        props: DropdownProps { style },
        trigger,
        header,
        entries,
        footer,
    })
}

pub fn command_component_node(
    props: Vec<ComponentProp>,
    entries: Vec<CommandEntry>,
) -> ComponentResult<ViewNode> {
    if entries.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Command requires at least one item or group",
        ));
    }
    let mut open = None;
    let mut placeholder = "Search...".to_string();
    let mut empty_text = "No results found".to_string();
    let mut close_text = "to close".to_string();
    let mut navigate_text = "Navigate".to_string();
    let mut select_text = "Select".to_string();
    let mut toggle_text = "Toggle".to_string();
    let mut shortcut = "k".to_string();
    let mut disable_global_shortcut = false;
    let mut show_footer = true;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "open" => open = Some(parse_signal_path(&prop.name, &prop.value, "signal bool path")?),
            "placeholder" => placeholder = parse_required_string(&prop.name, &prop.value)?,
            "emptyText" => empty_text = parse_required_string(&prop.name, &prop.value)?,
            "closeText" => close_text = parse_required_string(&prop.name, &prop.value)?,
            "navigateText" => navigate_text = parse_required_string(&prop.name, &prop.value)?,
            "selectText" => select_text = parse_required_string(&prop.name, &prop.value)?,
            "toggleText" => toggle_text = parse_required_string(&prop.name, &prop.value)?,
            "shortcut" => shortcut = parse_command_shortcut(&prop.name, &prop.value)?,
            "disableGlobalShortcut" => {
                disable_global_shortcut = parse_static_bool(&prop.name, &prop.value)?
            }
            "showFooter" => show_footer = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Command)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Command, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::Command {
        props: CommandProps {
            style,
            open,
            placeholder,
            empty_text,
            close_text,
            navigate_text,
            select_text,
            toggle_text,
            shortcut,
            disable_global_shortcut,
            show_footer,
        },
        entries,
    })
}

pub fn avatar_group_item_component(
    props: Vec<ComponentProp>,
) -> ComponentResult<AvatarGroupItem> {
    let mut src = None;
    let mut name = None;
    let mut alt = None;
    let mut href = None;
    let mut navigate = None;
    let mut history = None;
    let mut target = None;
    let mut external_mode = None;
    let mut on_click = None;
    for prop in props {
        match prop.name.as_str() {
            "src" => src = Some(parse_avatar_src(&prop.name, &prop.value)?),
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "alt" => alt = Some(parse_required_string(&prop.name, &prop.value)?),
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "navigate" => navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?),
            "history" => history = Some(parse_history_prop(&prop.name, &prop.value)?),
            "target" => target = Some(parse_web_target(&prop.name, &prop.value)?),
            "externalMode" => {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            "onClick" => on_click = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::AvatarGroup, &prop.name)),
        }
    }
    if href.is_some() && on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`href` and `onClick` cannot be used on the same AvatarGroup item",
        ));
    }
    if href.is_some() && history.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`href` and `history` cannot be used on the same AvatarGroup item",
        ));
    }
    let navigation = if history.is_some() {
        if navigate.is_some() || target.is_some() || external_mode.is_some() {
            return Err(ComponentError::invalid_prop_combination(
                "`navigate`, `target` and `externalMode` require `href` on AvatarGroup item",
            ));
        }
        history
    } else {
        parse_link_navigation_props(
            "AvatarGroup item",
            href,
            navigate,
            target,
            external_mode,
        )?
    };
    Ok(AvatarGroupItem {
        src,
        name,
        alt,
        on_click,
        navigation,
    })
}

pub fn avatar_group_component_node(
    props: Vec<ComponentProp>,
    items: Vec<AvatarGroupItem>,
) -> ComponentResult<ViewNode> {
    let mut source = None;
    let mut size = ButtonSize::Md;
    let mut max = None;
    let mut auto_fit = false;
    let mut inline = false;
    let mut bordered = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "items" => source = Some(parse_signal_path(&prop.name, &prop.value, "signal array path")?),
            "size" => size = parse_button_size_prop(&prop.name, &prop.value)?,
            "max" => max = Some(parse_positive_u16(&prop.name, &prop.value)?),
            "autoFit" => auto_fit = parse_static_bool(&prop.name, &prop.value)?,
            "inline" => inline = parse_static_bool(&prop.name, &prop.value)?,
            "bordered" => bordered = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::AvatarGroup)),
            _ => style_props.push(prop),
        }
    }
    if source.is_none() && items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "AvatarGroup requires `items` or at least one item entry",
        ));
    }
    let mut style = parse_variant_props(BuiltinComponent::AvatarGroup, &style_props)?;
    require_solid_or_soft(BuiltinComponent::AvatarGroup, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::AvatarGroup {
        props: AvatarGroupProps {
            style,
            items: source,
            size,
            max,
            auto_fit,
            inline,
            bordered,
        },
        items,
    })
}

pub fn chat_box_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut messages = None;
    let mut mode = ChatBoxMode::Conversation;
    let mut current_user_id = String::new();
    let mut user_name = String::new();
    let mut user_avatar = None;
    let mut user_status = "Online".to_string();
    let mut assistant_name = "Assistant".to_string();
    let mut assistant_avatar = None;
    let mut show_header = true;
    let mut placeholder = "Type a message...".to_string();
    let mut show_attachments = true;
    let mut show_voice_note = true;
    let mut show_camera = false;
    let mut loading = None;
    let mut sending = None;
    let mut streaming = None;
    let mut has_more = None;
    let mut on_send = None;
    let mut on_load_more = None;
    let mut on_stop = None;
    let mut on_voice_note = None;
    let mut on_file_attach = None;
    let mut on_camera_capture = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "messages" => {
                messages = Some(parse_signal_path(&prop.name, &prop.value, "signal array path")?)
            }
            "mode" => mode = parse_chat_box_mode(&prop.name, &prop.value)?,
            "currentUserId" => current_user_id = parse_static_string(&prop.name, &prop.value)?,
            "userName" => user_name = parse_static_string(&prop.name, &prop.value)?,
            "userAvatar" => user_avatar = Some(parse_avatar_src(&prop.name, &prop.value)?),
            "userStatus" => user_status = parse_static_string(&prop.name, &prop.value)?,
            "assistantName" => assistant_name = parse_static_string(&prop.name, &prop.value)?,
            "assistantAvatar" => assistant_avatar = Some(parse_avatar_src(&prop.name, &prop.value)?),
            "showHeader" => show_header = parse_static_bool(&prop.name, &prop.value)?,
            "placeholder" => placeholder = parse_static_string(&prop.name, &prop.value)?,
            "showAttachments" => show_attachments = parse_static_bool(&prop.name, &prop.value)?,
            "showVoiceNote" => show_voice_note = parse_static_bool(&prop.name, &prop.value)?,
            "showCamera" => show_camera = parse_static_bool(&prop.name, &prop.value)?,
            "loading" => loading = Some(parse_signal_path(&prop.name, &prop.value, "signal bool path")?),
            "sending" => sending = Some(parse_signal_path(&prop.name, &prop.value, "signal bool path")?),
            "streaming" => streaming = Some(parse_signal_path(&prop.name, &prop.value, "signal bool path")?),
            "hasMore" => has_more = Some(parse_signal_path(&prop.name, &prop.value, "signal bool path")?),
            "onSend" => on_send = Some(parse_required_string(&prop.name, &prop.value)?),
            "onLoadMore" => on_load_more = Some(parse_required_string(&prop.name, &prop.value)?),
            "onStop" => on_stop = Some(parse_required_string(&prop.name, &prop.value)?),
            "onVoiceNote" => on_voice_note = Some(parse_required_string(&prop.name, &prop.value)?),
            "onFileAttach" => on_file_attach = Some(parse_required_string(&prop.name, &prop.value)?),
            "onCameraCapture" => on_camera_capture = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::ChatBox)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::ChatBox, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::ChatBox {
        props: ChatBoxProps {
            style,
            messages: messages.ok_or_else(|| ComponentError::invalid_prop("messages", "signal array path"))?,
            mode,
            current_user_id,
            user_name,
            user_avatar,
            user_status,
            assistant_name,
            assistant_avatar,
            show_header,
            placeholder,
            show_attachments,
            show_voice_note,
            show_camera,
            loading,
            sending,
            streaming,
            has_more,
            on_send,
            on_load_more,
            on_stop,
            on_voice_note,
            on_file_attach,
            on_camera_capture,
        },
    })
}

pub fn empty_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut kind = EmptyKind::Template;
    let mut title = None;
    let mut description = None;
    let mut action_label = "View more".to_string();
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "type" => kind = parse_empty_kind(&prop.name, &prop.value)?,
            "title" => title = Some(parse_static_string(&prop.name, &prop.value)?),
            "description" => description = Some(parse_static_string(&prop.name, &prop.value)?),
            "actionLabel" => action_label = parse_static_string(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Empty)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Empty, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Soft);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Empty {
        props: EmptyProps {
            style,
            kind,
            title,
            description,
            action_label,
        },
    })
}

pub fn marquee_component_node(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Marquee requires at least one child",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Marquee, &children, allow_children)?;
    let mut speed = MarqueeSpeed::Normal;
    let mut pause_on_hover = false;
    let mut reverse = false;
    let mut orientation = MarqueeOrientation::Horizontal;
    let mut fade = false;
    let mut fade_color = ColorToken::Background;
    let mut gap = ScaleValue::from_half_steps(0);
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "speed" => speed = parse_marquee_speed(&prop.name, &prop.value)?,
            "pauseOnHover" => pause_on_hover = parse_static_bool(&prop.name, &prop.value)?,
            "reverse" => reverse = parse_static_bool(&prop.name, &prop.value)?,
            "orientation" => orientation = parse_marquee_orientation(&prop.name, &prop.value)?,
            "fade" => fade = parse_static_bool(&prop.name, &prop.value)?,
            "fadeColor" => fade_color = parse_single_color_token(&prop.name, &prop.value)?,
            "gap" => gap = parse_static_scale(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    Ok(ViewNode::Marquee {
        props: MarqueeProps {
            style: parse_style_props(
                BuiltinComponent::Marquee,
                &style_props,
                StylePropMode::Variant,
            )?,
            speed,
            pause_on_hover,
            reverse,
            orientation,
            fade,
            fade_color,
            gap,
        },
        children,
    })
}

pub fn type_writer_item_component(props: Vec<ComponentProp>) -> ComponentResult<TypeWriterItem> {
    let mut text = None;
    for prop in props {
        match prop.name.as_str() {
            "text" => text = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::TypeWriter, &prop.name)),
        }
    }
    Ok(TypeWriterItem {
        text: text.ok_or_else(|| ComponentError::invalid_prop("text", "non-empty string"))?,
    })
}

pub fn type_writer_component_node(
    props: Vec<ComponentProp>,
    items: Vec<TypeWriterItem>,
) -> ComponentResult<ViewNode> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "TypeWriter requires at least one item",
        ));
    }
    let mut type_speed = 100;
    let mut delete_speed = 50;
    let mut after_typed = 1000;
    let mut after_deleted = 500;
    let mut repeat = true;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "typeSpeed" => type_speed = parse_positive_u64(&prop.name, &prop.value)?,
            "deleteSpeed" => delete_speed = parse_positive_u64(&prop.name, &prop.value)?,
            "afterTyped" => after_typed = parse_non_negative_u64(&prop.name, &prop.value)?,
            "afterDeleted" => after_deleted = parse_non_negative_u64(&prop.name, &prop.value)?,
            "repeat" => repeat = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    Ok(ViewNode::TypeWriter {
        props: TypeWriterProps {
            style: parse_style_props(
                BuiltinComponent::TypeWriter,
                &style_props,
                StylePropMode::Text,
            )?,
            type_speed,
            delete_speed,
            after_typed,
            after_deleted,
            repeat,
        },
        items,
    })
}

pub fn rich_text_mark_component(props: Vec<ComponentProp>) -> ComponentResult<RichTextMark> {
    let mut text = None;
    let mut style = RichTextMarkStyle::Mark;
    let mut color = ColorFamily::Info;
    for prop in props {
        match prop.name.as_str() {
            "text" => text = Some(parse_required_string(&prop.name, &prop.value)?),
            "style" => style = parse_rich_text_mark_style(&prop.name, &prop.value)?,
            "scheme" => color = parse_family_prop(BuiltinComponent::RichText, &prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::RichText)),
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::RichText, &prop.name)),
        }
    }
    Ok(RichTextMark {
        text: text.ok_or_else(|| ComponentError::invalid_prop("text", "non-empty string"))?,
        style,
        color,
    })
}

pub fn rich_text_component_node(
    props: Vec<ComponentProp>,
    marks: Vec<RichTextMark>,
) -> ComponentResult<ViewNode> {
    if marks.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "RichText requires at least one mark entry",
        ));
    }
    Ok(ViewNode::RichText {
        props: parse_text_props(BuiltinComponent::RichText, &props)?,
        marks,
    })
}

pub fn record_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut name = None;
    let mut url = None;
    let mut disabled = false;
    let mut max_duration = None;
    let mut on_start = None;
    let mut on_pause = None;
    let mut on_resume = None;
    let mut on_stop = None;
    let mut on_discard = None;
    let mut on_confirm = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "name" => name = Some(parse_required_string(&prop.name, &prop.value)?),
            "url" => url = Some(parse_media_source(&prop.name, &prop.value)?),
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "maxDuration" => max_duration = Some(parse_positive_u16(&prop.name, &prop.value)?),
            "onStart" => on_start = Some(parse_required_string(&prop.name, &prop.value)?),
            "onPause" => on_pause = Some(parse_required_string(&prop.name, &prop.value)?),
            "onResume" => on_resume = Some(parse_required_string(&prop.name, &prop.value)?),
            "onStop" => on_stop = Some(parse_required_string(&prop.name, &prop.value)?),
            "onDiscard" => on_discard = Some(parse_required_string(&prop.name, &prop.value)?),
            "onConfirm" => on_confirm = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Record)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Record, &style_props)?;
    require_solid_or_soft(BuiltinComponent::Record, style.variant)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Record {
        props: RecordProps {
            style,
            name: name.ok_or_else(|| ComponentError::invalid_prop("name", "non-empty string"))?,
            url,
            disabled,
            max_duration,
            on_start,
            on_pause,
            on_resume,
            on_stop,
            on_discard,
            on_confirm,
        },
    })
}

pub fn toggle_group_item_component(
    props: Vec<ComponentProp>,
) -> ComponentResult<ToggleGroupItem> {
    let mut id = None;
    let mut label = None;
    let mut icon = None;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_static_string_or_number(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "icon" => icon = Some(parse_view_icon(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::ToggleGroup, &prop.name)),
        }
    }
    Ok(ToggleGroupItem {
        id: id.ok_or_else(|| ComponentError::invalid_prop("id", "static string or number"))?,
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        icon,
    })
}

pub fn toggle_group_component_node(
    props: Vec<ComponentProp>,
    items: Vec<ToggleGroupItem>,
) -> ComponentResult<ViewNode> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "ToggleGroup requires at least one item",
        ));
    }
    let mut seen = BTreeSet::new();
    for item in &items {
        if !seen.insert(item.id.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate ToggleGroup item id `{}`",
                item.id
            )));
        }
    }
    let mut value = None;
    let mut selected = None;
    let mut size = ButtonSize::Md;
    let mut wide = false;
    let mut vertical = false;
    let mut disabled = false;
    let mut aria_label = None;
    let mut on_change = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(parse_signal_path(
                &prop.name,
                &prop.value,
                "signal string path",
            )?),
            "selected" => selected = Some(parse_static_string_or_number(&prop.name, &prop.value)?),
            "size" => size = parse_button_size_prop(&prop.name, &prop.value)?,
            "wide" => wide = parse_static_bool(&prop.name, &prop.value)?,
            "vertical" => vertical = parse_static_bool(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "ariaLabel" => aria_label = Some(parse_required_string(&prop.name, &prop.value)?),
            "onChange" => on_change = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::ToggleGroup)),
            _ => style_props.push(prop),
        }
    }
    let selected = selected.unwrap_or_else(|| items[0].id.clone());
    if !items.iter().any(|item| item.id == selected) {
        return Err(ComponentError::invalid_prop_combination(format!(
            "ToggleGroup selected value `{}` must match an item id",
            selected
        )));
    }
    let mut style = parse_variant_props(BuiltinComponent::ToggleGroup, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::ToggleGroup {
        props: ToggleGroupProps {
            style,
            value,
            selected,
            size,
            wide,
            vertical,
            disabled,
            aria_label,
            on_change,
        },
        items,
    })
}

pub fn collapsible_component_node(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    reject_children_placeholder(BuiltinComponent::Collapsible, &children, allow_children)?;
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Collapsible requires body children",
        ));
    }
    let mut label = None;
    let mut default_open = false;
    let mut disabled = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "defaultOpen" => default_open = parse_static_bool(&prop.name, &prop.value)?,
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            "color" => return Err(scheme_prop_error(BuiltinComponent::Collapsible)),
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Collapsible, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Background);
    Ok(ViewNode::Collapsible {
        props: CollapsibleProps {
            style,
            label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
            default_open,
            disabled,
        },
        children,
    })
}

pub fn countdown_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut target = None;
    let mut show_days = true;
    let mut show_hours = true;
    let mut show_minutes = true;
    let mut show_seconds = true;
    let mut size = CountdownSize::Md;
    let mut days_label = "Days".to_string();
    let mut hours_label = "Hours".to_string();
    let mut minutes_label = "Minutes".to_string();
    let mut seconds_label = "Seconds".to_string();
    let mut on_complete = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "target" => target = Some(parse_required_string(&prop.name, &prop.value)?),
            "showDays" => show_days = parse_static_bool(&prop.name, &prop.value)?,
            "showHours" => show_hours = parse_static_bool(&prop.name, &prop.value)?,
            "showMinutes" => show_minutes = parse_static_bool(&prop.name, &prop.value)?,
            "showSeconds" => show_seconds = parse_static_bool(&prop.name, &prop.value)?,
            "size" => size = parse_countdown_size(&prop.name, &prop.value)?,
            "daysLabel" => days_label = parse_required_string(&prop.name, &prop.value)?,
            "hoursLabel" => hours_label = parse_required_string(&prop.name, &prop.value)?,
            "minutesLabel" => minutes_label = parse_required_string(&prop.name, &prop.value)?,
            "secondsLabel" => seconds_label = parse_required_string(&prop.name, &prop.value)?,
            "onComplete" => on_complete = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Countdown)),
            _ => style_props.push(prop),
        }
    }
    if !show_days && !show_hours && !show_minutes && !show_seconds {
        return Err(ComponentError::invalid_prop_combination(
            "Countdown must show at least one unit",
        ));
    }
    let mut style = parse_variant_props(BuiltinComponent::Countdown, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Countdown {
        props: CountdownProps {
            style,
            target: target.ok_or_else(|| ComponentError::invalid_prop("target", "date string"))?,
            show_days,
            show_hours,
            show_minutes,
            show_seconds,
            size,
            days_label,
            hours_label,
            minutes_label,
            seconds_label,
            on_complete,
        },
    })
}

pub fn map_marker_component(props: Vec<ComponentProp>) -> ComponentResult<MapMarker> {
    let mut id = None;
    let mut lat = None;
    let mut lng = None;
    let mut label = None;
    let mut popup = None;
    let mut icon = MapMarkerIcon::Default;
    let mut on_click = None;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_static_string_or_number(&prop.name, &prop.value)?),
            "lat" => lat = Some(parse_number_literal(&prop.name, &prop.value)?),
            "lng" => lng = Some(parse_number_literal(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "popup" => popup = Some(parse_required_string(&prop.name, &prop.value)?),
            "icon" => icon = parse_map_marker_icon(&prop.name, &prop.value)?,
            "onClick" => on_click = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::Map, &prop.name)),
        }
    }
    Ok(MapMarker {
        id: id.ok_or_else(|| ComponentError::invalid_prop("id", "static string or number"))?,
        lat: lat.ok_or_else(|| ComponentError::invalid_prop("lat", "number"))?,
        lng: lng.ok_or_else(|| ComponentError::invalid_prop("lng", "number"))?,
        label,
        popup,
        icon,
        on_click,
    })
}

pub fn map_waypoint_component(props: Vec<ComponentProp>) -> ComponentResult<MapWaypoint> {
    let mut lat = None;
    let mut lng = None;
    for prop in props {
        match prop.name.as_str() {
            "lat" => lat = Some(parse_number_literal(&prop.name, &prop.value)?),
            "lng" => lng = Some(parse_number_literal(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::Map, &prop.name)),
        }
    }
    Ok(MapWaypoint {
        lat: lat.ok_or_else(|| ComponentError::invalid_prop("lat", "number"))?,
        lng: lng.ok_or_else(|| ComponentError::invalid_prop("lng", "number"))?,
    })
}

pub fn map_component_node(
    props: Vec<ComponentProp>,
    markers: Vec<MapMarker>,
    waypoints: Vec<MapWaypoint>,
) -> ComponentResult<ViewNode> {
    let mut seen = BTreeSet::new();
    for marker in &markers {
        if !seen.insert(marker.id.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate Map marker id `{}`",
                marker.id
            )));
        }
    }
    let mut center_lat = "0".to_string();
    let mut center_lng = "0".to_string();
    let mut zoom = 13;
    let mut height = "400px".to_string();
    let mut width = "100%".to_string();
    let mut show_controls = true;
    let mut show_scale = false;
    let mut show_location_control = false;
    let mut interactive = true;
    let mut route_start_lat = None;
    let mut route_start_lng = None;
    let mut route_end_lat = None;
    let mut route_end_lng = None;
    let mut on_location = None;
    let mut on_location_error = None;
    let mut on_route = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "centerLat" => center_lat = parse_number_literal(&prop.name, &prop.value)?,
            "centerLng" => center_lng = parse_number_literal(&prop.name, &prop.value)?,
            "zoom" => zoom = parse_non_negative_u16(&prop.name, &prop.value)?,
            "height" => height = parse_required_string(&prop.name, &prop.value)?,
            "width" => width = parse_required_string(&prop.name, &prop.value)?,
            "showControls" => show_controls = parse_static_bool(&prop.name, &prop.value)?,
            "showScale" => show_scale = parse_static_bool(&prop.name, &prop.value)?,
            "showLocationControl" => {
                show_location_control = parse_static_bool(&prop.name, &prop.value)?
            }
            "interactive" => interactive = parse_static_bool(&prop.name, &prop.value)?,
            "routeStartLat" => route_start_lat = Some(parse_number_literal(&prop.name, &prop.value)?),
            "routeStartLng" => route_start_lng = Some(parse_number_literal(&prop.name, &prop.value)?),
            "routeEndLat" => route_end_lat = Some(parse_number_literal(&prop.name, &prop.value)?),
            "routeEndLng" => route_end_lng = Some(parse_number_literal(&prop.name, &prop.value)?),
            "onLocation" => on_location = Some(parse_required_string(&prop.name, &prop.value)?),
            "onLocationError" => {
                on_location_error = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "onRoute" => on_route = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Map)),
            _ => style_props.push(prop),
        }
    }
    let route_count = [
        route_start_lat.is_some(),
        route_start_lng.is_some(),
        route_end_lat.is_some(),
        route_end_lng.is_some(),
    ]
    .into_iter()
    .filter(|value| *value)
    .count();
    if route_count != 0 && route_count != 4 {
        return Err(ComponentError::invalid_prop_combination(
            "Map route requires routeStartLat, routeStartLng, routeEndLat and routeEndLng",
        ));
    }
    let mut style = parse_variant_props(BuiltinComponent::Map, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Primary);
    Ok(ViewNode::Map {
        props: MapProps {
            style,
            center_lat,
            center_lng,
            zoom,
            height,
            width,
            show_controls,
            show_scale,
            show_location_control,
            interactive,
            route_start_lat,
            route_start_lng,
            route_end_lat,
            route_end_lng,
            on_location,
            on_location_error,
            on_route,
        },
        markers,
        waypoints,
    })
}

pub fn overlay_item_component(
    owner: BuiltinComponent,
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
) -> ComponentResult<OverlayItemProps> {
    let mut label = None;
    let mut description = None;
    let mut href = None;
    let mut navigate = None;
    let mut history = None;
    let mut target = None;
    let mut external_mode = None;
    let mut on_click = None;
    let mut disabled = false;
    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "description" => description = Some(parse_required_string(&prop.name, &prop.value)?),
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "navigate" => navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?),
            "history" => history = Some(parse_history_prop(&prop.name, &prop.value)?),
            "target" => target = Some(parse_web_target(&prop.name, &prop.value)?),
            "externalMode" => {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            "onClick" => on_click = Some(parse_required_string(&prop.name, &prop.value)?),
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            _ => return Err(ComponentError::unknown_prop(owner, &prop.name)),
        }
    }
    if href.is_some() && on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "`href` and `onClick` cannot be used on the same {} item",
            owner.as_str()
        )));
    }
    if href.is_some() && history.is_some() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "`href` and `history` cannot be used on the same {} item",
            owner.as_str()
        )));
    }
    let navigation = if history.is_some() {
        if navigate.is_some() || target.is_some() || external_mode.is_some() {
            return Err(ComponentError::invalid_prop_combination(format!(
                "`navigate`, `target` and `externalMode` require `href` on {} item",
                owner.as_str()
            )));
        }
        history
    } else {
        parse_link_navigation_props(owner.as_str(), href, navigate, target, external_mode)?
    };
    Ok(OverlayItemProps {
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        description,
        icon,
        on_click,
        navigation,
        disabled,
    })
}

pub fn command_group_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    items: Vec<OverlayItemProps>,
) -> ComponentResult<CommandEntry> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Command group requires at least one item",
        ));
    }
    let mut label = None;
    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::Command, &prop.name)),
        }
    }
    Ok(CommandEntry::Group {
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        icon,
        items,
    })
}

pub fn overlay_icon_component(node: ViewNode, owner: BuiltinComponent) -> ComponentResult<SideNavIcon> {
    match node {
        ViewNode::Svg { props, paths } => Ok(SideNavIcon { props, paths }),
        _ => Err(ComponentError::invalid_prop_combination(format!(
            "{} icon requires exactly one Svg child",
            owner.as_str()
        ))),
    }
}

fn parse_avatar_src(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    parse_cover_source(&value)
        .map(|source| source.0)
        .ok_or_else(|| ComponentError::invalid_prop(name, "asset path or https URL"))
}

fn parse_media_source(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    parse_cover_source(&value)
        .map(|source| source.0)
        .ok_or_else(|| ComponentError::invalid_prop(name, "asset path or https URL"))
}

fn parse_static_string_or_number(name: &str, value: &PropValue) -> ComponentResult<String> {
    match value {
        PropValue::String(value) | PropValue::Number(value) => Ok(value.clone()),
        PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "static string or number"))
        }
    }
}

fn parse_image_aspect(name: &str, value: &PropValue) -> ComponentResult<ImageAspect> {
    let value = parse_required_string(name, value)?;
    ImageAspect::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "horizontal, vertical, square or auto"))
}

fn parse_image_object_fit(name: &str, value: &PropValue) -> ComponentResult<ImageObjectFit> {
    let value = parse_required_string(name, value)?;
    ImageObjectFit::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "cover, contain, fill or none"))
}

fn parse_image_loading(name: &str, value: &PropValue) -> ComponentResult<ImageLoading> {
    let value = parse_required_string(name, value)?;
    ImageLoading::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "lazy or eager"))
}

fn parse_chat_box_mode(name: &str, value: &PropValue) -> ComponentResult<ChatBoxMode> {
    let value = parse_required_string(name, value)?;
    ChatBoxMode::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "conversation or prompt"))
}

fn parse_empty_kind(name: &str, value: &PropValue) -> ComponentResult<EmptyKind> {
    let value = parse_required_string(name, value)?;
    EmptyKind::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "playlist, result, data or template"))
}

fn parse_marquee_speed(name: &str, value: &PropValue) -> ComponentResult<MarqueeSpeed> {
    let value = parse_required_string(name, value)?;
    MarqueeSpeed::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "slow, normal or fast"))
}

fn parse_marquee_orientation(
    name: &str,
    value: &PropValue,
) -> ComponentResult<MarqueeOrientation> {
    let value = parse_required_string(name, value)?;
    MarqueeOrientation::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "horizontal or vertical"))
}

fn parse_carousel_orientation(
    name: &str,
    value: &PropValue,
) -> ComponentResult<CarouselOrientation> {
    let value = parse_required_string(name, value)?;
    CarouselOrientation::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "horizontal or vertical"))
}

fn parse_carousel_indicator(
    name: &str,
    value: &PropValue,
) -> ComponentResult<CarouselIndicatorType> {
    let value = parse_required_string(name, value)?;
    CarouselIndicatorType::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "bar or dot"))
}

fn parse_rich_text_mark_style(
    name: &str,
    value: &PropValue,
) -> ComponentResult<RichTextMarkStyle> {
    let value = parse_required_string(name, value)?;
    RichTextMarkStyle::from_name(&value).ok_or_else(|| {
        ComponentError::invalid_prop(
            name,
            "mark, grad, pill, slant, glow, under, strike, box, wave, neon, pop or tag",
        )
    })
}

fn parse_countdown_size(name: &str, value: &PropValue) -> ComponentResult<CountdownSize> {
    let value = parse_required_string(name, value)?;
    CountdownSize::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "sm, md, lg or xl"))
}

fn parse_map_marker_icon(name: &str, value: &PropValue) -> ComponentResult<MapMarkerIcon> {
    let value = parse_required_string(name, value)?;
    MapMarkerIcon::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "default, start, end or waypoint"))
}

fn parse_control_size_prop(name: &str, value: &PropValue) -> ComponentResult<ButtonSize> {
    let size = parse_button_size_prop(name, value)?;
    match size {
        ButtonSize::Sm | ButtonSize::Md | ButtonSize::Lg => Ok(size),
        ButtonSize::Xs | ButtonSize::Xl => Err(ComponentError::invalid_prop(name, "sm, md or lg")),
    }
}

fn parse_view_icon(name: &str, value: &PropValue) -> ComponentResult<ViewIcon> {
    let value = parse_required_string(name, value)?;
    ViewIcon::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "plus, link, edit, trash, search, settings, upload, file or dismiss"))
}

fn parse_static_scale(name: &str, value: &PropValue) -> ComponentResult<ScaleValue> {
    match value {
        PropValue::Number(value) => scale_value(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "Dowe scale value from 0 to 96")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "Dowe scale value from 0 to 96"))
        }
    }
}

fn parse_number_literal(name: &str, value: &PropValue) -> ComponentResult<String> {
    match value {
        PropValue::Number(value) if value.parse::<f64>().is_ok() => Ok(value.clone()),
        PropValue::String(_) | PropValue::Number(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "number"))
        }
    }
}

fn parse_positive_number_literal(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_number_literal(name, value)?;
    if value.parse::<f64>().ok().is_some_and(|value| value > 0.0) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "positive number"))
    }
}

fn parse_positive_u64(name: &str, value: &PropValue) -> ComponentResult<u64> {
    match value {
        PropValue::Number(value) => value
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| ComponentError::invalid_prop(name, "positive integer")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "positive integer"))
        }
    }
}

fn parse_non_negative_u64(name: &str, value: &PropValue) -> ComponentResult<u64> {
    match value {
        PropValue::Number(value) => value
            .parse::<u64>()
            .ok()
            .ok_or_else(|| ComponentError::invalid_prop(name, "non-negative integer")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "non-negative integer"))
        }
    }
}

fn parse_single_color_token(name: &str, value: &PropValue) -> ComponentResult<ColorToken> {
    let value = parse_required_string(name, value)?;
    ColorToken::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "color token"))
}

fn validate_slider_range(min: &str, max: &str, value: &str) -> ComponentResult<()> {
    let min_value = min.parse::<f64>().map_err(|_| ComponentError::invalid_prop("min", "number"))?;
    let max_value = max.parse::<f64>().map_err(|_| ComponentError::invalid_prop("max", "number"))?;
    let current_value = value
        .parse::<f64>()
        .map_err(|_| ComponentError::invalid_prop("value", "number"))?;
    if max_value <= min_value {
        return Err(ComponentError::invalid_prop_combination(
            "`max` must be greater than `min`",
        ));
    }
    if current_value < min_value || current_value > max_value {
        return Err(ComponentError::invalid_prop(
            "value",
            "number between min and max",
        ));
    }
    Ok(())
}

fn apply_icon_button_size_defaults(style: &mut StyleProps, size: ButtonSize) {
    let value = SizeValue::Scale(size.min_height());
    if style.sizing.w.is_none() {
        style.sizing.w = Some(ResponsiveValue::scalar(value.clone()));
    }
    if style.sizing.h.is_none() {
        style.sizing.h = Some(ResponsiveValue::scalar(value));
    }
    if style.rounded.is_none() {
        style.rounded = Some(ResponsiveValue::scalar(RoundedSize::Full));
    }
}

fn parse_non_negative_u16(name: &str, value: &PropValue) -> ComponentResult<u16> {
    match value {
        PropValue::Number(value) => value
            .parse::<u16>()
            .ok()
            .ok_or_else(|| ComponentError::invalid_prop(name, "non-negative integer")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "non-negative integer"))
        }
    }
}

fn parse_hex_color_prop(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if valid_hex_color(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "hex color like #3b82f6"))
    }
}

fn valid_hex_color(value: &str) -> bool {
    let Some(hex) = value.strip_prefix('#') else {
        return false;
    };
    matches!(hex.len(), 3 | 6) && hex.chars().all(|value| value.is_ascii_hexdigit())
}

fn parse_date_literal(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if valid_date_literal(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "YYYY-MM-DD"))
    }
}

fn valid_date_literal(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, value)| index == 4 || index == 7 || value.is_ascii_digit())
    {
        return false;
    }
    let year = value[0..4].parse::<u16>().ok();
    let month = value[5..7].parse::<u8>().ok();
    let day = value[8..10].parse::<u8>().ok();
    let (Some(year), Some(month), Some(day)) = (year, month, day) else {
        return false;
    };
    if year == 0 || !(1..=12).contains(&month) {
        return false;
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max_day).contains(&day)
}

fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn validate_date_bounds(min: Option<&str>, max: Option<&str>) -> ComponentResult<()> {
    if let (Some(min), Some(max)) = (min, max)
        && min > max
    {
        return Err(ComponentError::invalid_prop_combination(
            "`min` cannot be greater than `max`",
        ));
    }
    Ok(())
}

fn parse_avatar_status(name: &str, value: &PropValue) -> ComponentResult<AvatarStatus> {
    let value = parse_required_string(name, value)?;
    AvatarStatus::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "online, offline, busy or away"))
}

fn parse_overlay_corner_position(
    name: &str,
    value: &PropValue,
) -> ComponentResult<OverlayCornerPosition> {
    let value = parse_required_string(name, value)?;
    OverlayCornerPosition::from_name(&value).ok_or_else(|| {
        ComponentError::invalid_prop(
            name,
            "top-left, top-right, bottom-left or bottom-right",
        )
    })
}

fn parse_overlay_position(name: &str, value: &PropValue) -> ComponentResult<OverlayPosition> {
    let value = parse_required_string(name, value)?;
    OverlayPosition::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "top, bottom, start or end"))
}

fn parse_skeleton_variant(name: &str, value: &PropValue) -> ComponentResult<SkeletonVariant> {
    let value = parse_required_string(name, value)?;
    SkeletonVariant::from_name(&value).ok_or_else(|| {
        ComponentError::invalid_prop(name, "text, circular, rectangular or rounded")
    })
}

fn parse_skeleton_animation(name: &str, value: &PropValue) -> ComponentResult<SkeletonAnimation> {
    let value = parse_required_string(name, value)?;
    SkeletonAnimation::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "pulse, wave or none"))
}

fn parse_toast_kind(name: &str, value: &PropValue) -> ComponentResult<ToastKind> {
    let value = parse_required_string(name, value)?;
    ToastKind::from_name(&value).ok_or_else(|| {
        ComponentError::invalid_prop(
            name,
            "primary, secondary, muted, success, info, warning, danger or error",
        )
    })
}

fn parse_signal_path(name: &str, value: &PropValue, expected: &str) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if is_reference_path(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, expected))
    }
}

fn parse_command_shortcut(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if value.chars().count() == 1 && value.chars().all(|value| value.is_ascii_alphanumeric()) {
        Ok(value.to_ascii_lowercase())
    } else {
        Err(ComponentError::invalid_prop(name, "single ASCII letter or digit"))
    }
}

fn require_solid_or_soft(
    component: BuiltinComponent,
    variant: Option<ComponentVariant>,
) -> ComponentResult<()> {
    if matches!(
        variant,
        Some(ComponentVariant::Outlined | ComponentVariant::Ghost)
    ) {
        return Err(ComponentError::invalid_prop("variant", "solid or soft"));
    }
    if component == BuiltinComponent::Avatar {
        return Ok(());
    }
    Ok(())
}

fn scheme_prop_error(component: BuiltinComponent) -> ComponentError {
    ComponentError::new(format!(
        "unknown prop `color` on `{}`; use `scheme` for visual family",
        component.as_str()
    ))
}

pub fn side_nav_header_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
) -> ComponentResult<SideNavItem> {
    Ok(SideNavItem::Header(parse_side_nav_item_props(
        props, icon, false,
    )?))
}

pub fn side_nav_item_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
) -> ComponentResult<SideNavItem> {
    Ok(SideNavItem::Item(parse_side_nav_item_props(
        props, icon, true,
    )?))
}

pub fn side_nav_submenu_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    open: bool,
    items: Vec<SideNavItemProps>,
) -> ComponentResult<SideNavItem> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "SideNav submenu requires at least one item",
        ));
    }
    let props = parse_side_nav_item_props(props, icon, true)?;
    if props.navigation.is_some() || props.on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "SideNav submenu cannot declare navigation or onClick",
        ));
    }
    Ok(SideNavItem::Submenu { props, open, items })
}

pub fn side_nav_icon_component(node: ViewNode) -> ComponentResult<SideNavIcon> {
    match node {
        ViewNode::Svg { props, paths } => Ok(SideNavIcon { props, paths }),
        _ => Err(ComponentError::invalid_prop_combination(
            "SideNav icon requires exactly one Svg child",
        )),
    }
}

pub fn nav_menu_item_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
) -> ComponentResult<NavMenuItem> {
    Ok(NavMenuItem::Item(parse_nav_menu_item_props(
        props, icon, true,
    )?))
}

pub fn nav_menu_submenu_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    items: Vec<NavMenuItemProps>,
) -> ComponentResult<NavMenuItem> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "NavMenu submenu requires at least one item",
        ));
    }
    let props = parse_nav_menu_item_props(props, icon, false)?;
    if props.navigation.is_some() || props.on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "NavMenu submenu cannot declare navigation or onClick",
        ));
    }
    Ok(NavMenuItem::Submenu { props, items })
}

pub fn nav_menu_megamenu_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    content: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<NavMenuItem> {
    if content.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "NavMenu megamenu requires content",
        ));
    }
    if contains_children(&content) && !allow_children {
        return Err(ComponentError::children_outside_layout());
    }
    let props = parse_nav_menu_item_props(props, icon, false)?;
    if props.navigation.is_some() || props.on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "NavMenu megamenu cannot declare navigation or onClick",
        ));
    }
    Ok(NavMenuItem::Megamenu { props, content })
}

fn parse_side_nav_item_props(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    allow_status: bool,
) -> ComponentResult<SideNavItemProps> {
    let mut label = None;
    let mut description = None;
    let mut status = None;
    let mut href = None;
    let mut navigate = None;
    let mut target = None;
    let mut external_mode = None;
    let mut on_click = None;

    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "description" => description = Some(parse_required_string(&prop.name, &prop.value)?),
            "status" if allow_status => {
                status = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "navigate" => navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?),
            "target" => target = Some(parse_web_target(&prop.name, &prop.value)?),
            "externalMode" => {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            "onClick" => on_click = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::SideNav,
                    &prop.name,
                ));
            }
        }
    }

    if href.is_some() && on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`href` and `onClick` cannot be used on the same SideNav entry",
        ));
    }
    let navigation =
        parse_link_navigation_props("SideNav entry", href, navigate, target, external_mode)?;
    Ok(SideNavItemProps {
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        description,
        status,
        icon,
        on_click,
        navigation,
    })
}

fn parse_nav_menu_item_props(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    allow_navigation: bool,
) -> ComponentResult<NavMenuItemProps> {
    let mut label = None;
    let mut description = None;
    let mut href = None;
    let mut navigate = None;
    let mut target = None;
    let mut external_mode = None;
    let mut on_click = None;

    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "description" => description = Some(parse_required_string(&prop.name, &prop.value)?),
            "href" if allow_navigation => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "navigate" if allow_navigation => {
                navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?)
            }
            "target" if allow_navigation => target = Some(parse_web_target(&prop.name, &prop.value)?),
            "externalMode" if allow_navigation => {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            "onClick" if allow_navigation => {
                on_click = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::NavMenu,
                    &prop.name,
                ));
            }
        }
    }

    if href.is_some() && on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`href` and `onClick` cannot be used on the same NavMenu entry",
        ));
    }
    let navigation =
        parse_link_navigation_props("NavMenu entry", href, navigate, target, external_mode)?;
    Ok(NavMenuItemProps {
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        description,
        icon,
        on_click,
        navigation,
    })
}

pub fn container_component_node(
    component: BuiltinComponent,
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    match component {
        BuiltinComponent::Box => {
            let props = parse_style_props(component, &props, StylePropMode::Box)?;
            container_node(component, Vec::new(), children, allow_children, props)
        }
        BuiltinComponent::Section => {
            let props = parse_style_props(component, &props, StylePropMode::Section)?;
            container_node(component, Vec::new(), children, allow_children, props)
        }
        BuiltinComponent::Flex => {
            let props = parse_layout_props(component, &props)?;
            if contains_children(&children) && !allow_children {
                return Err(ComponentError::children_outside_layout());
            }
            Ok(ViewNode::Flex { props, children })
        }
        BuiltinComponent::Grid => {
            let props = parse_grid_props(component, &props)?;
            if contains_children(&children) && !allow_children {
                return Err(ComponentError::children_outside_layout());
            }
            Ok(ViewNode::Grid { props, children })
        }
        BuiltinComponent::Card => {
            let props = parse_variant_props(component, &props)?;
            reject_children_placeholder(component, &children, allow_children)?;
            Ok(ViewNode::Card { props, children })
        }
        BuiltinComponent::Drawer => drawer_component_node(props, children, allow_children),
        BuiltinComponent::Avatar => {
            if children.is_empty() {
                avatar_component_node(props, None)
            } else {
                Err(ComponentError::invalid_prop_combination(
                    "Avatar only accepts an optional icon region",
                ))
            }
        }
        BuiltinComponent::Badge => badge_component_node(props, children, allow_children),
        BuiltinComponent::Chip => {
            reject_children_placeholder(component, &children, allow_children)?;
            if children.iter().all(is_text_like) {
                let value = children
                    .iter()
                    .filter_map(first_text)
                    .collect::<Vec<_>>()
                    .join(" ");
                chip_component_node(props, value, None, None)
            } else {
                Err(ComponentError::text_cannot_contain_component_children(
                    component,
                ))
            }
        }
        BuiltinComponent::Skeleton => {
            if children.is_empty() {
                skeleton_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Modal => modal_component_node(
            props,
            Vec::new(),
            children,
            Vec::new(),
            allow_children,
        ),
        BuiltinComponent::AlertDialog => {
            if children.is_empty() {
                alert_dialog_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Tooltip => tooltip_component_node(props, children, allow_children),
        BuiltinComponent::Toast => {
            if children.is_empty() {
                toast_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Dropdown => Err(ComponentError::invalid_prop_combination(
            "Dropdown requires trigger and item entries",
        )),
        BuiltinComponent::Command => Err(ComponentError::invalid_prop_combination(
            "Command requires item or group entries",
        )),
        BuiltinComponent::AvatarGroup => {
            reject_children_placeholder(component, &children, allow_children)?;
            if children.is_empty() {
                avatar_group_component_node(props, Vec::new())
            } else {
                Err(ComponentError::invalid_prop_combination(
                    "AvatarGroup only accepts item entries",
                ))
            }
        }
        BuiltinComponent::ChatBox => {
            if children.is_empty() {
                chat_box_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Empty => {
            if children.is_empty() {
                empty_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Marquee => marquee_component_node(props, children, allow_children),
        BuiltinComponent::TypeWriter => Err(ComponentError::invalid_prop_combination(
            "TypeWriter requires item entries",
        )),
        BuiltinComponent::RichText => Err(ComponentError::invalid_prop_combination(
            "RichText requires mark entries",
        )),
        BuiltinComponent::Record => {
            if children.is_empty() {
                record_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::ToggleGroup => Err(ComponentError::invalid_prop_combination(
            "ToggleGroup requires item entries",
        )),
        BuiltinComponent::Collapsible => collapsible_component_node(props, children, allow_children),
        BuiltinComponent::Countdown => {
            if children.is_empty() {
                countdown_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Map => Err(ComponentError::invalid_prop_combination(
            "Map only accepts marker and waypoint entries",
        )),
        BuiltinComponent::Accordion => Err(ComponentError::invalid_prop_combination(
            "Accordion requires item entries",
        )),
        BuiltinComponent::Carousel => Err(ComponentError::invalid_prop_combination(
            "Carousel requires slide entries",
        )),
        BuiltinComponent::RadioGroup => Err(ComponentError::invalid_prop_combination(
            "RadioGroup requires item entries",
        )),
        BuiltinComponent::Audio => {
            if children.is_empty() {
                audio_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Image => {
            if children.is_empty() {
                image_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Checkbox => {
            if children.is_empty() {
                checkbox_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Color => {
            if children.is_empty() {
                color_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Date => {
            if children.is_empty() {
                date_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::DateRange => {
            if children.is_empty() {
                date_range_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Toggle => {
            if children.is_empty() {
                toggle_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Button => {
            let props = parse_variant_props(component, &props)?;
            reject_children_placeholder(component, &children, allow_children)?;
            if children.iter().all(is_text_like) {
                Ok(ViewNode::Button { props, children })
            } else {
                Err(ComponentError::text_cannot_contain_component_children(
                    component,
                ))
            }
        }
        BuiltinComponent::ToggleTheme => {
            if children.is_empty() {
                theme_toggle_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Fab => {
            reject_children_placeholder(component, &children, allow_children)?;
            fab_component_node(props, Vec::new())
        }
        BuiltinComponent::FabAction => Err(ComponentError::invalid_prop_combination(
            "fabAction can only be used inside Fab",
        )),
        BuiltinComponent::Input => {
            if children.is_empty() {
                input_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Slider => {
            if children.is_empty() {
                slider_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Dropzone => {
            if children.is_empty() {
                dropzone_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Select => Err(ComponentError::invalid_prop_combination(
            "Select can only contain Option children",
        )),
        BuiltinComponent::Option => Err(ComponentError::invalid_prop_combination(
            "Option can only be used inside Select",
        )),
        BuiltinComponent::Code => {
            if children.is_empty() {
                code_node(props, Vec::new())
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Video => {
            if children.is_empty() {
                video_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Candlestick => {
            if children.is_empty() {
                candlestick_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Table => Err(ComponentError::invalid_prop_combination(
            "Table requires column entries",
        )),
        BuiltinComponent::Tabs => Err(ComponentError::invalid_prop_combination(
            "Tabs requires tab entries",
        )),
        BuiltinComponent::Tab => Err(ComponentError::invalid_prop_combination(
            "tab can only be used inside Tabs",
        )),
        BuiltinComponent::NavMenu => Err(ComponentError::invalid_prop_combination(
            "NavMenu requires item, submenu or megamenu entries",
        )),
        BuiltinComponent::Divider => {
            if children.is_empty() {
                divider_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Alert => {
            if children.is_empty() {
                alert_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Svg => svg_component_node(props, Vec::new()),
        BuiltinComponent::Path => Err(ComponentError::invalid_prop_combination(
            "Path can only be used inside Svg",
        )),
        BuiltinComponent::AppBar | BuiltinComponent::Footer | BuiltinComponent::BottomBar => {
            Err(ComponentError::invalid_prop_combination(
                "bar components require start, center or end regions",
            ))
        }
        BuiltinComponent::SideNav => Err(ComponentError::invalid_prop_combination(
            "SideNav requires header, item, divider or submenu entries",
        )),
        BuiltinComponent::Sidebar => Err(ComponentError::invalid_prop_combination(
            "Sidebar requires header, item, divider or submenu entries",
        )),
        BuiltinComponent::Scaffold => Err(ComponentError::invalid_prop_combination(
            "Scaffold requires appBar, main and optional side regions",
        )),
        BuiltinComponent::Title | BuiltinComponent::Text => Err(
            ComponentError::text_cannot_contain_component_children(component),
        ),
    }
}

pub fn children_node(allow_children: bool) -> ComponentResult<ViewNode> {
    if allow_children {
        Ok(ViewNode::Children)
    } else {
        Err(ComponentError::children_outside_layout())
    }
}

pub fn first_text(node: &ViewNode) -> Option<String> {
    match node {
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => {
            children.iter().find_map(first_text)
        }
        ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Drawer { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Button { children, .. } => children.iter().find_map(first_text),
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => header
            .iter()
            .chain(body)
            .chain(footer)
            .find_map(first_text),
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            entries,
            ..
        } => trigger
            .iter()
            .chain(header)
            .chain(footer)
            .find_map(first_text)
            .or_else(|| entries.iter().find_map(overlay_entry_first_text)),
        ViewNode::Command { entries, .. } => entries.iter().find_map(command_entry_first_text),
        ViewNode::AvatarGroup { items, .. } => items.iter().find_map(|item| {
            item.name
                .clone()
                .or_else(|| item.alt.clone())
                .or_else(|| item.src.clone())
        }),
        ViewNode::ChatBox { props } => Some(props.assistant_name.clone()),
        ViewNode::Empty { props } => props
            .title
            .clone()
            .or_else(|| props.description.clone())
            .or_else(|| Some(props.action_label.clone())),
        ViewNode::TypeWriter { items, .. } => items.first().map(|item| item.text.clone()),
        ViewNode::RichText { marks, .. } => marks.first().map(|mark| mark.text.clone()),
        ViewNode::Record { props } => props.style.label.clone().or_else(|| Some(props.name.clone())),
        ViewNode::ToggleGroup { props, items } => props
            .style
            .label
            .clone()
            .or_else(|| items.first().map(|item| item.label.clone())),
        ViewNode::Collapsible {
            props, children, ..
        } => Some(props.label.clone())
            .or_else(|| children.iter().find_map(first_text)),
        ViewNode::Countdown { props } => Some(props.target.clone()),
        ViewNode::Map { markers, .. } => markers
            .iter()
            .find_map(|marker| marker.label.clone().or_else(|| marker.popup.clone())),
        ViewNode::Accordion { items, .. } => {
            items.iter().find_map(|item| Some(item.label.clone()))
        }
        ViewNode::Carousel { props, slides } => props.title.clone().or_else(|| {
            slides
                .iter()
                .find_map(|slide| slide.children.iter().find_map(first_text))
        }),
        ViewNode::Tabs { tabs, .. } => tabs
            .iter()
            .find_map(|tab| tab.children.iter().find_map(first_text)),
        ViewNode::NavMenu { items, .. } => items.iter().find_map(nav_menu_first_text),
        ViewNode::AppBar {
            start, center, end, ..
        }
        | ViewNode::Footer {
            start, center, end, ..
        }
        | ViewNode::BottomBar {
            start, center, end, ..
        } => start.iter().chain(center).chain(end).find_map(first_text),
        ViewNode::SideNav { items, .. } | ViewNode::Sidebar { items, .. } => {
            items.iter().find_map(side_nav_first_text)
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            ..
        } => app_bar
            .iter()
            .chain(start)
            .chain(main)
            .chain(end)
            .chain(bottom_bar)
            .find_map(first_text),
        ViewNode::Table { props } => props
            .columns
            .first()
            .map(|column| column.label.clone())
            .or_else(|| Some(props.empty_title.clone())),
        ViewNode::Title { value, .. } | ViewNode::Text { value, .. } => Some(value.clone()),
        ViewNode::Alert { props } => Some(props.message.clone()),
        ViewNode::Avatar { props, .. } => props.name.clone().or_else(|| Some(props.alt.clone())),
        ViewNode::Chip { value, .. } => Some(value.clone()),
        ViewNode::AlertDialog { props } => Some(props.title.clone()),
        ViewNode::Toast { props } => props
            .title
            .clone()
            .or_else(|| (!props.description.is_empty()).then(|| props.description.clone())),
        ViewNode::Audio { props } => props.subtitle.clone(),
        ViewNode::Image { props } => (!props.alt.is_empty()).then(|| props.alt.clone()),
        ViewNode::Checkbox { props } => props.style.label.clone(),
        ViewNode::Color { props } => props.style.label.clone(),
        ViewNode::Date { props } => props.style.label.clone(),
        ViewNode::DateRange { props } => props.style.label.clone(),
        ViewNode::RadioGroup { props, options } => props
            .style
            .label
            .clone()
            .or_else(|| options.first().map(|option| option.label.clone())),
        ViewNode::Toggle { props } => props.style.label.clone(),
        ViewNode::ToggleTheme { props } => Some(props.dark_label.clone()),
        ViewNode::Fab { props, actions } => props
            .style
            .label
            .clone()
            .or_else(|| actions.first().map(|action| action.label.clone())),
        ViewNode::Slider { props } => props.style.label.clone(),
        ViewNode::Dropzone { props } => props.style.label.clone(),
        ViewNode::Input { .. }
        | ViewNode::Select { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Children => None,
    }
}

pub fn node_element_props(node: &ViewNode) -> Option<&ElementProps> {
    match node {
        ViewNode::Scope { .. } | ViewNode::Each { .. } => None,
        ViewNode::Box { props, .. } | ViewNode::Section { props, .. } => Some(&props.element),
        ViewNode::Flex { props, .. } => Some(&props.style.element),
        ViewNode::Grid { props, .. } => Some(&props.style.element),
        ViewNode::Card { props, .. }
        | ViewNode::Button { props, .. }
        | ViewNode::Input { props }
        | ViewNode::Select { props, .. } => Some(&props.element),
        ViewNode::AvatarGroup { props, .. } => Some(&props.style.element),
        ViewNode::ChatBox { props } => Some(&props.style.element),
        ViewNode::Empty { props } => Some(&props.style.element),
        ViewNode::Marquee { props, .. } => Some(&props.style.element),
        ViewNode::TypeWriter { props, .. } => Some(&props.style.element),
        ViewNode::RichText { props, .. } => Some(&props.style.element),
        ViewNode::Record { props } => Some(&props.style.element),
        ViewNode::ToggleGroup { props, .. } => Some(&props.style.element),
        ViewNode::Collapsible { props, .. } => Some(&props.style.element),
        ViewNode::Countdown { props } => Some(&props.style.element),
        ViewNode::Map { props, .. } => Some(&props.style.element),
        ViewNode::ToggleTheme { props } => Some(&props.style.element),
        ViewNode::Fab { props, .. } => Some(&props.style.element),
        ViewNode::Slider { props } => Some(&props.style.element),
        ViewNode::Dropzone { props } => Some(&props.style.element),
        ViewNode::Avatar { props, .. } => Some(&props.style.element),
        ViewNode::Badge { props, .. } => Some(&props.style.element),
        ViewNode::Chip { props, .. } => Some(&props.style.element),
        ViewNode::Modal { props, .. } => Some(&props.style.element),
        ViewNode::AlertDialog { props } => Some(&props.style.element),
        ViewNode::Tooltip { props, .. } => Some(&props.style.element),
        ViewNode::Toast { props } => Some(&props.style.element),
        ViewNode::Dropdown { props, .. } => Some(&props.style.element),
        ViewNode::Command { props, .. } => Some(&props.style.element),
        ViewNode::Audio { props } => Some(&props.style.element),
        ViewNode::Image { props } => Some(&props.style.element),
        ViewNode::Accordion { props, .. } => Some(&props.style.element),
        ViewNode::Carousel { props, .. } => Some(&props.style.element),
        ViewNode::Checkbox { props } => Some(&props.style.element),
        ViewNode::Color { props } => Some(&props.style.element),
        ViewNode::Date { props } => Some(&props.style.element),
        ViewNode::DateRange { props } => Some(&props.style.element),
        ViewNode::RadioGroup { props, .. } => Some(&props.style.element),
        ViewNode::Toggle { props } => Some(&props.style.element),
        ViewNode::Skeleton { props } => Some(&props.style.element),
        ViewNode::Tabs { props, .. } => Some(&props.style.element),
        ViewNode::NavMenu { props, .. } => Some(&props.style.element),
        ViewNode::Code { props } => Some(&props.style.element),
        ViewNode::Video { props } => Some(&props.style.element),
        ViewNode::Candlestick { props } => Some(&props.style.element),
        ViewNode::Table { props } => Some(&props.style.element),
        ViewNode::Divider { props } => Some(&props.style.element),
        ViewNode::Alert { props } => Some(&props.style.element),
        ViewNode::Svg { props, .. } => Some(&props.style.element),
        ViewNode::AppBar { props, .. }
        | ViewNode::Footer { props, .. }
        | ViewNode::BottomBar { props, .. } => Some(&props.style.element),
        ViewNode::SideNav { props, .. } | ViewNode::Sidebar { props, .. } => {
            Some(&props.style.element)
        }
        ViewNode::Scaffold { props, .. } => Some(&props.style.element),
        ViewNode::Drawer { props, .. } => Some(&props.style.element),
        ViewNode::Title { props, .. } | ViewNode::Text { props, .. } => Some(&props.style.element),
        ViewNode::Children => None,
    }
}

pub fn navigation_action(node: &ViewNode) -> Option<&NavigationAction> {
    match node {
        ViewNode::Button { props, .. } => props.navigation.as_ref(),
        ViewNode::Avatar { props, .. } => props.style.navigation.as_ref(),
        ViewNode::Empty { props } => props.style.navigation.as_ref(),
        _ => None,
    }
}

pub fn node_children(node: &ViewNode) -> &[ViewNode] {
    match node {
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => children,
        ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Drawer { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Collapsible { children, .. }
        | ViewNode::Button { children, .. } => children,
        ViewNode::Modal { body, .. } => body,
        ViewNode::Tabs { .. }
        | ViewNode::NavMenu { .. }
        | ViewNode::Dropdown { .. }
        | ViewNode::Command { .. }
        | ViewNode::Accordion { .. }
        | ViewNode::Carousel { .. }
        | ViewNode::RadioGroup { .. } => &[],
        ViewNode::AppBar { .. }
        | ViewNode::Footer { .. }
        | ViewNode::BottomBar { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::Scaffold { .. } => &[],
        ViewNode::Input { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::Table { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Avatar { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::Children => &[],
    }
}

pub fn node_child_groups(node: &ViewNode) -> Vec<&[ViewNode]> {
    match node {
        ViewNode::AppBar {
            start, center, end, ..
        }
        | ViewNode::Footer {
            start, center, end, ..
        }
        | ViewNode::BottomBar {
            start, center, end, ..
        } => vec![start.as_slice(), center.as_slice(), end.as_slice()],
        ViewNode::Tabs { tabs, .. } => tabs
            .iter()
            .map(|tab| tab.children.as_slice())
            .collect::<Vec<_>>(),
        ViewNode::NavMenu { items, .. } => items.iter().filter_map(nav_menu_child_group).collect(),
        ViewNode::SideNav { .. } | ViewNode::Sidebar { .. } => Vec::new(),
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => vec![header.as_slice(), body.as_slice(), footer.as_slice()],
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => vec![trigger.as_slice(), header.as_slice(), footer.as_slice()],
        ViewNode::Command { .. } => Vec::new(),
        ViewNode::Accordion { items, .. } => items
            .iter()
            .map(|item| item.children.as_slice())
            .collect::<Vec<_>>(),
        ViewNode::Collapsible { children, .. } => vec![children.as_slice()],
        ViewNode::Carousel { slides, .. } => slides
            .iter()
            .map(|slide| slide.children.as_slice())
            .collect::<Vec<_>>(),
        ViewNode::RadioGroup { .. } | ViewNode::ToggleGroup { .. } => Vec::new(),
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            ..
        } => vec![
            app_bar.as_slice(),
            start.as_slice(),
            main.as_slice(),
            end.as_slice(),
            bottom_bar.as_slice(),
        ],
        _ => vec![node_children(node)],
    }
}

fn nav_menu_child_group(item: &NavMenuItem) -> Option<&[ViewNode]> {
    match item {
        NavMenuItem::Megamenu { content, .. } => Some(content.as_slice()),
        NavMenuItem::Item(_) | NavMenuItem::Submenu { .. } => None,
    }
}

fn overlay_entry_first_text(item: &OverlayEntry) -> Option<String> {
    match item {
        OverlayEntry::Item(props) => Some(props.label.clone()),
        OverlayEntry::Divider => None,
    }
}

fn command_entry_first_text(item: &CommandEntry) -> Option<String> {
    match item {
        CommandEntry::Item(props) => Some(props.label.clone()),
        CommandEntry::Group { label, items, .. } => items
            .iter()
            .find_map(|item| Some(item.label.clone()))
            .or_else(|| Some(label.clone())),
    }
}

fn side_nav_first_text(item: &SideNavItem) -> Option<String> {
    match item {
        SideNavItem::Header(props) | SideNavItem::Item(props) => Some(props.label.clone()),
        SideNavItem::Submenu { props, .. } => Some(props.label.clone()),
        SideNavItem::Divider => None,
    }
}

fn nav_menu_first_text(item: &NavMenuItem) -> Option<String> {
    match item {
        NavMenuItem::Item(props)
        | NavMenuItem::Submenu { props, .. }
        | NavMenuItem::Megamenu { props, .. } => Some(props.label.clone()),
    }
}

fn prop_value_string(name: &str, value: &PropValue) -> ComponentResult<String> {
    match value {
        PropValue::String(value) => Ok(value.clone()),
        PropValue::Boolean(value) => Ok(value.to_string()),
        PropValue::Number(value) => Ok(value.clone()),
        PropValue::Responsive(_) => Err(ComponentError::invalid_prop(name, "static scalar")),
    }
}
