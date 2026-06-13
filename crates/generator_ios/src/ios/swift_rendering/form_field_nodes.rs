fn render_swift_checkbox(
    props: &CheckboxProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = swift_bool_binding(&props.style, props.checked, context);
    output.push_str(&format!(
        "{pad}DoweCheckboxView(checked: {binding}, enabled: {}, label: {}, name: {}, accentColor: {})\n",
        !props.disabled,
        swift_optional_literal(props.style.label.as_deref()),
        swift_optional_literal(props.name.as_deref()),
        swift_scheme_color(&props.style)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_color(
    props: &ColorProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = swift_string_binding(&props.style, &props.value, context);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined)
        == ComponentVariant::Outlined
    {
        format!("Optional({})", color_ref(ColorToken::Muted))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweColorField(value: {binding}, label: {}, placeholder: {}, floating: {}, size: {}, name: {}, helpText: {}, errorText: {}, showHex: {}, showRgb: {}, showCmyk: {}, showOklch: {}, backgroundColor: {}, contentColor: {}, borderColor: {border})\n",
        swift_optional_literal(props.style.label.as_deref()),
        swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Select color")),
        props.style.label_floating,
        swift_string_literal(props.size.as_str()),
        swift_optional_literal(props.name.as_deref()),
        swift_optional_literal(props.help_text.as_deref()),
        swift_optional_literal(props.error_text.as_deref()),
        props.show_hex,
        props.show_rgb,
        props.show_cmyk,
        props.show_oklch,
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_date(
    props: &DateProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = swift_string_binding(
        &props.style,
        props.value.as_deref().unwrap_or_default(),
        context,
    );
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined)
        == ComponentVariant::Outlined
    {
        format!("Optional({})", color_ref(ColorToken::Muted))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweDateField(value: {binding}, label: {}, placeholder: {}, floating: {}, size: {}, name: {}, helpText: {}, errorText: {}, min: {}, max: {}, backgroundColor: {}, contentColor: {}, borderColor: {border})\n",
        swift_optional_literal(props.style.label.as_deref()),
        swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Select date")),
        props.style.label_floating,
        swift_string_literal(props.size.as_str()),
        swift_optional_literal(props.name.as_deref()),
        swift_optional_literal(props.help_text.as_deref()),
        swift_optional_literal(props.error_text.as_deref()),
        swift_optional_literal(props.min.as_deref()),
        swift_optional_literal(props.max.as_deref()),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_date_range(
    props: &DateRangeProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let start_binding = swift_optional_string_binding(
        props.start.as_deref(),
        props.start_value.as_deref().unwrap_or_default(),
        context,
    );
    let end_binding = swift_optional_string_binding(
        props.end.as_deref(),
        props.end_value.as_deref().unwrap_or_default(),
        context,
    );
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined)
        == ComponentVariant::Outlined
    {
        format!("Optional({})", color_ref(ColorToken::Muted))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweDateRangeField(startValue: {start_binding}, endValue: {end_binding}, label: {}, placeholder: {}, floating: {}, size: {}, name: {}, helpText: {}, errorText: {}, min: {}, max: {}, backgroundColor: {}, contentColor: {}, borderColor: {border})\n",
        swift_optional_literal(props.style.label.as_deref()),
        swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Select date range")),
        props.style.label_floating,
        swift_string_literal(props.size.as_str()),
        swift_optional_literal(props.name.as_deref()),
        swift_optional_literal(props.help_text.as_deref()),
        swift_optional_literal(props.error_text.as_deref()),
        swift_optional_literal(props.min.as_deref()),
        swift_optional_literal(props.max.as_deref()),
        variant_container(&props.style),
        variant_content(&props.style),
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_radio_group(
    props: &RadioGroupProps,
    options: &[RadioOption],
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = swift_string_binding(&props.style, "", context);
    output.push_str(&format!(
        "{pad}DoweRadioGroupView(value: {binding}, options: {}, size: {}, name: {}, label: {}, helpText: {}, errorText: {}, accentColor: {})\n",
        swift_radio_options(options),
        swift_string_literal(props.size.as_str()),
        swift_optional_literal(props.name.as_deref()),
        swift_optional_literal(props.style.label.as_deref()),
        swift_optional_literal(props.info.as_deref()),
        swift_optional_literal(props.error.as_deref()),
        swift_scheme_color(&props.style)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_toggle(
    props: &ToggleProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let binding = swift_bool_binding(&props.style, props.checked, context);
    output.push_str(&format!(
        "{pad}DoweToggleView(checked: {binding}, enabled: {}, label: {}, labelLeft: {}, labelRight: {}, name: {}, accentColor: {})\n",
        !props.disabled,
        swift_optional_literal(props.style.label.as_deref()),
        swift_optional_literal(props.label_left.as_deref()),
        swift_optional_literal(props.label_right.as_deref()),
        swift_optional_literal(props.name.as_deref()),
        swift_scheme_color(&props.style)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}
