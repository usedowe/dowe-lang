fn render_password_html(props: &PasswordProps, context: &ReactiveRenderContext) -> String {
    let has_validation = has_form_validation_contract(&props.style.element);
    let show_icon = solar_control_icon("eye").expect("bundled Password reveal icon");
    let hide_icon = solar_control_icon("eye-closed").expect("bundled Password conceal icon");
    let toggle = format!(
        r#"<button class="password-toggle" type="button" aria-label="Show password" data-dowe-password-toggle><span data-dowe-password-show-icon>{}</span><span data-dowe-password-hide-icon hidden>{}</span></button>"#,
        render_svg_html(&show_icon.props, &show_icon.paths, context),
        render_svg_html(&hide_icon.props, &hide_icon.paths, context)
    );
    let input = format!(
        r#"<input class="password-input input" type="password"{}{}{}{}{}{} data-dowe-password-input>{toggle}"#,
        input_placeholder_attr(&props.style),
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
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.disabled {
            " disabled"
        } else if props.readonly {
            " readonly"
        } else {
            ""
        },
        if has_validation {
            " data-dowe-validation-control"
        } else {
            ""
        }
    );
    let mut classes = variant_classes("control", &props.style);
    classes.push("password".to_string());
    classes.push(format!(
        "is-{}",
        props.style.size.unwrap_or(ButtonSize::Md).as_str()
    ));
    if props.style.label_floating {
        classes.push("is-floating".to_string());
    }
    if props.error_text.is_some() {
        classes.push("is-error".to_string());
    }
    let control = format!(
        "<span{}>{}{input}</span>",
        attrs(classes, Some(&props.style.element), None, context),
        floating_label_html(&props.style)
    );
    let body = format!("{control}{}", render_password_strength(props));
    render_field_block(
        &props.style,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &body,
        context,
    )
}

