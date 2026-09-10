fn render_compose_audio(props: &AudioProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    let play = solar_control_icon("play").expect("bundled Audio play icon");
    let pause = solar_control_icon("pause").expect("bundled Audio pause icon");
    let mut button_style = props.style.clone();
    button_style.variant = Some(
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Solid {
            ComponentVariant::Solid
        } else {
            ComponentVariant::Solid
        },
    );
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            card_variant_content(&props.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweAudio(source = {}, subtitle = {}, avatarSource = {}, playIconViewBox = {}, playIconPaths = {}, pauseIconViewBox = {}, pauseIconPaths = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, buttonBackgroundColor = {}, buttonContentColor = {}, borderColor = {border})\n",
        compose_string_literal(&props.src),
        compose_optional_string(props.subtitle.as_deref()),
        compose_optional_string(props.avatar_src.as_deref()),
        compose_svg_view_box(&play.props.view_box),
        compose_svg_paths(&play.paths),
        compose_svg_view_box(&pause.props.view_box),
        compose_svg_paths(&pause.paths),
        modifier_for_style(&props.style.style),
        compose_card_radius(&props.style.style),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        variant_container(&button_style),
        variant_content(&button_style),
    ));
}

fn render_compose_camera(
    props: &CameraProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweCamera(state = state, facing = {}, label = {}, disabled = {}, onStart = {}, onCapture = {}, onError = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {})\n",
        compose_string_literal(props.facing.as_str()),
        compose_string_literal(&props.label),
        props.disabled,
        compose_optional_string(props.on_start.as_deref().and_then(|value| context.action_id(value))),
        compose_optional_string(props.on_capture.as_deref().and_then(|value| context.action_id(value))),
        compose_optional_string(props.on_error.as_deref().and_then(|value| context.action_id(value))),
        modifier_for_style(&props.style.style),
        compose_card_radius(&props.style.style),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
    ));
}

fn render_compose_microphone(
    props: &MicrophoneProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let max_duration = props
        .max_duration
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string());
    output.push_str(&format!(
        "{pad}DoweMicrophone(state = state, label = {}, maxDuration = {}, disabled = {}, onStart = {}, onStop = {}, onError = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {})\n",
        compose_string_literal(&props.label),
        max_duration,
        props.disabled,
        compose_optional_string(props.on_start.as_deref().and_then(|value| context.action_id(value))),
        compose_optional_string(props.on_stop.as_deref().and_then(|value| context.action_id(value))),
        compose_optional_string(props.on_error.as_deref().and_then(|value| context.action_id(value))),
        modifier_for_style(&props.style.style),
        compose_card_radius(&props.style.style),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
    ));
}

fn compose_image_source(props: &ImageProps, context: &ComposeReactiveContext) -> String {
    let Some(binding) = props.reactive_src.as_deref() else {
        return compose_string_literal(&props.src);
    };
    let Some(path) = context.dynamic_path(binding) else {
        return compose_string_literal(&props.src);
    };
    context
        .item_value(binding)
        .map(|item| format!("state.text(\"{}\", {item})", escape_kotlin(&path)))
        .unwrap_or_else(|| format!("state.text(\"{}\")", escape_kotlin(&path)))
}

fn render_compose_image(
    props: &ImageProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            card_variant_content(&props.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweImage(source = {}, alt = {}, aspect = {}, objectFit = {}, loading = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, borderColor = {border})\n",
        compose_image_source(props, context),
        compose_string_literal(&props.alt),
        compose_string_literal(props.aspect.as_str()),
        compose_string_literal(props.object_fit.as_str()),
        compose_string_literal(props.loading.as_str()),
        modifier_for_style(&props.style.style),
        compose_card_radius(&props.style.style),
        card_variant_container(&props.style),
    ));
}

fn render_compose_accordion(
    props: &AccordionProps,
    items: &[AccordionItem],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let mut style = props.style.clone();
    style.variant.get_or_insert(ComponentVariant::Ghost);
    let variant = style.variant.unwrap_or(ComponentVariant::Ghost);
    let arrow = side_nav_submenu_arrow_icon();
    let default_open_ids = items
        .iter()
        .filter(|item| item.default_open)
        .map(|item| compose_string_literal(&item.id))
        .collect::<Vec<_>>()
        .join(", ");
    let border = if variant == ComponentVariant::Outlined {
        color_ref(family_color(style.color.unwrap_or(ColorFamily::Primary)))
    } else {
        "null"
    };
    let item_background = match variant {
        ComponentVariant::Solid | ComponentVariant::Outlined => color_ref(ColorToken::Surface),
        _ => "Color.Transparent",
    };
    let item_border = match variant {
        ComponentVariant::Solid => card_variant_content(&style),
        ComponentVariant::Outlined => {
            color_ref(family_color(style.color.unwrap_or(ColorFamily::Primary)))
        }
        ComponentVariant::Ghost | ComponentVariant::Line => card_variant_content(&style),
    };
    let item_border_alpha = match variant {
        ComponentVariant::Solid => "0.16f",
        ComponentVariant::Outlined => "0.24f",
        ComponentVariant::Ghost => "0.22f",
        _ => "0.24f",
    };
    let item_radius = if matches!(variant, ComponentVariant::Ghost | ComponentVariant::Line) {
        "0.dp".to_string()
    } else {
        format!("({}) * 0.85f", compose_card_radius(&style.style))
    };
    output.push_str(&format!(
        "{pad}DoweAccordion(multiple = {}, variant = {}, defaultOpenIds = setOf({default_open_ids}), modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border}, itemBackgroundColor = {}, itemBorderColor = {}, itemBorderAlpha = {}, radius = {}) {{ openIds, toggleItem ->\n",
        props.multiple,
        compose_string_literal(variant.as_str()),
        modifier_for_style(&style.style),
        card_variant_container(&style),
        card_variant_content(&style),
        item_background,
        item_border,
        item_border_alpha,
        compose_card_radius(&style.style),
    ));
    for item in items {
        output.push_str(&format!(
            "{pad}    DoweAccordionItem(label = {}, disabled = {}, open = openIds.contains({}), backgroundColor = {}, borderColor = {}, borderAlpha = {}, radius = {item_radius}, onToggle = {{ toggleItem({}) }}, arrowIcon = {{\n",
            compose_string_literal(&item.label),
            item.disabled,
            compose_string_literal(&item.id),
            item_background,
            item_border,
            item_border_alpha,
            compose_string_literal(&item.id),
        ));
        render_compose_side_icon(&arrow, indent + 8, output);
        output.push_str(&format!("{pad}    }}) {{\n"));
        for child in &item.children {
            render_compose_node_in_flow(
                child,
                indent + 8,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_carousel(
    props: &CarouselProps,
    slides: &[CarouselSlide],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweCarousel(variant = {}, slides = listOf(\n",
        compose_string_literal(props.variant.as_str()),
    ));
    for (index, slide) in slides.iter().enumerate() {
        output.push_str(&format!(
            "{pad}    DoweCarouselSlideSpec(id = {}, content = {{\n",
            compose_string_literal(&slide.id),
        ));
        for child in &slide.children {
            render_compose_node_in_flow(
                child,
                indent + 8,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!(
            "{pad}    }}){}\n",
            if index + 1 == slides.len() { "" } else { "," }
        ));
    }
    output.push_str(&format!(
        "{pad}), autoplay = {}, autoplayInterval = {}, disableLoop = {}, hideControls = {}, hideIndicators = {}, showNavigation = {}, showCounter = {}, orientation = {}, size = {}, indicatorType = {}, title = {}, slideWidth = {}, slideHeight = {}, slidesPerView = {}, gap = {}, modifier = {}, accentColor = {})\n",
        props.autoplay,
        props.autoplay_interval,
        props.disable_loop,
        props.hide_controls,
        props.hide_indicators,
        props.show_navigation,
        props.show_counter,
        compose_string_literal(props.orientation.as_str()),
        compose_string_literal(props.size.as_str()),
        compose_string_literal(props.indicator_type.as_str()),
        compose_optional_string(props.title.as_deref()),
        compose_optional_u16(props.slide_width),
        compose_optional_u16(props.slide_height),
        props.slides_per_view,
        props.gap,
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style),
    ));
}

