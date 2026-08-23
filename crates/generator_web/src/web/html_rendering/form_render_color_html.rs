fn render_color_html(props: &ColorProps, context: &ReactiveRenderContext) -> String {
    let name = props
        .name
        .as_deref()
        .map(|name| format!(r#" name="{}""#, escape_attr(name)))
        .unwrap_or_default();
    let bind = props
        .style
        .element
        .bind
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-color-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let values = render_color_values(props);
    let picker = format!(
        r#"<div class="color-control-shell" data-dowe-color-picker data-dowe-color-value="{}"{}><button class="color-control-trigger" data-dowe-color-trigger type="button" aria-haspopup="dialog" aria-expanded="false"><span class="color-field-swatch is-{}" data-dowe-color-swatch></span><span class="color-field-value" data-dowe-color-value-label></span></button><input class="color-input" type="hidden" value="{}"{}><div class="color-picker-popover" data-dowe-color-popover role="dialog" aria-label="Color picker"><div class="color-picker-canvas" data-dowe-color-sv role="slider" tabindex="0" aria-label="Saturation and brightness" aria-valuemin="0" aria-valuemax="100"><span class="color-picker-cursor" data-dowe-color-cursor></span></div><div class="color-picker-hue" data-dowe-color-hue role="slider" tabindex="0" aria-label="Hue" aria-valuemin="0" aria-valuemax="360"><span class="color-picker-slider-thumb" data-dowe-color-hue-thumb></span></div><div class="color-picker-preview"><span class="color-picker-preview-swatch"><span class="color-picker-preview-color" data-dowe-color-preview></span></span><span class="color-picker-preview-info"><strong class="color-picker-preview-hex" data-dowe-color-preview-hex></strong><span class="color-picker-preview-foreground" data-dowe-color-foreground></span></span></div>{values}</div></div>"#,
        escape_attr(&props.value),
        bind,
        props.size.as_str(),
        escape_attr(&props.value),
        name
    );
    render_field_control(
        "color-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &picker,
        true,
        true,
        context,
    )
}

