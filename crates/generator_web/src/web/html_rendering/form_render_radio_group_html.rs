fn render_radio_group_html(
    props: &RadioGroupProps,
    options: &[RadioOption],
    context: &ReactiveRenderContext,
) -> String {
    if matches!(props.presentation, RadioGroupPresentation::Card) {
        return render_radio_card_html(props, options, context);
    }
    let name = props
        .name
        .clone()
        .unwrap_or_else(|| format!("radio-{}", short_id("radio", &options[0].value)));
    let reactive_attrs = reactive_variant_attrs(&props.style, context, "is-");
    let mut group_classes = vec!["radio-group".to_string()];
    if props.style.variant.is_some() {
        group_classes.push(format!(
            "is-{}",
            props.style.variant.unwrap_or(ComponentVariant::Solid).as_str()
        ));
    }
    group_classes.push(format!("is-{}", props.orientation.as_str()));
    let mut group = format!(
        "<div{}>",
        attrs(
            group_classes,
            Some(&props.style.element),
            Some(reactive_attrs.as_str()),
            context,
        )
    );
    for option in options {
        let mut radio_classes = vec!["radio".to_string()];
        if props.style.variant.is_some() {
            radio_classes.push(format!(
                "is-{}",
                props.style.variant.unwrap_or(ComponentVariant::Solid).as_str()
            ));
        }
        radio_classes.push(format!(
            "is-{}",
            props.style.color.unwrap_or(ColorFamily::Primary).as_str()
        ));
        radio_classes.push(format!("is-{}", props.size.as_str()));
        group.push_str(&format!(
            r#"<label class="radio-item"><input type="radio"{} name="{}" value="{}"{}{}><span class="label">{}</span></label>"#,
            attrs(radio_classes, None, Some(reactive_attrs.as_str()), context),
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

fn render_radio_card_html(
    props: &RadioGroupProps,
    options: &[RadioOption],
    context: &ReactiveRenderContext,
) -> String {
    let name = props
        .name
        .clone()
        .unwrap_or_else(|| format!("radio-card-{}", short_id("radio-card", &options[0].value)));
    let reactive_attrs = reactive_variant_attrs(&props.style, context, "is-");
    let mut group_classes = vec!["radio-card-group".to_string()];
    if props.style.variant.is_some() {
        group_classes.push(format!(
            "is-{}",
            props.style.variant.unwrap_or(ComponentVariant::Outlined).as_str()
        ));
    }
    group_classes.push(format!("is-{}", props.orientation.as_str()));
    let mut group = format!(
        "<div{}>",
        attrs(
            group_classes,
            Some(&props.style.element),
            Some(reactive_attrs.as_str()),
            context,
        )
    );
    for option in options {
        let mut card_classes = vec!["radio-card".to_string()];
        if props.style.variant.is_some() {
            card_classes.push(format!(
                "is-{}",
                props.style.variant.unwrap_or(ComponentVariant::Outlined).as_str()
            ));
        }
        card_classes.push(format!(
            "is-{}",
            props.style.color.unwrap_or(ColorFamily::Primary).as_str()
        ));
        card_classes.push(format!("is-{}", props.size.as_str()));
        let icon = option
            .icon
            .as_ref()
            .map(|icon| {
                format!(
                    r#"<span class="radio-card-icon" aria-hidden="true">{}</span>"#,
                    render_svg_html(&icon.props, &icon.paths, context)
                )
            })
            .unwrap_or_default();
        let description = option
            .description
            .as_deref()
            .map(|value| {
                format!(
                    r#"<span class="radio-card-description">{}</span>"#,
                    escape_html(value)
                )
            })
            .unwrap_or_default();
        group.push_str(&format!(
            r#"<label class="{}"><input class="radio-card-control" type="radio" name="{}" value="{}"{}{}><span class="radio-card-content">{}<span class="radio-card-copy"><span class="radio-card-title">{}</span>{}</span></span><span class="radio-card-indicator" aria-hidden="true"></span></label>"#,
            card_classes.join(" "),
            escape_attr(&name),
            escape_attr(&option.value),
            bind_attr(props.style.element.bind.as_deref(), context),
            if option.disabled { " disabled" } else { "" },
            icon,
            escape_html(&option.label),
            description
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
