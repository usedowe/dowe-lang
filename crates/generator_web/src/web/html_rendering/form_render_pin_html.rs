fn render_pin_html(props: &PinProps, context: &ReactiveRenderContext) -> String {
    let value = props.value.as_deref().unwrap_or_default();
    let size = props.style.size.unwrap_or(ButtonSize::Md);
    let inputs = (0..props.length)
        .map(|index| {
            let char_value = value
                .chars()
                .nth(index as usize)
                .map(|character| character.to_string())
                .unwrap_or_default();
            let input_type = match props.kind {
                PinKind::Password => "password",
                PinKind::Text | PinKind::Number => "text",
            };
            let input_mode = if props.kind == PinKind::Number {
                "numeric"
            } else {
                "text"
            };
            let mut cell_classes = variant_classes("control", &props.style);
            cell_classes.push("pin-cell".to_string());
            cell_classes.push(format!("is-{}", size.as_str()));
            if props.error_text.is_some()
                || props
                    .style
                    .element
                    .form_validation()
                    .and_then(|validation| validation.error_text.as_ref())
                    .is_some()
            {
                cell_classes.push("is-error".to_string());
            }
            format!(
                r#"<label{}><input class="pin-input" inputmode="{}" type="{}" maxlength="1" value="{}" autocomplete="one-time-code" data-dowe-pin-cell data-dowe-validation-control></label>"#,
                attrs(cell_classes, None, None, context),
                input_mode,
                input_type,
                escape_attr(&char_value),
            )
        })
        .collect::<String>();
    let hidden = props
        .name
        .as_deref()
        .map(|name| {
            format!(
                r#"<input type="hidden" name="{}" value="{}" data-dowe-pin-hidden>"#,
                escape_attr(name),
                escape_attr(value)
            )
        })
        .unwrap_or_default();
    let extra = format!(
        r#" data-dowe-pin data-dowe-pin-length="{}" data-dowe-pin-type="{}"{}"#,
        props.length,
        props.kind.as_str(),
        bind_attr(props.style.element.bind.as_deref(), context)
    );
    let body = format!(
        r#"<div{}>{hidden}<div class="pin-cells">{inputs}</div></div>"#,
        attrs(
            vec!["pin".to_string()],
            Some(&props.style.element),
            Some(&extra),
            context
        )
    );
    render_field_block(
        &props.style,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &body,
        context,
    )
}

