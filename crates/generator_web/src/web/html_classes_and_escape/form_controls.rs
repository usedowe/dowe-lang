fn render_input_html(props: &VariantProps, context: &ReactiveRenderContext) -> String {
    let mut input = String::new();
    let has_validation = has_form_validation_contract(&props.element);
    if let Some(icon) = props.icon_start.as_ref() {
        input.push_str(r#"<span class="control-icon icon-start">"#);
        input.push_str(&render_svg_html(&icon.props, &icon.paths, context));
        input.push_str("</span>");
    }
    input.push_str(&format!(
        r#"<input class="input"{}{}{}{}>"#,
        if has_validation {
            " data-dowe-validation-control"
        } else {
            ""
        },
        input_placeholder_attr(props),
        bind_attr(props.element.bind.as_deref(), context),
        props
            .label
            .as_deref()
            .map(|label| format!(r#" aria-label="{}""#, escape_attr(label)))
            .unwrap_or_default()
    ));
    if let Some(icon) = props.icon_end.as_ref() {
        input.push_str(r#"<span class="control-icon icon-end">"#);
        input.push_str(&render_svg_html(&icon.props, &icon.paths, context));
        input.push_str("</span>");
    }
    if props.label.is_some() && !props.label_floating {
        let control = format!(
            "<span{}>{}</span>",
            attrs(
                input_control_classes(props),
                Some(&props.element),
                None,
                context
            ),
            input
        );
        if has_validation {
            return render_field_block(props, None, None, &control, context);
        }
        return format!(
            r#"<label{}><span class="field-label">{}</span>{}</label>"#,
            attrs(
                vec!["field".to_string()],
                Some(&props.element),
                None,
                context,
            ),
            escape_html(props.label.as_deref().unwrap_or_default()),
            control
        );
    }
    if props.label_floating {
        let mut classes = input_control_classes(props);
        classes.push("is-floating".to_string());
        let body = format!(
            "<label{}>{}{}</label>",
            attrs(classes, Some(&props.element), None, context),
            floating_label_html(props),
            input
        );
        return if has_validation {
            render_field_block(props, None, None, &body, context)
        } else {
            body
        };
    }
    let body = format!(
        r#"<div{}>{}</div>"#,
        attrs(
            input_control_classes(props),
            Some(&props.element),
            None,
            context
        ),
        input
    );
    if has_validation {
        render_field_block(props, None, None, &body, context)
    } else {
        body
    }
}

fn has_form_validation_contract(element: &ElementProps) -> bool {
    element.form_validation().is_some_and(|validation| {
        validation.help_text.is_some()
            || validation.error_text.is_some()
            || !validation.rules.is_empty()
    })
}

fn input_control_classes(props: &VariantProps) -> Vec<String> {
    let mut classes = variant_classes("control", props);
    classes.insert(
        1,
        format!("is-{}", props.size.unwrap_or(ButtonSize::Md).as_str()),
    );
    if props.icon_start.is_some() {
        classes.push("has-start-adornment".to_string());
    }
    if props
        .element
        .form_validation()
        .and_then(|validation| validation.error_text.as_ref())
        .is_some()
    {
        classes.push("is-error".to_string());
    }
    classes
}

fn render_select_html(
    props: &VariantProps,
    options: &[SelectOption],
    option_each: Option<&SelectOptionEach>,
    context: &ReactiveRenderContext,
) -> String {
    render_select_html_with_attrs(props, options, option_each, context, "")
}

fn render_select_html_with_attrs(
    props: &VariantProps,
    options: &[SelectOption],
    option_each: Option<&SelectOptionEach>,
    context: &ReactiveRenderContext,
    extra_attrs: &str,
) -> String {
    let has_validation = has_form_validation_contract(&props.element);
    let mut classes = variant_classes("control", props);
    classes.insert(
        1,
        format!("is-{}", props.size.unwrap_or(ButtonSize::Md).as_str()),
    );
    classes.push("select-control".to_string());
    if props.label_floating {
        classes.push("is-floating".to_string());
    }
    let placeholder = props.placeholder.as_deref().unwrap_or("Select an option");
    let extra = format!(
        r#" type="button" role="combobox" aria-haspopup="listbox" aria-expanded="false" data-dowe-select{} data-dowe-placeholder="{}"{}{}{}"#,
        if has_validation {
            " data-dowe-validation-control"
        } else {
            ""
        },
        escape_attr(placeholder),
        bind_attr(props.element.bind.as_deref(), context),
        extra_attrs,
        props
            .label
            .as_deref()
            .map(|label| format!(r#" aria-label="{}""#, escape_attr(label)))
            .unwrap_or_default()
    );
    let mut options_html = options
        .iter()
        .map(render_select_option_html)
        .collect::<Vec<_>>()
        .join("");
    if let Some(each) = option_each {
        let description = each
            .description
            .as_ref()
            .map(|path| {
                format!(
                    r#"<span class="select-option-description" data-dowe-text="{}"></span>"#,
                    escape_attr(path)
                )
            })
            .unwrap_or_default();
        options_html.push_str(&format!(
            r#"<div data-dowe-each="{}" data-dowe-item="{}" data-dowe-key="{}"><template><button type="button" class="select-option" role="option" data-dowe-option-value-path="{}" data-dowe-option-label-path="{}"><span class="select-option-label" data-dowe-text="{}"></span>{}</button></template></div>"#,
            escape_attr(&context.signal_path(&each.collection)),
            escape_attr(&each.item),
            escape_attr(&each.key),
            escape_attr(&each.value),
            escape_attr(&each.label),
            escape_attr(&each.label),
            description
        ));
    }
    if props
        .element
        .form_validation()
        .and_then(|validation| validation.error_text.as_ref())
        .is_some()
    {
        classes.push("is-error".to_string());
    }
    let control = format!(
        r#"<div class="select"><button{}>{}<span class="select-value">{}</span>{}</button><div class="select-popover" data-dowe-select-popover role="listbox">{}</div></div>"#,
        attrs(classes, Some(&props.element), Some(&extra), context),
        floating_label_html(props),
        escape_html(placeholder),
        select_arrow_svg(),
        options_html
    );
    if has_validation {
        render_field_block(props, None, None, &control, context)
    } else if props.label.is_some() && !props.label_floating {
        format!(
            r#"<div class="field"><span class="field-label">{}</span>{}</div>"#,
            escape_html(props.label.as_deref().unwrap_or_default()),
            control
        )
    } else {
        control
    }
}

fn select_arrow_svg() -> &'static str {
    r#"<svg class="select-arrow" aria-hidden="true" focusable="false" width="1em" height="1em" viewBox="0 0 24 24"><path d="M0 0h24v24H0z" fill="none"></path><path fill="currentColor" d="M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4a1 1 0 1 0-2 0v13.665L5.714 12.3a1 1 0 0 0-1.424 1.403l6.822 6.925a1.25 1.25 0 0 0 1.78 0z"></path></svg>"#
}

fn render_select_option_html(option: &SelectOption) -> String {
    let description = option
        .description
        .as_deref()
        .map(|description| {
            format!(
                r#"<span class="select-option-description">{}</span>"#,
                escape_html(description)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<button type="button" class="select-option" role="option" data-dowe-option-value="{}" data-dowe-option-label="{}"><span class="select-option-label">{}</span>{}</button>"#,
        escape_attr(&option.value),
        escape_attr(&option.label),
        escape_html(&option.label),
        description
    )
}

