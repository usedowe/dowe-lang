fn render_field_control(
    base: &str,
    props: &VariantProps,
    size: ButtonSize,
    help_text: Option<&str>,
    error_text: Option<&str>,
    control_html: &str,
    has_start_adornment: bool,
    has_value: bool,
    context: &ReactiveRenderContext,
) -> String {
    let validation = props.element.form_validation();
    let help_text = validation
        .and_then(|validation| validation.help_text.as_deref())
        .or(help_text);
    let error_text = validation
        .and_then(|validation| validation.error_text.as_deref())
        .or(error_text);
    let mut classes = variant_classes("control", props);
    classes.push(base.to_string());
    classes.push(format!("is-{}", size.as_str()));
    if props.label_floating {
        classes.push("is-floating".to_string());
    }
    if has_start_adornment {
        classes.push("has-start-adornment".to_string());
    }
    if has_value {
        classes.push("has-value".to_string());
    }
    if error_text.is_some() {
        classes.push("is-error".to_string());
    }
    let control = format!(
        "<span{}>{}{}</span>",
        attrs(classes, Some(&props.element), None, context),
        floating_label_html(props),
        control_html
    );
    render_field_block(props, help_text, error_text, &control, context)
}

