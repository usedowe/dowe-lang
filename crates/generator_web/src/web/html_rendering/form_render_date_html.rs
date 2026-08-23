fn render_date_html(props: &DateProps, context: &ReactiveRenderContext) -> String {
    let bind = props
        .style
        .element
        .bind
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-date-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let value = props.value.as_deref().unwrap_or_default();
    let hidden = format!(
        r#"<input class="date-hidden" type="hidden" value="{}"{}>"#,
        escape_attr(value),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default()
    );
    let input = format!(
        r#"<div class="date-control-shell" data-dowe-date-field data-dowe-date-value="{}" data-dowe-date-placeholder="{}"{}{}{}><button class="date-control-trigger" data-dowe-date-trigger data-dowe-validation-control type="button" aria-haspopup="dialog" aria-expanded="false"><span class="date-control-value"></span>{}</button>{}<div class="date-popover" data-dowe-date-popover role="dialog" aria-label="Date picker"><div class="date-picker-header"><button class="date-picker-nav" type="button" data-dowe-date-prev aria-label="Previous month">‹</button><span class="date-picker-month"></span><button class="date-picker-nav" type="button" data-dowe-date-next aria-label="Next month">›</button></div><div class="date-picker-weekdays"></div><div class="date-picker-days"></div></div></div>"#,
        escape_attr(value),
        escape_attr(
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Select a date")
        ),
        bind,
        props
            .min
            .as_deref()
            .map(|value| format!(r#" data-dowe-date-min="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .max
            .as_deref()
            .map(|value| format!(r#" data-dowe-date-max="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        select_arrow_svg(),
        hidden
    );
    render_field_control(
        "date-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &input,
        false,
        props
            .value
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        context,
    )
}

