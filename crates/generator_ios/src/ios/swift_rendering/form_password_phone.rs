fn render_swift_password(
    props: &PasswordProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
let show_icon = solar_control_icon("eye").expect("bundled Password reveal icon");
let hide_icon =
    solar_control_icon("eye-closed").expect("bundled Password conceal icon");
let control_size = props.style.size.unwrap_or(ButtonSize::Md);
let text_size = form_control_text_size(control_size);
output.push_str(&format!(
    "{pad}DowePassword(value: {}, initialValue: {}, label: {}, placeholder: {}, floating: {}, minHeight: CGFloat({}), fontSize: {}, lineHeight: CGFloat({}), hideStrength: {}, weakLabel: {}, mediumLabel: {}, strongLabel: {}, readOnly: {}, showIcon: {}, hideIcon: {}, backgroundColor: {}, contentColor: {}, helpText: {}, errorText: {}, validationRules: {})\n",
    swift_text_binding(props.style.element.bind.as_deref(), context),
    swift_string_literal(props.value.as_deref().unwrap_or_default()),
    swift_optional_literal(props.style.label.as_deref()),
    swift_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
    props.style.label_floating,
    form_control_min_height(control_size, props.style.label_floating)
    .native_units(),
    swift_text_size_expr(false, text_size),
    text_typography(false, text_size).line_height,
    props.hide_strength,
    swift_string_literal(&props.weak_label),
    swift_string_literal(&props.medium_label),
    swift_string_literal(&props.strong_label),
    props.readonly || props.disabled,
    swift_control_icon(Some(&show_icon)),
    swift_control_icon(Some(&hide_icon)),
    variant_container(&props.style),
    variant_content(&props.style),
    swift_optional_literal(props.help_text.as_deref()),
    swift_optional_literal(props.error_text.as_deref()),
    swift_validation_rules(&props.style.element, context, false)
));
append_swift_modifiers(
    output,
    indent,
    &swift_modifiers_for_style(&props.style.style),
);
}

fn render_swift_phone(
    props: &PhoneProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
let control_size = props.style.size.unwrap_or(ButtonSize::Md);
let text_size = form_control_text_size(control_size);
output.push_str(&format!(
    "{pad}DowePhone(value: {}, initialValue: {}, label: {}, placeholder: {}, country: {}, countries: {}, priorityCountries: {}, dialCodeName: {}, searchPlaceholder: {}, emptyText: {}, loadingText: {}, floating: {}, minHeight: CGFloat({}), fontSize: {}, lineHeight: CGFloat({}), disabled: {}, backgroundColor: {}, contentColor: {}, helpText: {}, errorText: {}, validationRules: {})\n",
    swift_text_binding(props.style.element.bind.as_deref(), context),
    swift_string_literal(props.value.as_deref().unwrap_or_default()),
    swift_optional_literal(props.style.label.as_deref()),
    swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Enter phone number")),
    swift_string_literal(props.country.as_deref().unwrap_or("US")),
    "DowePhoneCatalog.countries",
    swift_string_array(&props.priority_countries),
    swift_string_literal(&props.dial_code_name),
    swift_string_literal(&props.search_placeholder),
    swift_string_literal(&props.empty_text),
    swift_string_literal(&props.loading_text),
    props.style.label_floating,
    form_control_min_height(control_size, props.style.label_floating)
    .native_units(),
    swift_text_size_expr(false, text_size),
    text_typography(false, text_size).line_height,
    props.disabled,
    variant_container(&props.style),
    variant_content(&props.style),
    swift_optional_literal(props.help_text.as_deref()),
    swift_optional_literal(props.error_text.as_deref()),
    swift_validation_rules(&props.style.element, context, false)
));
append_swift_modifiers(
    output,
    indent,
    &swift_modifiers_for_style(&props.style.style),
);
}
