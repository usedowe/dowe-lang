fn render_field_block_kind(
    props: &VariantProps,
    help_text: Option<&str>,
    error_text: Option<&str>,
    body_html: &str,
    value_kind: &str,
    context: &ReactiveRenderContext,
) -> String {
    let validation = props.element.form_validation();
    let help_text = validation
        .and_then(|validation| validation.help_text.as_deref())
        .or(help_text);
    let error_text = validation
        .and_then(|validation| validation.error_text.as_deref())
        .or(error_text);
    let label = if props.label.is_some() && !props.label_floating {
        format!(
            r#"<span class="field-label">{}</span>"#,
            escape_html(props.label.as_deref().unwrap_or_default())
        )
    } else {
        String::new()
    };
    let message = error_text.or(help_text);
    let has_rules = validation.is_some_and(|validation| !validation.rules.is_empty());
    let help = if message.is_some() || has_rules {
        format!(
            r#"<span class="field-help{}" data-dowe-validation-feedback{}>{}</span>"#,
            if error_text.is_some() {
                " is-error"
            } else {
                ""
            },
            if message.is_none() { " hidden" } else { "" },
            escape_html(message.unwrap_or_default())
        )
    } else {
        String::new()
    };
    let validation_attrs =
        render_form_validation_attrs(&props.element, help_text, error_text, value_kind, context);
    let mut field_classes = vec!["field".to_string()];
    append_responsive_classes(
        &mut field_classes,
        "w",
        props.style.sizing.w.as_ref(),
        size_suffix,
    );
    append_responsive_classes(
        &mut field_classes,
        "min-w",
        props.style.sizing.min_w.as_ref(),
        size_suffix,
    );
    append_responsive_classes(
        &mut field_classes,
        "max-w",
        props.style.sizing.max_w.as_ref(),
        size_suffix,
    );
    format!(
        r#"<div{}>{}{body_html}{}</div>"#,
        attrs(
            field_classes,
            Some(&props.element),
            Some(&validation_attrs),
            context,
        ),
        label,
        help
    )
}

