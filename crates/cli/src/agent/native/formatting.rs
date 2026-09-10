use serde_json::Value;

const SESSION_TITLE_WIDTH: usize = 32;
const SESSION_ID_WIDTH: usize = 8;

/// Build the compact, aligned label used by the interactive resume selector.
///
/// The selector uses the full ID from its source row for selection; this label is
/// only presentation and intentionally contains the same status fields as `/sessions`.
pub(super) fn resume_session_label(row: &Value) -> String {
    let id = row["id"].as_str().unwrap_or_default();
    let (compatibility, state) = session_markers(row);
    format!(
        "{:<SESSION_TITLE_WIDTH$} {:<SESSION_ID_WIDTH$} {compatibility}{state}",
        bounded_title(&session_title(row)),
        short_id(id),
    )
}

fn truncate_label(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let end = limit.saturating_sub(1);
    value.chars().take(end).collect::<String>() + "…"
}

/// Render command results as stable, readable lines without exposing JSON syntax.
pub(super) fn format_value(value: &Value) -> String {
    if is_sessions_result(value) {
        return render_sessions(value);
    }
    let mut lines = Vec::new();
    render_value(value, 0, "", &mut lines);
    lines.join("\n")
}

fn is_sessions_result(value: &Value) -> bool {
    value["sessions"].is_array() && value["inventory"].is_array() && value["active"].is_string()
}

fn render_sessions(value: &Value) -> String {
    let inventory = value["inventory"]
        .as_array()
        .expect("sessions inventory is an array");
    let active = value["active"].as_str().unwrap_or_default();
    let heading = format!("Sessions ({})", inventory.len());
    let active_line = inventory
        .iter()
        .find(|row| row["id"].as_str() == Some(active))
        .map(|row| {
            format!(
                "Active: {} · {}",
                short_id(active),
                bounded_title(&session_title(row))
            )
        })
        .unwrap_or_else(|| "Active: none".into());
    let mut lines = vec![heading, active_line];

    for (index, row) in inventory.iter().enumerate() {
        let id = row["id"].as_str().unwrap_or_default();
        let marker = if id == active { '*' } else { ' ' };
        let (compatibility, state) = session_markers(row);
        lines.push(format!(
            "{marker} {:>2}. {:<SESSION_TITLE_WIDTH$} {:<SESSION_ID_WIDTH$} {compatibility}{state}",
            index + 1,
            bounded_title(&session_title(row)),
            short_id(id),
        ));
    }
    if inventory.is_empty() {
        lines.push("  (none)".into());
    }
    lines.join("\n")
}

fn short_id(id: &str) -> String {
    id.chars().take(SESSION_ID_WIDTH).collect()
}

fn session_markers(row: &Value) -> (char, char) {
    let compatibility = match row["catalog_compatible"].as_bool() {
        Some(true) => '✓',
        Some(false) => '×',
        None => '?',
    };
    let state = match row["state"].as_str() {
        Some("recorded") => ' ',
        Some("interrupted") => '!',
        Some(_) | None => '?',
    };
    (compatibility, state)
}

fn session_title(row: &Value) -> String {
    row["title"]
        .as_str()
        .or_else(|| row["initial_prompt_preview"].as_str())
        .map(normalize_display_text)
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| "Untitled session".into())
}

fn bounded_title(value: &str) -> String {
    truncate_label(value, SESSION_TITLE_WIDTH)
}

fn normalize_display_text(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_value(value: &Value, indent: usize, prefix: &str, lines: &mut Vec<String>) {
    let padding = " ".repeat(indent);
    match value {
        Value::Object(object) => {
            if object.is_empty() {
                lines.push(format!("{padding}{prefix}(none)"));
                return;
            }
            let mut fields = object
                .iter()
                .filter(|(key, value)| {
                    !(key.as_str() == "sessions"
                        && object.contains_key("inventory")
                        && value
                            .as_array()
                            .is_some_and(|items| items.iter().all(Value::is_string)))
                })
                .collect::<Vec<_>>();
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
    fn formats_session_inventory_with_human_metadata() {
        let rendered = format_value(&json!({
            "inventory": [{
                "id": "0123456789abcdef",
                "title": "Build dashboard",
                "initial_prompt_preview": "Build a dashboard with safe colors"
            }]
        }));
        assert!(rendered.contains("title: Build dashboard"));
        assert!(rendered.contains("initial_prompt_preview: Build a dashboard with safe colors"));
        assert!(rendered.contains("id: 0123456789abcdef"));
    }

    #[test]
    fn formats_sessions_with_active_marker_and_compact_entries() {
        let rendered = format_value(&json!({
            "sessions": ["0123456789abcdef", "fedcba9876543210"],
            "active": "0123456789abcdef",
            "inventory": [
                {"id": "0123456789abcdef", "title": "Build dashboard", "state": "recorded", "catalog_compatible": true},
                {"id": "fedcba9876543210", "title": null, "initial_prompt_preview": "Fallback prompt", "state": "interrupted", "catalog_compatible": false}
            ]
        }));

        assert_eq!(
            rendered,
            "Sessions (2)\nActive: 01234567 · Build dashboard\n*  1. Build dashboard                  01234567 ✓ \n   2. Fallback prompt                  fedcba98 ×!"
        );
    }

    #[test]
    fn formats_sessions_truncate_titles_without_wrapping() {
        let rendered = format_value(&json!({
            "sessions": ["0123456789abcdef"],
            "active": "0123456789abcdef",
            "inventory": [{"id": "0123456789abcdef", "title": "A very long title that should be safely bounded so it never causes ugly terminal wrapping", "state": "recorded", "catalog_compatible": true}]
        }));
        let lines = rendered.lines().collect::<Vec<_>>();
        let entry = lines[2];

        assert!(lines[1].chars().count() <= 64);
        assert!(entry.chars().count() <= 60);
        assert!(entry.contains('…'));
        assert!(!entry.chars().any(char::is_control));
    }

    #[test]
    fn formats_sessions_normalize_controls_and_align_entries() {
        let rendered = format_value(&json!({
            "sessions": ["a", "b"],
            "active": "a",
            "inventory": [
                {"id": "a", "title": "  First\u{0000} session\n", "state": "recorded", "catalog_compatible": true},
                {"id": "b", "title": "Second", "state": "interrupted", "catalog_compatible": null}
            ]
        }));
        let entries = rendered.lines().skip(2).collect::<Vec<_>>();

        assert_eq!(entries[0].chars().count(), entries[1].chars().count());
        assert!(entries.iter().all(|entry| entry.chars().count() <= 60));
        assert!(
            entries
                .iter()
                .all(|entry| !entry.chars().any(char::is_control))
        );
        assert!(entries[0].contains("First session"));
        assert!(!entries[0].contains("recorded"));
    }

    #[test]
    fn formats_sessions_without_mutating_or_exposing_json_shape() {
        let value = json!({
            "sessions": ["0123456789abcdef"],
            "active": "0123456789abcdef",
            "inventory": [{"id": "0123456789abcdef", "title": "Build dashboard", "state": "recorded", "catalog_compatible": true}]
        });
        let original = value.clone();

        let rendered = format_value(&value);

        assert!(!rendered.contains("inventory:"));
        assert!(!rendered.contains("0123456789abcdef"));
        assert_eq!(value, original);
        assert_eq!(value["sessions"][0], "0123456789abcdef");
    }

    #[test]
    fn resume_labels_match_compact_aligned_session_style() {
        let label = super::resume_session_label(&json!({
            "id": "0123456789abcdef0123456789abcdef",
            "title": "  Build\u{0000}   dashboard  ",
            "initial_prompt_preview": "Build dashboard with safe colors",
            "state": "interrupted",
            "catalog_compatible": false
        }));
        assert_eq!(label, "Build dashboard                  01234567 ×!");
        assert_eq!(label.chars().count(), 44);
        assert!(!label.contains("safe colors"));
    }

    #[test]
    fn resume_labels_are_bounded_and_keep_status_fields() {
        let label = super::resume_session_label(&json!({
            "id": "0123456789abcdef0123456789abcdef",
            "title": "A title that is intentionally very long and should be clipped without wrapping",
            "state": "recorded",
            "catalog_compatible": true
        }));
        assert!(label.chars().count() <= 44);
        assert!(label.contains("01234567"));
        assert!(label.ends_with("✓ "));
        assert!(label.contains('…'));
    }

    #[test]
    fn resume_labels_use_normalized_preview_as_title_when_title_is_absent() {
        let label = super::resume_session_label(&json!({
            "id": "0123456789abcdef0123456789abcdef",
            "title": null,
            "initial_prompt_preview": " Start   a new dashboard\n",
            "state": "recorded",
            "catalog_compatible": null
        }));
        assert!(label.starts_with("Start a new dashboard"));
        assert!(label.ends_with("01234567 ? "));
    }

    #[test]
    fn formats_scalar_values_without_json_quotes() {
        assert_eq!(format_value(&json!("hello")), "hello");
        assert_eq!(format_value(&json!(42)), "42");
        assert_eq!(format_value(&json!(false)), "false");
        assert_eq!(format_value(&json!(null)), "null");
    }
}
