fn render_editor_html(props: &EditorProps, context: &ReactiveRenderContext) -> String {
    let toolbar = if props.hide_toolbar {
        String::new()
    } else {
        [
            ("bold", "B"),
            ("italic", "I"),
            ("underline", "U"),
            ("insertUnorderedList", "List"),
            ("justifyLeft", "Left"),
            ("justifyCenter", "Center"),
            ("justifyRight", "Right"),
            ("removeFormat", "Clear"),
        ]
        .into_iter()
        .map(|(command, label)| {
            format!(
                r#"<button class="editor-toolbar-button" type="button" data-dowe-editor-command="{}">{}</button>"#,
                escape_attr(command),
                escape_html(label)
            )
        })
        .collect::<String>()
    };
    let hidden = props
        .name
        .as_deref()
        .map(|name| {
            format!(
                r#"<textarea name="{}" data-dowe-editor-hidden hidden>{}</textarea>"#,
                escape_attr(name),
                escape_html(props.value.as_deref().unwrap_or_default())
            )
        })
        .unwrap_or_default();
    let extra = format!(
        r#" data-dowe-editor style="--dowe-editor-min-height:{}px"{}{}{}"#,
        props.min_height,
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.disabled {
            " data-dowe-disabled=\"true\""
        } else {
            ""
        },
        if props.readonly {
            " data-dowe-readonly=\"true\""
        } else {
            ""
        }
    );
    let body = format!(
        r#"<div{}>{hidden}<div class="editor-toolbar">{toolbar}</div><div class="editor-content" contenteditable="{}" role="textbox" aria-multiline="true" data-dowe-editor-content placeholder="{}">{}</div></div>"#,
        attrs(
            variant_classes("editor", &props.style),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        if props.disabled || props.readonly {
            "false"
        } else {
            "true"
        },
        escape_attr(props.style.placeholder.as_deref().unwrap_or_default()),
        escape_html(props.value.as_deref().unwrap_or_default())
    );
    render_field_block(
        &props.style,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &body,
        context,
    )
}

