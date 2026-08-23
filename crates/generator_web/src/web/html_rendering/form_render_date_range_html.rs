fn render_date_range_html(props: &DateRangeProps, context: &ReactiveRenderContext) -> String {
    let start_bind = props
        .start
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-date-start-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let end_bind = props
        .end
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-date-end-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let start_value = props.start_value.as_deref().unwrap_or_default();
    let end_value = props.end_value.as_deref().unwrap_or_default();
    let input = format!(
        r#"<div class="date-control-shell" data-dowe-date-range data-dowe-date-placeholder="{}" data-dowe-date-start-value="{}" data-dowe-date-end-value="{}"{}{}{}{}><button class="date-control-trigger" data-dowe-date-trigger type="button" aria-haspopup="dialog" aria-expanded="false"><span class="date-control-value"></span>{}</button><input class="date-hidden date-hidden-start" type="hidden" value="{}"{}><input class="date-hidden date-hidden-end" type="hidden" value="{}"{}><div class="date-range-popover" data-dowe-date-popover role="dialog" aria-label="Date range picker"><div class="date-range-calendars"><div class="date-range-calendar"><div class="date-picker-header"><button class="date-picker-nav" type="button" data-dowe-date-prev aria-label="Previous month">‹</button><span class="date-picker-month" data-dowe-date-month-current></span><span class="date-range-spacer"></span></div><div class="date-picker-weekdays"></div><div class="date-picker-days" data-dowe-date-days-current></div></div><div class="date-range-calendar"><div class="date-picker-header"><span class="date-range-spacer"></span><span class="date-picker-month" data-dowe-date-month-next></span><button class="date-picker-nav" type="button" data-dowe-date-next aria-label="Next month">›</button></div><div class="date-picker-weekdays"></div><div class="date-picker-days" data-dowe-date-days-next></div></div></div></div></div>"#,
        escape_attr(
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Select a date range")
        ),
        escape_attr(start_value),
        escape_attr(end_value),
        start_bind,
        end_bind,
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
        escape_attr(start_value),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}Start""#, escape_attr(name)))
            .unwrap_or_default(),
        escape_attr(end_value),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}End""#, escape_attr(name)))
            .unwrap_or_default()
    );
    render_field_control(
        "date-range-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &input,
        false,
        props
            .start_value
            .as_deref()
            .is_some_and(|value| !value.is_empty())
            || props
                .end_value
                .as_deref()
                .is_some_and(|value| !value.is_empty()),
        context,
    )
}

