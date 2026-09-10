fn render_swift_input_node(
    props: &VariantProps,
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
let binding = props
    .element
    .bind
    .as_deref()
    .map(|path| {
        format!(
            "state.binding(\"{}\")",
            escape_swift(&context.signal_path(path))
        )
    })
    .unwrap_or_else(|| "nil".to_string());
let control_size = props.size.unwrap_or(ButtonSize::Md);
let text_size = form_control_text_size(control_size);
let size = swift_text_size_expr(false, text_size);
let base_border =
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        (
            format!("Optional({})", color_ref(ColorToken::Muted)),
            "CGFloat(1)".to_string(),
        )
    } else {
        ("nil".to_string(), "CGFloat(0)".to_string())
    };
let (border, border_width) =
    swift_style_border(&props.style, &base_border.0, &base_border.1);
let shadow = swift_shadow_spec(&props.style)
    .map(|value| format!("Optional({value})"))
    .unwrap_or_else(|| "nil".to_string());
output.push_str(&format!(
    "{pad}DoweInputField(value: {binding}, label: {}, placeholder: {}, floating: {}, font: {}, fontSize: {size}, lineHeight: CGFloat({}), minHeight: CGFloat({}), horizontalPadding: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, borderWidth: {border_width}, radius: {}, shadow: {shadow}, startIcon: {}, endIcon: {}, helpText: {}, errorText: {}, validationRules: {})\n",
    swift_optional_literal(props.label.as_deref()),
    swift_string_literal(props.placeholder.as_deref().unwrap_or_default()),
    props.label_floating,
    swift_font_value(
        props.style.font.as_ref().or(inherited_font),
        &size,
        default_family,
    ),
    text_typography(false, text_size).line_height,
    form_control_min_height(control_size, props.label_floating)
    .native_units(),
    INPUT_HORIZONTAL_PADDING.native_units(),
    variant_container(props),
    variant_content(props),
    swift_control_radius(&props.style),
    swift_control_icon(props.icon_start.as_ref()),
    swift_control_icon(props.icon_end.as_ref()),
    swift_validation_help(&props.element),
    swift_validation_error(&props.element),
    swift_validation_rules(&props.element, context, false)
));
let mut input_style = props.style.clone();
input_style.shadow = None;
input_style.shadow_color = None;
input_style.rounded = None;
input_style.border = None;
input_style.border_color = None;
append_swift_modifiers(output, indent, &swift_modifiers_for_style(&input_style));
}
