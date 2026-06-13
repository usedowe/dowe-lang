fn render_compose_audio(props: &AudioProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            card_variant_content(&props.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweAudio(source = {}, subtitle = {}, avatarSource = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_string_literal(&props.src),
        compose_optional_string(props.subtitle.as_deref()),
        compose_optional_string(props.avatar_src.as_deref()),
        modifier_for_style(&props.style.style),
        compose_card_radius(&props.style.style),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
    ));
}

fn render_compose_image(props: &ImageProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            card_variant_content(&props.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweImage(source = {}, alt = {}, aspect = {}, objectFit = {}, loading = {}, hideControls = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_string_literal(&props.src),
        compose_string_literal(&props.alt),
        compose_string_literal(props.aspect.as_str()),
        compose_string_literal(props.object_fit.as_str()),
        compose_string_literal(props.loading.as_str()),
        props.hide_controls,
        modifier_for_style(&props.style.style),
        compose_card_radius(&props.style.style),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
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
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            variant_content(&props.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweAccordion(multiple = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border}) {{\n",
        props.multiple,
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    for item in items {
        output.push_str(&format!(
            "{pad}    DoweAccordionItem(id = {}, label = {}, disabled = {}, defaultOpen = {}) {{\n",
            compose_string_literal(&item.id),
            compose_string_literal(&item.label),
            item.disabled,
            item.default_open
        ));
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
        "{pad}DoweCarousel(autoplay = {}, autoplayInterval = {}, disableLoop = {}, hideControls = {}, hideIndicators = {}, showNavigation = {}, showCounter = {}, orientation = {}, size = {}, indicatorType = {}, title = {}, slideWidth = {}, slideHeight = {}, slidesPerView = {}, gap = {}, modifier = {}, accentColor = {}) {{\n",
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
    for slide in slides {
        output.push_str(&format!(
            "{pad}    DoweCarouselSlide(id = {}) {{\n",
            compose_string_literal(&slide.id)
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
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_theme_toggle(props: &ThemeToggleProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweThemeToggle(modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {})\n",
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_variant_border(&props.style)
    ));
}

fn render_compose_fab(
    props: &FabProps,
    actions: &[FabAction],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let modifier = if props.fixed {
        format!(
            "{}.fillMaxSize().padding(horizontal = {}, vertical = {})",
            modifier_for_style(&props.style.style),
            compose_scale_literal(props.offset_x),
            compose_scale_literal(props.offset_y)
        )
    } else {
        modifier_for_style(&props.style.style)
    };
    output.push_str(&format!(
        "{pad}Column(modifier = {modifier}, horizontalAlignment = {}, verticalArrangement = {}) {{\n",
        compose_fab_horizontal_alignment(props.position),
        compose_fab_vertical_arrangement(props.position)
    ));
    for action in actions {
        let action_props = VariantProps {
            color: Some(action.color),
            variant: props.style.variant,
            ..VariantProps::default()
        };
        output.push_str(&format!(
            "{pad}    Button(onClick = {}, colors = ButtonDefaults.buttonColors(containerColor = {}, contentColor = {}), border = {}, contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp)) {{ Text({}) }}\n",
            compose_component_action(action.on_click.as_deref(), action.navigation.as_ref(), context),
            variant_container(&action_props),
            variant_content(&action_props),
            compose_variant_border(&action_props),
            compose_string_literal(&action.label)
        ));
    }
    output.push_str(&format!(
        "{pad}    Button(onClick = {}, colors = ButtonDefaults.buttonColors(containerColor = {}, contentColor = {}), border = {}, contentPadding = PaddingValues(0.dp), modifier = Modifier.width(56.dp).height(56.dp)) {{ Text({}) }}\n",
        compose_component_action(props.style.element.on_click.as_deref(), props.style.navigation.as_ref(), context),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_string_literal(compose_view_icon_label(props.icon))
    ));
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_slider(
    props: &SliderProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let value = props.value.parse::<f32>().unwrap_or(0.0);
    let min = props.min.parse::<f32>().unwrap_or(0.0);
    let max = props.max.parse::<f32>().unwrap_or(100.0);
    let (value_expr, change_expr, bound) = props
        .style
        .element
        .bind
        .as_deref()
        .map(|path| {
            let path = escape_kotlin(&context.signal_path(path));
            (
                format!("state.text(\"{path}\").toFloatOrNull() ?: {value}f"),
                format!("{{ state.write(\"{path}\", it.toDouble()) }}"),
                "true",
            )
        })
        .unwrap_or_else(|| (format!("{value}f"), "{}".to_string(), "false"));
    output.push_str(&format!(
        "{pad}DoweSliderField(value = {value_expr}, onValueChange = {change_expr}, bound = {bound}, label = {}, hideLabel = {}, min = {min}f, max = {max}f, size = {}, modifier = {}, accentColor = {})\n",
        compose_optional_string(props.style.label.as_deref()),
        props.hide_label,
        compose_string_literal(props.size.as_str()),
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style)
    ));
}

fn render_compose_dropzone(props: &DropzoneProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweDropzone(label = {}, placeholder = {}, helpText = {}, errorText = {}, size = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Drag & drop files here or click to select")
        ),
        compose_optional_string(props.help_text.as_deref()),
        compose_optional_string(props.error_text.as_deref()),
        compose_string_literal(props.size.as_str()),
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_variant_border(&props.style)
    ));
}

fn render_compose_checkbox(
    props: &CheckboxProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (checked, change) = compose_bool_value_and_change(&props.style, props.checked, context);
    output.push_str(&format!(
        "{pad}DoweCheckbox(checked = {checked}, onCheckedChange = {change}, enabled = {}, label = {}, name = {}, modifier = {}, accentColor = {})\n",
        !props.disabled,
        compose_optional_string(props.style.label.as_deref()),
        compose_optional_string(props.name.as_deref()),
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style)
    ));
}

fn render_compose_color(
    props: &ColorProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (value, change) = compose_text_value_and_change(&props.style, &props.value, context);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined)
        == ComponentVariant::Outlined
    {
        color_ref(ColorToken::Muted)
    } else {
        "null"
    };
    output.push_str(&format!(
        "{pad}DoweColorField(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, floating = {}, size = {}, name = {}, helpText = {}, errorText = {}, showHex = {}, showRgb = {}, showCmyk = {}, showOklch = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Select color")),
        props.style.label_floating,
        compose_string_literal(props.size.as_str()),
        compose_optional_string(props.name.as_deref()),
        compose_optional_string(props.help_text.as_deref()),
        compose_optional_string(props.error_text.as_deref()),
        props.show_hex,
        props.show_rgb,
        props.show_cmyk,
        props.show_oklch,
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
}

fn render_compose_date(
    props: &DateProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (value, change) = compose_text_value_and_change(
        &props.style,
        props.value.as_deref().unwrap_or_default(),
        context,
    );
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined)
        == ComponentVariant::Outlined
    {
        color_ref(ColorToken::Muted)
    } else {
        "null"
    };
    output.push_str(&format!(
        "{pad}DoweDateField(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, floating = {}, size = {}, name = {}, helpText = {}, errorText = {}, min = {}, max = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Select date")),
        props.style.label_floating,
        compose_string_literal(props.size.as_str()),
        compose_optional_string(props.name.as_deref()),
        compose_optional_string(props.help_text.as_deref()),
        compose_optional_string(props.error_text.as_deref()),
        compose_optional_string(props.min.as_deref()),
        compose_optional_string(props.max.as_deref()),
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
}

fn render_compose_date_range(
    props: &DateRangeProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (start_value, start_change) = compose_optional_text_path_and_change(
        props.start.as_deref(),
        props.start_value.as_deref().unwrap_or_default(),
        context,
    );
    let (end_value, end_change) = compose_optional_text_path_and_change(
        props.end.as_deref(),
        props.end_value.as_deref().unwrap_or_default(),
        context,
    );
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined)
        == ComponentVariant::Outlined
    {
        color_ref(ColorToken::Muted)
    } else {
        "null"
    };
    output.push_str(&format!(
        "{pad}DoweDateRangeField(startValue = {start_value}, endValue = {end_value}, onStartChange = {start_change}, onEndChange = {end_change}, label = {}, placeholder = {}, floating = {}, size = {}, name = {}, helpText = {}, errorText = {}, min = {}, max = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Select date range")),
        props.style.label_floating,
        compose_string_literal(props.size.as_str()),
        compose_optional_string(props.name.as_deref()),
        compose_optional_string(props.help_text.as_deref()),
        compose_optional_string(props.error_text.as_deref()),
        compose_optional_string(props.min.as_deref()),
        compose_optional_string(props.max.as_deref()),
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
}

fn render_compose_radio_group(
    props: &RadioGroupProps,
    options: &[RadioOption],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (value, change) = compose_text_value_and_change(&props.style, "", context);
    output.push_str(&format!(
        "{pad}DoweRadioGroup(value = {value}, onValueChange = {change}, options = {}, size = {}, name = {}, label = {}, helpText = {}, errorText = {}, modifier = {}, accentColor = {})\n",
        compose_radio_options(options),
        compose_string_literal(props.size.as_str()),
        compose_optional_string(props.name.as_deref()),
        compose_optional_string(props.style.label.as_deref()),
        compose_optional_string(props.info.as_deref()),
        compose_optional_string(props.error.as_deref()),
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style)
    ));
}

fn render_compose_toggle(
    props: &ToggleProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (checked, change) = compose_bool_value_and_change(&props.style, props.checked, context);
    output.push_str(&format!(
        "{pad}DoweToggle(checked = {checked}, onCheckedChange = {change}, enabled = {}, label = {}, labelLeft = {}, labelRight = {}, name = {}, modifier = {}, accentColor = {})\n",
        !props.disabled,
        compose_optional_string(props.style.label.as_deref()),
        compose_optional_string(props.label_left.as_deref()),
        compose_optional_string(props.label_right.as_deref()),
        compose_optional_string(props.name.as_deref()),
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style)
    ));
}
