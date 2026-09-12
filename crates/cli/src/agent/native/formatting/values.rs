/// Render command results as stable, readable lines without exposing JSON syntax.
pub(super) fn format_value(value: &Value) -> String {
    if is_sessions_result(value) {
        return render_sessions(value);
    }
    if is_sdd_status_result(value) {
        return render_sdd_status(value);
    }
    let mut lines = Vec::new();
    render_value(value, 0, "", &mut lines);
    lines.join("\n")
}

fn is_sessions_result(value: &Value) -> bool {
    value["sessions"].is_array() && value["inventory"].is_array() && value["active"].is_string()
}

fn is_sdd_status_result(value: &Value) -> bool {
    value["changes"].is_array() && value["canonical_surface"].is_boolean()
}

fn render_sdd_status(value: &Value) -> String {
    let canonical = value["canonical_surface"].as_bool().unwrap_or(false);
    let changes = value["changes"]
        .as_array()
        .expect("sdd changes is an array");
    let surface = if canonical {
        "canonical SDD (specs/ or openspec/)"
    } else {
        "project-local SDD (.agents/changes/)"
    };
    let mut lines = vec!["SDD status".into(), format!("Surface: {surface}")];
    if changes.is_empty() {
        lines.push("Changes: none".into());
    } else {
        lines.push(format!("Changes ({}):", changes.len()));
        for change in changes.iter().take(8) {
            let name = change.as_str().unwrap_or("(invalid name)");
            lines.push(format!("  - {}", bounded_display(name, 96)));
        }
        if changes.len() > 8 {
            lines.push(format!("  - … {} more", changes.len() - 8));
        }
    }
    let next = if canonical {
        "Next: add or update a change in specs/ or openspec/."
    } else {
        "Next: use /sdd init <change-id> [title] to create a change packet."
    };
    lines.push(next.into());
    lines.join("\n")
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
