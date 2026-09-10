fn render_swift_pin(
    props: &PinProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
let size = props.style.size.unwrap_or(ButtonSize::Md);
let text_size = form_control_text_size(size);
let base_border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
    == ComponentVariant::Outlined
{
    (
        format!("Optional({})", color_ref(ColorToken::Muted)),
        "CGFloat(1)".to_string(),
    )
} else {
    ("nil".to_string(), "CGFloat(0)".to_string())
};
let (border, border_width) =
    swift_style_border(&props.style.style, &base_border.0, &base_border.1);
output.push_str(&format!(
    "{pad}DowePin(value: {}, initialValue: {}, label: {}, length: {}, kind: {}, size: {}, fontSize: {}, lineHeight: CGFloat({}), variant: {}, helpText: {}, errorText: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, borderWidth: {}, radius: {}, validationRules: {})\n",
    swift_text_binding(props.style.element.bind.as_deref(), context),
    swift_string_literal(props.value.as_deref().unwrap_or_default()),
    swift_optional_literal(props.style.label.as_deref()),
    props.length,
    swift_string_literal(props.kind.as_str()),
    swift_string_literal(size.as_str()),
    swift_text_size_expr(false, text_size),
    text_typography(false, text_size).line_height,
    swift_string_literal(props.style.variant.unwrap_or(ComponentVariant::Solid).as_str()),
    swift_optional_literal(props.help_text.as_deref()),
    swift_optional_literal(props.error_text.as_deref()),
    variant_container(&props.style),
    variant_content(&props.style),
    border,
    border_width,
    swift_control_radius(&props.style.style),
    swift_validation_rules(&props.style.element, context, false)
));
append_swift_modifiers(
    output,
    indent,
    &swift_modifiers_for_style(&props.style.style),
);
}

fn render_swift_textarea(
    props: &TextareaProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
let text_size = form_control_text_size(props.style.size.unwrap_or(ButtonSize::Md));
output.push_str(&format!(
    "{pad}DoweTextarea(value: {}, initialValue: {}, label: {}, placeholder: {}, floating: {}, rows: {}, maxLength: {}, fontSize: {}, lineHeight: CGFloat({}), readOnly: {}, backgroundColor: {}, contentColor: {})\n",
    swift_text_binding(props.style.element.bind.as_deref(), context),
    swift_string_literal(props.value.as_deref().unwrap_or_default()),
    swift_optional_literal(props.style.label.as_deref()),
    swift_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
    props.style.label_floating,
    props.rows,
    props.max_length
        .map(|value| value.to_string())
        .unwrap_or_else(|| "nil".to_string()),
    swift_text_size_expr(false, text_size),
    text_typography(false, text_size).line_height,
    props.readonly || props.disabled,
    variant_container(&props.style),
    variant_content(&props.style)
));
append_swift_modifiers(
    output,
    indent,
    &swift_modifiers_for_style(&props.style.style),
);
}
