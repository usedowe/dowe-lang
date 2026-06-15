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
            src: src
                .ok_or_else(|| ComponentError::invalid_prop("src", "asset path or https URL"))?,
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
            src: src
                .ok_or_else(|| ComponentError::invalid_prop("src", "asset path or https URL"))?,
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
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Accordion,
                    &prop.name,
                ));
            }
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
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Carousel,
                    &prop.name,
                ));
            }
        }
    }
    Ok(CarouselSlide {
        id: id.ok_or_else(|| ComponentError::invalid_prop("id", "static string or number"))?,
        children,
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
