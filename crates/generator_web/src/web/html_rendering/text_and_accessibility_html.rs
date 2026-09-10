fn render_text_html(
    base: &str,
    classes: Vec<String>,
    element: Option<&ElementProps>,
    value: &str,
    i18n: Option<&str>,
    tag: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let dynamic = visible_dynamic_text_attr(value, context);
    let mut extra = dynamic.clone();
    if let Some(key) = i18n {
        extra.push_str(&format!(r#" data-dowe-i18n="{}""#, escape_attr(key)));
    }
    let content = if dynamic.is_empty() {
        escape_html(value)
    } else if let Some(initial) = context.initial_text(value) {
        escape_html(&initial)
    } else {
        String::new()
    };
    let tag = tag.unwrap_or(if base == "title" { "h2" } else { "p" });
    format!(
        "<{tag}{}>{}</{tag}>",
        attrs(
            classes,
            element,
            (!extra.is_empty()).then_some(extra.as_str()),
            context,
        ),
        content
    )
}

fn bind_attr(value: Option<&str>, context: &ReactiveRenderContext) -> String {
    value
        .map(|value| {
            format!(
                r#" data-dowe-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default()
}

fn dynamic_text_attr(value: &str, context: &ReactiveRenderContext) -> String {
    if is_dynamic_path(value) {
        format!(
            r#" data-dowe-text="{}""#,
            escape_attr(&context.signal_path(value))
        )
    } else {
        String::new()
    }
}

fn visible_dynamic_text_attr(value: &str, context: &ReactiveRenderContext) -> String {
    let bindings = text_template_segments(value)
        .into_iter()
        .filter_map(|(_, binding)| binding)
        .collect::<Vec<_>>();
    if bindings.is_empty() {
        return String::new();
    }
    let mut template = value.to_string();
    for binding in bindings {
        template = template.replace(
            &format!("{{{binding}}}"),
            &format!("{{{}}}", context.signal_path(&binding)),
        );
    }
    format!(r#" data-dowe-template="{}""#, escape_attr(&template))
}

fn is_dynamic_path(value: &str) -> bool {
    let value = value.trim();
    value.contains('.')
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '.')
}

fn alert_attrs(props: &AlertProps, context: &ReactiveRenderContext) -> String {
    let mut attrs = format!(
        r#" data-dowe-alert data-dowe-alert-kind="{}""#,
        props.kind.as_str()
    );
    if let Some(visible) = props.visible.as_deref() {
        attrs.push_str(&format!(
            r#" data-dowe-alert-visible="{}""#,
            escape_attr(&context.signal_path(visible))
        ));
    }
    attrs
}
