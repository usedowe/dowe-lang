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
    require_solid_variant(BuiltinComponent::Audio, style.variant)?;
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
    let mut reactive_src = None;
    let mut alt = String::new();
    let mut aspect = ImageAspect::Auto;
    let mut object_fit = ImageObjectFit::Cover;
    let mut loading = ImageLoading::Lazy;
    let mut hide_controls = true;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "src" => {
                if let Some(path) = reactive_reference(&prop.value) {
                    reactive_src = Some(path);
                } else {
                    src = Some(parse_media_source(&prop.name, &prop.value)?);
                }
            }
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
            src: match (src, reactive_src.as_ref()) {
                (Some(src), _) => src,
                (None, Some(_)) => String::new(),
                (None, None) => {
                    return Err(ComponentError::invalid_prop(
                        "src",
                        "asset path, https URL or Signal path",
                    ));
                }
            },
            reactive_src,
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
    mut items: Vec<AccordionItem>,
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
    let style = parse_variant_props(BuiltinComponent::Accordion, &style_props)?;
    if !multiple {
        let mut found_open = false;
        for item in &mut items {
            if item.default_open {
                item.default_open = !found_open;
                found_open = true;
            }
        }
    }
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
    let mut variant = CarouselVariant::Simple;
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
            "variant" => variant = parse_carousel_variant(&prop.name, &prop.value)?,
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
            variant,
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

