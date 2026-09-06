fn input_placeholder_attr(props: &VariantProps) -> String {
    let placeholder = props
        .placeholder
        .as_deref()
        .or((props.label_floating && props.label.is_some()).then_some(" "));
    placeholder
        .map(|value| format!(r#" placeholder="{}""#, escape_attr(value)))
        .unwrap_or_default()
}

fn floating_label_html(props: &VariantProps) -> String {
    if props.label_floating {
        props
            .label
            .as_deref()
            .map(|label| {
                format!(
                    r#"<span class="control-label">{}</span>"#,
                    escape_html(label)
                )
            })
            .unwrap_or_default()
    } else {
        String::new()
    }
}

fn navigation_attrs(href: &str, operation: NavigationOperation) -> String {
    format!(
        r#" href="{}" data-dowe-nav="{}" data-dowe-href="{}""#,
        escape_attr(href),
        operation.as_str(),
        escape_attr(href)
    )
}

fn external_attrs(
    url: &str,
    web_target: WebTarget,
    native_external_mode: NativeExternalMode,
) -> String {
    let mut attrs = format!(
        r#" href="{}" data-dowe-external-mode="{}""#,
        escape_attr(url),
        native_external_mode.as_str()
    );
    if web_target == WebTarget::Blank {
        attrs.push_str(r#" target="_blank" rel="noopener noreferrer""#);
    }
    attrs
}

fn attrs(
    classes: Vec<String>,
    element: Option<&ElementProps>,
    extra: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut output = class_attr(classes);
    if let Some(id) = element.and_then(|element| element.id.as_ref()) {
        output.push_str(&format!(r#" id="{}""#, escape_attr(id)));
    }
    if element.is_some()
        && let Some(id) = next_view_inspector_id()
    {
        output.push_str(&format!(r#" data-dowe-node="{}""#, escape_attr(&id)));
    }
    if let Some(action) = element.and_then(|element| element.on_click.as_ref()) {
        output.push_str(&format!(
            r#" data-dowe-click="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(action) = element.and_then(|element| element.on_change.as_ref()) {
        output.push_str(&format!(
            r#" data-dowe-change="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(action) = element.and_then(|element| element.on_input.as_ref()) {
        output.push_str(&format!(
            r#" data-dowe-input="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(show) = element.and_then(|element| element.show.as_ref()) {
        match show {
            VisibilityCondition::Signal(path) => output.push_str(&format!(
                r#" data-dowe-show="{}""#,
                escape_attr(&context.signal_path(path))
            )),
            VisibilityCondition::NumberComparison { path, comparison } => {
                output.push_str(&format!(
                    r#" data-dowe-show="{}" data-dowe-show-operator="{}" data-dowe-show-value="{}""#,
                    escape_attr(&context.signal_path(path)),
                    comparison.operator.as_str(),
                    escape_attr(&comparison.value)
                ));
            }
            VisibilityCondition::StringEquality { path, value } => output.push_str(&format!(
                r#" data-dowe-show="{}" data-dowe-show-equals="{}""#,
                escape_attr(&context.signal_path(path)),
                escape_attr(value)
            )),
            VisibilityCondition::Static(_) => {}
        }
        if !matches!(show, VisibilityCondition::Static(_))
            && initial_visibility(show, context) != Some(true)
        {
            output.push_str(" hidden");
        }
    }
    if let Some(extra) = extra {
        output.push_str(extra);
    }
    output
}

fn class_attr(classes: Vec<String>) -> String {
    if classes.is_empty() {
        String::new()
    } else {
        format!(r#" class="{}""#, classes.join(" "))
    }
}

fn initial_visibility(
    show: &VisibilityCondition,
    context: &ReactiveRenderContext,
) -> Option<bool> {
    match show {
        VisibilityCondition::Static(_) => None,
        VisibilityCondition::Signal(path) => context.initial_value(path).map(initial_truthy),
        VisibilityCondition::NumberComparison { path, comparison } => context
            .initial_value(path)
            .and_then(initial_number)
            .and_then(|value| comparison.value.parse::<f64>().ok().map(|target| (value, target)))
            .map(|(value, target)| match comparison.operator {
                dowe_components::NumberComparisonOperator::GreaterThan => value > target,
                dowe_components::NumberComparisonOperator::GreaterThanOrEqual => value >= target,
                dowe_components::NumberComparisonOperator::LessThan => value < target,
                dowe_components::NumberComparisonOperator::LessThanOrEqual => value <= target,
            }),
        VisibilityCondition::StringEquality { path, value } => context
            .initial_value(path)
            .and_then(initial_string)
            .map(|current| current.as_str() == value),
    }
}

fn initial_truthy(value: &ViewSignalValue) -> bool {
    match value {
        ViewSignalValue::Null => false,
        ViewSignalValue::Bool(value) => *value,
        ViewSignalValue::Number(value) => value.parse::<f64>().map(|value| value != 0.0).unwrap_or(false),
        ViewSignalValue::String(value) => !value.is_empty(),
        ViewSignalValue::Array(_) | ViewSignalValue::Object(_) => true,
    }
}

fn initial_number(value: &ViewSignalValue) -> Option<f64> {
    match value {
        ViewSignalValue::Number(value) => value.parse().ok(),
        ViewSignalValue::String(value) => value.parse().ok(),
        _ => None,
    }
}

fn initial_string(value: &ViewSignalValue) -> Option<String> {
    match value {
        ViewSignalValue::String(value) => Some(value.clone()),
        ViewSignalValue::Bool(value) => Some(value.to_string()),
        ViewSignalValue::Number(value) => Some(value.clone()),
        ViewSignalValue::Null => Some("null".to_string()),
        ViewSignalValue::Array(_) | ViewSignalValue::Object(_) => None,
    }
}

fn push_literal(segments: &mut Vec<JsSegment>, value: &str) {
    if value.is_empty() {
        return;
    }

    if let Some(JsSegment::Literal(existing)) = segments.last_mut() {
        existing.push_str(value);
    } else {
        segments.push(JsSegment::Literal(value.to_string()));
    }
}

enum JsSegment {
    Literal(String),
    Children,
}

fn short_id(namespace: &str, source: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;

    for byte in namespace.bytes().chain([0]).chain(source.bytes()) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    let alphabet = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut value = hash;
    let mut id = String::with_capacity(8);

    for index in 0..8 {
        let digit = (value % 36) as usize;
        id.push(alphabet[digit] as char);
        value /= 36;
        if value == 0 {
            value = hash.rotate_left((index + 1) as u32);
        }
    }

    id
}

fn js_string_literal(value: &str) -> String {
    format!("\"{}\"", escape_js(value))
}

fn escape_js(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "")
}

fn escape_json(value: &str) -> String {
    escape_js(value)
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_attr(value: &str) -> String {
    escape_html(value)
}
