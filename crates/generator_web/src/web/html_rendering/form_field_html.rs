fn render_color_html(props: &ColorProps, context: &ReactiveRenderContext) -> String {
    let input = format!(
        r#"<input class="color-input" type="color" value="{}"{}{}>"#,
        escape_attr(&props.value),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context)
    );
    let preview = format!(
        r#"<span class="color-field-swatch is-{}" style="background-color:{}"></span><span class="color-field-value">{}</span>"#,
        props.size.as_str(),
        escape_attr(&props.value),
        escape_html(&props.value.to_ascii_uppercase())
    );
    let values = render_color_values(props);
    render_field_control(
        "color-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &format!("{input}<span class=\"color-field-display\">{preview}</span>{values}"),
        context,
    )
}

fn render_date_html(props: &DateProps, context: &ReactiveRenderContext) -> String {
    let input = format!(
        r#"<input class="date-input" type="date"{}{}{}{}{}>"#,
        props
            .value
            .as_deref()
            .map(|value| format!(r#" value="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        props
            .min
            .as_deref()
            .map(|value| format!(r#" min="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .max
            .as_deref()
            .map(|value| format!(r#" max="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context)
    );
    render_field_control(
        "date-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &input,
        context,
    )
}

fn render_date_range_html(props: &DateRangeProps, context: &ReactiveRenderContext) -> String {
    let start_bind = props
        .start
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let end_bind = props
        .end
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let common = format!(
        "{}{}",
        props
            .min
            .as_deref()
            .map(|value| format!(r#" min="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .max
            .as_deref()
            .map(|value| format!(r#" max="{}""#, escape_attr(value)))
            .unwrap_or_default()
    );
    let input = format!(
        r#"<span class="date-range-inputs"><input class="date-input" type="date"{}{}{}><span class="date-range-separator">-</span><input class="date-input" type="date"{}{}{}></span>"#,
        props
            .start_value
            .as_deref()
            .map(|value| format!(r#" value="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}Start""#, escape_attr(name)))
            .unwrap_or_default(),
        format!("{common}{start_bind}"),
        props
            .end_value
            .as_deref()
            .map(|value| format!(r#" value="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}End""#, escape_attr(name)))
            .unwrap_or_default(),
        format!("{common}{end_bind}")
    );
    render_field_control(
        "date-range-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &input,
        context,
    )
}

fn render_radio_group_html(
    props: &RadioGroupProps,
    options: &[RadioOption],
    context: &ReactiveRenderContext,
) -> String {
    let name = props
        .name
        .clone()
        .unwrap_or_else(|| format!("radio-{}", short_id("radio", &options[0].value)));
    let mut group = String::from("<div class=\"radio-group\">");
    for option in options {
        group.push_str(&format!(
            r#"<label class="radio-item"><input type="radio" class="radio is-{} is-{}" name="{}" value="{}"{}{}><span class="label">{}</span></label>"#,
            props.style.color.unwrap_or(ColorFamily::Primary).as_str(),
            props.size.as_str(),
            escape_attr(&name),
            escape_attr(&option.value),
            bind_attr(props.style.element.bind.as_deref(), context),
            if option.disabled { " disabled" } else { "" },
            escape_html(&option.label)
        ));
    }
    group.push_str("</div>");
    render_field_block(
        &props.style,
        props.info.as_deref(),
        props.error.as_deref(),
        &group,
        context,
    )
}

fn render_toggle_html(props: &ToggleProps, context: &ReactiveRenderContext) -> String {
    let left = props
        .label_left
        .as_deref()
        .map(|label| {
            format!(
                r#"<span class="toggle-label-left{}">{}</span>"#,
                if props.checked { "" } else { " is-active" },
                escape_html(label)
            )
        })
        .unwrap_or_default();
    let right = props
        .label_right
        .as_deref()
        .map(|label| {
            format!(
                r#"<span class="toggle-label-right{}">{}</span>"#,
                if props.checked { " is-active" } else { "" },
                escape_html(label)
            )
        })
        .unwrap_or_default();
    let label = props
        .style
        .label
        .as_deref()
        .map(|label| format!(r#"<span class="label-md">{}</span>"#, escape_html(label)))
        .unwrap_or_default();
    let input = format!(
        r#"<input type="checkbox" role="switch" class="toggle-input is-{}" aria-checked="{}"{}{}{}{}>"#,
        props.style.color.unwrap_or(ColorFamily::Primary).as_str(),
        props.checked,
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.checked { " checked" } else { "" },
        if props.disabled { " disabled" } else { "" }
    );
    format!(
        "<label{}>{left}{input}{right}{label}</label>",
        attrs(
            vec!["toggle".to_string()],
            Some(&props.style.element),
            None,
            context
        )
    )
}

fn render_field_control(
    base: &str,
    props: &VariantProps,
    size: ButtonSize,
    help_text: Option<&str>,
    error_text: Option<&str>,
    control_html: &str,
    context: &ReactiveRenderContext,
) -> String {
    let mut classes = variant_classes("control", props);
    classes.push(base.to_string());
    classes.push(format!("is-{}", size.as_str()));
    if props.label_floating {
        classes.push("is-floating".to_string());
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

fn render_field_block(
    props: &VariantProps,
    help_text: Option<&str>,
    error_text: Option<&str>,
    body_html: &str,
    context: &ReactiveRenderContext,
) -> String {
    let label = if props.label.is_some() && !props.label_floating {
        format!(
            r#"<span class="field-label">{}</span>"#,
            escape_html(props.label.as_deref().unwrap_or_default())
        )
    } else {
        String::new()
    };
    let help = error_text
        .or(help_text)
        .map(|value| {
            format!(
                r#"<span class="field-help{}">{}</span>"#,
                if error_text.is_some() {
                    " is-error"
                } else {
                    ""
                },
                escape_html(value)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<div{}>{}{body_html}{}</div>"#,
        attrs(vec!["field".to_string()], None, None, context),
        label,
        help
    )
}

fn render_color_values(props: &ColorProps) -> String {
    if !(props.show_hex || props.show_rgb || props.show_cmyk || props.show_oklch) {
        return String::new();
    }
    let mut html = String::from("<span class=\"color-picker-values\">");
    if props.show_hex {
        html.push_str(&format!(
            r#"<code class="color-picker-value-code">hex: {}</code>"#,
            escape_html(&props.value)
        ));
    }
    if props.show_rgb {
        html.push_str(&format!(
            r#"<code class="color-picker-value-code">rgb: {}</code>"#,
            escape_html(&hex_rgb_label(&props.value))
        ));
    }
    if props.show_cmyk {
        html.push_str(&format!(
            r#"<code class="color-picker-value-code">cmyk: {}</code>"#,
            escape_html(&hex_cmyk_label(&props.value))
        ));
    }
    if props.show_oklch {
        html.push_str(r#"<code class="color-picker-value-code">oklch: target-derived</code>"#);
    }
    html.push_str("</span>");
    html
}

fn hex_rgb_label(value: &str) -> String {
    let Some((r, g, b)) = parse_hex_rgb(value) else {
        return "rgb(0, 0, 0)".to_string();
    };
    format!("rgb({r}, {g}, {b})")
}

fn hex_cmyk_label(value: &str) -> String {
    let Some((r, g, b)) = parse_hex_rgb(value) else {
        return "cmyk(0%, 0%, 0%, 100%)".to_string();
    };
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let k = 1.0 - r.max(g).max(b);
    if k >= 1.0 {
        return "cmyk(0%, 0%, 0%, 100%)".to_string();
    }
    let c = ((1.0 - r - k) / (1.0 - k) * 100.0).round() as u8;
    let m = ((1.0 - g - k) / (1.0 - k) * 100.0).round() as u8;
    let y = ((1.0 - b - k) / (1.0 - k) * 100.0).round() as u8;
    let k = (k * 100.0).round() as u8;
    format!("cmyk({c}%, {m}%, {y}%, {k}%)")
}

fn parse_hex_rgb(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.strip_prefix('#')?;
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        return Some((r, g, b));
    }
    if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        return Some((r, g, b));
    }
    None
}
