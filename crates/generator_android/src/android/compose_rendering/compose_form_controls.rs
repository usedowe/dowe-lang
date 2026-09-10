fn render_compose_checkbox(
    props: &CheckboxProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (checked, change) = compose_bool_value_and_change(&props.style, props.checked, context);
    output.push_str(&format!(
        "{pad}DoweCheckbox(checked = {checked}, onCheckedChange = {change}, enabled = {}, label = {}, name = {}, modifier = {}, accentColor = {}, helpText = {}, errorText = {}, validationRules = {})\n",
        !props.disabled,
        compose_optional_string(props.style.label.as_deref()),
        compose_optional_string(props.name.as_deref()),
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style),
        compose_validation_help(&props.style.element),
        compose_validation_error(&props.style.element),
        compose_boolean_validation_rules(&props.style.element, context)
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
    let text_size = form_control_text_size(props.size);
    let font_size = compose_text_size_expr(false, text_size);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined)
        == ComponentVariant::Outlined
    {
        color_ref(ColorToken::Muted)
    } else {
        "null"
    };
    output.push_str(&format!(
        "{pad}DoweColorField(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, floating = {}, size = {}, fontSize = {font_size}, lineHeight = doweTextLineHeight({font_size}, {}f), name = {}, helpText = {}, errorText = {}, showHex = {}, showRgb = {}, showCmyk = {}, showOklch = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Select color")),
        props.style.label_floating,
        compose_string_literal(props.size.as_str()),
        text_typography(false, text_size).line_height,
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
    let text_size = form_control_text_size(props.size);
    let font_size = compose_text_size_expr(false, text_size);
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
        "{pad}DoweDateField(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, floating = {}, size = {}, fontSize = {font_size}, lineHeight = doweTextLineHeight({font_size}, {}f), name = {}, helpText = {}, errorText = {}, min = {}, max = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border}, validationRules = {})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Select date")),
        props.style.label_floating,
        compose_string_literal(props.size.as_str()),
        text_typography(false, text_size).line_height,
        compose_optional_string(props.name.as_deref()),
        compose_optional_string(props.help_text.as_deref()),
        compose_optional_string(props.error_text.as_deref()),
        compose_optional_string(props.min.as_deref()),
        compose_optional_string(props.max.as_deref()),
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_validation_rules(&props.style.element, context),
    ));
}

fn render_compose_date_range(
    props: &DateRangeProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let text_size = form_control_text_size(props.size);
    let font_size = compose_text_size_expr(false, text_size);
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
        "{pad}DoweDateRangeField(startValue = {start_value}, endValue = {end_value}, onStartChange = {start_change}, onEndChange = {end_change}, label = {}, placeholder = {}, floating = {}, size = {}, fontSize = {font_size}, lineHeight = doweTextLineHeight({font_size}, {}f), name = {}, helpText = {}, errorText = {}, min = {}, max = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Select date range")),
        props.style.label_floating,
        compose_string_literal(props.size.as_str()),
        text_typography(false, text_size).line_height,
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
    if matches!(props.presentation, RadioGroupPresentation::Card) {
        return render_compose_radio_card(props, options, indent, output, context);
    }
    let pad = " ".repeat(indent);
    let (value, change) = compose_text_value_and_change(&props.style, "", context);
    output.push_str(&format!(
        "{pad}DoweRadioGroup(value = {value}, onValueChange = {change}, options = {}, size = {}, orientation = {}, name = {}, label = {}, helpText = {}, errorText = {}, modifier = {}, accentColor = {})\n",
        compose_radio_options(options),
        compose_string_literal(props.size.as_str()),
        compose_string_literal(props.orientation.as_str()),
        compose_optional_string(props.name.as_deref()),
        compose_optional_string(props.style.label.as_deref()),
        compose_optional_string(props.info.as_deref()),
        compose_optional_string(props.error.as_deref()),
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style)
    ));
}

fn render_compose_radio_card(
    props: &RadioGroupProps,
    options: &[RadioOption],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (value, change) = compose_text_value_and_change(&props.style, "", context);
    output.push_str(&format!(
        "{pad}DoweRadioCard(value = {value}, onValueChange = {change}, options = {}, size = {}, orientation = {}, name = {}, label = {}, helpText = {}, errorText = {}, modifier = {}, accentColor = {})\n",
        compose_radio_card_options(options),
        compose_string_literal(props.size.as_str()),
        compose_string_literal(props.orientation.as_str()),
        compose_optional_string(props.name.as_deref()),
        compose_optional_string(props.style.label.as_deref()),
        compose_optional_string(props.info.as_deref()),
        compose_optional_string(props.error.as_deref()),
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style)
    ));
}

