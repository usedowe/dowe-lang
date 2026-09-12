/// Render a host approval as a small, human-readable change summary.
///
/// Approval payloads are internal protocol data and can contain large file
/// bodies, tool arguments, and environment metadata. Showing that object
/// directly made the interactive agent look like a JSON debugger and could
/// flood the terminal. Keep the confirmation useful without exposing the wire
/// shape or unbounded content.
pub(super) fn format_approval(value: &Value) -> String {
    let call = &value["call"];
    let details = &value["details"];
    let name = call["name"].as_str().unwrap_or("tool");
    let heading = match name {
        "write_file" | "edit_file" => "File change requested",
        "write_asset" => "Asset change requested",
        "propose_instruction_update" => "Instruction change requested",
        "shell" => "Command requested",
        "generate_image" => "Image generation requested",
        "capture_web_screenshot" => "Screenshot requested",
        _ => "Operation requested",
    };
    let mut lines = vec![heading.to_string()];
    push_field(&mut lines, "path", details["path"].as_str());
    push_field(&mut lines, "destination", details["destination"].as_str());
    push_field(&mut lines, "skill", details["skill"].as_str());
    push_field(&mut lines, "reason", details["reason"].as_str());

    match name {
        "write_file" | "edit_file" | "propose_instruction_update" => {
            push_preview(&mut lines, "before", details["before"].as_str());
            push_preview(&mut lines, "after", details["after"].as_str());
        }
        "write_asset" => {
            push_field_value(&mut lines, "bytes", &details["byte_count"]);
            push_field(&mut lines, "sha256", details["sha256"].as_str());
            if details["before_byte_count"].is_number() {
                push_field_value(&mut lines, "previous bytes", &details["before_byte_count"]);
            }
        }
        "shell" => {
            push_field(&mut lines, "command", details["command"].as_str());
            push_field(&mut lines, "working directory", details["cwd"].as_str());
            push_field(&mut lines, "resource", details["resource"].as_str());
        }
        "generate_image" => {
            push_preview(&mut lines, "prompt", details["prompt"].as_str());
        }
        _ => {}
    }
    lines.push("This operation is single-use and will be checked again before applying.".into());
    lines.join("\n")
}

pub(super) fn format_approval_batch(values: &[Value]) -> String {
    if values.len() < 2 {
        return values
            .first()
            .map_or_else(|| "No file changes requested".into(), format_approval);
    }
    const MAX_BATCH_OPERATIONS: usize = 8;
    const MAX_BATCH_LINES: usize = 96;
    const RESERVED_BATCH_LINES: usize = 2;
    let content_limit = MAX_BATCH_LINES - RESERVED_BATCH_LINES;
    let mut lines = vec![format!("File changes requested ({})", values.len())];
    for (index, value) in values.iter().take(MAX_BATCH_OPERATIONS).enumerate() {
        lines.push(format!("Operation {}:", index + 1));
        for line in format_approval(value).lines() {
            if line.starts_with("This operation is single-use") {
                continue;
            }
            if lines.len() >= content_limit {
                break;
            }
            lines.push(format!("  {line}"));
        }
        if lines.len() >= content_limit {
            break;
        }
    }
    if values.len() > MAX_BATCH_OPERATIONS || lines.len() >= content_limit {
        lines.push("  … additional operations omitted from preview".into());
    }
    lines.push("These operations are single-use and will be checked again before applying.".into());
    lines.join("\n")
}

fn push_field(lines: &mut Vec<String>, label: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(format!(
            "  {label}: {}",
            bounded_display(value, APPROVAL_PREVIEW_LINE_CHARS)
        ));
    }
}

fn push_field_value(lines: &mut Vec<String>, label: &str, value: &Value) {
    if !value.is_null() {
        lines.push(format!(
            "  {label}: {}",
            bounded_display(&value.to_string(), APPROVAL_PREVIEW_LINE_CHARS)
        ));
    }
}

fn push_preview(lines: &mut Vec<String>, label: &str, value: Option<&str>) {
    lines.push(format!("  {label}:"));
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        lines.push("    (empty)".into());
        return;
    };
    let mut omitted = false;
    for line in value.lines().take(APPROVAL_PREVIEW_LINES) {
        if lines.len() >= APPROVAL_PREVIEW_LINES * 2 + 8 {
            omitted = true;
            break;
        }
        lines.push(format!(
            "    {}",
            bounded_display(line, APPROVAL_PREVIEW_LINE_CHARS)
        ));
    }
    if value.lines().count() > APPROVAL_PREVIEW_LINES {
        omitted = true;
    }
    if omitted {
        lines.push("    … preview truncated".into());
    }
}
