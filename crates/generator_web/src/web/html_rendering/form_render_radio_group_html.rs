fn render_radio_group_html(
    props: &RadioGroupProps,
    options: &[RadioOption],
    context: &ReactiveRenderContext,
) -> String {
    let name = props
        .name
        .clone()
        .unwrap_or_else(|| format!("radio-{}", short_id("radio", &options[0].value)));
    let mut group = format!(
        "<div class=\"radio-group is-{}\">",
        props.orientation.as_str()
    );
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

