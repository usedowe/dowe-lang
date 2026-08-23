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

