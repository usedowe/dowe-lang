fn render_form_validation_attrs(
    element: &ElementProps,
    help_text: Option<&str>,
    error_text: Option<&str>,
    value_kind: &str,
    context: &ReactiveRenderContext,
) -> String {
    let Some(validation) = element.form_validation() else {
        if help_text.is_none() && error_text.is_none() {
            return String::new();
        }
        return format!(
            r#" data-dowe-validation-kind="{}"{}{}"#,
            escape_attr(value_kind),
            help_text
                .map(|value| format!(r#" data-dowe-validation-help="{}""#, escape_attr(value)))
                .unwrap_or_default(),
            error_text
                .map(|value| format!(r#" data-dowe-validation-error="{}""#, escape_attr(value)))
                .unwrap_or_default()
        );
    };
    let rules = validation
        .rules
        .iter()
        .map(|rule| {
            let argument = match &rule.kind {
                dowe_components::FormValidationRuleKind::Matches(path) => {
                    Some(context.signal_path(path))
                }
                _ => rule.kind.argument(),
            };
            format!(
                r#"{{"kind":"{}","argument":{},"message":"{}"}}"#,
                escape_json(rule.kind.name()),
                argument
                    .as_deref()
                    .map(|value| format!(r#""{}""#, escape_json(value)))
                    .unwrap_or_else(|| "null".to_string()),
                escape_json(&rule.message)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let form_attrs = element
        .bind
        .as_deref()
        .and_then(|bind| {
            let (signal, field) = bind.split_once('.')?;
            Some(format!(
                r#" data-dowe-validation-form="{}" data-dowe-validation-field="{}""#,
                escape_attr(&context.signal_path(signal)),
                escape_attr(field)
            ))
        })
        .unwrap_or_default();
    format!(
        r#" data-dowe-validation-kind="{}" data-dowe-validation="{}"{}{}{}"#,
        escape_attr(value_kind),
        escape_attr(&format!("[{rules}]")),
        form_attrs,
        help_text
            .map(|value| format!(r#" data-dowe-validation-help="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        error_text
            .map(|value| format!(r#" data-dowe-validation-error="{}""#, escape_attr(value)))
            .unwrap_or_default()
    )
}

