fn render_textarea_html(props: &TextareaProps, context: &ReactiveRenderContext) -> String {
    let control = format!(
        r#"<textarea class="textarea-control input" rows="{}"{}{}{}{}{}{}>{}</textarea>"#,
        props.rows,
        props
            .cols
            .map(|value| format!(r#" cols="{value}""#))
            .unwrap_or_default(),
        props
            .max_length
            .map(|value| format!(r#" maxlength="{value}""#))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        input_placeholder_attr(&props.style),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.disabled {
            " disabled"
        } else if props.readonly {
            " readonly"
        } else {
            ""
        },
        escape_html(props.value.as_deref().unwrap_or_default())
    );
    let mut html = render_field_control(
        "textarea-field",
        &props.style,
        props.style.size.unwrap_or(ButtonSize::Md),
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &control,
        false,
        props
            .value
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        context,
    );
    if props.resize {
        html = html.replace(
            "textarea-control input",
            "textarea-control input is-resizable",
        );
    }
    html
}

