fn render_checkbox_html(props: &CheckboxProps, context: &ReactiveRenderContext) -> String {
    let mut input = format!(
        r#"<input type="checkbox" class="checkbox-input is-{}"{}{}{}{}>"#,
        props.style.color.unwrap_or(ColorFamily::Primary).as_str(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.checked { " checked" } else { "" },
        if props.disabled { " disabled" } else { "" }
    );
    input.push_str(
        &props
            .style
            .label
            .as_deref()
            .map(|label| format!(r#"<span class="label-md">{}</span>"#, escape_html(label)))
            .unwrap_or_default(),
    );
    format!(
        "<label{}>{}</label>",
        attrs(
            vec!["checkbox".to_string()],
            Some(&props.style.element),
            None,
            context
        ),
        input
    )
}

fn render_theme_toggle_html(props: &ThemeToggleProps, context: &ReactiveRenderContext) -> String {
    let mut classes = variant_classes("theme-toggle", &props.style);
    classes.push("theme-toggle-icon".to_string());
    format!(
        r#"<button{} type="button" aria-label="{}" data-dowe-theme-toggle data-dowe-light-label="{}" data-dowe-dark-label="{}">{}{}</button>"#,
        attrs(classes, Some(&props.style.element), None, context),
        escape_attr(&props.dark_label),
        escape_attr(&props.light_label),
        escape_attr(&props.dark_label),
        view_icon_svg(ViewIcon::Moon, "theme-icon theme-icon-moon"),
        view_icon_svg(ViewIcon::Sun, "theme-icon theme-icon-sun")
    )
}

fn render_fab_html(
    props: &FabProps,
    actions: &[FabAction],
    context: &ReactiveRenderContext,
) -> String {
    let mut container_classes = vec![
        "fab-container".to_string(),
        format!("is-{}", props.position.as_str()),
    ];
    if props.fixed {
        container_classes.push("is-fixed".to_string());
    }
    let style = format!(
        r#" style="--dowe-fab-offset-x:{};--dowe-fab-offset-y:{};""#,
        scale_rem(props.offset_x),
        scale_rem(props.offset_y)
    );
    let mut trigger_classes = variant_classes("fab-trigger", &props.style);
    trigger_classes.push("fab-icon".to_string());
    let expanded = if actions.is_empty() {
        ""
    } else {
        r#" aria-expanded="false""#
    };
    let mut html = format!(
        r#"<div{}{}><div class="fab-actions" data-dowe-fab-actions hidden>"#,
        class_attr(container_classes),
        style
    );
    for (index, action) in actions.iter().enumerate() {
        html.push_str(&render_fab_action_html(
            action,
            index,
            &props.style,
            context,
        ));
    }
    html.push_str("</div>");
    html.push_str(&format!(
        r#"<button{} type="button" aria-label="{}" data-dowe-fab-trigger{}>{}</button></div>"#,
        attrs(
            trigger_classes,
            Some(&props.style.element),
            Some(&fab_trigger_extra(props, actions, context)),
            context
        ),
        escape_attr(&props.label),
        expanded,
        view_icon_svg(props.icon, "fab-icon-svg")
    ));
    html
}

fn fab_trigger_extra(
    _props: &FabProps,
    actions: &[FabAction],
    _context: &ReactiveRenderContext,
) -> String {
    let mut extra = String::new();
    if !actions.is_empty() {
        extra.push_str(r#" data-dowe-fab-has-actions="true""#);
    }
    extra
}

fn render_fab_action_html(
    action: &FabAction,
    index: usize,
    style: &VariantProps,
    context: &ReactiveRenderContext,
) -> String {
    let delay = index * 50;
    let label = format!(
        r#"<span class="fab-action-label">{}</span>"#,
        escape_html(&action.label)
    );
    let mut button_classes = variant_classes(
        "fab-action-button",
        &VariantProps {
            color: Some(action.color),
            variant: style.variant,
            ..VariantProps::default()
        },
    );
    button_classes.push("button-md".to_string());
    button_classes.push("fab-icon".to_string());
    let icon = view_icon_svg(action.icon, "fab-icon-svg");
    let control = if let Some(navigation) = action.navigation.as_ref() {
        match navigation {
            NavigationAction::Internal {
                path,
                fragment,
                operation,
            } => {
                let href = internal_href(path, fragment.as_deref());
                format!(
                    r#"<a{}{} data-dowe-fab-action>{}</a>"#,
                    class_attr(button_classes),
                    navigation_attrs(&href, *operation),
                    icon
                )
            }
            NavigationAction::Section {
                fragment,
                operation,
            } => {
                let href = format!("#{fragment}");
                format!(
                    r#"<a{}{} data-dowe-fab-action>{}</a>"#,
                    class_attr(button_classes),
                    navigation_attrs(&href, *operation),
                    icon
                )
            }
            NavigationAction::External {
                url,
                web_target,
                native_external_mode,
            } => format!(
                r#"<a{}{} data-dowe-fab-action>{}</a>"#,
                class_attr(button_classes),
                external_attrs(url, *web_target, *native_external_mode),
                icon
            ),
            NavigationAction::Back => format!(
                r#"<button{} type="button" data-dowe-history="back" data-dowe-fab-action>{}</button>"#,
                class_attr(button_classes),
                icon
            ),
        }
    } else {
        let click = action
            .on_click
            .as_deref()
            .map(|value| {
                format!(
                    r#" data-dowe-click="{}""#,
                    escape_attr(&context.action_id(value))
                )
            })
            .unwrap_or_default();
        format!(
            r#"<button{} type="button"{} data-dowe-fab-action>{}</button>"#,
            class_attr(button_classes),
            click,
            icon
        )
    };
    format!(
        r#"<div class="fab-action" style="--dowe-fab-action-delay:{}ms">{label}{control}</div>"#,
        delay
    )
}

fn render_slider_html(props: &SliderProps, context: &ReactiveRenderContext) -> String {
    let value = props.value.parse::<f64>().unwrap_or(0.0);
    let min = props.min.parse::<f64>().unwrap_or(0.0);
    let max = props.max.parse::<f64>().unwrap_or(100.0);
    let progress = if max > min {
        (((value - min) / (max - min)) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    let info = if props.hide_label {
        String::new()
    } else {
        format!(
            r#"<div class="slider-info"><span>{}</span><span data-dowe-slider-value>{}</span></div>"#,
            props
                .style
                .label
                .as_deref()
                .map(escape_html)
                .unwrap_or_default(),
            escape_html(&props.value)
        )
    };
    let input = format!(
        r#"<input type="range"{}{}{}{}{}{} class="slider is-{} is-{}" style="--dowe-slider-progress:{}%" data-dowe-slider{}>"#,
        format!(r#" min="{}""#, escape_attr(&props.min)),
        format!(r#" max="{}""#, escape_attr(&props.max)),
        props
            .step
            .as_deref()
            .map(|step| format!(r#" step="{}""#, escape_attr(step)))
            .unwrap_or_default(),
        format!(r#" value="{}""#, escape_attr(&props.value)),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context),
        props.size.as_str(),
        props.style.color.unwrap_or(ColorFamily::Primary).as_str(),
        progress,
        props
            .style
            .label
            .as_deref()
            .map(|label| format!(r#" data-dowe-slider-label="{}""#, escape_attr(label)))
            .unwrap_or_default()
    );
    format!(
        "<div{}>{info}{input}</div>",
        attrs(
            vec!["slider-wrapper".to_string()],
            Some(&props.style.element),
            None,
            context
        )
    )
}

fn render_dropzone_html(props: &DropzoneProps, context: &ReactiveRenderContext) -> String {
    let source = format!(
        "{}:{}:{}:{}",
        props.name.as_deref().unwrap_or_default(),
        props.style.placeholder.as_deref().unwrap_or_default(),
        props.accept.as_deref().unwrap_or_default(),
        props.style.label.as_deref().unwrap_or_default()
    );
    let uid = short_id("dropzone", &source);
    let field_label = props
        .style
        .label
        .as_deref()
        .map(|label| format!(r#"<span class="field-label">{}</span>"#, escape_html(label)))
        .unwrap_or_default();
    let help = props
        .error_text
        .as_deref()
        .or(props.help_text.as_deref())
        .map(|text| {
            format!(
                r#"<div class="field-help{}">{}</div>"#,
                if props.error_text.is_some() {
                    " is-danger"
                } else {
                    ""
                },
                escape_html(text)
            )
        })
        .unwrap_or_default();
    let mut input_classes = vec![
        "dropzone-input".to_string(),
        format!(
            "is-{}",
            props
                .style
                .variant
                .unwrap_or(ComponentVariant::Solid)
                .as_str()
        ),
        format!(
            "is-{}",
            props.style.color.unwrap_or(ColorFamily::Primary).as_str()
        ),
        format!("is-{}", props.size.as_str()),
    ];
    if props.disabled {
        input_classes.push("is-disabled".to_string());
    }
    if props.error_text.is_some() {
        input_classes.push("is-error".to_string());
    }
    let max = props
        .max_size
        .map(|value| format!(r#" data-dowe-dropzone-max-size="{value}""#))
        .unwrap_or_default();
    let input = format!(
        r#"<label{} for="{uid}" data-dowe-dropzone{max}><input id="{uid}" type="file" hidden{}{}{}{}><div class="dropzone-content">{}<span class="dropzone-placeholder">{}</span></div></label>"#,
        class_attr(input_classes),
        props
            .accept
            .as_deref()
            .map(|accept| format!(r#" accept="{}""#, escape_attr(accept)))
            .unwrap_or_default(),
        if props.multiple { " multiple" } else { "" },
        if props.disabled { " disabled" } else { "" },
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        view_icon_svg(ViewIcon::Upload, "dropzone-icon"),
        escape_html(props.style.placeholder.as_deref().unwrap_or_default())
    );
    format!(
        r#"<div{}>{field_label}{input}<div class="dropzone-files" data-dowe-dropzone-files hidden></div>{help}</div>"#,
        attrs(
            vec!["field".to_string(), "dropzone".to_string()],
            Some(&props.style.element),
            None,
            context
        )
    )
}

fn view_icon_svg(icon: ViewIcon, class_name: &str) -> String {
    let (paths, view_box) = match icon {
        ViewIcon::Plus => (
            r#"<path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
            "0 0 24 24",
        ),
        ViewIcon::Link => (
            r#"<path d="M10 13a5 5 0 0 0 7.07 0l2.12-2.12a5 5 0 0 0-7.07-7.07L10.9 5.03" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><path d="M14 11a5 5 0 0 0-7.07 0L4.81 13.12a5 5 0 0 0 7.07 7.07l1.22-1.22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
            "0 0 24 24",
        ),
        ViewIcon::Edit => (
            r#"<path d="M4 20h4l10.5-10.5a2.12 2.12 0 0 0-3-3L5 17v3Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/><path d="m13.5 7.5 3 3" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
            "0 0 24 24",
        ),
        ViewIcon::Trash => (
            r#"<path d="M5 7h14M10 11v6M14 11v6M8 7l1-3h6l1 3M7 7l1 13h8l1-13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
            "0 0 24 24",
        ),
        ViewIcon::Search => (
            r#"<circle cx="11" cy="11" r="6" fill="none" stroke="currentColor" stroke-width="2"/><path d="m16 16 4 4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
            "0 0 24 24",
        ),
        ViewIcon::Settings => (
            r#"<path d="M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z" fill="none" stroke="currentColor" stroke-width="2"/><path d="M4 12h2m12 0h2M12 4v2m0 12v2M6.3 6.3l1.4 1.4m8.6 8.6 1.4 1.4m0-11.4-1.4 1.4m-8.6 8.6-1.4 1.4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
            "0 0 24 24",
        ),
        ViewIcon::Upload => (
            r#"<path d="M12 16V4m0 0 5 5m-5-5-5 5M4 16v3a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-3" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
            "0 0 24 24",
        ),
        ViewIcon::File => (
            r#"<path d="M6 3h8l4 4v14H6V3Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/><path d="M14 3v5h5" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>"#,
            "0 0 24 24",
        ),
        ViewIcon::Dismiss => (
            r#"<path d="m6 6 12 12M18 6 6 18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
            "0 0 24 24",
        ),
        ViewIcon::Moon => (
            r#"<path d="M20 15.3A8 8 0 0 1 8.7 4 8.5 8.5 0 1 0 20 15.3Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>"#,
            "0 0 24 24",
        ),
        ViewIcon::Sun => (
            r#"<circle cx="12" cy="12" r="4" fill="none" stroke="currentColor" stroke-width="2"/><path d="M12 2v2m0 16v2M4.93 4.93l1.42 1.42m11.3 11.3 1.42 1.42M2 12h2m16 0h2M4.93 19.07l1.42-1.42m11.3-11.3 1.42-1.42" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
            "0 0 24 24",
        ),
    };
    format!(
        r#"<svg class="{}" viewBox="{}" aria-hidden="true">{}</svg>"#,
        escape_attr(class_name),
        escape_attr(view_box),
        paths
    )
}
