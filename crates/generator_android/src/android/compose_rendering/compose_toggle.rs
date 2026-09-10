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
