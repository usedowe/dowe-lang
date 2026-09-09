use serde_json::Value;

/// Render command results as stable, readable lines without exposing JSON syntax.
pub(super) fn format_value(value: &Value) -> String {
    let mut lines = Vec::new();
    render_value(value, 0, "", &mut lines);
    lines.join("\n")
}

fn render_value(value: &Value, indent: usize, prefix: &str, lines: &mut Vec<String>) {
    let padding = " ".repeat(indent);
    match value {
        Value::Object(object) => {
            if object.is_empty() {
                lines.push(format!("{padding}{prefix}(none)"));
                return;
            }
            let mut fields = object.iter().collect::<Vec<_>>();
            fields.sort_by_key(|(key, _)| *key);
            let mut fields = fields.into_iter();
            if let Some((key, value)) = fields.next() {
                render_field(key, value, indent, prefix, lines);
            }
            let continuation_indent = if prefix.is_empty() {
                indent
            } else {
                indent + 2
            };
            for (key, value) in fields {
                render_field(key, value, continuation_indent, "", lines);
            }
        }
        Value::Array(array) => {
            if array.is_empty() {
                lines.push(format!("{padding}{prefix}(none)"));
                return;
            }
            for value in array {
                render_value(value, indent, "- ", lines);
            }
        }
        _ => lines.push(format!("{padding}{prefix}{}", scalar(value))),
    }
}

fn render_field(key: &str, value: &Value, indent: usize, prefix: &str, lines: &mut Vec<String>) {
    let padding = " ".repeat(indent);
    match value {
        Value::Object(object) if !object.is_empty() => {
            lines.push(format!("{padding}{prefix}{key}:",));
            render_value(value, indent + 2, "", lines);
        }
        Value::Array(array) if !array.is_empty() => {
            lines.push(format!("{padding}{prefix}{key}:",));
            render_value(value, indent + 2, "", lines);
        }
        Value::Object(_) | Value::Array(_) => {
            lines.push(format!("{padding}{prefix}{key}: (none)"));
        }
        _ => lines.push(format!("{padding}{prefix}{key}: {}", scalar(value))),
    }
}

fn scalar(value: &Value) -> String {
    match value {
        Value::Null => "null".into(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Object(_) | Value::Array(_) => unreachable!("containers are rendered recursively"),
    }
}

#[cfg(test)]
mod tests {
    use super::format_value;
    use serde_json::json;

    #[test]
    fn formats_nested_objects_and_lists_as_labeled_lines() {
        let value = json!({
            "session": {
                "id": "abc",
                "active": true,
                "usage": {"input": 12, "output": null}
            },
            "events": [
                {"kind": "message", "text": "hello"},
                {"kind": "tool", "ok": false}
            ],
            "empty": []
        });

        assert_eq!(
            format_value(&value),
            "empty: (none)\nevents:\n  - kind: message\n    text: hello\n  - kind: tool\n    ok: false\nsession:\n  active: true\n  id: abc\n  usage:\n    input: 12\n    output: null"
        );
    }

    #[test]
    fn formats_scalar_values_without_json_quotes() {
        assert_eq!(format_value(&json!("hello")), "hello");
        assert_eq!(format_value(&json!(42)), "42");
        assert_eq!(format_value(&json!(false)), "false");
        assert_eq!(format_value(&json!(null)), "null");
    }
}
